//! Strategy Managers - изолированные менеджеры для управления стратегиями
//!
//! Этот модуль содержит менеджеры для:
//! - `BlockedStrategiesManager` - чёрный список стратегий (default + user)
//! - `LockedStrategiesManager` - зафиксированные рабочие стратегии по протоколам
//! - `StrategyHistoryManager` - история успехов/неудач стратегий по доменам
//! - `StrategyCacheManager` - кэш оптимальных стратегий по env_key
//!
//! # Архитектура
//! Менеджеры используют SQLite для persistence и RwLock для thread-safe доступа.
//! Дефолтные блокировки (для заблокированных РКН сайтов) не могут быть удалены через UI.

mod blocked;
mod cache;
mod history;
mod locked;

pub use blocked::BlockedStrategiesManager;
pub use cache::StrategyCacheManager;
#[allow(unused_imports)]
pub use cache::CachedStrategy;
pub use history::{StrategyHistoryManager, StrategyStats};
pub use locked::{LockedStrategiesManager, Protocol};
