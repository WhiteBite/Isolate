import { invoke } from '@tauri-apps/api/core';
import type { VlessConfig, SingboxInstance } from './types';

// ============================================================================
// VLESS Config API Functions
// ============================================================================

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
