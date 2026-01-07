import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { LogEntry, LogFilter } from './types';

// ============================================================================
// Log API Functions
// ============================================================================

/**
 * Get logs with optional filtering.
 * @param filter - Optional filter parameters
 * @returns Array of log entries
 */
export async function getLogs(filter?: LogFilter): Promise<LogEntry[]> {
    return invoke('get_logs', { filter });
}

/**
 * Clear all logs from memory.
 */
export async function clearLogs(): Promise<void> {
    return invoke('clear_logs');
}

/**
 * Export logs to a file.
 * @returns Path to the exported file
 */
export async function exportLogs(): Promise<string> {
    return invoke('export_logs');
}

/**
 * Subscribe to real-time log entries.
 * @param callback - Function to call when a new log entry is received
 * @returns Unsubscribe function
 */
export function onLogEntry(callback: (entry: LogEntry) => void): Promise<UnlistenFn> {
    return listen('log:entry', (event) => {
        callback(event.payload as LogEntry);
    });
}
