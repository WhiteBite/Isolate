import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Monitor API Functions
// ============================================================================

/** Start strategy health monitoring */
export async function startMonitor(): Promise<void> {
    return invoke('start_monitor');
}

/** Stop strategy health monitoring */
export async function stopMonitor(): Promise<void> {
    return invoke('stop_monitor');
}

/** Check if monitor is running */
export async function isMonitorRunning(): Promise<boolean> {
    return invoke('is_monitor_running');
}

/** Check if strategy is degraded */
export async function isStrategyDegraded(): Promise<boolean> {
    return invoke('is_strategy_degraded');
}

/** Perform manual health check */
export async function checkStrategyHealth(): Promise<boolean> {
    return invoke('check_strategy_health');
}

/** Set monitor test URLs */
export async function setMonitorUrls(urls: string[]): Promise<void> {
    return invoke('set_monitor_urls', { urls });
}

/** Enable/disable auto-restart on degradation */
export async function setMonitorAutoRestart(enabled: boolean): Promise<void> {
    return invoke('set_monitor_auto_restart', { enabled });
}
