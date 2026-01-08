/**
 * Backend utilities for Tauri IPC communication
 * Provides retry logic for waiting backend to be ready
 */

import { browser } from '$app/environment';

/**
 * Check if running in Tauri environment
 */
export function isTauriEnv(): boolean {
  if (!browser) return false;
  return '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
}

/**
 * Wait for backend to be ready with retry logic
 * 
 * Backend initialization includes:
 * - AppState creation
 * - Settings loading
 * - Binary integrity verification (can take 2-5 seconds)
 * 
 * @param retries - Number of retry attempts (default: 30)
 * @param delayMs - Delay between retries in ms (default: 300)
 * @returns Promise<boolean> - true if backend is ready, false otherwise
 */
export async function waitForBackend(retries = 30, delayMs = 300): Promise<boolean> {
  if (!browser || !isTauriEnv()) {
    return false;
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core');
    
    for (let i = 0; i < retries; i++) {
      try {
        const ready = await invoke<boolean>('is_backend_ready');
        if (ready) return true;
      } catch {
        // Backend not ready yet or command failed
      }
      // Wait before next attempt
      await new Promise(r => setTimeout(r, delayMs));
    }
    
    console.warn(`waitForBackend: Backend not ready after ${retries} attempts (${retries * delayMs}ms)`);
  } catch (e) {
    console.error('waitForBackend: Failed to import Tauri API', e);
  }
  
  return false;
}

/**
 * Execute a Tauri command with backend ready check
 * @param command - Command name to invoke
 * @param args - Command arguments
 * @param retries - Number of retry attempts for backend ready check (default: 30)
 * @returns Promise<T> - Command result
 * @throws Error if backend is not ready or command fails
 */
export async function invokeWithBackendCheck<T>(
  command: string,
  args?: Record<string, unknown>,
  retries = 30
): Promise<T> {
  const ready = await waitForBackend(retries);
  if (!ready) {
    throw new Error('Backend not ready after retries');
  }
  
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}
