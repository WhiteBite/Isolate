/**
 * Theme store for dark/light mode switching
 * Uses Svelte 5 runes pattern with localStorage persistence
 */

import { browser } from '$app/environment';

export type Theme = 'dark' | 'light' | 'system';

const STORAGE_KEY = 'isolate-theme';

/**
 * Get the initial theme from localStorage or default to 'dark'
 */
function getInitialTheme(): Theme {
  if (!browser) return 'dark';
  
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'dark' || stored === 'light' || stored === 'system') {
    return stored;
  }
  return 'dark';
}

/**
 * Get the effective theme (resolves 'system' to actual preference)
 */
function getEffectiveTheme(theme: Theme): 'dark' | 'light' {
  if (theme === 'system') {
    if (!browser) return 'dark';
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return theme;
}

/**
 * Apply theme class to document
 */
function applyTheme(theme: Theme): void {
  if (!browser) return;
  
  const effective = getEffectiveTheme(theme);
  const root = document.documentElement;
  
  // Remove both classes first
  root.classList.remove('dark', 'light');
  
  // Add the effective theme class
  root.classList.add(effective);
  
  // Also set data attribute for potential CSS usage
  root.setAttribute('data-theme', effective);
}

/**
 * Save theme to localStorage
 */
function saveTheme(theme: Theme): void {
  if (!browser) return;
  localStorage.setItem(STORAGE_KEY, theme);
}

// Create a simple reactive store using closure
let currentTheme: Theme = getInitialTheme();
const subscribers = new Set<(theme: Theme) => void>();

/**
 * Theme store with subscribe/set pattern
 */
export const themeStore = {
  subscribe(callback: (theme: Theme) => void) {
    subscribers.add(callback);
    callback(currentTheme);
    
    return () => {
      subscribers.delete(callback);
    };
  },
  
  set(theme: Theme) {
    currentTheme = theme;
    saveTheme(theme);
    applyTheme(theme);
    subscribers.forEach(cb => cb(theme));
  },
  
  get(): Theme {
    return currentTheme;
  },
  
  /**
   * Get the effective theme (dark or light, resolving system preference)
   */
  getEffective(): 'dark' | 'light' {
    return getEffectiveTheme(currentTheme);
  },
  
  /**
   * Toggle between dark and light (ignores system)
   */
  toggle() {
    const effective = getEffectiveTheme(currentTheme);
    this.set(effective === 'dark' ? 'light' : 'dark');
  },
  
  /**
   * Initialize theme on app start
   * Should be called once in layout
   */
  init() {
    if (!browser) return;
    
    // Apply initial theme
    applyTheme(currentTheme);
    
    // Listen for system preference changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => {
      if (currentTheme === 'system') {
        applyTheme('system');
      }
    };
    
    mediaQuery.addEventListener('change', handleChange);
    
    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }
};

// Export type for external use
export type ThemeStore = typeof themeStore;
