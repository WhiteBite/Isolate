import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Telemetry API Functions
// ============================================================================

/** Enable or disable telemetry (opt-in) */
export async function setTelemetryEnabled(enabled: boolean): Promise<void> {
    return invoke('set_telemetry_enabled', { enabled });
}

/** Check if telemetry is enabled */
export async function isTelemetryEnabled(): Promise<boolean> {
    return invoke('is_telemetry_enabled');
}

/** Get number of pending telemetry events */
export async function getTelemetryPendingCount(): Promise<number> {
    return invoke('get_telemetry_pending_count');
}

/** Manually flush telemetry events */
export async function flushTelemetry(): Promise<void> {
    return invoke('flush_telemetry');
}

/** Clear pending telemetry events without sending */
export async function clearTelemetry(): Promise<void> {
    return invoke('clear_telemetry');
}

/** Report optimization result to telemetry */
export async function reportOptimizationTelemetry(
    strategyId: string,
    score: number,
    success: boolean
): Promise<void> {
    return invoke('report_optimization_telemetry', { strategyId, score, success });
}

/** Report strategy usage to telemetry */
export async function reportStrategyUsageTelemetry(
    strategyId: string,
    durationSecs: number
): Promise<void> {
    return invoke('report_strategy_usage_telemetry', { strategyId, durationSecs });
}
