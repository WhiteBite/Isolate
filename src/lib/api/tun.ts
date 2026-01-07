import { invoke } from '@tauri-apps/api/core';
import type { TunInstance, TunConfig } from './types';

// ============================================================================
// TUN API Functions
// ============================================================================

/**
 * Start TUN mode.
 * Routes all system traffic through the specified SOCKS proxy.
 * Requires administrator privileges.
 * @param socksPort - SOCKS5 proxy port to route traffic through
 * @returns TUN instance information
 */
export async function startTun(socksPort: number): Promise<TunInstance> {
    return invoke('start_tun', { socksPort });
}

/**
 * Stop TUN mode.
 */
export async function stopTun(): Promise<void> {
    return invoke('stop_tun');
}

/**
 * Check if TUN is running.
 * @returns true if TUN is running
 */
export async function isTunRunning(): Promise<boolean> {
    return invoke('is_tun_running');
}

/**
 * Get TUN status.
 * @returns TUN instance information
 */
export async function getTunStatus(): Promise<TunInstance> {
    return invoke('get_tun_status');
}

/**
 * Get TUN configuration.
 * @returns Current TUN configuration
 */
export async function getTunConfig(): Promise<TunConfig> {
    return invoke('get_tun_config');
}

/**
 * Update TUN configuration.
 * Note: Changes take effect on next TUN start.
 * @param config - New TUN configuration
 */
export async function setTunConfig(config: TunConfig): Promise<void> {
    return invoke('set_tun_config', { config });
}

/**
 * Check if TUN mode is available.
 * Returns true if sing-box exists and running with admin privileges.
 * @returns true if TUN is available
 */
export async function isTunAvailable(): Promise<boolean> {
    return invoke('is_tun_available');
}

/**
 * Restart TUN with optional new SOCKS port.
 * @param socksPort - Optional new SOCKS port
 * @returns TUN instance information
 */
export async function restartTun(socksPort?: number): Promise<TunInstance> {
    return invoke('restart_tun', { socksPort });
}
