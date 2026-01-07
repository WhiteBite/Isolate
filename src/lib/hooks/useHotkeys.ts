/**
 * Hotkeys Hook
 * 
 * Provides global keyboard shortcut handling with Svelte 5 runes.
 * Supports both in-app hotkeys and Tauri global shortcuts.
 * 
 * Usage:
 * ```typescript
 * import { createHotkeyHandler, useGlobalHotkeys } from '$lib/hooks/useHotkeys';
 * 
 * // In a Svelte component with $effect:
 * const cleanup = useGlobalHotkeys({
 *   toggleStrategy: () => toggleProtection(),
 *   openSettings: () => goto('/settings'),
 *   quickTest: () => runTest(),
 *   stopAll: () => stopAllProcesses()
 * });
 * 
 * // Cleanup on unmount
 * $effect(() => cleanup);
 * ```
 */

import { browser } from '$app/environment';
import { hotkeysStore, matchesHotkey, type HotkeysState, type HotkeyConfig } from '$lib/stores/hotkeys';

export interface HotkeyHandlers {
  toggleStrategy?: () => void | Promise<void>;
  openSettings?: () => void | Promise<void>;
  quickTest?: () => void | Promise<void>;
  stopAll?: () => void | Promise<void>;
}

export interface UseHotkeysOptions {
  /** Whether to ignore hotkeys when focus is in input/textarea (default: true) */
  ignoreInputs?: boolean;
  /** Whether to prevent default browser behavior (default: true) */
  preventDefault?: boolean;
  /** Whether to stop event propagation (default: false) */
  stopPropagation?: boolean;
}

const DEFAULT_OPTIONS: Required<UseHotkeysOptions> = {
  ignoreInputs: true,
  preventDefault: true,
  stopPropagation: false
};

/**
 * Check if the event target is an input element
 */
function isInputElement(target: EventTarget | null): boolean {
  if (!target || !(target instanceof HTMLElement)) return false;
  
  const tagName = target.tagName.toUpperCase();
  return (
    tagName === 'INPUT' ||
    tagName === 'TEXTAREA' ||
    tagName === 'SELECT' ||
    target.isContentEditable
  );
}

/**
 * Create a keyboard event handler for hotkeys
 * 
 * @param handlers - Object mapping hotkey actions to handler functions
 * @param options - Configuration options
 * @returns Event handler function
 */
export function createHotkeyHandler(
  handlers: HotkeyHandlers,
  options?: UseHotkeysOptions
): (e: KeyboardEvent) => void {
  const opts = { ...DEFAULT_OPTIONS, ...options };
  let currentHotkeys: HotkeysState = hotkeysStore.get();
  
  // Subscribe to hotkey changes
  hotkeysStore.subscribe(state => {
    currentHotkeys = state;
  });
  
  return (e: KeyboardEvent) => {
    // Skip if focus is in input element
    if (opts.ignoreInputs && isInputElement(e.target)) {
      return;
    }
    
    // Check each hotkey action
    const actions: (keyof HotkeysState)[] = ['toggleStrategy', 'openSettings', 'quickTest', 'stopAll'];
    
    for (const action of actions) {
      if (handlers[action] && matchesHotkey(e, currentHotkeys[action])) {
        if (opts.preventDefault) {
          e.preventDefault();
        }
        if (opts.stopPropagation) {
          e.stopPropagation();
        }
        
        // Execute handler (supports async)
        Promise.resolve(handlers[action]!()).catch(err => {
          console.error(`[useHotkeys] Error in ${action} handler:`, err);
        });
        
        return;
      }
    }
  };
}

/**
 * Register global hotkey handlers
 * 
 * @param handlers - Object mapping hotkey actions to handler functions
 * @param options - Configuration options
 * @returns Cleanup function to remove event listener
 * 
 * @example
 * ```typescript
 * $effect(() => {
 *   const cleanup = useGlobalHotkeys({
 *     toggleStrategy: () => toggleProtection(),
 *     openSettings: () => goto('/settings')
 *   });
 *   return cleanup;
 * });
 * ```
 */
export function useGlobalHotkeys(
  handlers: HotkeyHandlers,
  options?: UseHotkeysOptions
): () => void {
  if (!browser) {
    return () => {};
  }
  
  const handler = createHotkeyHandler(handlers, options);
  
  window.addEventListener('keydown', handler);
  
  return () => {
    window.removeEventListener('keydown', handler);
  };
}

/**
 * Register Tauri global shortcuts (system-wide, works even when app is not focused)
 * 
 * Note: Requires Tauri globalShortcut plugin to be enabled
 * 
 * @param handlers - Object mapping hotkey actions to handler functions
 * @returns Cleanup function to unregister shortcuts
 */
export async function registerTauriGlobalShortcuts(
  handlers: HotkeyHandlers
): Promise<() => Promise<void>> {
  if (!browser) {
    return async () => {};
  }
  
  const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
  if (!isTauri) {
    console.warn('[useHotkeys] Tauri global shortcuts not available in browser');
    return async () => {};
  }
  
  try {
    // @ts-ignore - Plugin may not be installed
    const { register, unregister } = await import('@tauri-apps/plugin-global-shortcut');
    const currentHotkeys = hotkeysStore.get();
    const registeredShortcuts: string[] = [];
    
    // Helper to convert HotkeyConfig to Tauri shortcut string
    const toShortcutString = (config: HotkeyConfig): string => {
      const parts: string[] = [];
      if (config.ctrlKey) parts.push('CommandOrControl');
      if (config.altKey) parts.push('Alt');
      if (config.shiftKey) parts.push('Shift');
      
      // Map special keys
      let key = config.key;
      if (key === ' ') key = 'Space';
      else if (key === ',') key = 'Comma';
      else if (key === '.') key = 'Period';
      else if (key === '/') key = 'Slash';
      else if (key === '\\') key = 'Backslash';
      else if (key === '[') key = 'BracketLeft';
      else if (key === ']') key = 'BracketRight';
      else if (key === ';') key = 'Semicolon';
      else if (key === "'") key = 'Quote';
      else if (key === '`') key = 'Backquote';
      else if (key === '-') key = 'Minus';
      else if (key === '=') key = 'Equal';
      
      parts.push(key.toUpperCase());
      return parts.join('+');
    };
    
    // Register each hotkey
    const actions: (keyof HotkeysState)[] = ['toggleStrategy', 'openSettings', 'quickTest', 'stopAll'];
    
    for (const action of actions) {
      if (handlers[action]) {
        const shortcut = toShortcutString(currentHotkeys[action]);
        try {
          await register(shortcut, () => {
            Promise.resolve(handlers[action]!()).catch(err => {
              console.error(`[useHotkeys] Error in global ${action} handler:`, err);
            });
          });
          registeredShortcuts.push(shortcut);
        } catch (err) {
          console.warn(`[useHotkeys] Failed to register global shortcut ${shortcut}:`, err);
        }
      }
    }
    
    // Return cleanup function
    return async () => {
      for (const shortcut of registeredShortcuts) {
        try {
          await unregister(shortcut);
        } catch (err) {
          console.warn(`[useHotkeys] Failed to unregister shortcut ${shortcut}:`, err);
        }
      }
    };
  } catch (err) {
    console.warn('[useHotkeys] Tauri global shortcut plugin not available:', err);
    return async () => {};
  }
}

/**
 * Create a reactive hotkeys state for Svelte 5
 * 
 * @returns Object with current hotkeys and helper methods
 */
export function createHotkeysState() {
  let hotkeys = $state<HotkeysState>(hotkeysStore.get());
  
  // Subscribe to store changes
  $effect(() => {
    const unsubscribe = hotkeysStore.subscribe(state => {
      hotkeys = state;
    });
    return unsubscribe;
  });
  
  return {
    get hotkeys() { return hotkeys; },
    
    /**
     * Check if a keyboard event matches a specific action
     */
    matches(e: KeyboardEvent, action: keyof HotkeysState): boolean {
      return matchesHotkey(e, hotkeys[action]);
    },
    
    /**
     * Get the hotkey config for an action
     */
    getConfig(action: keyof HotkeysState): HotkeyConfig {
      return hotkeys[action];
    }
  };
}

export type { HotkeyConfig, HotkeysState } from '$lib/stores/hotkeys';
