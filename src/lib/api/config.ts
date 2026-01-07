import { invoke } from '@tauri-apps/api/core';
import type { ConfigUpdate, ConfigUpdateResult } from './types';

// ============================================================================
// Config Updater API Functions
// ============================================================================

/** Check for config updates from remote repository */
export async function checkConfigUpdates(): Promise<ConfigUpdate[]> {
    return invoke('check_config_updates');
}

/** Download and apply config updates */
export async function downloadConfigUpdates(): Promise<ConfigUpdateResult> {
    return invoke('download_config_updates');
}
