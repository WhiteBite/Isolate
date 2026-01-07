//! Automation module - оркестрация и мониторинг стратегий
//!
//! Этот модуль содержит компоненты для автоматизации работы со стратегиями:
//!
//! - [`events`] - События и типы для UI (OptimizationProgress, AutomationEvent)
//! - [`optimizer`] - Одноразовая оптимизация стратегий (StrategyOptimizer)
//! - [`monitor`] - Непрерывный мониторинг доменов (DomainMonitor)
//!
//! # Архитектура
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Automation Module                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  ┌──────────────────┐       ┌──────────────────┐            │
//! │  │ StrategyOptimizer│       │  DomainMonitor   │            │
//! │  │                  │       │                  │            │
//! │  │ - optimize()     │       │ - start()        │            │
//! │  │ - cancel()       │       │ - stop()         │            │
//! │  │ - subscribe()    │       │ - subscribe()    │            │
//! │  └────────┬─────────┘       └────────┬─────────┘            │
//! │           │                          │                       │
//! │           ▼                          ▼                       │
//! │  ┌──────────────────────────────────────────────┐           │
//! │  │              Events (broadcast)               │           │
//! │  │  - OptimizationProgress                       │           │
//! │  │  - AutomationEvent                            │           │
//! │  └──────────────────────────────────────────────┘           │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Dependencies                            │
//! │  - managers/ (Cache, Blocked, Locked, History)              │
//! │  - testing/ (HttpProber, ScoreCalculator, EndpointRegistry) │
//! │  - strategy_engine (SharedStrategyEngine)                   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Использование
//!
//! ## Одноразовая оптимизация
//!
//! ```rust,ignore
//! use crate::core::automation::{StrategyOptimizer, OptimizationProgress};
//!
//! let optimizer = StrategyOptimizer::new(engine, cache, blocked, history);
//!
//! // Подписка на прогресс
//! let mut rx = optimizer.subscribe();
//! tokio::spawn(async move {
//!     while let Ok(progress) = rx.recv().await {
//!         println!("Progress: {:?}", progress);
//!     }
//! });
//!
//! // Запуск оптимизации
//! let result = optimizer.optimize(&env_info, &strategies, &services).await?;
//! println!("Best strategy: {}", result.strategy_name);
//! ```
//!
//! ## Непрерывный мониторинг
//!
//! ```rust,ignore
//! use crate::core::automation::{DomainMonitor, MonitorConfig, AutomationEvent};
//!
//! let config = MonitorConfig::with_thresholds(3, 2);
//! let monitor = DomainMonitor::new(locked, blocked, history, config);
//!
//! // Подписка на события
//! let mut rx = monitor.subscribe();
//! tokio::spawn(async move {
//!     while let Ok(event) = rx.recv().await {
//!         match event {
//!             AutomationEvent::DomainLocked { domain, strategy_id, .. } => {
//!                 println!("🔒 {} locked with {}", domain, strategy_id);
//!             }
//!             _ => {}
//!         }
//!     }
//! });
//!
//! // Запуск мониторинга
//! let domains = vec!["youtube.com".to_string(), "discord.com".to_string()];
//! monitor.start(&domains, &strategies).await?;
//! ```

pub mod events;
pub mod monitor;
pub mod optimizer;

// Re-export main types
// Note: Some types are exported for public API but may not be used internally yet
pub use events::{AutomationEvent, DomainStatus, OptimizationResult};
#[allow(unused_imports)]
pub use events::{OptimizationProgress, OptimizationStage};
pub use monitor::{DomainMonitor, MonitorConfig};
#[allow(unused_imports)]
pub use monitor::{create_monitor, SharedDomainMonitor};
pub use optimizer::StrategyOptimizer;
#[allow(unused_imports)]
pub use optimizer::{create_optimizer, SharedStrategyOptimizer};
