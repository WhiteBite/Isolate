import { invoke } from '@tauri-apps/api/core';
import type { DnsSettings, DnsServer, WindivertMode } from './types';

// ============================================================================
// DNS Settings API Functions
// ============================================================================

/**
 * Get current DNS settings.
 * @returns Current DNS configuration
 */
export async function getDnsSettings(): Promise<DnsSettings> {
    return invoke('get_dns_settings');
}

/**
 * Set DNS server configuration.
 * @param server - DNS server to use
 * @param customAddress - Custom DNS address (required if server is 'custom')
 */
export async function setDnsServer(server: DnsServer, customAddress?: string): Promise<void> {
    return invoke('set_dns_server', { server, customAddress });
}

// ============================================================================
// DNS System API Functions
// ============================================================================

/**
 * Apply DNS settings to Windows system.
 * Requires administrator privileges.
 */
export async function applyDnsToSystem(): Promise<void> {
    return invoke('apply_dns_to_system');
}

/**
 * Restore system DNS to DHCP (automatic).
 * Requires administrator privileges.
 */
export async function restoreSystemDns(): Promise<void> {
    return invoke('restore_system_dns');
}

// ============================================================================
// WinDivert Mode API Functions
// ============================================================================

/**
 * Get current WinDivert mode.
 * @returns Current WinDivert mode
 */
export async function getWindivertMode(): Promise<WindivertMode> {
    return invoke('get_windivert_mode');
}

/**
 * Set WinDivert operation mode.
 * @param mode - WinDivert mode to set
 */
export async function setWindivertMode(mode: WindivertMode): Promise<void> {
    return invoke('set_windivert_mode', { mode });
}
