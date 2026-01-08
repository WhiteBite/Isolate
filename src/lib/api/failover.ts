import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Failover Types
// ============================================================================

/**
 * Status of the failover system.
 */
export type FailoverState = 'normal' | 'degraded' | 'on_backup';

/**
 * Configuration for auto-recovery/failover.
 */
export interface FailoverConfig {
    /** Whether auto-recovery is enabled */
    enabled: boolean;
    /** Primary strategy ID */
    primary_strategy_id: string | null;
    /** Backup strategy IDs in priority order */
    backup_strategy_ids: string[];
    /** Number of failures before switching to backup */
    max_failures: number;
    /** Cooldown in seconds before attempting to restore primary */
    cooldown_secs: number;
}

/**
 * Current status of the failover system.
 */
export interface FailoverStatus {
    /** Current failover state */
    state: FailoverState;
    /** Currently active strategy ID */
    active_strategy_id: string | null;
    /** Whether active strategy is primary or backup */
    is_primary: boolean;
    /** Current failure count */
    failure_count: number;
    /** Maximum failures before switch */
    max_failures: number;
    /** Seconds until restore attempt (null if not on backup) */
    restore_countdown_secs: number | null;
    /** Timestamp of last failure */
    last_failure_at: string | null;
    /** Timestamp of last successful check */
    last_success_at: string | null;
}

/**
 * Result of a failover operation.
 */
export interface FailoverOperationResult {
    success: boolean;
    message: string;
    new_state?: FailoverState;
    new_strategy_id?: string;
}

/**
 * Strategy info for failover selection.
 */
export interface FailoverStrategyInfo {
    id: string;
    name: string;
    family: string;
    is_available: boolean;
}

// ============================================================================
// Failover API Functions
// ============================================================================

/**
 * Get the current failover configuration.
 * @returns Current failover config
 */
export async function getFailoverConfig(): Promise<FailoverConfig> {
    return invoke('get_failover_config');
}

/**
 * Configure the failover system.
 * @param config - New failover configuration
 * @returns Operation result
 */
export async function configureFailover(config: FailoverConfig): Promise<FailoverOperationResult> {
    return invoke('configure_failover', { config });
}

/**
 * Get the current failover status.
 * @returns Current failover status
 */
export async function getFailoverStatus(): Promise<FailoverStatus> {
    return invoke('get_failover_status');
}

/**
 * Force switch to the next backup strategy.
 * @returns Operation result
 */
export async function forceSwitchToBackup(): Promise<FailoverOperationResult> {
    return invoke('force_switch_to_backup');
}

/**
 * Force restore to primary strategy.
 * @returns Operation result
 */
export async function forceRestorePrimary(): Promise<FailoverOperationResult> {
    return invoke('force_restore_primary');
}

/**
 * Test the failover mechanism without actually switching.
 * @returns Operation result with test details
 */
export async function testFailover(): Promise<FailoverOperationResult> {
    return invoke('test_failover');
}

/**
 * Get available strategies for failover configuration.
 * @returns List of available strategies
 */
export async function getFailoverStrategies(): Promise<FailoverStrategyInfo[]> {
    return invoke('get_failover_strategies');
}

/**
 * Reset failover state (clear failure count, restore to primary).
 * @returns Operation result
 */
export async function resetFailoverState(): Promise<FailoverOperationResult> {
    return invoke('reset_failover_state');
}


// ============================================================================
// Additional Failover Functions (for UI compatibility)
// ============================================================================

/**
 * Enable or disable failover.
 * @param enabled - Whether to enable failover
 */
export async function setFailoverEnabled(enabled: boolean): Promise<void> {
    const config = await getFailoverConfig();
    await configureFailover({ ...config, enabled });
}

/**
 * Update failover configuration.
 * @param config - Partial config to update
 */
export async function setFailoverConfig(config: Partial<FailoverConfig>): Promise<void> {
    const current = await getFailoverConfig();
    await configureFailover({ ...current, ...config });
}

/**
 * Trigger manual failover to backup strategy.
 */
export async function triggerManualFailover(): Promise<FailoverOperationResult> {
    return forceSwitchToBackup();
}

/**
 * Get learned strategies from history.
 */
export async function getLearnedStrategies(): Promise<FailoverStrategyInfo[]> {
    return getFailoverStrategies();
}

/**
 * Check if failover is enabled.
 */
export async function isFailoverEnabled(): Promise<boolean> {
    const config = await getFailoverConfig();
    return config.enabled;
}

/**
 * Get failover progress (for UI).
 */
export async function getFailoverProgress(): Promise<number> {
    const status = await getFailoverStatus();
    if (status.restore_countdown_secs === null) return 100;
    const config = await getFailoverConfig();
    return Math.round((1 - status.restore_countdown_secs / config.cooldown_secs) * 100);
}

/**
 * Format cooldown time for display.
 */
export function formatCooldown(seconds: number | null): string {
    if (seconds === null) return '-';
    if (seconds < 60) return `${seconds}s`;
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}m ${secs}s`;
}

/**
 * Get status color for UI.
 */
export function getFailoverStatusColor(state: FailoverState): string {
    switch (state) {
        case 'normal': return 'text-emerald-400';
        case 'degraded': return 'text-amber-400';
        case 'on_backup': return 'text-red-400';
        default: return 'text-zinc-400';
    }
}
