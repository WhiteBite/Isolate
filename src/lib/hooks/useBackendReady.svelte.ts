/**
 * Backend Ready Hook
 * 
 * Provides unified retry logic with exponential backoff for waiting
 * for the Tauri backend to be ready.
 * 
 * Usage:
 * ```typescript
 * import { waitForBackend, invokeWhenReady } from '$lib/hooks/useBackendReady';
 * 
 * // Wait for backend
 * const ready = await waitForBackend();
 * 
 * // Or invoke command with automatic retry
 * const data = await invokeWhenReady<MyData>('get_data');
 * ```
 */

import { browser } from '$app/environment';

export interface BackendReadyOptions {
  /** Maximum number of retry attempts (default: 10) */
  maxRetries?: number;
  /** Initial delay in ms (default: 100) */
  initialDelay?: number;
  /** Maximum delay in ms (default: 2000) */
  maxDelay?: number;
  /** Backoff multiplier (default: 1.5) */
  backoffMultiplier?: number;
}

const DEFAULT_OPTIONS: Required<BackendReadyOptions> = {
  maxRetries: 20,
  initialDelay: 150,
  maxDelay: 3000,
  backoffMultiplier: 1.5,
};

/**
 * Check if running in Tauri environment
 */
export function isTauriEnv(): boolean {
  if (!browser) return false;
  return '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
}

/**
 * Calculate delay with exponential backoff
 */
function calculateDelay(attempt: number, options: Required<BackendReadyOptions>): number {
  const delay = options.initialDelay * Math.pow(options.backoffMultiplier, attempt);
  return Math.min(delay, options.maxDelay);
}

/**
 * Wait for backend to be ready with exponential backoff retry logic
 * 
 * @param options - Configuration options for retry behavior
 * @returns Promise<boolean> - true if backend is ready, false otherwise
 * 
 * @example
 * ```typescript
 * const ready = await waitForBackend();
 * if (ready) {
 *   // Backend is ready, safe to call commands
 * }
 * ```
 */
export async function waitForBackend(options?: BackendReadyOptions): Promise<boolean> {
  if (!browser || !isTauriEnv()) {
    return false;
  }

  const opts = { ...DEFAULT_OPTIONS, ...options };

  try {
    const { invoke } = await import('@tauri-apps/api/core');
    
    for (let attempt = 0; attempt < opts.maxRetries; attempt++) {
      try {
        const ready = await invoke<boolean>('is_backend_ready');
        if (ready) {
          return true;
        }
      } catch {
        // Backend not ready yet or command doesn't exist
      }
      
      const delay = calculateDelay(attempt, opts);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  } catch {
    // Failed to import Tauri API
    console.warn('[useBackendReady] Failed to import Tauri API');
  }
  
  return false;
}

/**
 * Execute a Tauri command with automatic backend ready check and retry
 * 
 * @param command - Command name to invoke
 * @param args - Command arguments
 * @param options - Retry options
 * @returns Promise<T> - Command result
 * @throws Error if backend is not ready after retries or command fails
 * 
 * @example
 * ```typescript
 * const services = await invokeWhenReady<Service[]>('get_services');
 * ```
 */
export async function invokeWhenReady<T>(
  command: string,
  args?: Record<string, unknown>,
  options?: BackendReadyOptions
): Promise<T> {
  const ready = await waitForBackend(options);
  
  if (!ready) {
    throw new Error(`Backend not ready after ${options?.maxRetries ?? DEFAULT_OPTIONS.maxRetries} retries`);
  }
  
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

/**
 * Create a reactive backend ready state for Svelte 5
 * 
 * @returns Object with ready state and check function
 * 
 * @example
 * ```svelte
 * <script lang="ts">
 *   import { createBackendReadyState } from '$lib/hooks/useBackendReady';
 *   
 *   const backend = createBackendReadyState();
 *   
 *   $effect(() => {
 *     backend.check();
 *   });
 * </script>
 * 
 * {#if backend.ready}
 *   <MainContent />
 * {:else if backend.error}
 *   <ErrorMessage error={backend.error} />
 * {:else}
 *   <LoadingSpinner />
 * {/if}
 * ```
 */
export function createBackendReadyState(options?: BackendReadyOptions) {
  let ready = $state(false);
  let checking = $state(false);
  let error = $state<string | null>(null);

  async function check() {
    if (checking) return;
    
    checking = true;
    error = null;
    
    try {
      ready = await waitForBackend(options);
      if (!ready) {
        error = 'Backend not ready after retries';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Unknown error';
      ready = false;
    } finally {
      checking = false;
    }
  }

  return {
    get ready() { return ready; },
    get checking() { return checking; },
    get error() { return error; },
    check,
  };
}
