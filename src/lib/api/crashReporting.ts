/**
 * Crash Reporting API
 * 
 * Provides opt-in crash reporting functionality.
 * Privacy-first: disabled by default, anonymizes all data.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Crash reporting privacy information
 */
export interface CrashReportingInfo {
  enabled: boolean;
  privacy_url: string;
  data_collected: string[];
  data_not_collected: string[];
}

/**
 * Enable or disable crash reporting
 * 
 * When enabled, anonymous crash reports are sent to help improve the app.
 * Privacy: No IP addresses, usernames, or file paths are collected.
 */
export async function setCrashReportingEnabled(enabled: boolean): Promise<void> {
  await invoke('set_crash_reporting_enabled', { enabled });
}

/**
 * Check if crash reporting is enabled
 */
export async function isCrashReportingEnabled(): Promise<boolean> {
  return await invoke<boolean>('is_crash_reporting_enabled');
}

/**
 * Report an error to crash reporting
 * 
 * Used to manually report JavaScript errors.
 * The error message will be anonymized before sending.
 */
export async function reportCrashError(
  errorType: string,
  message: string,
  context?: string
): Promise<void> {
  await invoke('report_crash_error', { 
    errorType, 
    message, 
    context: context ?? null 
  });
}

/**
 * Get crash reporting privacy information
 * 
 * Returns details about what data is and isn't collected.
 */
export async function getCrashReportingInfo(): Promise<CrashReportingInfo> {
  return await invoke<CrashReportingInfo>('get_crash_reporting_info');
}

/**
 * Initialize frontend error tracking
 * 
 * Sets up global error handlers to capture unhandled errors.
 * Only reports errors if crash reporting is enabled.
 */
export function initFrontendErrorTracking(): void {
  // Handle unhandled promise rejections
  window.addEventListener('unhandledrejection', async (event) => {
    const message = event.reason?.message || event.reason?.toString() || 'Unknown rejection';
    await reportCrashError('unhandled_rejection', message).catch(() => {
      // Silently fail - crash reporting might be disabled
    });
  });

  // Handle uncaught errors
  window.addEventListener('error', async (event) => {
    const message = event.message || 'Unknown error';
    const context = event.filename ? `${event.filename}:${event.lineno}:${event.colno}` : undefined;
    await reportCrashError('uncaught_error', message, context).catch(() => {
      // Silently fail - crash reporting might be disabled
    });
  });
}

/**
 * Report a caught error
 * 
 * Use this to report errors that you catch but want to track.
 */
export async function reportError(error: Error, context?: string): Promise<void> {
  await reportCrashError(
    error.name || 'Error',
    error.message,
    context
  ).catch(() => {
    // Silently fail - crash reporting might be disabled
  });
}
