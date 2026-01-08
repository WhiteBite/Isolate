//! Library module - управление правилами доступа для сервисов
//!
//! Определяет какой метод обхода использовать для каких доменов/сервисов.

use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use chrono::Utc;

use crate::state::AppState;
use crate::core::errors::IsolateError;

/// Метод доступа к ресурсу
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessMethod {
    /// Использовать Zapret (DPI bypass)
    Zapret,
    /// Использовать VLESS прокси
    Vless,
    /// Прямое подключение (без обхода)
    Direct,
    /// Заблокировать доступ
    Block,
}

impl Default for AccessMethod {
    fn default() -> Self {
        Self::Direct
    }
}

/// Правило библиотеки - определяет метод доступа для домена/паттерна
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRule {
    /// Уникальный идентификатор правила
    pub id: String,
    /// ID сервиса (если правило привязано к сервису)
    pub service_id: Option<String>,
    /// Домен или паттерн (например: *.example.com, youtube.com)
    pub pattern: String,
    /// Метод доступа
    pub method: AccessMethod,
    /// Включено ли правило
    pub is_enabled: bool,
    /// ID конкретной стратегии (для Zapret)
    pub strategy_id: Option<String>,
    /// Приоритет правила (выше = важнее)
    pub priority: i32,
    /// Время создания (Unix timestamp)
    pub created_at: i64,
    /// Время последнего обновления (Unix timestamp)
    pub updated_at: i64,
}

impl LibraryRule {
    /// Создать новое правило
    pub fn new(pattern: String, method: AccessMethod) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            service_id: None,
            pattern,
            method,
            is_enabled: true,
            strategy_id: None,
            priority: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Пресет библиотеки - набор предустановленных правил
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPreset {
    /// Уникальный идентификатор пресета
    pub id: String,
    /// Название пресета
    pub name: String,
    /// Описание пресета
    pub description: String,
    /// Правила в пресете
    pub rules: Vec<LibraryRule>,
    /// Встроенный пресет (нельзя удалить)
    pub is_builtin: bool,
}

/// Входные данные для создания правила
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleInput {
    pub service_id: Option<String>,
    pub pattern: String,
    pub method: AccessMethod,
    pub strategy_id: Option<String>,
    pub priority: Option<i32>,
}

/// Входные данные для обновления правила
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRuleInput {
    pub id: String,
    pub service_id: Option<String>,
    pub pattern: Option<String>,
    pub method: Option<AccessMethod>,
    pub is_enabled: Option<bool>,
    pub strategy_id: Option<String>,
    pub priority: Option<i32>,
}

// ============================================================================
// Mock Data
// ============================================================================

fn get_mock_rules() -> Vec<LibraryRule> {
    let now = Utc::now().timestamp();
    vec![
        LibraryRule {
            id: "rule-youtube".to_string(),
            service_id: Some("youtube".to_string()),
            pattern: "*.youtube.com".to_string(),
            method: AccessMethod::Zapret,
            is_enabled: true,
            strategy_id: Some("zapret-youtube-v1".to_string()),
            priority: 100,
            created_at: now - 86400,
            updated_at: now,
        },
        LibraryRule {
            id: "rule-discord".to_string(),
            service_id: Some("discord".to_string()),
            pattern: "*.discord.com".to_string(),
            method: AccessMethod::Zapret,
            is_enabled: true,
            strategy_id: Some("zapret-discord-v1".to_string()),
            priority: 100,
            created_at: now - 86400,
            updated_at: now,
        },
        LibraryRule {
            id: "rule-discord-gg".to_string(),
            service_id: Some("discord".to_string()),
            pattern: "*.discord.gg".to_string(),
            method: AccessMethod::Zapret,
            is_enabled: true,
            strategy_id: Some("zapret-discord-v1".to_string()),
            priority: 99,
            created_at: now - 86400,
            updated_at: now,
        },
        LibraryRule {
            id: "rule-telegram".to_string(),
            service_id: Some("telegram".to_string()),
            pattern: "*.telegram.org".to_string(),
            method: AccessMethod::Vless,
            is_enabled: true,
            strategy_id: None,
            priority: 100,
            created_at: now - 86400,
            updated_at: now,
        },
        LibraryRule {
            id: "rule-spotify".to_string(),
            service_id: Some("spotify".to_string()),
            pattern: "*.spotify.com".to_string(),
            method: AccessMethod::Direct,
            is_enabled: true,
            strategy_id: None,
            priority: 50,
            created_at: now - 86400,
            updated_at: now,
        },
        LibraryRule {
            id: "rule-blocked-example".to_string(),
            service_id: None,
            pattern: "blocked-site.example".to_string(),
            method: AccessMethod::Block,
            is_enabled: false,
            strategy_id: None,
            priority: 10,
            created_at: now - 86400,
            updated_at: now,
        },
    ]
}

fn get_mock_presets() -> Vec<LibraryPreset> {
    let now = Utc::now().timestamp();
    vec![
        LibraryPreset {
            id: "preset-default".to_string(),
            name: "По умолчанию".to_string(),
            description: "Базовый набор правил для популярных сервисов".to_string(),
            rules: vec![
                LibraryRule {
                    id: "preset-rule-1".to_string(),
                    service_id: Some("youtube".to_string()),
                    pattern: "*.youtube.com".to_string(),
                    method: AccessMethod::Zapret,
                    is_enabled: true,
                    strategy_id: None,
                    priority: 100,
                    created_at: now,
                    updated_at: now,
                },
                LibraryRule {
                    id: "preset-rule-2".to_string(),
                    service_id: Some("discord".to_string()),
                    pattern: "*.discord.com".to_string(),
                    method: AccessMethod::Zapret,
                    is_enabled: true,
                    strategy_id: None,
                    priority: 100,
                    created_at: now,
                    updated_at: now,
                },
            ],
            is_builtin: true,
        },
        LibraryPreset {
            id: "preset-vless-all".to_string(),
            name: "Всё через VLESS".to_string(),
            description: "Направить весь трафик через VLESS прокси".to_string(),
            rules: vec![
                LibraryRule {
                    id: "preset-vless-rule".to_string(),
                    service_id: None,
                    pattern: "*".to_string(),
                    method: AccessMethod::Vless,
                    is_enabled: true,
                    strategy_id: None,
                    priority: 1,
                    created_at: now,
                    updated_at: now,
                },
            ],
            is_builtin: true,
        },
        LibraryPreset {
            id: "preset-gaming".to_string(),
            name: "Игровой".to_string(),
            description: "Оптимизированные правила для игровых сервисов".to_string(),
            rules: vec![
                LibraryRule {
                    id: "preset-gaming-discord".to_string(),
                    service_id: Some("discord".to_string()),
                    pattern: "*.discord.com".to_string(),
                    method: AccessMethod::Zapret,
                    is_enabled: true,
                    strategy_id: None,
                    priority: 100,
                    created_at: now,
                    updated_at: now,
                },
                LibraryRule {
                    id: "preset-gaming-steam".to_string(),
                    service_id: Some("steam".to_string()),
                    pattern: "*.steampowered.com".to_string(),
                    method: AccessMethod::Direct,
                    is_enabled: true,
                    strategy_id: None,
                    priority: 90,
                    created_at: now,
                    updated_at: now,
                },
            ],
            is_builtin: true,
        },
    ]
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Получить все правила библиотеки
#[tauri::command]
pub async fn get_library_rules(
    _state: State<'_, Arc<AppState>>,
) -> Result<Vec<LibraryRule>, IsolateError> {
    info!("Getting all library rules");
    
    // TODO: Загрузка из БД
    Ok(get_mock_rules())
}

/// Получить одно правило по ID
#[tauri::command]
pub async fn get_library_rule(
    rule_id: String,
    _state: State<'_, Arc<AppState>>,
) -> Result<Option<LibraryRule>, IsolateError> {
    info!(rule_id = %rule_id, "Getting library rule");
    
    // TODO: Загрузка из БД
    let rules = get_mock_rules();
    Ok(rules.into_iter().find(|r| r.id == rule_id))
}

/// Добавить новое правило
#[tauri::command]
pub async fn add_library_rule(
    input: CreateRuleInput,
    _state: State<'_, Arc<AppState>>,
) -> Result<LibraryRule, IsolateError> {
    info!(pattern = %input.pattern, method = ?input.method, "Adding library rule");
    
    let now = Utc::now().timestamp();
    let rule = LibraryRule {
        id: Uuid::new_v4().to_string(),
        service_id: input.service_id,
        pattern: input.pattern,
        method: input.method,
        is_enabled: true,
        strategy_id: input.strategy_id,
        priority: input.priority.unwrap_or(0),
        created_at: now,
        updated_at: now,
    };
    
    // TODO: Сохранение в БД
    
    Ok(rule)
}

/// Обновить существующее правило
#[tauri::command]
pub async fn update_library_rule(
    input: UpdateRuleInput,
    _state: State<'_, Arc<AppState>>,
) -> Result<LibraryRule, IsolateError> {
    info!(rule_id = %input.id, "Updating library rule");
    
    // TODO: Загрузка из БД и обновление
    let rules = get_mock_rules();
    let mut rule = rules
        .into_iter()
        .find(|r| r.id == input.id)
        .ok_or_else(|| IsolateError::NotFound(format!("Rule {} not found", input.id)))?;
    
    // Применяем обновления
    if let Some(service_id) = input.service_id {
        rule.service_id = Some(service_id);
    }
    if let Some(pattern) = input.pattern {
        rule.pattern = pattern;
    }
    if let Some(method) = input.method {
        rule.method = method;
    }
    if let Some(is_enabled) = input.is_enabled {
        rule.is_enabled = is_enabled;
    }
    if let Some(strategy_id) = input.strategy_id {
        rule.strategy_id = Some(strategy_id);
    }
    if let Some(priority) = input.priority {
        rule.priority = priority;
    }
    rule.updated_at = Utc::now().timestamp();
    
    // TODO: Сохранение в БД
    
    Ok(rule)
}

/// Удалить правило
#[tauri::command]
pub async fn delete_library_rule(
    rule_id: String,
    _state: State<'_, Arc<AppState>>,
) -> Result<bool, IsolateError> {
    info!(rule_id = %rule_id, "Deleting library rule");
    
    // TODO: Удаление из БД
    // Проверяем что правило существует
    let rules = get_mock_rules();
    let exists = rules.iter().any(|r| r.id == rule_id);
    
    if !exists {
        return Err(IsolateError::NotFound(format!("Rule {} not found", rule_id)));
    }
    
    Ok(true)
}

/// Изменить метод доступа для правила
#[tauri::command]
pub async fn set_rule_method(
    rule_id: String,
    method: AccessMethod,
    _state: State<'_, Arc<AppState>>,
) -> Result<LibraryRule, IsolateError> {
    info!(rule_id = %rule_id, method = ?method, "Setting rule method");
    
    // TODO: Загрузка из БД и обновление
    let rules = get_mock_rules();
    let mut rule = rules
        .into_iter()
        .find(|r| r.id == rule_id)
        .ok_or_else(|| IsolateError::NotFound(format!("Rule {} not found", rule_id)))?;
    
    rule.method = method;
    rule.updated_at = Utc::now().timestamp();
    
    // TODO: Сохранение в БД
    
    Ok(rule)
}

/// Включить/выключить правило
#[tauri::command]
pub async fn toggle_library_rule(
    rule_id: String,
    enabled: bool,
    _state: State<'_, Arc<AppState>>,
) -> Result<LibraryRule, IsolateError> {
    info!(rule_id = %rule_id, enabled = enabled, "Toggling library rule");
    
    // TODO: Загрузка из БД и обновление
    let rules = get_mock_rules();
    let mut rule = rules
        .into_iter()
        .find(|r| r.id == rule_id)
        .ok_or_else(|| IsolateError::NotFound(format!("Rule {} not found", rule_id)))?;
    
    rule.is_enabled = enabled;
    rule.updated_at = Utc::now().timestamp();
    
    // TODO: Сохранение в БД
    
    Ok(rule)
}

/// Получить все пресеты
#[tauri::command]
pub async fn get_library_presets(
    _state: State<'_, Arc<AppState>>,
) -> Result<Vec<LibraryPreset>, IsolateError> {
    info!("Getting library presets");
    
    // TODO: Загрузка из БД + встроенные пресеты
    Ok(get_mock_presets())
}

/// Получить пресет по ID
#[tauri::command]
pub async fn get_library_preset(
    preset_id: String,
    _state: State<'_, Arc<AppState>>,
) -> Result<Option<LibraryPreset>, IsolateError> {
    info!(preset_id = %preset_id, "Getting library preset");
    
    let presets = get_mock_presets();
    Ok(presets.into_iter().find(|p| p.id == preset_id))
}

/// Применить пресет (заменить текущие правила правилами из пресета)
#[tauri::command]
pub async fn apply_library_preset(
    preset_id: String,
    _state: State<'_, Arc<AppState>>,
) -> Result<Vec<LibraryRule>, IsolateError> {
    info!(preset_id = %preset_id, "Applying library preset");
    
    let presets = get_mock_presets();
    let preset = presets
        .into_iter()
        .find(|p| p.id == preset_id)
        .ok_or_else(|| IsolateError::NotFound(format!("Preset {} not found", preset_id)))?;
    
    // TODO: Очистить текущие правила и применить правила из пресета
    // Генерируем новые ID для правил
    let now = Utc::now().timestamp();
    let rules: Vec<LibraryRule> = preset
        .rules
        .into_iter()
        .map(|mut r| {
            r.id = Uuid::new_v4().to_string();
            r.created_at = now;
            r.updated_at = now;
            r
        })
        .collect();
    
    Ok(rules)
}

/// Получить правила для конкретного сервиса
#[tauri::command]
pub async fn get_rules_for_service(
    service_id: String,
    _state: State<'_, Arc<AppState>>,
) -> Result<Vec<LibraryRule>, IsolateError> {
    info!(service_id = %service_id, "Getting rules for service");
    
    let rules = get_mock_rules();
    let filtered: Vec<LibraryRule> = rules
        .into_iter()
        .filter(|r| r.service_id.as_ref() == Some(&service_id))
        .collect();
    
    Ok(filtered)
}

/// Получить правила по методу доступа
#[tauri::command]
pub async fn get_rules_by_method(
    method: AccessMethod,
    _state: State<'_, Arc<AppState>>,
) -> Result<Vec<LibraryRule>, IsolateError> {
    info!(method = ?method, "Getting rules by method");
    
    let rules = get_mock_rules();
    let filtered: Vec<LibraryRule> = rules
        .into_iter()
        .filter(|r| r.method == method)
        .collect();
    
    Ok(filtered)
}
