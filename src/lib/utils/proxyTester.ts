/**
 * Parallel Proxy Tester Utility
 * 
 * Provides parallel proxy testing with concurrency control,
 * progress tracking, and cancellation support.
 */

import type { ProxyConfig, ProxyTestResult } from '$lib/api/types';

// ============================================================================
// Types
// ============================================================================

export type ProxyTestStatus = 'pending' | 'testing' | 'success' | 'failed' | 'cancelled';

export interface ProxyTestState {
  proxyId: string;
  proxyName: string;
  latency: number | null;
  status: ProxyTestStatus;
  error?: string;
}

export interface ProxyTesterProgress {
  total: number;
  completed: number;
  successful: number;
  failed: number;
  percent: number;
}

export interface ProxyTesterOptions {
  /** Maximum concurrent tests (default: 5) */
  concurrency?: number;
  /** Callback for each test result */
  onResult?: (result: ProxyTestState) => void;
  /** Callback for progress updates */
  onProgress?: (progress: ProxyTesterProgress) => void;
}

// ============================================================================
// Proxy Tester Class
// ============================================================================

export class ProxyTester {
  private abortController: AbortController | null = null;
  private isRunning = false;
  
  /**
   * Test multiple proxies in parallel with concurrency control.
   * 
   * @param proxies - Array of proxy configurations to test
   * @param testFn - Function to test a single proxy (usually from API)
   * @param options - Tester options
   * @returns Array of test results
   */
  async testAll(
    proxies: ProxyConfig[],
    testFn: (id: string) => Promise<ProxyTestResult>,
    options: ProxyTesterOptions = {}
  ): Promise<ProxyTestState[]> {
    const { concurrency = 5, onResult, onProgress } = options;
    
    if (this.isRunning) {
      throw new Error('Tester is already running');
    }
    
    this.isRunning = true;
    this.abortController = new AbortController();
    
    // Initialize results
    const results: ProxyTestState[] = proxies.map(p => ({
      proxyId: p.id,
      proxyName: p.name,
      latency: null,
      status: 'pending' as ProxyTestStatus,
    }));
    
    // Progress tracking
    let completed = 0;
    let successful = 0;
    let failed = 0;
    const total = proxies.length;
    
    const updateProgress = () => {
      onProgress?.({
        total,
        completed,
        successful,
        failed,
        percent: total > 0 ? Math.round((completed / total) * 100) : 0,
      });
    };
    
    // Initial progress
    updateProgress();
    
    // Create test queue
    const queue = [...proxies];
    const activeTests: Promise<void>[] = [];
    
    const runTest = async (proxy: ProxyConfig): Promise<void> => {
      const resultIndex = results.findIndex(r => r.proxyId === proxy.id);
      if (resultIndex === -1) return;
      
      // Check if cancelled
      if (this.abortController?.signal.aborted) {
        results[resultIndex].status = 'cancelled';
        return;
      }
      
      // Mark as testing
      results[resultIndex].status = 'testing';
      onResult?.(results[resultIndex]);
      
      try {
        const testResult = await testFn(proxy.id);
        
        // Check if cancelled during test
        if (this.abortController?.signal.aborted) {
          results[resultIndex].status = 'cancelled';
          return;
        }
        
        if (testResult.success && testResult.latency !== undefined) {
          results[resultIndex].status = 'success';
          results[resultIndex].latency = testResult.latency;
          successful++;
        } else {
          results[resultIndex].status = 'failed';
          results[resultIndex].error = testResult.error || 'Connection failed';
          failed++;
        }
      } catch (err) {
        // Check if cancelled
        if (this.abortController?.signal.aborted) {
          results[resultIndex].status = 'cancelled';
          return;
        }
        
        results[resultIndex].status = 'failed';
        results[resultIndex].error = err instanceof Error ? err.message : 'Unknown error';
        failed++;
      }
      
      completed++;
      updateProgress();
      onResult?.(results[resultIndex]);
    };
    
    // Process queue with concurrency limit
    const processQueue = async () => {
      while (queue.length > 0 && !this.abortController?.signal.aborted) {
        // Fill up to concurrency limit
        while (activeTests.length < concurrency && queue.length > 0) {
          const proxy = queue.shift()!;
          const testPromise = runTest(proxy).then(() => {
            // Remove from active tests when done
            const idx = activeTests.indexOf(testPromise);
            if (idx > -1) activeTests.splice(idx, 1);
          });
          activeTests.push(testPromise);
        }
        
        // Wait for at least one test to complete
        if (activeTests.length > 0) {
          await Promise.race(activeTests);
        }
      }
      
      // Wait for remaining tests
      await Promise.all(activeTests);
    };
    
    try {
      await processQueue();
    } finally {
      this.isRunning = false;
      this.abortController = null;
    }
    
    return results;
  }
  
  /**
   * Cancel ongoing tests.
   */
  cancel(): void {
    if (this.abortController) {
      this.abortController.abort();
    }
  }
  
  /**
   * Check if tester is currently running.
   */
  get running(): boolean {
    return this.isRunning;
  }
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Sort proxies by test results (fastest first, failed last).
 */
export function sortByTestResults(
  proxies: ProxyConfig[],
  results: ProxyTestState[]
): ProxyConfig[] {
  const resultMap = new Map(results.map(r => [r.proxyId, r]));
  
  return [...proxies].sort((a, b) => {
    const resultA = resultMap.get(a.id);
    const resultB = resultMap.get(b.id);
    
    // Both have no results - keep original order
    if (!resultA && !resultB) return 0;
    
    // One has no result - put it last
    if (!resultA) return 1;
    if (!resultB) return -1;
    
    // Both failed - keep original order
    if (resultA.status === 'failed' && resultB.status === 'failed') return 0;
    
    // One failed - put it last
    if (resultA.status === 'failed') return 1;
    if (resultB.status === 'failed') return -1;
    
    // Both have latency - sort by latency (fastest first)
    if (resultA.latency !== null && resultB.latency !== null) {
      return resultA.latency - resultB.latency;
    }
    
    // One has latency - put it first
    if (resultA.latency !== null) return -1;
    if (resultB.latency !== null) return 1;
    
    return 0;
  });
}

/**
 * Get latency color class based on value.
 */
export function getLatencyColor(latency: number | null): string {
  if (latency === null) return 'text-zinc-500';
  if (latency < 100) return 'text-emerald-400';
  if (latency < 300) return 'text-amber-400';
  return 'text-red-400';
}

/**
 * Format latency for display.
 */
export function formatLatency(latency: number | null): string {
  if (latency === null) return '—';
  return `${latency}ms`;
}

// ============================================================================
// Singleton Instance
// ============================================================================

/** Global proxy tester instance */
export const proxyTester = new ProxyTester();
