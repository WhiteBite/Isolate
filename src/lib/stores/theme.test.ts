/**
 * Unit tests for theme store
 * @vitest-environment happy-dom
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { store = {}; }),
    get length() { return Object.keys(store).length; },
    key: vi.fn((i: number) => Object.keys(store)[i] || null),
  };
})();

// Mock matchMedia
const matchMediaMock = vi.fn((query: string) => ({
  matches: query.includes('dark'),
  media: query,
  onchange: null,
  addListener: vi.fn(),
  removeListener: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn(),
}));

describe('theme store', () => {
  beforeEach(() => {
    // Setup mocks
    Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock, writable: true });
    Object.defineProperty(globalThis, 'matchMedia', { value: matchMediaMock, writable: true });
    localStorageMock.clear();
    vi.clearAllMocks();
    
    // Reset module cache to get fresh store instance
    vi.resetModules();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should initialize with dark theme by default', async () => {
    const { themeStore } = await import('./theme');
    expect(themeStore.get()).toBe('dark');
  });

  it('should load theme from localStorage', async () => {
    localStorageMock.setItem('isolate-theme', 'light');
    const { themeStore } = await import('./theme');
    expect(themeStore.get()).toBe('light');
  });

  it('should set theme and save to localStorage', async () => {
    const { themeStore } = await import('./theme');
    
    themeStore.set('light');
    expect(themeStore.get()).toBe('light');
    expect(localStorageMock.setItem).toHaveBeenCalledWith('isolate-theme', 'light');
  });

  it('should support system theme', async () => {
    const { themeStore } = await import('./theme');
    
    themeStore.set('system');
    expect(themeStore.get()).toBe('system');
    expect(localStorageMock.setItem).toHaveBeenCalledWith('isolate-theme', 'system');
  });

  it('should toggle between dark and light', async () => {
    const { themeStore } = await import('./theme');
    
    // Start with dark
    themeStore.set('dark');
    expect(themeStore.get()).toBe('dark');
    
    // Toggle to light
    themeStore.toggle();
    expect(themeStore.get()).toBe('light');
    
    // Toggle back to dark
    themeStore.toggle();
    expect(themeStore.get()).toBe('dark');
  });

  it('should resolve system theme to effective theme', async () => {
    const { themeStore } = await import('./theme');
    
    // When system prefers dark
    themeStore.set('system');
    expect(themeStore.getEffective()).toBe('dark'); // matchMedia mock returns dark
  });

  it('should return dark/light directly for non-system themes', async () => {
    const { themeStore } = await import('./theme');
    
    themeStore.set('dark');
    expect(themeStore.getEffective()).toBe('dark');
    
    themeStore.set('light');
    expect(themeStore.getEffective()).toBe('light');
  });

  it('should notify subscribers on theme change', async () => {
    const { themeStore } = await import('./theme');
    const callback = vi.fn();
    
    const unsubscribe = themeStore.subscribe(callback);
    
    // Should be called immediately with current value
    expect(callback).toHaveBeenCalledWith('dark');
    
    // Should be called on change
    themeStore.set('light');
    expect(callback).toHaveBeenCalledWith('light');
    
    unsubscribe();
  });

  it('should stop notifying after unsubscribe', async () => {
    const { themeStore } = await import('./theme');
    const callback = vi.fn();
    
    const unsubscribe = themeStore.subscribe(callback);
    expect(callback).toHaveBeenCalledTimes(1);
    
    unsubscribe();
    
    themeStore.set('light');
    // Should not be called again after unsubscribe
    expect(callback).toHaveBeenCalledTimes(1);
  });

  it('should apply theme class to document', async () => {
    const { themeStore } = await import('./theme');
    
    themeStore.set('dark');
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
    
    themeStore.set('light');
    expect(document.documentElement.classList.contains('light')).toBe(true);
    expect(document.documentElement.classList.contains('dark')).toBe(false);
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
  });

  it('should handle invalid stored theme gracefully', async () => {
    localStorageMock.setItem('isolate-theme', 'invalid-theme');
    const { themeStore } = await import('./theme');
    
    // Should fallback to dark
    expect(themeStore.get()).toBe('dark');
  });

  it('should initialize and setup system preference listener', async () => {
    const { themeStore } = await import('./theme');
    
    const cleanup = themeStore.init();
    
    // Should have called matchMedia
    expect(matchMediaMock).toHaveBeenCalledWith('(prefers-color-scheme: dark)');
    
    // Cleanup should be a function
    expect(typeof cleanup).toBe('function');
    
    if (cleanup) cleanup();
  });
});
