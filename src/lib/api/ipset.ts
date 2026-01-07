import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Ipset Types
// ============================================================================

/**
 * Available ipset sources.
 */
export type IpsetSource = 'antifilter' | 'community' | 'custom';

/**
 * Information about the current ipset.
 */
export interface IpsetInfo {
    /** Source of the ipset */
    source: IpsetSource;
    /** Last update timestamp (ISO 8601) */
    last_updated: string | null;
    /** Number of IP addresses/ranges */
    ip_count: number;
    /** File size in bytes */
    size_bytes: number;
    /** Whether an update is available */
    update_available: boolean;
    /** Auto-update enabled */
    auto_update: boolean;
    /** Auto-update interval in hours */
    auto_update_interval_hours: number;
}

/**
 * Result of an ipset update operation.
 */
export interface IpsetUpdateResult {
    success: boolean;
    ip_count: number;
    size_bytes: number;
    error?: string;
}

// ============================================================================
// Ipset API Functions
// ============================================================================

/**
 * Get information about the current ipset.
 * @returns Current ipset info
 */
export async function getIpsetInfo(): Promise<IpsetInfo> {
    return invoke('get_ipset_info');
}

/**
 * Update the ipset from the configured source.
 * @returns Update result
 */
export async function updateIpset(): Promise<IpsetUpdateResult> {
    return invoke('update_ipset');
}

/**
 * Check if an ipset update is available.
 * @returns true if update is available
 */
export async function checkIpsetUpdate(): Promise<boolean> {
    return invoke('check_ipset_update');
}

/**
 * Set the ipset source.
 * @param source - Source to use
 */
export async function setIpsetSource(source: IpsetSource): Promise<void> {
    return invoke('set_ipset_source', { source });
}

/**
 * Toggle auto-update for ipset.
 * @param enabled - Whether auto-update should be enabled
 */
export async function toggleIpsetAutoUpdate(enabled: boolean): Promise<void> {
    return invoke('toggle_ipset_auto_update', { enabled });
}

/**
 * Set auto-update interval.
 * @param hours - Interval in hours
 */
export async function setIpsetAutoUpdateInterval(hours: number): Promise<void> {
    return invoke('set_ipset_auto_update_interval', { hours });
}

/**
 * Import a custom ipset from file.
 * @param filePath - Path to the ipset file
 * @returns Import result
 */
export async function importCustomIpset(filePath: string): Promise<IpsetUpdateResult> {
    return invoke('import_custom_ipset', { filePath });
}

/**
 * Export the current ipset to a file.
 * @param filePath - Path to save the ipset
 */
export async function exportIpset(filePath: string): Promise<void> {
    return invoke('export_ipset', { filePath });
}
