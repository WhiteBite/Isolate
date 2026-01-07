import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  formatHotkey,
  parseKeyboardEvent,
  matchesHotkey,
  hotkeysEqual,
  hotkeysStore,
  HOTKEY_ACTIONS,
  type HotkeyConfig,
  type HotkeysState
} from '../hotkeys';

// ============================================================================
// Test Helpers
// ============================================================================

function createKeyboardEvent(options: Partial<KeyboardEvent>): KeyboardEvent {
  return {
    key: 'A',
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    metaKey: false,
    ...options
  } as KeyboardEvent;
}

function createHotkeyConfig(overrides: Partial<HotkeyConfig> = {}): HotkeyConfig {
  return {
    key: 'A',
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    ...overrides
  };
}

// ============================================================================
// formatHotkey Tests
// ============================================================================

describe('formatHotkey', () => {
  it('should format simple key', () => {
    const config = createHotkeyConfig({ key: 'A' });
    expect(formatHotkey(config)).toBe('A');
  });

  it('should format Ctrl+key', () => {
    const config = createHotkeyConfig({ key: 'S', ctrlKey: true });
    expect(formatHotkey(config)).toBe('Ctrl+S');
  });

  it('should format Ctrl+Shift+key', () => {
    const config = createHotkeyConfig({ key: 'P', ctrlKey: true, shiftKey: true });
    expect(formatHotkey(config)).toBe('Ctrl+Shift+P');
  });

  it('should format Ctrl+Alt+Shift+key', () => {
    const config = createHotkeyConfig({ key: 'X', ctrlKey: true, altKey: true, shiftKey: true });
    expect(formatHotkey(config)).toBe('Ctrl+Alt+Shift+X');
  });

  it('should format Alt+key', () => {
    const config = createHotkeyConfig({ key: 'F4', altKey: true });
    expect(formatHotkey(config)).toBe('Alt+F4');
  });

  it('should format special keys correctly', () => {
    expect(formatHotkey(createHotkeyConfig({ key: ' ' }))).toBe('Space');
    expect(formatHotkey(createHotkeyConfig({ key: 'Escape' }))).toBe('Esc');
    expect(formatHotkey(createHotkeyConfig({ key: ',' }))).toBe(',');
    expect(formatHotkey(createHotkeyConfig({ key: '.' }))).toBe('.');
  });

  it('should uppercase single character keys', () => {
    const config = createHotkeyConfig({ key: 'a' });
    expect(formatHotkey(config)).toBe('A');
  });

  it('should maintain order: Ctrl+Alt+Shift', () => {
    const config = createHotkeyConfig({ 
      key: 'K', 
      ctrlKey: true, 
      altKey: true, 
      shiftKey: true 
    });
    const formatted = formatHotkey(config);
    
    const ctrlIndex = formatted.indexOf('Ctrl');
    const altIndex = formatted.indexOf('Alt');
    const shiftIndex = formatted.indexOf('Shift');
    
    expect(ctrlIndex).toBeLessThan(altIndex);
    expect(altIndex).toBeLessThan(shiftIndex);
  });
});

// ============================================================================
// parseKeyboardEvent Tests
// ============================================================================

describe('parseKeyboardEvent', () => {
  it('should parse simple key press', () => {
    const event = createKeyboardEvent({ key: 'a' });
    const result = parseKeyboardEvent(event);

    expect(result).toEqual({
      key: 'A',
      ctrlKey: false,
      shiftKey: false,
      altKey: false
    });
  });

  it('should parse key with modifiers', () => {
    const event = createKeyboardEvent({ 
      key: 's', 
      ctrlKey: true, 
      shiftKey: true 
    });
    const result = parseKeyboardEvent(event);

    expect(result).toEqual({
      key: 'S',
      ctrlKey: true,
      shiftKey: true,
      altKey: false
    });
  });

  it('should return null for modifier-only presses', () => {
    expect(parseKeyboardEvent(createKeyboardEvent({ key: 'Control' }))).toBeNull();
    expect(parseKeyboardEvent(createKeyboardEvent({ key: 'Shift' }))).toBeNull();
    expect(parseKeyboardEvent(createKeyboardEvent({ key: 'Alt' }))).toBeNull();
    expect(parseKeyboardEvent(createKeyboardEvent({ key: 'Meta' }))).toBeNull();
  });

  it('should preserve special key names', () => {
    const event = createKeyboardEvent({ key: 'Escape' });
    const result = parseKeyboardEvent(event);

    expect(result?.key).toBe('Escape');
  });

  it('should uppercase single character keys', () => {
    const event = createKeyboardEvent({ key: 'z' });
    const result = parseKeyboardEvent(event);

    expect(result?.key).toBe('Z');
  });

  it('should handle function keys', () => {
    const event = createKeyboardEvent({ key: 'F12', altKey: true });
    const result = parseKeyboardEvent(event);

    expect(result).toEqual({
      key: 'F12',
      ctrlKey: false,
      shiftKey: false,
      altKey: true
    });
  });
});

// ============================================================================
// matchesHotkey Tests
// ============================================================================

describe('matchesHotkey', () => {
  it('should match exact key combination', () => {
    const event = createKeyboardEvent({ key: 'S', ctrlKey: true });
    const config = createHotkeyConfig({ key: 'S', ctrlKey: true });

    expect(matchesHotkey(event, config)).toBe(true);
  });

  it('should not match different key', () => {
    const event = createKeyboardEvent({ key: 'A', ctrlKey: true });
    const config = createHotkeyConfig({ key: 'S', ctrlKey: true });

    expect(matchesHotkey(event, config)).toBe(false);
  });

  it('should not match different modifiers', () => {
    const event = createKeyboardEvent({ key: 'S', ctrlKey: true });
    const config = createHotkeyConfig({ key: 'S', ctrlKey: true, shiftKey: true });

    expect(matchesHotkey(event, config)).toBe(false);
  });

  it('should be case-insensitive for single characters', () => {
    const event = createKeyboardEvent({ key: 's', ctrlKey: true });
    const config = createHotkeyConfig({ key: 'S', ctrlKey: true });

    expect(matchesHotkey(event, config)).toBe(true);
  });

  it('should match complex combinations', () => {
    const event = createKeyboardEvent({ 
      key: 'P', 
      ctrlKey: true, 
      shiftKey: true, 
      altKey: false 
    });
    const config = createHotkeyConfig({ 
      key: 'P', 
      ctrlKey: true, 
      shiftKey: true, 
      altKey: false 
    });

    expect(matchesHotkey(event, config)).toBe(true);
  });

  it('should not match when extra modifier is pressed', () => {
    const event = createKeyboardEvent({ 
      key: 'S', 
      ctrlKey: true, 
      altKey: true 
    });
    const config = createHotkeyConfig({ key: 'S', ctrlKey: true });

    expect(matchesHotkey(event, config)).toBe(false);
  });
});

// ============================================================================
// hotkeysEqual Tests
// ============================================================================

describe('hotkeysEqual', () => {
  it('should return true for identical configs', () => {
    const a = createHotkeyConfig({ key: 'S', ctrlKey: true });
    const b = createHotkeyConfig({ key: 'S', ctrlKey: true });

    expect(hotkeysEqual(a, b)).toBe(true);
  });

  it('should return false for different keys', () => {
    const a = createHotkeyConfig({ key: 'S', ctrlKey: true });
    const b = createHotkeyConfig({ key: 'A', ctrlKey: true });

    expect(hotkeysEqual(a, b)).toBe(false);
  });

  it('should return false for different modifiers', () => {
    const a = createHotkeyConfig({ key: 'S', ctrlKey: true });
    const b = createHotkeyConfig({ key: 'S', ctrlKey: true, shiftKey: true });

    expect(hotkeysEqual(a, b)).toBe(false);
  });

  it('should be case-insensitive', () => {
    const a = createHotkeyConfig({ key: 's' });
    const b = createHotkeyConfig({ key: 'S' });

    expect(hotkeysEqual(a, b)).toBe(true);
  });

  it('should compare all modifier keys', () => {
    const base = { key: 'X', ctrlKey: true, shiftKey: true, altKey: true };
    const a = createHotkeyConfig(base);
    const b = createHotkeyConfig(base);

    expect(hotkeysEqual(a, b)).toBe(true);

    // Change one modifier
    const c = createHotkeyConfig({ ...base, altKey: false });
    expect(hotkeysEqual(a, c)).toBe(false);
  });
});

// ============================================================================
// HOTKEY_ACTIONS Tests
// ============================================================================

describe('HOTKEY_ACTIONS', () => {
  it('should have all required actions', () => {
    const actionIds = HOTKEY_ACTIONS.map(a => a.id);

    expect(actionIds).toContain('toggleStrategy');
    expect(actionIds).toContain('openSettings');
    expect(actionIds).toContain('quickTest');
    expect(actionIds).toContain('stopAll');
  });

  it('should have label and description for each action', () => {
    HOTKEY_ACTIONS.forEach(action => {
      expect(action.label).toBeTruthy();
      expect(action.description).toBeTruthy();
      expect(typeof action.label).toBe('string');
      expect(typeof action.description).toBe('string');
    });
  });

  it('should have unique action IDs', () => {
    const ids = HOTKEY_ACTIONS.map(a => a.id);
    const uniqueIds = new Set(ids);

    expect(uniqueIds.size).toBe(ids.length);
  });
});

// ============================================================================
// hotkeysStore Tests
// ============================================================================

describe('hotkeysStore', () => {
  beforeEach(() => {
    // Reset to defaults before each test
    hotkeysStore.resetToDefaults();
  });

  describe('get', () => {
    it('should return current hotkeys state', () => {
      const state = hotkeysStore.get();

      expect(state).toHaveProperty('toggleStrategy');
      expect(state).toHaveProperty('openSettings');
      expect(state).toHaveProperty('quickTest');
      expect(state).toHaveProperty('stopAll');
    });

    it('should return default hotkeys initially', () => {
      const state = hotkeysStore.get();

      // Check default for toggleStrategy (Ctrl+Shift+P)
      expect(state.toggleStrategy.key).toBe('P');
      expect(state.toggleStrategy.ctrlKey).toBe(true);
      expect(state.toggleStrategy.shiftKey).toBe(true);
    });
  });

  describe('subscribe', () => {
    it('should call callback immediately with current state', () => {
      const callback = vi.fn();
      const unsubscribe = hotkeysStore.subscribe(callback);

      expect(callback).toHaveBeenCalledTimes(1);
      expect(callback).toHaveBeenCalledWith(expect.objectContaining({
        toggleStrategy: expect.any(Object)
      }));

      unsubscribe();
    });

    it('should call callback when state changes', () => {
      const callback = vi.fn();
      const unsubscribe = hotkeysStore.subscribe(callback);

      callback.mockClear();
      hotkeysStore.setHotkey('toggleStrategy', createHotkeyConfig({ key: 'X', ctrlKey: true }));

      expect(callback).toHaveBeenCalledTimes(1);

      unsubscribe();
    });

    it('should not call callback after unsubscribe', () => {
      const callback = vi.fn();
      const unsubscribe = hotkeysStore.subscribe(callback);

      unsubscribe();
      callback.mockClear();

      hotkeysStore.setHotkey('toggleStrategy', createHotkeyConfig({ key: 'Y' }));

      expect(callback).not.toHaveBeenCalled();
    });
  });

  describe('setHotkey', () => {
    it('should update specific hotkey', () => {
      const newConfig = createHotkeyConfig({ key: 'Z', ctrlKey: true, altKey: true });
      hotkeysStore.setHotkey('quickTest', newConfig);

      const state = hotkeysStore.get();
      expect(state.quickTest).toEqual(newConfig);
    });

    it('should not affect other hotkeys', () => {
      const originalState = hotkeysStore.get();
      const originalToggle = { ...originalState.toggleStrategy };

      hotkeysStore.setHotkey('quickTest', createHotkeyConfig({ key: 'X' }));

      const newState = hotkeysStore.get();
      expect(newState.toggleStrategy).toEqual(originalToggle);
    });
  });

  describe('resetToDefaults', () => {
    it('should reset all hotkeys to defaults', () => {
      // Change some hotkeys
      hotkeysStore.setHotkey('toggleStrategy', createHotkeyConfig({ key: 'X' }));
      hotkeysStore.setHotkey('quickTest', createHotkeyConfig({ key: 'Y' }));

      // Reset
      hotkeysStore.resetToDefaults();

      const state = hotkeysStore.get();
      expect(state.toggleStrategy.key).toBe('P');
      expect(state.toggleStrategy.ctrlKey).toBe(true);
      expect(state.toggleStrategy.shiftKey).toBe(true);
    });
  });

  describe('resetHotkey', () => {
    it('should reset specific hotkey to default', () => {
      hotkeysStore.setHotkey('toggleStrategy', createHotkeyConfig({ key: 'X' }));
      hotkeysStore.resetHotkey('toggleStrategy');

      const state = hotkeysStore.get();
      expect(state.toggleStrategy.key).toBe('P');
    });

    it('should not affect other hotkeys', () => {
      hotkeysStore.setHotkey('toggleStrategy', createHotkeyConfig({ key: 'X' }));
      hotkeysStore.setHotkey('quickTest', createHotkeyConfig({ key: 'Y' }));

      hotkeysStore.resetHotkey('toggleStrategy');

      const state = hotkeysStore.get();
      expect(state.quickTest.key).toBe('Y');
    });
  });

  describe('hasConflict', () => {
    it('should detect conflict with another action', () => {
      // Set quickTest to same as toggleStrategy default
      const toggleDefault = hotkeysStore.getDefault('toggleStrategy');
      
      const conflict = hotkeysStore.hasConflict('quickTest', toggleDefault);
      expect(conflict).toBe('toggleStrategy');
    });

    it('should return null when no conflict', () => {
      const uniqueConfig = createHotkeyConfig({ 
        key: 'Z', 
        ctrlKey: true, 
        altKey: true, 
        shiftKey: true 
      });

      const conflict = hotkeysStore.hasConflict('quickTest', uniqueConfig);
      expect(conflict).toBeNull();
    });

    it('should not report conflict with self', () => {
      const currentConfig = hotkeysStore.get().toggleStrategy;
      const conflict = hotkeysStore.hasConflict('toggleStrategy', currentConfig);

      expect(conflict).toBeNull();
    });
  });

  describe('getDefault', () => {
    it('should return default config for action', () => {
      const defaultToggle = hotkeysStore.getDefault('toggleStrategy');

      expect(defaultToggle.key).toBe('P');
      expect(defaultToggle.ctrlKey).toBe(true);
      expect(defaultToggle.shiftKey).toBe(true);
      expect(defaultToggle.altKey).toBe(false);
    });

    it('should return different defaults for different actions', () => {
      const toggleDefault = hotkeysStore.getDefault('toggleStrategy');
      const settingsDefault = hotkeysStore.getDefault('openSettings');

      expect(toggleDefault.key).not.toBe(settingsDefault.key);
    });
  });
});
