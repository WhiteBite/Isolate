import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from './types';

// ============================================================================
// Settings API Functions
// ============================================================================

/**
 * Get application settings.
 * @returns Current settings
 */
export async function getAppSettings(): Promise<AppSettings> {
    return invoke('get_settings');
}

/**
 * Save application settings.
 * @param settings - Settings to save
 */
export async function saveAppSettings(settings: AppSettings): Promise<void> {
    return invoke('save_settings', { settings });
}
