import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Hosts Manager Types
// ============================================================================

/**
 * Status of the hosts file modification.
 */
export interface HostsStatus {
    /** Whether Discord hosts are enabled */
    discord_enabled: boolean;
    /** Number of Discord host entries */
    discord_entries_count: number;
    /** Whether a backup exists */
    backup_exists: boolean;
    /** Backup timestamp (ISO 8601) */
    backup_timestamp: string | null;
    /** Whether hosts file is writable (admin check) */
    is_writable: boolean;
    /** Last modification timestamp */
    last_modified: string | null;
}

/**
 * A single host entry.
 */
export interface HostEntry {
    ip: string;
    hostname: string;
    comment?: string;
}

/**
 * Result of a hosts operation.
 */
export interface HostsOperationResult {
    success: boolean;
    entries_affected: number;
    error?: string;
    requires_admin?: boolean;
}

// ============================================================================
// Hosts Manager API Functions
// ============================================================================

/**
 * Get the current hosts file status.
 * @returns Current hosts status
 */
export async function getHostsStatus(): Promise<HostsStatus> {
    return invoke('get_hosts_status');
}

/**
 * Enable Discord hosts entries.
 * Adds IP mappings for Discord domains to bypass DPI.
 * Requires administrator privileges.
 * @returns Operation result
 */
export async function enableDiscordHosts(): Promise<HostsOperationResult> {
    return invoke('enable_discord_hosts');
}

/**
 * Disable Discord hosts entries.
 * Removes Discord IP mappings from hosts file.
 * Requires administrator privileges.
 * @returns Operation result
 */
export async function disableDiscordHosts(): Promise<HostsOperationResult> {
    return invoke('disable_discord_hosts');
}

/**
 * Create a backup of the current hosts file.
 * @returns Operation result
 */
export async function backupHostsFile(): Promise<HostsOperationResult> {
    return invoke('backup_hosts_file');
}

/**
 * Restore hosts file from backup.
 * Requires administrator privileges.
 * @returns Operation result
 */
export async function restoreHostsFile(): Promise<HostsOperationResult> {
    return invoke('restore_hosts_file');
}

/**
 * Get all custom host entries added by Isolate.
 * @returns Array of host entries
 */
export async function getIsolateHostEntries(): Promise<HostEntry[]> {
    return invoke('get_isolate_host_entries');
}

/**
 * Add a custom host entry.
 * Requires administrator privileges.
 * @param entry - Host entry to add
 * @returns Operation result
 */
export async function addHostEntry(entry: HostEntry): Promise<HostsOperationResult> {
    return invoke('add_host_entry', { entry });
}

/**
 * Remove a custom host entry.
 * Requires administrator privileges.
 * @param hostname - Hostname to remove
 * @returns Operation result
 */
export async function removeHostEntry(hostname: string): Promise<HostsOperationResult> {
    return invoke('remove_host_entry', { hostname });
}

/**
 * Check if the application has write access to hosts file.
 * @returns true if hosts file is writable
 */
export async function canModifyHosts(): Promise<boolean> {
    return invoke('can_modify_hosts');
}

/**
 * Flush DNS cache after hosts modification.
 * Requires administrator privileges.
 */
export async function flushDnsCache(): Promise<void> {
    return invoke('flush_dns_cache');
}
