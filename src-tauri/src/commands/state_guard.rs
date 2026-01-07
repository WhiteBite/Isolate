//! State Guard — защита от race condition при инициализации AppState
//!
//! Этот модуль предоставляет макрос и утилиты для безопасного доступа к AppState
//! в Tauri командах. Решает проблему race condition когда фронтенд вызывает
//! команды до завершения асинхронной инициализации AppState.
//!
//! # Проблема
//! 
//! ```text
//! setup() запускает async инициализацию AppState
//!     │
//!     ├─── Frontend загружается параллельно
//!     │         │
//!     │         └─── invoke('get_services') ← FAIL! AppState ещё не готов
//!     │

#![allow(dead_code)] // Public state guard API
//!     └─── AppState готов (через ~300-500ms)
//! ```
//!
//! # Решение
//!
//! Используйте `require_state!` макрос или `try_get_state()` функцию:
//!
//! ```rust,ignore
//! use crate::commands::state_guard::{require_state, StateNotReady};
//!
//! #[tauri::command]
//! pub async fn my_command(app: AppHandle) -> Result<Data, IsolateError> {
//!     let state = require_state!(app);
//!     // Теперь state гарантированно инициализирован
//!     Ok(state.do_something().await)
//! }
//! ```

use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};

use crate::core::errors::IsolateError;
use crate::state::AppState;

/// Ошибка когда AppState ещё не инициализирован
#[derive(Debug, Clone)]
pub struct StateNotReady;

impl std::fmt::Display for StateNotReady {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Backend is not ready yet. Please wait for initialization to complete.")
    }
}

impl std::error::Error for StateNotReady {}

/// Пытается получить AppState из AppHandle
/// 
/// Возвращает `None` если AppState ещё не инициализирован.
/// Это безопасная альтернатива `State<'_, Arc<AppState>>` которая
/// не паникует при отсутствии state.
///
/// # Example
///
/// ```rust,ignore
/// use crate::commands::state_guard::try_get_state;
///
/// #[tauri::command]
/// pub async fn my_command(app: AppHandle) -> Result<Data, IsolateError> {
///     let state = try_get_state(&app)
///         .ok_or_else(|| IsolateError::Tauri("Backend not ready".to_string()))?;
///     Ok(state.do_something().await)
/// }
/// ```
pub fn try_get_state<R: Runtime>(app: &AppHandle<R>) -> Option<Arc<AppState>> {
    app.try_state::<Arc<AppState>>().map(|s| s.inner().clone())
}

/// Проверяет готовность AppState
///
/// Возвращает `true` если AppState инициализирован и готов к использованию.
pub fn is_state_ready<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.try_state::<Arc<AppState>>().is_some()
}

/// Получает AppState или возвращает ошибку IsolateError::Tauri
///
/// # Example
///
/// ```rust,ignore
/// use crate::commands::state_guard::get_state_or_error;
///
/// #[tauri::command]
/// pub async fn my_command(app: AppHandle) -> Result<Data, IsolateError> {
///     let state = get_state_or_error(&app)?;
///     Ok(state.do_something().await)
/// }
/// ```
pub fn get_state_or_error<R: Runtime>(app: &AppHandle<R>) -> Result<Arc<AppState>, IsolateError> {
    try_get_state(app).ok_or_else(|| IsolateError::Tauri(StateNotReady.to_string()))
}

/// Макрос для безопасного получения AppState в командах
///
/// Автоматически возвращает ошибку если state не готов.
/// Требует чтобы функция возвращала `Result<T, IsolateError>`.
///
/// # Example
///
/// ```rust,ignore
/// use crate::commands::state_guard::require_state;
///
/// #[tauri::command]
/// pub async fn get_services(app: AppHandle) -> Result<Vec<Service>, IsolateError> {
///     let state = require_state!(app);
///     // state: Arc<AppState>
///     let services = state.config_manager.load_services().await?;
///     Ok(services.into_values().collect())
/// }
/// ```
#[macro_export]
macro_rules! require_state {
    ($app:expr) => {
        match $crate::commands::state_guard::try_get_state(&$app) {
            Some(state) => state,
            None => return Err($crate::core::errors::IsolateError::Tauri(
                $crate::commands::state_guard::StateNotReady.to_string()
            )),
        }
    };
}

// Re-export макроса на уровне модуля
// Note: require_state macro is exported via #[macro_export] and available at crate root
// This re-export is kept for documentation purposes but may show as unused
#[allow(unused_imports)]
pub use require_state;
