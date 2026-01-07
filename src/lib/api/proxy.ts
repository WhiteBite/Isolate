import { invoke } from '@tauri-apps/api/core';
import type { ProxyConfig, ProxyTestResult } from './types';

// ============================================================================
// Proxy API Functions
// ============================================================================

/**
 * Get all configured proxies.
 * @returns Array of proxy configurations
 */
export async function getProxies(): Promise<ProxyConfig[]> {
    return invoke('get_proxies');
}

/**
 * Add a new proxy configuration.
 * @param proxy - Partial proxy configuration (id will be generated)
 * @returns The created proxy configuration
 */
export async function addProxy(proxy: Partial<ProxyConfig>): Promise<ProxyConfig> {
    return invoke('add_proxy', { proxy });
}

/**
 * Update an existing proxy configuration.
 * @param proxy - Full proxy configuration with id
 */
export async function updateProxy(proxy: ProxyConfig): Promise<void> {
    return invoke('update_proxy', { proxy });
}

/**
 * Delete a proxy configuration.
 * @param id - Proxy ID to delete
 */
export async function deleteProxy(id: string): Promise<void> {
    return invoke('delete_proxy', { id });
}

/**
 * Apply (activate) a proxy.
 * @param id - Proxy ID to apply
 */
export async function applyProxy(id: string): Promise<void> {
    return invoke('apply_proxy', { id });
}

/**
 * Deactivate a proxy.
 * Stops sing-box if running and marks proxy as inactive.
 * @param id - Proxy ID to deactivate
 */
export async function deactivateProxy(id: string): Promise<void> {
    return invoke('deactivate_proxy', { id });
}

/**
 * Test proxy connectivity.
 * @param id - Proxy ID to test
 * @returns Test result with success status, latency, and optional error
 */
export async function testProxy(id: string): Promise<ProxyTestResult> {
    return invoke('test_proxy', { id });
}

/**
 * Import proxy from URL (ss://, vmess://, vless://, trojan://, etc.)
 * @param url - Proxy URL to import
 * @returns Parsed proxy configuration
 */
export async function importProxyUrl(url: string): Promise<ProxyConfig> {
    return invoke('import_proxy_url', { url });
}

/**
 * Import proxies from subscription URL.
 * @param url - Subscription URL
 * @returns Array of parsed proxy configurations
 */
export async function importSubscription(url: string): Promise<ProxyConfig[]> {
    return invoke('import_subscription', { url });
}

/**
 * Export proxy configuration as URL.
 * @param id - Proxy ID to export
 * @returns Proxy URL string (vless://, vmess://, ss://, etc.)
 */
export async function exportProxyUrl(id: string): Promise<string> {
    return invoke('export_proxy_url', { id });
}

// ============================================================================
// System Proxy API Functions
// ============================================================================

/**
 * Set system proxy.
 * Configures Windows system proxy settings.
 * @param host - Proxy host
 * @param port - Proxy port
 * @param scheme - Proxy scheme ('socks5', 'http')
 */
export async function setSystemProxy(host: string, port: number, scheme: string): Promise<void> {
    return invoke('set_system_proxy', { host, port, scheme });
}

/**
 * Clear system proxy settings.
 */
export async function clearSystemProxy(): Promise<void> {
    return invoke('clear_system_proxy');
}

/**
 * Check if system proxy is currently set.
 * @returns true if system proxy is enabled
 */
export async function isSystemProxySet(): Promise<boolean> {
    return invoke('is_system_proxy_set');
}
