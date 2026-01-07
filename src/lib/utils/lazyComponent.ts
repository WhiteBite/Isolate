/**
 * Lazy Component Loading Utility
 * 
 * Provides utilities for dynamically importing Svelte components
 * to improve initial load time by deferring heavy components.
 */

import type { Component } from 'svelte';

export interface LazyComponentState<T extends Component = Component> {
  component: T | null;
  loading: boolean;
  error: Error | null;
}

/**
 * Creates a lazy loader for a Svelte component.
 * The component is only loaded when load() is called.
 * 
 * @example
 * ```ts
 * const commandPaletteLoader = createLazyLoader(
 *   () => import('$lib/components/CommandPalette.svelte')
 * );
 * 
 * // Later, when needed:
 * const CommandPalette = await commandPaletteLoader.load();
 * ```
 */
export function createLazyLoader<T extends Component>(
  importFn: () => Promise<{ default: T }>
) {
  let cachedComponent: T | null = null;
  let loadPromise: Promise<T> | null = null;

  return {
    /**
     * Load the component. Returns cached version if already loaded.
     */
    async load(): Promise<T> {
      if (cachedComponent) {
        return cachedComponent;
      }

      if (loadPromise) {
        return loadPromise;
      }

      loadPromise = importFn()
        .then((module) => {
          cachedComponent = module.default;
          return cachedComponent;
        })
        .catch((error) => {
          loadPromise = null;
          throw error;
        });

      return loadPromise;
    },

    /**
     * Check if component is already loaded
     */
    isLoaded(): boolean {
      return cachedComponent !== null;
    },

    /**
     * Get cached component (null if not loaded)
     */
    getCached(): T | null {
      return cachedComponent;
    },

    /**
     * Preload the component without waiting
     */
    preload(): void {
      this.load().catch(() => {
        // Silently ignore preload errors
      });
    }
  };
}

/**
 * Pre-configured lazy loaders for heavy components
 */
export const lazyComponents = {
  CommandPalette: createLazyLoader(
    () => import('$lib/components/CommandPalette.svelte')
  ),
  TerminalPanel: createLazyLoader(
    () => import('$lib/components/TerminalPanel.svelte')
  ),
  KeyboardShortcutsModal: createLazyLoader(
    () => import('$lib/components/KeyboardShortcutsModal.svelte')
  ),
};

/**
 * Preload all lazy components after initial render
 * Call this after the app is interactive to warm the cache
 */
export function preloadAllLazyComponents(): void {
  // Use requestIdleCallback if available, otherwise setTimeout
  const schedulePreload = typeof requestIdleCallback !== 'undefined'
    ? requestIdleCallback
    : (fn: () => void) => setTimeout(fn, 1000);

  schedulePreload(() => {
    Object.values(lazyComponents).forEach(loader => loader.preload());
  });
}
