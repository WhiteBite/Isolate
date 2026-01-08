/**
 * IP Set management API
 * 
 * Functions for managing IP address lists (ipset) for DPI bypass.
 * Supports downloading from remote sources and auto-updates.
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

/**
 * Source configuration for an IP set
 */
export interface IpsetSource {
    /** Source ID */
    id: string;
    /** Human-readable name */
    name: string;
    /** Remote URL to download from */
    url: string;
    /** Optional description */
    description: string | null;
    /** Whether this source is enabled */
    enabled: boolean;
    /** Priority (lower = higher priority) */
    priority: number;
}

/**
 * Information about an ipset for UI display
 */
export interface IpsetInfo {
    /** Ipset ID (filename without extension) */
    id: string;
    /** Human-readable name */
    name: string;
    /** Last update timestamp (ISO 8601) */
    last_updated: string | null;
    /** File size in bytes */
    size: number | null;
    /** Total number of IP entries */
    ip_count: number | null;
    /** Number of IPv4 entries */
    ipv4_count: number | null;
    /** Number of IPv6 entries */
    ipv6_count: number | null;
    /** Number of CIDR entries */
    cidr_count: number | null;
    /** Whether update is available */
    update_available: boolean;
    /** Source URL (if configured) */
    source_url: string | null;
    /** Whether auto-update is enabled */
    auto_update_enabled: boolean;
}

/**
 * Result of updating ipset
 */
export interface IpsetUpdateResult {
    /** Whether update was successful */
    success: boolean;
    /** Number of IP entries after update */
    ip_count: number;
    /** Number of IPv4 entries */
    ipv4_count: number;
    /** Number of IPv6 entries */
    ipv6_count: number;
    /** Number of CIDR entries */
    cidr_count: number;
    /** Source URL used */
    source_url: string;
    /** Error message if failed */
    error: string | null;
    /** Timestamp of update (ISO 8601) */
    timestamp: string;
}

// ============================================================================
// API Functions
// ============================================================================

/**
 * Get information about the current ipset.
 * 
 * Returns metadata including IP count, last update time, and source URL.
 * 
 * @returns Current ipset info
 */
export async function getIpsetInfo(): Promise<IpsetInfo> {
    return invoke<IpsetInfo>('get_ipset_info');
}

/**
 * Update ipset from a specific source URL.
 * 
 * Downloads IP addresses from the given URL and updates ipset-all.txt.
 * Validates content before saving (only IPv4/IPv6 and CIDR allowed).
 * 
 * @param sourceUrl - URL to download ipset from
 * @returns Update result with IP counts
 */
export async function updateIpset(sourceUrl: string): Promise<IpsetUpdateResult> {
    return invoke<IpsetUpdateResult>('update_ipset', { sourceUrl });
}

/**
 * Update ipset from configured sources.
 * 
 * Tries each configured source in priority order until one succeeds.
 * Primary source is zapret-discord-youtube GitHub repository.
 * 
 * @returns Update result with IP counts
 */
export async function updateIpsetFromSources(): Promise<IpsetUpdateResult> {
    return invoke<IpsetUpdateResult>('update_ipset_from_sources');
}

/**
 * Set ipset auto-update enabled/disabled.
 * 
 * When enabled, ipset will be automatically updated once per day.
 * 
 * @param enabled - Whether auto-update should be enabled
 */
export async function setIpsetAutoUpdate(enabled: boolean): Promise<void> {
    return invoke<void>('set_ipset_auto_update', { enabled });
}

/**
 * Get ipset sources configuration.
 * 
 * Returns the list of configured ipset sources with their URLs and priorities.
 * 
 * @returns List of configured sources
 */
export async function getIpsetSources(): Promise<IpsetSource[]> {
    return invoke<IpsetSource[]>('get_ipset_sources');
}

/**
 * Restore ipset from backup.
 * 
 * Restores the previous version of ipset-all.txt from backup.
 * Useful if an update caused issues.
 */
export async function restoreIpsetBackup(): Promise<void> {
    return invoke<void>('restore_ipset_backup');
}
