//! Storage types and data structures

use serde::{Deserialize, Serialize};

/// Кэшированная стратегия
#[derive(Debug, Clone)]
pub struct CachedStrategy {
    pub strategy_id: String,
    pub score: f64,
    pub timestamp: i64,
}

/// Обученная стратегия Orchestra
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedStrategy {
    pub domain: String,
    pub strategy_id: String,
    pub successes: u32,
    pub failures: u32,
    pub locked_at: Option<String>,
    pub updated_at: String,
}

/// Запись истории тестов
#[derive(Debug, Clone)]
pub struct TestHistoryEntry {
    pub env_key: String,
    pub success: bool,
    pub score: f64,
    pub latency_ms: f64,
    pub timestamp: i64,
}

/// Routing rule model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub source: String,        // "domain" | "app" | "ip" | "all"
    pub source_value: Option<String>,
    pub action: String,        // "direct" | "proxy" | "block" | "dpi-bypass"
    pub proxy_id: Option<String>,
    pub priority: i32,
}

/// Helper struct for reading proxy from database
pub(crate) struct ProxyConfigRow {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub server: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub uuid: Option<String>,
    pub tls: bool,
    pub sni: Option<String>,
    pub transport: Option<String>,
    pub custom_fields: String,
    pub active: bool,
}

impl ProxyConfigRow {
    pub fn into_proxy_config(self) -> crate::core::errors::Result<crate::core::models::ProxyConfig> {
        use crate::core::models::{ProxyConfig, ProxyProtocol};
        
        let protocol: ProxyProtocol = serde_json::from_str(&format!("\"{}\"", self.protocol))
            .unwrap_or(ProxyProtocol::Socks5);
        
        let custom_fields: std::collections::HashMap<String, String> = 
            serde_json::from_str(&self.custom_fields).unwrap_or_default();
        
        // Decrypt password when reading from database
        let decrypted_password = match &self.password {
            Some(pwd) if !pwd.is_empty() => {
                match crate::core::crypto::decrypt_password(pwd) {
                    Ok(decrypted) => Some(decrypted),
                    Err(e) => {
                        tracing::warn!("Failed to decrypt password, returning as-is: {}", e);
                        Some(pwd.clone())
                    }
                }
            }
            other => other.clone(),
        };

        Ok(ProxyConfig {
            id: self.id,
            name: self.name,
            protocol,
            server: self.server,
            port: self.port,
            username: self.username,
            password: decrypted_password,
            uuid: self.uuid,
            tls: self.tls,
            sni: self.sni,
            transport: self.transport,
            custom_fields,
            active: self.active,
        })
    }
}

/// Запись истории результатов стратегии
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyHistoryEntry {
    pub id: i64,
    pub strategy_id: String,
    pub service_id: String,
    pub success: bool,
    pub latency_ms: Option<f64>,
    pub timestamp: i64,
}

/// Агрегированная статистика стратегии
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyStats {
    pub strategy_id: String,
    pub total_tests: u32,
    pub successes: u32,
    pub failures: u32,
    pub success_rate: f64,
    pub avg_latency_ms: Option<f64>,
    pub last_test_at: Option<i64>,
}

/// Ключи настроек
/// 
/// Эти константы используются для типобезопасного доступа к настройкам в Storage.
/// Некоторые ключи зарезервированы для будущего использования.
#[allow(dead_code)] // Public API - keys for type-safe settings access
pub mod settings_keys {
    /// Выбранные сервисы для тестирования
    pub const SELECTED_SERVICES: &str = "selected_services";
    /// Автозапуск при старте системы
    pub const AUTOSTART: &str = "autostart";
    /// Автоматическое применение стратегии
    pub const AUTO_APPLY: &str = "auto_apply";
    /// Последняя использованная стратегия
    pub const LAST_STRATEGY: &str = "last_strategy";
    /// Тема оформления
    pub const THEME: &str = "theme";
    /// Язык интерфейса
    pub const LANGUAGE: &str = "language";
    /// Показывать уведомления
    pub const NOTIFICATIONS: &str = "notifications";
    /// Минимизировать в трей
    pub const MINIMIZE_TO_TRAY: &str = "minimize_to_tray";
    /// Блокировать QUIC
    pub const BLOCK_QUIC: &str = "block_quic";
    /// Режим по умолчанию
    pub const DEFAULT_MODE: &str = "default_mode";
    /// Настройки приложения (объект)
    pub const APP_SETTINGS: &str = "app_settings";
    /// Префикс для состояния сервисов
    pub const SERVICE_ENABLED_PREFIX: &str = "service_enabled:";
}
