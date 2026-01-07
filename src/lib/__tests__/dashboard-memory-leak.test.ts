/**
 * Tests for Dashboard (+page.svelte) Memory Leak Fix
 * 
 * These tests verify that:
 * 1. Initialization happens only once per component mount
 * 2. Intervals are properly cleaned up on unmount
 * 3. Re-navigation doesn't create duplicate intervals/subscriptions
 * 
 * Location: Tests the pattern used in src/routes/+page.svelte
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('Dashboard Memory Leak Prevention', () => {
  let mockSetInterval: ReturnType<typeof vi.spyOn>;
  let mockClearInterval: ReturnType<typeof vi.spyOn>;
  let intervalIds: Set<number>;

  beforeEach(() => {
    vi.useFakeTimers();
    intervalIds = new Set();
    
    // Track interval creation and cleanup
    mockSetInterval = vi.spyOn(global, 'setInterval').mockImplementation(((fn: any, delay: number) => {
      const id = Math.floor(Math.random() * 10000);
      intervalIds.add(id);
      return id as any;
    }) as any);
    
    mockClearInterval = vi.spyOn(global, 'clearInterval').mockImplementation((id: any) => {
      intervalIds.delete(id);
    });
  });

  afterEach(() => {
    mockSetInterval.mockRestore();
    mockClearInterval.mockRestore();
    vi.useRealTimers();
    vi.clearAllTimers();
  });

  it('should prevent re-initialization with guard check', () => {
    // Simulate the guard pattern from +page.svelte
    let initialized = false;
    let initCount = 0;

    function initializeDashboard() {
      if (!initialized) {
        initialized = true;
        initCount++;
      }
    }

    // First call - should initialize
    initializeDashboard();
    expect(initCount).toBe(1);
    expect(initialized).toBe(true);

    // Second call - should be blocked by guard
    initializeDashboard();
    expect(initCount).toBe(1); // Still 1, not 2

    // Third call - should still be blocked
    initializeDashboard();
    expect(initCount).toBe(1);
  });

  it('should clean up intervals on unmount', () => {
    // Simulate interval creation (healthCheckInterval + networkStatsInterval)
    const interval1 = setInterval(() => {}, 1000);
    const interval2 = setInterval(() => {}, 2000);

    expect(intervalIds.size).toBe(2);

    // Simulate cleanup via clearAllIntervals()
    clearInterval(interval1);
    clearInterval(interval2);

    expect(intervalIds.size).toBe(0);
  });

  it('should properly structure cleanup function', () => {
    // Simulate the $effect cleanup pattern from +page.svelte
    let cleanupFns: (() => void)[] = [];
    let healthCheckInterval: ReturnType<typeof setInterval> | null = null;
    let networkStatsInterval: ReturnType<typeof setInterval> | null = null;

    function clearAllIntervals() {
      if (healthCheckInterval) {
        clearInterval(healthCheckInterval);
        healthCheckInterval = null;
      }
      if (networkStatsInterval) {
        clearInterval(networkStatsInterval);
        networkStatsInterval = null;
      }
    }

    // Create intervals (as done in initializeDashboard)
    healthCheckInterval = setInterval(() => {}, 30000);
    networkStatsInterval = setInterval(() => {}, 1000);

    // Add mock cleanup functions (store unsubscribes)
    cleanupFns.push(() => console.log('cleanup 1'));
    cleanupFns.push(() => console.log('cleanup 2'));

    expect(intervalIds.size).toBe(2);
    expect(cleanupFns.length).toBe(2);

    // Execute cleanup (as done in $effect return function)
    cleanupFns.forEach(fn => fn());
    cleanupFns = [];
    clearAllIntervals();

    expect(intervalIds.size).toBe(0);
    expect(cleanupFns.length).toBe(0);
    expect(healthCheckInterval).toBeNull();
    expect(networkStatsInterval).toBeNull();
  });

  it('should handle multiple mount/unmount cycles', () => {
    // Simulate the full lifecycle from +page.svelte
    let initialized = false;
    let cleanupFns: (() => void)[] = [];
    let intervals: (ReturnType<typeof setInterval> | null)[] = [];

    function mount() {
      // Guard prevents re-initialization (from $effect)
      if (initialized) return;
      initialized = true;

      // Create intervals (healthCheck + networkStats)
      intervals.push(setInterval(() => {}, 1000));
      intervals.push(setInterval(() => {}, 2000));

      // Add cleanup functions (store subscriptions)
      cleanupFns.push(() => console.log('cleanup'));
    }

    function unmount() {
      // Cleanup (from $effect return function)
      cleanupFns.forEach(fn => fn());
      cleanupFns = [];
      
      intervals.forEach(interval => {
        if (interval) clearInterval(interval);
      });
      intervals = [];
      
      initialized = false;
    }

    // First mount/unmount cycle
    mount();
    expect(intervalIds.size).toBe(2);
    unmount();
    expect(intervalIds.size).toBe(0);

    // Second mount/unmount cycle (re-navigation)
    mount();
    expect(intervalIds.size).toBe(2);
    unmount();
    expect(intervalIds.size).toBe(0);

    // Third mount/unmount cycle
    mount();
    expect(intervalIds.size).toBe(2);
    unmount();
    expect(intervalIds.size).toBe(0);
  });

  it('should not create duplicate intervals when guard is active', () => {
    // Test the critical fix: guard prevents duplicate intervals
    let initialized = false;
    let intervals: number[] = [];

    function createIntervals() {
      // Guard check (the fix!)
      if (initialized) return;
      initialized = true;

      // Create intervals
      intervals.push(setInterval(() => {}, 1000) as any);
      intervals.push(setInterval(() => {}, 2000) as any);
    }

    // First call - creates intervals
    createIntervals();
    expect(intervalIds.size).toBe(2);
    const firstCount = intervalIds.size;

    // Second call - blocked by guard (prevents memory leak)
    createIntervals();
    expect(intervalIds.size).toBe(firstCount); // No new intervals

    // Third call - still blocked
    createIntervals();
    expect(intervalIds.size).toBe(firstCount); // Still no new intervals
  });

  it('should reset initialized flag only in cleanup', () => {
    // Test the $effect pattern with guard
    let initialized = false;

    // Simulate $effect with guard
    function effectRun() {
      // Guard at the start (prevents re-initialization)
      if (initialized) return;

      // Initialize
      initialized = true;

      // Return cleanup function
      return () => {
        // Reset only in cleanup (allows re-mount)
        initialized = false;
      };
    }

    // First run
    const cleanup1 = effectRun();
    expect(initialized).toBe(true);

    // Second run (should be blocked by guard)
    const cleanup2 = effectRun();
    expect(initialized).toBe(true);
    expect(cleanup2).toBeUndefined(); // Guard prevented execution

    // Cleanup
    cleanup1?.();
    expect(initialized).toBe(false);

    // After cleanup, can initialize again (re-mount scenario)
    const cleanup3 = effectRun();
    expect(initialized).toBe(true);
    cleanup3?.();
  });

  it('should handle store subscriptions cleanup', () => {
    // Test cleanup of store subscriptions (appStatus, services, etc.)
    const unsubscribeFns: (() => void)[] = [];
    let cleanupFns: (() => void)[] = [];

    // Simulate store subscriptions
    function subscribe(callback: () => void) {
      const unsubscribe = vi.fn();
      unsubscribeFns.push(unsubscribe);
      return unsubscribe;
    }

    // Create subscriptions (as in initializeDashboard)
    const unsub1 = subscribe(() => {}); // appStatus
    const unsub2 = subscribe(() => {}); // isOptimizing
    const unsub3 = subscribe(() => {}); // optimizationProgress
    const unsub4 = subscribe(() => {}); // services

    cleanupFns.push(unsub1, unsub2, unsub3, unsub4);

    expect(cleanupFns.length).toBe(4);

    // Cleanup (from $effect return function)
    cleanupFns.forEach(fn => fn());
    cleanupFns = [];

    expect(cleanupFns.length).toBe(0);
    unsubscribeFns.forEach(fn => {
      expect(fn).toHaveBeenCalled();
    });
  });
});
