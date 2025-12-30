import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// Types
export interface AppStatus {
    is_active: boolean;
    current_strategy: string | null;
    services_status: Record<string, ServiceStatus>;
}

export interface ServiceStatus {
    name: string;
    is_available: boolean;
    latency_ms: number | null;
}

export interface Strategy {
    id: string;
    name: string;
    description: string;
    family: string;
    engine: string;
}

export interface Service {
    id: string;
    name: string;
    critical: boolean;
}

export interface OptimizationProgress {
    stage: string;
    percent: number;
    message: string;
    current_strategy: string | null;
    tested_count: number;
    total_count: number;
    best_score: number | null;
}

export interface DiagnosticResult {
    profile: {
        kind: string;
        details: string | null;
        candidate_families: string[];
    };
    tested_services: string[];
    blocked_services: string[];
}

// API Functions
export async function getStatus(): Promise<AppStatus> {
    return invoke('get_status');
}

export async function getStrategies(): Promise<Strategy[]> {
    return invoke('get_strategies');
}

export async function getServices(): Promise<Service[]> {
    return invoke('get_services');
}

export async function runOptimization(mode: 'turbo' | 'deep'): Promise<string> {
    return invoke('run_optimization', { mode });
}

export async function cancelOptimization(): Promise<void> {
    return invoke('cancel_optimization');
}

export async function applyStrategy(strategyId: string): Promise<void> {
    return invoke('apply_strategy', { strategyId });
}

export async function stopStrategy(): Promise<void> {
    return invoke('stop_strategy');
}

export async function diagnose(): Promise<DiagnosticResult> {
    return invoke('diagnose');
}

export async function panicReset(): Promise<void> {
    return invoke('panic_reset');
}

// Event Listeners
export function onOptimizationProgress(
    callback: (progress: OptimizationProgress) => void
): Promise<UnlistenFn> {
    return listen('optimization:progress', (event) => {
        callback(event.payload as OptimizationProgress);
    });
}

export function onOptimizationComplete(
    callback: (result: { strategy_id: string; score: number }) => void
): Promise<UnlistenFn> {
    return listen('optimization:complete', (event) => {
        callback(event.payload as { strategy_id: string; score: number });
    });
}

export function onOptimizationFailed(
    callback: (error: string) => void
): Promise<UnlistenFn> {
    return listen('optimization:failed', (event) => {
        callback(event.payload as string);
    });
}

export function onStrategyDegraded(
    callback: () => void
): Promise<UnlistenFn> {
    return listen('strategy:degraded', () => {
        callback();
    });
}

// VLESS Types
export interface VlessConfig {
    id: string;
    name: string;
    server: string;
    port: number;
    uuid: string;
    flow: string | null;
    security: string;
    sni: string | null;
    active: boolean;
}

// Sing-box instance status
export type SingboxStatus =
    | 'starting'
    | 'running'
    | 'stopping'
    | 'stopped'
    | 'failed'
    | 'health_check_failed';

// Sing-box instance info
export interface SingboxInstance {
    config_id: string;
    config_name: string;
    socks_port: number;
    status: SingboxStatus;
    pid: number | null;
    started_at: number | null;
    last_health_check: number | null;
    health_check_failures: number;
}

// VLESS API Functions
export async function importVless(url: string): Promise<VlessConfig> {
    return invoke('import_vless', { url });
}

export async function getVlessConfigs(): Promise<VlessConfig[]> {
    return invoke('get_vless_configs');
}

export async function deleteVlessConfig(id: string): Promise<void> {
    return invoke('delete_vless_config', { id });
}

export async function toggleVlessConfig(id: string, active: boolean): Promise<void> {
    return invoke('toggle_vless_config', { id, active });
}

// ============================================================================
// VLESS Proxy Control API
// ============================================================================

/**
 * Start VLESS proxy for a specific config.
 * @param configId - The ID of the VLESS config to start
 * @param socksPort - Optional SOCKS port (auto-allocated if not provided)
 * @returns The running instance info
 */
export async function startVlessProxy(
    configId: string,
    socksPort?: number
): Promise<SingboxInstance> {
    return invoke('start_vless_proxy', { configId, socksPort });
}

/**
 * Stop VLESS proxy for a specific config.
 * @param configId - The ID of the VLESS config to stop
 */
export async function stopVlessProxy(configId: string): Promise<void> {
    return invoke('stop_vless_proxy', { configId });
}

/**
 * Stop all running VLESS proxies.
 */
export async function stopAllVlessProxies(): Promise<void> {
    return invoke('stop_all_vless_proxies');
}

/**
 * Get status of a specific VLESS proxy.
 * @param configId - The ID of the VLESS config
 * @returns The instance info or null if not running
 */
export async function getVlessStatus(configId: string): Promise<SingboxInstance | null> {
    return invoke('get_vless_status', { configId });
}

/**
 * Get status of all running VLESS proxies.
 * @returns Array of all running instances
 */
export async function getAllVlessStatus(): Promise<SingboxInstance[]> {
    return invoke('get_all_vless_status');
}

/**
 * Perform health check on a running VLESS proxy.
 * @param configId - The ID of the VLESS config
 * @returns true if healthy
 */
export async function healthCheckVless(configId: string): Promise<boolean> {
    return invoke('health_check_vless', { configId });
}

/**
 * Test VLESS proxy connectivity by making a test request.
 * @param configId - The ID of the VLESS config
 * @param testUrl - Optional URL to test (defaults to google.com)
 * @returns Latency in milliseconds
 */
export async function testVlessConnectivity(
    configId: string,
    testUrl?: string
): Promise<number> {
    return invoke('test_vless_connectivity', { configId, testUrl });
}

/**
 * Check if sing-box binary is available.
 * @returns true if sing-box is installed
 */
export async function isSingboxAvailable(): Promise<boolean> {
    return invoke('is_singbox_available');
}

/**
 * Get sing-box version.
 * @returns Version string
 */
export async function getSingboxVersion(): Promise<string> {
    return invoke('get_singbox_version');
}

// ============================================================================
// QUIC Blocking API
// ============================================================================

/**
 * Enable QUIC blocking via Windows Firewall.
 * Blocks UDP port 443 to force browsers to use TCP/TLS.
 * Requires administrator privileges.
 */
export async function enableQuicBlock(): Promise<void> {
    return invoke('enable_quic_block');
}

/**
 * Disable QUIC blocking.
 * Removes the firewall rule that blocks QUIC protocol.
 * Requires administrator privileges.
 */
export async function disableQuicBlock(): Promise<void> {
    return invoke('disable_quic_block');
}

/**
 * Check if QUIC is currently blocked.
 * @returns true if the QUIC blocking firewall rule exists
 */
export async function isQuicBlocked(): Promise<boolean> {
    return invoke('is_quic_blocked');
}

/**
 * Check if the application is running with administrator privileges.
 * @returns true if running as admin
 */
export async function isAdmin(): Promise<boolean> {
    return invoke('is_admin');
}
