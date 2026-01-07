/**
 * Hotkeys store for keyboard shortcuts configuration
 * Uses localStorage persistence with Svelte 5 runes pattern
 */

import { browser } from '$app/environment';

const STORAGE_KEY = 'isolate-hotkeys';

export interface HotkeyConfig {
  key: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
}

export interface HotkeysState {
  toggleStrategy: HotkeyConfig;
  openSettings: HotkeyConfig;
  quickTest: HotkeyConfig;
  stopAll: HotkeyConfig;
}

export interface HotkeyAction {
  id: keyof HotkeysState;
  label: string;
  description: string;
}

// Available hotkey actions with descriptions
export const HOTKEY_ACTIONS: HotkeyAction[] = [
  {
    id: 'toggleStrategy',
    label: 'Toggle Protection',
    description: 'Start or stop the current protection strategy'
  },
  {
    id: 'openSettings',
    label: 'Open Settings',
    description: 'Navigate to settings page'
  },
  {
    id: 'quickTest',
    label: 'Quick Test',
    description: 'Run a quick connectivity test'
  },
  {
    id: 'stopAll',
    label: 'Stop All',
    description: 'Stop all running strategies and processes'
  }
];

// Default hotkey configurations
const DEFAULT_HOTKEYS: HotkeysState = {
  toggleStrategy: { key: 'P', ctrlKey: true, shiftKey: true, altKey: false },
  openSettings: { key: ',', ctrlKey: true, shiftKey: false, altKey: false },
  quickTest: { key: 'T', ctrlKey: true, shiftKey: false, altKey: false },
  stopAll: { key: 'S', ctrlKey: true, shiftKey: true, altKey: false }
};

/**
 * Format hotkey for display (e.g., "Ctrl+Shift+S")
 */
export function formatHotkey(config: HotkeyConfig): string {
  const parts: string[] = [];
  if (config.ctrlKey) parts.push('Ctrl');
  if (config.altKey) parts.push('Alt');
  if (config.shiftKey) parts.push('Shift');
  
  // Format special keys
  let keyDisplay = config.key;
  if (config.key === ',') keyDisplay = ',';
  else if (config.key === '.') keyDisplay = '.';
  else if (config.key === ' ') keyDisplay = 'Space';
  else if (config.key === 'Escape') keyDisplay = 'Esc';
  else keyDisplay = config.key.toUpperCase();
  
  parts.push(keyDisplay);
  return parts.join('+');
}

/**
 * Parse keyboard event to HotkeyConfig
 */
export function parseKeyboardEvent(e: KeyboardEvent): HotkeyConfig | null {
  // Ignore modifier-only presses
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
    return null;
  }
  
  return {
    key: e.key.length === 1 ? e.key.toUpperCase() : e.key,
    ctrlKey: e.ctrlKey,
    shiftKey: e.shiftKey,
    altKey: e.altKey
  };
}

/**
 * Check if keyboard event matches hotkey config
 */
export function matchesHotkey(e: KeyboardEvent, config: HotkeyConfig): boolean {
  const eventKey = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  const configKey = config.key.length === 1 ? config.key.toUpperCase() : config.key;
  
  return (
    eventKey === configKey &&
    e.ctrlKey === config.ctrlKey &&
    e.shiftKey === config.shiftKey &&
    e.altKey === config.altKey
  );
}

/**
 * Check if two hotkey configs are equal
 */
export function hotkeysEqual(a: HotkeyConfig, b: HotkeyConfig): boolean {
  return (
    a.key.toUpperCase() === b.key.toUpperCase() &&
    a.ctrlKey === b.ctrlKey &&
    a.shiftKey === b.shiftKey &&
    a.altKey === b.altKey
  );
}

/**
 * Load hotkeys from localStorage
 */
function loadHotkeys(): HotkeysState {
  if (!browser) return DEFAULT_HOTKEYS;
  
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      // Merge with defaults to handle missing fields
      return {
        toggleStrategy: { ...DEFAULT_HOTKEYS.toggleStrategy, ...parsed.toggleStrategy },
        openSettings: { ...DEFAULT_HOTKEYS.openSettings, ...parsed.openSettings },
        quickTest: { ...DEFAULT_HOTKEYS.quickTest, ...parsed.quickTest },
        stopAll: { ...DEFAULT_HOTKEYS.stopAll, ...parsed.stopAll }
      };
    }
  } catch (e) {
    console.warn('Failed to load hotkeys:', e);
  }
  
  return DEFAULT_HOTKEYS;
}

/**
 * Save hotkeys to localStorage
 */
function saveHotkeys(state: HotkeysState): void {
  if (!browser) return;
  
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (e) {
    console.warn('Failed to save hotkeys:', e);
  }
}

// Store state
let currentHotkeys: HotkeysState = loadHotkeys();
const subscribers = new Set<(state: HotkeysState) => void>();

/**
 * Hotkeys store with subscribe/set pattern
 */
export const hotkeysStore = {
  subscribe(callback: (state: HotkeysState) => void) {
    subscribers.add(callback);
    callback(currentHotkeys);
    
    return () => {
      subscribers.delete(callback);
    };
  },
  
  /**
   * Get current hotkeys state
   */
  get(): HotkeysState {
    return currentHotkeys;
  },
  
  /**
   * Set a specific hotkey
   */
  setHotkey(action: keyof HotkeysState, config: HotkeyConfig): void {
    currentHotkeys = {
      ...currentHotkeys,
      [action]: config
    };
    saveHotkeys(currentHotkeys);
    subscribers.forEach(cb => cb(currentHotkeys));
  },
  
  /**
   * Reset all hotkeys to defaults
   */
  resetToDefaults(): void {
    currentHotkeys = { ...DEFAULT_HOTKEYS };
    saveHotkeys(currentHotkeys);
    subscribers.forEach(cb => cb(currentHotkeys));
  },
  
  /**
   * Reset a specific hotkey to default
   */
  resetHotkey(action: keyof HotkeysState): void {
    currentHotkeys = {
      ...currentHotkeys,
      [action]: DEFAULT_HOTKEYS[action]
    };
    saveHotkeys(currentHotkeys);
    subscribers.forEach(cb => cb(currentHotkeys));
  },
  
  /**
   * Check if a hotkey conflicts with another action
   */
  hasConflict(action: keyof HotkeysState, config: HotkeyConfig): keyof HotkeysState | null {
    for (const [key, value] of Object.entries(currentHotkeys)) {
      if (key !== action && hotkeysEqual(value, config)) {
        return key as keyof HotkeysState;
      }
    }
    return null;
  },
  
  /**
   * Get default hotkey for an action
   */
  getDefault(action: keyof HotkeysState): HotkeyConfig {
    return DEFAULT_HOTKEYS[action];
  }
};

export type HotkeysStore = typeof hotkeysStore;
