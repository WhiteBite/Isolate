//! Automation events - события и типы для модуля автоматизации
//!
//! Содержит:
//! - `OptimizationStage` - этапы процесса оптимизации
//! - `OptimizationProgress` - прогресс оптимизации для UI
//! - `OptimizationResult` - результат оптимизации
//! - `DomainStatus` - статус домена в мониторинге
//! - `AutomationEvent` - события для UI

use serde::{Deserialize, Serialize};

use crate::core::testing::StrategyScore;

// ============================================================================
// Optimization Events
// ============================================================================

/// Этап оптимизации
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationStage {
    #[default]
    /// Инициализация
    Initializing,
    /// Проверка кэша
    CheckingCache,
    /// DPI-диагностика
    Diagnosing,
    /// Выбор кандидатов
    SelectingCandidates,
    /// Тестирование VLESS стратегий
    TestingVless,
    /// Тестирование Zapret стратегий
    TestingZapret,
    /// Выбор лучшей стратегии
    SelectingBest,
    /// Применение стратегии
    Applying,
    /// Завершено успешно
    Completed,
    /// Ошибка
    Failed,
    /// Отменено
    Cancelled,
}

impl std::fmt::Display for OptimizationStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationStage::Initializing => write!(f, "Инициализация"),
            OptimizationStage::CheckingCache => write!(f, "Проверка кэша"),
            OptimizationStage::Diagnosing => write!(f, "DPI-диагностика"),
            OptimizationStage::SelectingCandidates => write!(f, "Выбор кандидатов"),
            OptimizationStage::TestingVless => write!(f, "Тестирование VLESS"),
            OptimizationStage::TestingZapret => write!(f, "Тестирование Zapret"),
            OptimizationStage::SelectingBest => write!(f, "Выбор лучшей"),
            OptimizationStage::Applying => write!(f, "Применение"),
            OptimizationStage::Completed => write!(f, "Завершено"),
            OptimizationStage::Failed => write!(f, "Ошибка"),
            OptimizationStage::Cancelled => write!(f, "Отменено"),
        }
    }
}

/// Прогресс оптимизации
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationProgress {
    /// Текущий этап
    pub stage: OptimizationStage,
    /// Процент выполнения (0-100)
    pub percent: u8,
    /// Текстовое сообщение
    pub message: String,
    /// Текущая тестируемая стратегия
    pub current_strategy: Option<String>,
    /// Количество протестированных стратегий
    pub tested_count: u32,
    /// Общее количество стратегий для тестирования
    pub total_count: u32,
    /// Лучший текущий score
    pub best_score: Option<f64>,
}

impl OptimizationProgress {
    /// Создаёт новый прогресс
    pub fn new(stage: OptimizationStage, percent: u8, message: impl Into<String>) -> Self {
        Self {
            stage,
            percent,
            message: message.into(),
            current_strategy: None,
            tested_count: 0,
            total_count: 0,
            best_score: None,
        }
    }

    /// Устанавливает текущую стратегию
    pub fn with_strategy(mut self, strategy: &str) -> Self {
        self.current_strategy = Some(strategy.to_string());
        self
    }

    /// Устанавливает счётчики
    pub fn with_counts(mut self, tested: u32, total: u32) -> Self {
        self.tested_count = tested;
        self.total_count = total;
        self
    }

    /// Устанавливает лучший score
    pub fn with_score(mut self, score: f64) -> Self {
        self.best_score = Some(score);
        self
    }

    /// Проверяет, завершён ли процесс
    pub fn is_finished(&self) -> bool {
        matches!(
            self.stage,
            OptimizationStage::Completed
                | OptimizationStage::Failed
                | OptimizationStage::Cancelled
        )
    }

    /// Проверяет, успешно ли завершён процесс
    pub fn is_success(&self) -> bool {
        self.stage == OptimizationStage::Completed
    }
}

/// Результат оптимизации
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationResult {
    /// Выбранная стратегия
    pub strategy_id: String,
    /// Название стратегии
    pub strategy_name: String,
    /// Score стратегии
    pub score: f64,
    /// Была ли стратегия из кэша
    pub from_cache: bool,
    /// Результаты всех тестов
    pub all_scores: Vec<StrategyScore>,
}

impl OptimizationResult {
    /// Создаёт результат из кэша
    pub fn from_cache(strategy_id: String, strategy_name: String, score: f64) -> Self {
        Self {
            strategy_id,
            strategy_name,
            score,
            from_cache: true,
            all_scores: vec![],
        }
    }

    /// Создаёт результат из тестирования
    pub fn from_testing(
        strategy_id: String,
        strategy_name: String,
        score: f64,
        all_scores: Vec<StrategyScore>,
    ) -> Self {
        Self {
            strategy_id,
            strategy_name,
            score,
            from_cache: false,
            all_scores,
        }
    }
}

// ============================================================================
// Monitor Events
// ============================================================================

/// Статус домена в мониторинге
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DomainStatus {
    /// Сейчас тестируется
    Testing,
    /// Зафиксирована рабочая стратегия
    Locked,
    /// Не работает (много неудач)
    Failed,
    /// Ещё не тестировалась
    #[default]
    Unknown,
}

impl std::fmt::Display for DomainStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainStatus::Testing => write!(f, "Тестирование"),
            DomainStatus::Locked => write!(f, "Зафиксирован"),
            DomainStatus::Failed => write!(f, "Не работает"),
            DomainStatus::Unknown => write!(f, "Неизвестно"),
        }
    }
}

/// Событие автоматизации для UI
#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationEvent {
    /// Прогресс оптимизации
    OptimizationProgress(OptimizationProgress),

    /// Домен зафиксирован с рабочей стратегией
    DomainLocked {
        domain: String,
        strategy_id: String,
        protocol: String,
    },

    /// Домен разблокирован (стратегия перестала работать)
    DomainUnlocked {
        domain: String,
        protocol: String,
    },

    /// Стратегия заблокирована для домена
    StrategyBlocked {
        domain: String,
        strategy_id: String,
        reason: String,
    },

    /// Мониторинг запущен
    MonitorStarted {
        domains: Vec<String>,
    },

    /// Мониторинг остановлен
    MonitorStopped,
}

impl std::fmt::Debug for AutomationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutomationEvent::OptimizationProgress(p) => {
                write!(f, "OptimizationProgress({:?}, {}%)", p.stage, p.percent)
            }
            AutomationEvent::DomainLocked {
                domain,
                strategy_id,
                ..
            } => {
                write!(f, "DomainLocked({} -> {})", domain, strategy_id)
            }
            AutomationEvent::DomainUnlocked { domain, .. } => {
                write!(f, "DomainUnlocked({})", domain)
            }
            AutomationEvent::StrategyBlocked {
                domain,
                strategy_id,
                ..
            } => {
                write!(f, "StrategyBlocked({}, {})", domain, strategy_id)
            }
            AutomationEvent::MonitorStarted { domains } => {
                write!(f, "MonitorStarted({} domains)", domains.len())
            }
            AutomationEvent::MonitorStopped => write!(f, "MonitorStopped"),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_stage_default() {
        let stage: OptimizationStage = Default::default();
        assert_eq!(stage, OptimizationStage::Initializing);
    }

    #[test]
    fn test_optimization_stage_display() {
        assert_eq!(OptimizationStage::Initializing.to_string(), "Инициализация");
        assert_eq!(OptimizationStage::Completed.to_string(), "Завершено");
        assert_eq!(OptimizationStage::Failed.to_string(), "Ошибка");
    }

    #[test]
    fn test_optimization_stage_serialization() {
        let stages = vec![
            (OptimizationStage::Initializing, "\"initializing\""),
            (OptimizationStage::CheckingCache, "\"checking_cache\""),
            (OptimizationStage::Diagnosing, "\"diagnosing\""),
            (OptimizationStage::SelectingCandidates, "\"selecting_candidates\""),
            (OptimizationStage::TestingVless, "\"testing_vless\""),
            (OptimizationStage::TestingZapret, "\"testing_zapret\""),
            (OptimizationStage::SelectingBest, "\"selecting_best\""),
            (OptimizationStage::Applying, "\"applying\""),
            (OptimizationStage::Completed, "\"completed\""),
            (OptimizationStage::Failed, "\"failed\""),
            (OptimizationStage::Cancelled, "\"cancelled\""),
        ];

        for (stage, expected_json) in stages {
            let json = serde_json::to_string(&stage).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: OptimizationStage = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, stage);
        }
    }

    #[test]
    fn test_optimization_progress_new() {
        let progress = OptimizationProgress::new(
            OptimizationStage::TestingVless,
            50,
            "Тестируем VLESS стратегии...",
        );

        assert_eq!(progress.stage, OptimizationStage::TestingVless);
        assert_eq!(progress.percent, 50);
        assert_eq!(progress.message, "Тестируем VLESS стратегии...");
        assert!(progress.current_strategy.is_none());
        assert_eq!(progress.tested_count, 0);
        assert_eq!(progress.total_count, 0);
        assert!(progress.best_score.is_none());
    }

    #[test]
    fn test_optimization_progress_builder() {
        let progress = OptimizationProgress::new(OptimizationStage::TestingZapret, 75, "Testing")
            .with_strategy("zapret-1")
            .with_counts(5, 10)
            .with_score(0.85);

        assert_eq!(progress.current_strategy, Some("zapret-1".to_string()));
        assert_eq!(progress.tested_count, 5);
        assert_eq!(progress.total_count, 10);
        assert_eq!(progress.best_score, Some(0.85));
    }

    #[test]
    fn test_optimization_progress_is_finished() {
        assert!(!OptimizationProgress::new(OptimizationStage::Initializing, 0, "").is_finished());
        assert!(!OptimizationProgress::new(OptimizationStage::TestingVless, 50, "").is_finished());
        assert!(OptimizationProgress::new(OptimizationStage::Completed, 100, "").is_finished());
        assert!(OptimizationProgress::new(OptimizationStage::Failed, 100, "").is_finished());
        assert!(OptimizationProgress::new(OptimizationStage::Cancelled, 0, "").is_finished());
    }

    #[test]
    fn test_optimization_progress_is_success() {
        assert!(OptimizationProgress::new(OptimizationStage::Completed, 100, "").is_success());
        assert!(!OptimizationProgress::new(OptimizationStage::Failed, 100, "").is_success());
        assert!(!OptimizationProgress::new(OptimizationStage::Cancelled, 0, "").is_success());
    }

    #[test]
    fn test_optimization_progress_serialization() {
        let progress = OptimizationProgress::new(OptimizationStage::TestingVless, 50, "Testing")
            .with_strategy("vless-1")
            .with_counts(3, 10)
            .with_score(0.9);

        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("\"stage\":\"testing_vless\""));
        assert!(json.contains("\"percent\":50"));
        assert!(json.contains("\"currentStrategy\":\"vless-1\""));
        assert!(json.contains("\"testedCount\":3"));
        assert!(json.contains("\"totalCount\":10"));
        assert!(json.contains("\"bestScore\":0.9"));

        let deserialized: OptimizationProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.stage, progress.stage);
        assert_eq!(deserialized.percent, progress.percent);
    }

    #[test]
    fn test_optimization_result_from_cache() {
        let result = OptimizationResult::from_cache(
            "strategy-1".to_string(),
            "Strategy 1".to_string(),
            0.95,
        );

        assert_eq!(result.strategy_id, "strategy-1");
        assert_eq!(result.strategy_name, "Strategy 1");
        assert_eq!(result.score, 0.95);
        assert!(result.from_cache);
        assert!(result.all_scores.is_empty());
    }

    #[test]
    fn test_optimization_result_from_testing() {
        let scores = vec![StrategyScore {
            strategy_id: "strategy-1".to_string(),
            score: 0.9,
            success_rate: 1.0,
            critical_success_rate: 1.0,
            latency_avg: 100.0,
            latency_jitter: 10.0,
        }];

        let result = OptimizationResult::from_testing(
            "strategy-1".to_string(),
            "Strategy 1".to_string(),
            0.9,
            scores,
        );

        assert!(!result.from_cache);
        assert_eq!(result.all_scores.len(), 1);
    }

    #[test]
    fn test_domain_status_default() {
        let status: DomainStatus = Default::default();
        assert_eq!(status, DomainStatus::Unknown);
    }

    #[test]
    fn test_domain_status_display() {
        assert_eq!(DomainStatus::Testing.to_string(), "Тестирование");
        assert_eq!(DomainStatus::Locked.to_string(), "Зафиксирован");
        assert_eq!(DomainStatus::Failed.to_string(), "Не работает");
        assert_eq!(DomainStatus::Unknown.to_string(), "Неизвестно");
    }

    #[test]
    fn test_domain_status_serialization() {
        let statuses = vec![
            (DomainStatus::Testing, "\"testing\""),
            (DomainStatus::Locked, "\"locked\""),
            (DomainStatus::Failed, "\"failed\""),
            (DomainStatus::Unknown, "\"unknown\""),
        ];

        for (status, expected_json) in statuses {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: DomainStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, status);
        }
    }

    #[test]
    fn test_automation_event_serialization() {
        // OptimizationProgress event
        let progress_event = AutomationEvent::OptimizationProgress(OptimizationProgress::new(
            OptimizationStage::TestingVless,
            50,
            "Testing",
        ));
        let json = serde_json::to_string(&progress_event).unwrap();
        assert!(json.contains("\"type\":\"optimization_progress\""));

        // DomainLocked event
        let locked_event = AutomationEvent::DomainLocked {
            domain: "youtube.com".to_string(),
            strategy_id: "vless-1".to_string(),
            protocol: "tls".to_string(),
        };
        let json = serde_json::to_string(&locked_event).unwrap();
        assert!(json.contains("\"type\":\"domain_locked\""));
        assert!(json.contains("\"domain\":\"youtube.com\""));

        // DomainUnlocked event
        let unlocked_event = AutomationEvent::DomainUnlocked {
            domain: "discord.com".to_string(),
            protocol: "tls".to_string(),
        };
        let json = serde_json::to_string(&unlocked_event).unwrap();
        assert!(json.contains("\"type\":\"domain_unlocked\""));

        // StrategyBlocked event
        let blocked_event = AutomationEvent::StrategyBlocked {
            domain: "twitter.com".to_string(),
            strategy_id: "zapret-1".to_string(),
            reason: "Too many failures".to_string(),
        };
        let json = serde_json::to_string(&blocked_event).unwrap();
        assert!(json.contains("\"type\":\"strategy_blocked\""));

        // MonitorStarted event
        let started_event = AutomationEvent::MonitorStarted {
            domains: vec!["youtube.com".to_string(), "discord.com".to_string()],
        };
        let json = serde_json::to_string(&started_event).unwrap();
        assert!(json.contains("\"type\":\"monitor_started\""));

        // MonitorStopped event
        let stopped_event = AutomationEvent::MonitorStopped;
        let json = serde_json::to_string(&stopped_event).unwrap();
        assert!(json.contains("\"type\":\"monitor_stopped\""));
    }

    #[test]
    fn test_automation_event_debug() {
        let event = AutomationEvent::DomainLocked {
            domain: "youtube.com".to_string(),
            strategy_id: "vless-1".to_string(),
            protocol: "tls".to_string(),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("DomainLocked"));
        assert!(debug_str.contains("youtube.com"));
    }
}
