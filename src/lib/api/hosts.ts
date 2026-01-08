import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Hosts Manager Types
// ============================================================================

/**
 * Status of the hosts file modification.
 * Matches Rust HostsStatus struct from hosts_manager.rs (camelCase)
 */
export interface HostsStatus {
    /** Whether Discord hosts entries are enabled */
    discordEnabled: boolean;
    /** Number of Discord host entries */
    discordEntriesCount: number;
    /** Whether a backup exists */
    backupExists: boolean;
    /** Whether hosts file is writable */
    isWritable: boolean;
    /** Path to the hosts file */
    hostsPath: string;
    /** Path to the backup file */
    backupPath: string;
    /** Last modified timestamp (ISO 8601) */
    lastModified: string | null;
    /** Backup timestamp (ISO 8601) */
    backupTimestamp: string | null;
}

// ============================================================================
// Hosts Manager API Functions
// ============================================================================

/**
 * Get the current hosts file status.
 * @returns Current hosts status including enabled state and entry count
 */
export async function getHostsStatus(): Promise<HostsStatus> {
    return invoke('get_hosts_status');
}

/**
 * Enable Discord hosts entries.
 * Adds IP mappings for Discord voice servers to bypass DPI blocking.
 * Creates a backup before modification.
 * Requires administrator privileges.
 * @throws Error if admin rights are required or hosts file is not writable
 */
export async function enableDiscordHosts(): Promise<void> {
    return invoke('enable_discord_hosts');
}

/**
 * Disable Discord hosts entries.
 * Removes Discord IP mappings from hosts file.
 * Requires administrator privileges.
 * @throws Error if admin rights are required or hosts file is not writable
 */
export async function disableDiscordHosts(): Promise<void> {
    return invoke('disable_discord_hosts');
}

/**
 * Toggle Discord hosts entries.
 * Enables if disabled, disables if enabled.
 * @param enable - Whether to enable or disable Discord hosts
 * @throws Error if admin rights are required
 */
export async function toggleDiscordHosts(enable: boolean): Promise<void> {
    if (enable) {
        return enableDiscordHosts();
    } else {
        return disableDiscordHosts();
    }
}

/**
 * Create a backup of the current hosts file.
 * Backup is stored in the app data directory.
 * @throws Error if backup creation fails
 */
export async function backupHosts(): Promise<void> {
    return invoke('backup_hosts');
}

/**
 * Restore hosts file from backup.
 * Requires administrator privileges.
 * @throws Error if backup doesn't exist or admin rights are required
 */
export async function restoreHosts(): Promise<void> {
    return invoke('restore_hosts');
}

/**
 * Flush DNS cache after hosts modification.
 * Runs `ipconfig /flushdns` to clear the Windows DNS resolver cache.
 * This ensures hosts file changes take effect immediately.
 * @throws Error if flush operation fails
 */
export async function flushDns(): Promise<void> {
    return invoke('flush_dns');
}

/**
 * Enable Discord hosts and flush DNS cache.
 * Convenience function that performs both operations.
 * @throws Error if any operation fails
 */
export async function enableDiscordHostsAndFlush(): Promise<void> {
    await enableDiscordHosts();
    await flushDns();
}

/**
 * Disable Discord hosts and flush DNS cache.
 * Convenience function that performs both operations.
 * @throws Error if any operation fails
 */
export async function disableDiscordHostsAndFlush(): Promise<void> {
    await disableDiscordHosts();
    await flushDns();
}
