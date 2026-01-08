//! AI Pilot commands
//!
//! Commands for AI Pilot - background optimization system that automatically
//! monitors services and switches strategies when needed.
//!
//! Events emitted:
//! - `ai_pilot:started` - when AI Pilot starts
//! - `ai_pilot:stopped` - when AI Pilot stops
//! - `ai_pilot:action` - when AI Pilot performs an action

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

use crate::commands::state_guard::get_state_or_error;
use crate::core::errors::IsolateError;

// ============================================================================
// Data Structures
// ============================================================================

/// Current status of AI Pilot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPilotStatus {
    /// Whether AI Pilot is currently running
    pub is_running: bool,
    /// Unix timestamp when AI Pilot was started (milliseconds)
    pub started_at: Option<i64>,
    /// Total number of health checks performed
    pub checks_count: u32,
    /// Total number of actions taken
    pub actions_count: u32,
    /// Unix timestamp of last health check (milliseconds)
    pub last_check_at: Option<i64>,
}

/// A single action performed by AI Pilot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPilotAction {
    /// Unique identifier for this action
    pub id: String,
    /// Type of action: "switch_strategy" | "restart" | "fallback"
    pub action_type: String,
    /// Service ID that was affected
    pub service_id: String,
    /// Previous strategy ID (for switch_strategy)
    pub from_strategy: Option<String>,
    /// New strategy ID (for switch_strategy)
    pub to_strategy: Option<String>,
    /// Human-readable reason for the action
    pub reason: String,
    /// Unix timestamp when action was performed (milliseconds)
    pub timestamp: i64,
    /// Whether the action was successful
    pub success: bool,
}

/// History of AI Pilot actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPilotHistory {
    /// List of actions (most recent first)
    pub actions: Vec<AiPilotAction>,
    /// Total count of all actions (may be more than returned)
    pub total_count: u32,
}

/// Event payload for ai_pilot:started
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiPilotStartedEvent {
    started_at: i64,
}

/// Event payload for ai_pilot:stopped
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiPilotStoppedEvent {
    stopped_at: i64,
    total_checks: u32,
    total_actions: u32,
}

/// Event payload for ai_pilot:action
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiPilotActionEvent {
    action: AiPilotAction,
}

// ============================================================================
// Mock State (temporary until real implementation)
// ============================================================================

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::OnceLock;

/// Mock state for AI Pilot (will be replaced with real implementation)
struct MockAiPilotState {
    is_running: AtomicBool,
    started_at: std::sync::RwLock<Option<i64>>,
    checks_count: AtomicU32,
    actions_count: AtomicU32,
    last_check_at: std::sync::RwLock<Option<i64>>,
    actions_history: std::sync::RwLock<Vec<AiPilotAction>>,
}

impl MockAiPilotState {
    fn new() -> Self {
        Self {
            is_running: AtomicBool::new(false),
            started_at: std::sync::RwLock::new(None),
            checks_count: AtomicU32::new(0),
            actions_count: AtomicU32::new(0),
            last_check_at: std::sync::RwLock::new(None),
            actions_history: std::sync::RwLock::new(Vec::new()),
        }
    }
}

static MOCK_STATE: OnceLock<MockAiPilotState> = OnceLock::new();

fn get_mock_state() -> &'static MockAiPilotState {
    MOCK_STATE.get_or_init(MockAiPilotState::new)
}

fn current_timestamp_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Start AI Pilot background optimization
///
/// AI Pilot will periodically check service health and automatically
/// switch strategies when degradation is detected.
///
/// Emits: `ai_pilot:started`
#[tauri::command]
pub async fn start_ai_pilot(app: AppHandle) -> Result<AiPilotStatus, IsolateError> {
    // Verify backend is ready
    let _state = get_state_or_error(&app)?;
    
    let mock = get_mock_state();
    
    // Check if already running
    if mock.is_running.load(Ordering::SeqCst) {
        warn!("AI Pilot is already running");
        return Err(IsolateError::Strategy("AI Pilot is already running".into()));
    }
    
    let now = current_timestamp_ms();
    
    // Update state
    mock.is_running.store(true, Ordering::SeqCst);
    *mock.started_at.write().unwrap() = Some(now);
    mock.checks_count.store(0, Ordering::SeqCst);
    mock.actions_count.store(0, Ordering::SeqCst);
    *mock.last_check_at.write().unwrap() = None;
    
    info!("AI Pilot started");
    
    // Emit started event
    let _ = app.emit("ai_pilot:started", AiPilotStartedEvent { started_at: now });
    
    Ok(AiPilotStatus {
        is_running: true,
        started_at: Some(now),
        checks_count: 0,
        actions_count: 0,
        last_check_at: None,
    })
}

/// Stop AI Pilot background optimization
///
/// Emits: `ai_pilot:stopped`
#[tauri::command]
pub async fn stop_ai_pilot(app: AppHandle) -> Result<AiPilotStatus, IsolateError> {
    // Verify backend is ready
    let _state = get_state_or_error(&app)?;
    
    let mock = get_mock_state();
    
    // Check if running
    if !mock.is_running.load(Ordering::SeqCst) {
        warn!("AI Pilot is not running");
        return Err(IsolateError::Strategy("AI Pilot is not running".into()));
    }
    
    let checks = mock.checks_count.load(Ordering::SeqCst);
    let actions = mock.actions_count.load(Ordering::SeqCst);
    let now = current_timestamp_ms();
    
    // Update state
    mock.is_running.store(false, Ordering::SeqCst);
    
    info!(checks_count = checks, actions_count = actions, "AI Pilot stopped");
    
    // Emit stopped event
    let _ = app.emit("ai_pilot:stopped", AiPilotStoppedEvent {
        stopped_at: now,
        total_checks: checks,
        total_actions: actions,
    });
    
    Ok(AiPilotStatus {
        is_running: false,
        started_at: *mock.started_at.read().unwrap(),
        checks_count: checks,
        actions_count: actions,
        last_check_at: *mock.last_check_at.read().unwrap(),
    })
}

/// Get current AI Pilot status
#[tauri::command]
pub async fn get_ai_pilot_status(app: AppHandle) -> Result<AiPilotStatus, IsolateError> {
    // Verify backend is ready
    let _state = get_state_or_error(&app)?;
    
    let mock = get_mock_state();
    
    Ok(AiPilotStatus {
        is_running: mock.is_running.load(Ordering::SeqCst),
        started_at: *mock.started_at.read().unwrap(),
        checks_count: mock.checks_count.load(Ordering::SeqCst),
        actions_count: mock.actions_count.load(Ordering::SeqCst),
        last_check_at: *mock.last_check_at.read().unwrap(),
    })
}

/// Get AI Pilot action history
///
/// Returns the most recent actions, limited by the `limit` parameter.
#[tauri::command]
pub async fn get_ai_pilot_history(
    app: AppHandle,
    limit: Option<u32>,
) -> Result<AiPilotHistory, IsolateError> {
    // Verify backend is ready
    let _state = get_state_or_error(&app)?;
    
    let mock = get_mock_state();
    let history = mock.actions_history.read().unwrap();
    
    let limit = limit.unwrap_or(50) as usize;
    let total_count = history.len() as u32;
    
    // Return most recent actions first
    let actions: Vec<AiPilotAction> = history
        .iter()
        .rev()
        .take(limit)
        .cloned()
        .collect();
    
    Ok(AiPilotHistory {
        actions,
        total_count,
    })
}

/// Undo a specific AI Pilot action
///
/// Attempts to revert the action by switching back to the previous strategy.
/// Only works for "switch_strategy" actions.
///
/// Emits: `ai_pilot:action` (with undo action)
#[tauri::command]
pub async fn undo_ai_pilot_action(
    app: AppHandle,
    action_id: String,
) -> Result<bool, IsolateError> {
    // Verify backend is ready
    let _state = get_state_or_error(&app)?;
    
    let mock = get_mock_state();
    
    // Find the action
    let action = {
        let history = mock.actions_history.read().unwrap();
        history.iter().find(|a| a.id == action_id).cloned()
    };
    
    let Some(action) = action else {
        warn!(action_id = %action_id, "Action not found");
        return Err(IsolateError::Validation(format!("Action {} not found", action_id)));
    };
    
    // Only switch_strategy can be undone
    if action.action_type != "switch_strategy" {
        warn!(action_type = %action.action_type, "Cannot undo this action type");
        return Err(IsolateError::Validation(
            format!("Cannot undo action type: {}", action.action_type)
        ));
    }
    
    // Check if there's a previous strategy to revert to
    let Some(from_strategy) = &action.from_strategy else {
        warn!("No previous strategy to revert to");
        return Err(IsolateError::Validation("No previous strategy to revert to".into()));
    };
    
    info!(
        action_id = %action_id,
        from = ?action.to_strategy,
        to = %from_strategy,
        "Undoing AI Pilot action"
    );
    
    // Create undo action
    let undo_action = AiPilotAction {
        id: format!("undo-{}", action_id),
        action_type: "switch_strategy".to_string(),
        service_id: action.service_id.clone(),
        from_strategy: action.to_strategy.clone(),
        to_strategy: Some(from_strategy.clone()),
        reason: format!("Undo action {}", action_id),
        timestamp: current_timestamp_ms(),
        success: true, // Mock always succeeds
    };
    
    // Add to history
    {
        let mut history = mock.actions_history.write().unwrap();
        history.push(undo_action.clone());
    }
    mock.actions_count.fetch_add(1, Ordering::SeqCst);
    
    // Emit action event
    let _ = app.emit("ai_pilot:action", AiPilotActionEvent { action: undo_action });
    
    Ok(true)
}

// ============================================================================
// Internal helpers (for future real implementation)
// ============================================================================

/// Record an action (called internally by AI Pilot engine)
#[allow(dead_code)]
pub(crate) fn record_action(app: &AppHandle, action: AiPilotAction) {
    let mock = get_mock_state();
    
    // Add to history
    {
        let mut history = mock.actions_history.write().unwrap();
        history.push(action.clone());
    }
    mock.actions_count.fetch_add(1, Ordering::SeqCst);
    
    // Emit event
    let _ = app.emit("ai_pilot:action", AiPilotActionEvent { action });
}

/// Record a health check (called internally by AI Pilot engine)
#[allow(dead_code)]
pub(crate) fn record_check() {
    let mock = get_mock_state();
    mock.checks_count.fetch_add(1, Ordering::SeqCst);
    *mock.last_check_at.write().unwrap() = Some(current_timestamp_ms());
}
