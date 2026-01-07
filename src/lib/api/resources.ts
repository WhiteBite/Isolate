import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Resource Limits Types
// ============================================================================

/**
 * CPU priority levels for winws/sing-box processes.
 */
export type CpuPriority = 'idle' | 'below_normal' | 'normal' | 'above_normal' | 'high' | 'realtime';

/**
 * Resource limits configuration.
 */
export interface ResourceLimits {
    /** Memory limit in MB (0 = unlimited) */
    memory_limit_mb: number;
    /** CPU priority for child processes */
    cpu_priority: CpuPriority;
    /** Maximum number of concurrent connections (0 = unlimited) */
    max_connections: number;
    /** Enable resource monitoring */
    monitoring_enabled: boolean;
}

/**
 * Current resource usage statistics.
 */
export interface ResourceUsage {
    /** Current memory usage in MB */
    memory_mb: number;
    /** Peak memory usage in MB */
    memory_peak_mb: number;
    /** CPU usage percentage (0-100) */
    cpu_percent: number;
    /** Number of active connections */
    active_connections: number;
    /** Number of child processes */
    process_count: number;
    /** Uptime in seconds */
    uptime_secs: number;
}

// ============================================================================
// Resource Limits API Functions
// ============================================================================

/**
 * Get current resource limits configuration.
 * @returns Current resource limits
 */
export async function getResourceLimits(): Promise<ResourceLimits> {
    return invoke('get_resource_limits');
}

/**
 * Save resource limits configuration.
 * @param limits - Limits to apply
 */
export async function saveResourceLimits(limits: ResourceLimits): Promise<void> {
    return invoke('save_resource_limits', { limits });
}

/**
 * Get current resource usage statistics.
 * @returns Current usage stats
 */
export async function getResourceUsage(): Promise<ResourceUsage> {
    return invoke('get_resource_usage');
}

/**
 * Reset resource limits to defaults.
 */
export async function resetResourceLimits(): Promise<void> {
    return invoke('reset_resource_limits');
}

/**
 * Set memory limit for child processes.
 * @param limitMb - Memory limit in MB (0 = unlimited)
 */
export async function setMemoryLimit(limitMb: number): Promise<void> {
    return invoke('set_memory_limit', { limitMb });
}

/**
 * Set CPU priority for child processes.
 * @param priority - CPU priority level
 */
export async function setCpuPriority(priority: CpuPriority): Promise<void> {
    return invoke('set_cpu_priority', { priority });
}

/**
 * Toggle resource monitoring on/off.
 * @param enabled - Whether monitoring should be enabled
 */
export async function toggleResourceMonitoring(enabled: boolean): Promise<void> {
    return invoke('toggle_resource_monitoring', { enabled });
}
