import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { OptimizationProgress, OptimizationResult } from './types';

// ============================================================================
// Optimization API Functions (v2 - new automation system)
// ============================================================================

/**
 * Run strategy optimization
 * Uses new StrategyOptimizer from automation module
 */
export async function runOptimization(): Promise<OptimizationResult> {
    return invoke('run_optimization_v2');
}

/**
 * Cancel ongoing optimization
 */
export async function cancelOptimization(): Promise<void> {
    return invoke('cancel_optimization_v2');
}

/**
 * Check if optimization is running
 */
export async function isOptimizationRunning(): Promise<boolean> {
    return invoke('is_optimization_v2_running');
}

// ============================================================================
// Domain Monitor API Functions
// ============================================================================

export interface MonitorConfig {
    lock_threshold?: number;
    unlock_threshold?: number;
    test_timeout?: number;
    cycle_delay?: number;
    domain_delay?: number;
    min_bytes_success?: number;
}

export type DomainStatus = 'testing' | 'locked' | 'failed' | 'unknown';

/**
 * Start domain monitoring
 */
export async function startDomainMonitor(
    domains: string[],
    config?: MonitorConfig
): Promise<void> {
    return invoke('start_domain_monitor', { domains, config });
}

/**
 * Stop domain monitoring
 */
export async function stopDomainMonitor(): Promise<void> {
    return invoke('stop_domain_monitor');
}

/**
 * Check if domain monitor is running
 */
export async function isDomainMonitorRunning(): Promise<boolean> {
    return invoke('is_domain_monitor_running');
}

/**
 * Get status of a specific domain
 */
export async function getDomainStatus(domain: string): Promise<DomainStatus> {
    return invoke('get_domain_status', { domain });
}

/**
 * Get all domain statuses
 */
export async function getAllDomainStatuses(): Promise<Record<string, DomainStatus>> {
    return invoke('get_all_domain_statuses');
}

// ============================================================================
// Strategy Manager API Functions
// ============================================================================

/**
 * Get blocked strategies for a domain
 */
export async function getBlockedStrategies(domain: string): Promise<string[]> {
    return invoke('get_blocked_strategies', { domain });
}

/**
 * Block a strategy for a domain
 */
export async function blockStrategy(domain: string, strategyId: string): Promise<void> {
    return invoke('block_strategy', { domain, strategyId });
}

/**
 * Unblock a strategy for a domain
 */
export async function unblockStrategy(domain: string, strategyId: string): Promise<boolean> {
    return invoke('unblock_strategy', { domain, strategyId });
}

/**
 * Get locked strategy for a domain
 */
export async function getLockedStrategy(domain: string, protocol: string = 'tls'): Promise<string | null> {
    return invoke('get_locked_strategy', { domain, protocol });
}

/**
 * Lock a strategy for a domain
 */
export async function lockStrategy(domain: string, strategyId: string, protocol: string = 'tls'): Promise<void> {
    return invoke('lock_strategy', { domain, strategyId, protocol });
}

/**
 * Unlock a strategy for a domain
 */
export async function unlockStrategy(domain: string, protocol: string = 'tls'): Promise<string | null> {
    return invoke('unlock_strategy', { domain, protocol });
}

/**
 * Invalidate strategy cache
 */
export async function invalidateStrategyCache(envKey?: string): Promise<void> {
    return invoke('invalidate_strategy_cache', { envKey });
}

// ============================================================================
// Event Listeners (new automation events)
// ============================================================================

/**
 * Listen to optimization progress events
 */
export function onOptimizationProgress(
    callback: (progress: OptimizationProgress) => void
): Promise<UnlistenFn> {
    return listen('automation:progress', (event) => {
        callback(event.payload as OptimizationProgress);
    });
}

/**
 * Listen to optimization complete events
 */
export function onOptimizationComplete(
    callback: (result: OptimizationResult) => void
): Promise<UnlistenFn> {
    return listen('automation:complete', (event) => {
        callback(event.payload as OptimizationResult);
    });
}

/**
 * Listen to optimization error events
 */
export function onOptimizationError(
    callback: (error: string) => void
): Promise<UnlistenFn> {
    return listen('automation:error', (event) => {
        callback(event.payload as string);
    });
}

/**
 * Listen to domain locked events
 */
export function onDomainLocked(
    callback: (data: { domain: string; strategy_id: string; protocol: string }) => void
): Promise<UnlistenFn> {
    return listen('automation:domain_locked', (event) => {
        callback(event.payload as { domain: string; strategy_id: string; protocol: string });
    });
}

/**
 * Listen to domain unlocked events
 */
export function onDomainUnlocked(
    callback: (data: { domain: string; protocol: string }) => void
): Promise<UnlistenFn> {
    return listen('automation:domain_unlocked', (event) => {
        callback(event.payload as { domain: string; protocol: string });
    });
}

/**
 * Listen to monitor started events
 */
export function onMonitorStarted(
    callback: (data: { domains: string[] }) => void
): Promise<UnlistenFn> {
    return listen('automation:monitor_started', (event) => {
        callback(event.payload as { domains: string[] });
    });
}

/**
 * Listen to monitor stopped events
 */
export function onMonitorStopped(callback: () => void): Promise<UnlistenFn> {
    return listen('automation:monitor_stopped', () => {
        callback();
    });
}

// Legacy aliases for backward compatibility (deprecated)
/** @deprecated Use onOptimizationError instead */
export const onOptimizationFailed = onOptimizationError;
