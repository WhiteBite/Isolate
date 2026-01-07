import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  ProxyTester,
  sortByTestResults,
  getLatencyColor,
  formatLatency,
  type ProxyTestState,
  type ProxyTestStatus
} from '../proxyTester';
import type { ProxyConfig, ProxyTestResult } from '$lib/api/types';

// ============================================================================
// Test Helpers
// ============================================================================

function createMockProxy(id: string, name: string = `Proxy ${id}`): ProxyConfig {
  return {
    id,
    name,
    protocol: 'vless',
    server: 'test.com',
    port: 443,
    tls: true,
    custom_fields: {},
    active: false
  };
}

function createMockTestState(
  proxyId: string,
  status: ProxyTestStatus,
  latency: number | null = null
): ProxyTestState {
  return {
    proxyId,
    proxyName: `Proxy ${proxyId}`,
    latency,
    status
  };
}

// ============================================================================
// ProxyTester Class Tests
// ============================================================================

describe('ProxyTester', () => {
  let tester: ProxyTester;

  beforeEach(() => {
    tester = new ProxyTester();
  });

  afterEach(() => {
    tester.cancel();
  });

  describe('testAll', () => {
    it('should test all proxies and return results', async () => {
      const proxies = [
        createMockProxy('1'),
        createMockProxy('2'),
        createMockProxy('3')
      ];

      const testFn = vi.fn().mockImplementation(async (id: string): Promise<ProxyTestResult> => ({
        success: true,
        latency: parseInt(id) * 100
      }));

      const results = await tester.testAll(proxies, testFn);

      expect(results).toHaveLength(3);
      expect(testFn).toHaveBeenCalledTimes(3);
      expect(results.every(r => r.status === 'success')).toBe(true);
    });

    it('should handle failed tests correctly', async () => {
      const proxies = [createMockProxy('1')];

      const testFn = vi.fn().mockResolvedValue({
        success: false,
        error: 'Connection timeout'
      });

      const results = await tester.testAll(proxies, testFn);

      expect(results[0].status).toBe('failed');
      expect(results[0].error).toBe('Connection timeout');
    });

    it('should respect concurrency limit', async () => {
      const proxies = Array.from({ length: 10 }, (_, i) => createMockProxy(String(i)));
      let concurrentCount = 0;
      let maxConcurrent = 0;

      const testFn = vi.fn().mockImplementation(async (): Promise<ProxyTestResult> => {
        concurrentCount++;
        maxConcurrent = Math.max(maxConcurrent, concurrentCount);
        await new Promise(r => setTimeout(r, 50));
        concurrentCount--;
        return { success: true, latency: 100 };
      });

      await tester.testAll(proxies, testFn, { concurrency: 3 });

      expect(maxConcurrent).toBeLessThanOrEqual(3);
    });

    it('should call onResult callback for each test', async () => {
      const proxies = [createMockProxy('1'), createMockProxy('2')];
      const onResult = vi.fn();

      const testFn = vi.fn().mockResolvedValue({ success: true, latency: 100 });

      await tester.testAll(proxies, testFn, { onResult });

      // Called twice per proxy: once for 'testing' status, once for final status
      expect(onResult).toHaveBeenCalledTimes(4);
    });

    it('should call onProgress callback with correct values', async () => {
      const proxies = [createMockProxy('1'), createMockProxy('2')];
      const onProgress = vi.fn();

      const testFn = vi.fn().mockResolvedValue({ success: true, latency: 100 });

      await tester.testAll(proxies, testFn, { onProgress });

      // Initial call + one call per completed test
      expect(onProgress).toHaveBeenCalled();
      
      const lastCall = onProgress.mock.calls[onProgress.mock.calls.length - 1][0];
      expect(lastCall.total).toBe(2);
      expect(lastCall.completed).toBe(2);
      expect(lastCall.percent).toBe(100);
    });

    it('should throw error if already running', async () => {
      const proxies = [createMockProxy('1')];
      const testFn = vi.fn().mockImplementation(async () => {
        await new Promise(r => setTimeout(r, 100));
        return { success: true, latency: 100 };
      });

      // Start first test
      const firstTest = tester.testAll(proxies, testFn);

      // Try to start second test while first is running
      await expect(tester.testAll(proxies, testFn)).rejects.toThrow('Tester is already running');

      // Wait for first test to complete
      await firstTest;
    });

    it('should handle exceptions in testFn', async () => {
      const proxies = [createMockProxy('1')];
      const testFn = vi.fn().mockRejectedValue(new Error('Network error'));

      const results = await tester.testAll(proxies, testFn);

      expect(results[0].status).toBe('failed');
      expect(results[0].error).toBe('Network error');
    });
  });

  describe('cancel', () => {
    it('should cancel ongoing tests', async () => {
      const proxies = Array.from({ length: 10 }, (_, i) => createMockProxy(String(i)));
      let completedCount = 0;

      const testFn = vi.fn().mockImplementation(async (): Promise<ProxyTestResult> => {
        await new Promise(r => setTimeout(r, 100));
        completedCount++;
        return { success: true, latency: 100 };
      });

      const testPromise = tester.testAll(proxies, testFn, { concurrency: 2 });

      // Cancel after a short delay
      setTimeout(() => tester.cancel(), 50);

      const results = await testPromise;

      // Some tests should be cancelled
      const cancelledCount = results.filter(r => r.status === 'cancelled').length;
      expect(cancelledCount).toBeGreaterThan(0);
    });
  });

  describe('running', () => {
    it('should return true while tests are running', async () => {
      const proxies = [createMockProxy('1')];
      const testFn = vi.fn().mockImplementation(async () => {
        await new Promise(r => setTimeout(r, 50));
        return { success: true, latency: 100 };
      });

      expect(tester.running).toBe(false);

      const testPromise = tester.testAll(proxies, testFn);
      expect(tester.running).toBe(true);

      await testPromise;
      expect(tester.running).toBe(false);
    });
  });
});

// ============================================================================
// Utility Functions Tests
// ============================================================================

describe('sortByTestResults', () => {
  it('should sort proxies by latency (fastest first)', () => {
    const proxies = [
      createMockProxy('1'),
      createMockProxy('2'),
      createMockProxy('3')
    ];

    const results: ProxyTestState[] = [
      createMockTestState('1', 'success', 300),
      createMockTestState('2', 'success', 100),
      createMockTestState('3', 'success', 200)
    ];

    const sorted = sortByTestResults(proxies, results);

    expect(sorted[0].id).toBe('2'); // 100ms
    expect(sorted[1].id).toBe('3'); // 200ms
    expect(sorted[2].id).toBe('1'); // 300ms
  });

  it('should put failed proxies last', () => {
    const proxies = [
      createMockProxy('1'),
      createMockProxy('2'),
      createMockProxy('3')
    ];

    const results: ProxyTestState[] = [
      createMockTestState('1', 'failed'),
      createMockTestState('2', 'success', 100),
      createMockTestState('3', 'success', 200)
    ];

    const sorted = sortByTestResults(proxies, results);

    expect(sorted[0].id).toBe('2');
    expect(sorted[1].id).toBe('3');
    expect(sorted[2].id).toBe('1'); // failed - last
  });

  it('should handle proxies without results', () => {
    const proxies = [
      createMockProxy('1'),
      createMockProxy('2'),
      createMockProxy('3')
    ];

    const results: ProxyTestState[] = [
      createMockTestState('1', 'success', 100)
    ];

    const sorted = sortByTestResults(proxies, results);

    expect(sorted[0].id).toBe('1'); // has result
    // Proxies without results should be at the end
    expect(sorted.slice(1).map(p => p.id)).toContain('2');
    expect(sorted.slice(1).map(p => p.id)).toContain('3');
  });

  it('should keep original order for equal results', () => {
    const proxies = [
      createMockProxy('1'),
      createMockProxy('2')
    ];

    const results: ProxyTestState[] = [
      createMockTestState('1', 'failed'),
      createMockTestState('2', 'failed')
    ];

    const sorted = sortByTestResults(proxies, results);

    // Both failed, should maintain original order
    expect(sorted[0].id).toBe('1');
    expect(sorted[1].id).toBe('2');
  });

  it('should handle empty arrays', () => {
    const sorted = sortByTestResults([], []);
    expect(sorted).toEqual([]);
  });
});

describe('getLatencyColor', () => {
  it('should return gray for null latency', () => {
    expect(getLatencyColor(null)).toBe('text-zinc-500');
  });

  it('should return green for fast latency (<100ms)', () => {
    expect(getLatencyColor(50)).toBe('text-emerald-400');
    expect(getLatencyColor(99)).toBe('text-emerald-400');
  });

  it('should return amber for medium latency (100-299ms)', () => {
    expect(getLatencyColor(100)).toBe('text-amber-400');
    expect(getLatencyColor(200)).toBe('text-amber-400');
    expect(getLatencyColor(299)).toBe('text-amber-400');
  });

  it('should return red for slow latency (>=300ms)', () => {
    expect(getLatencyColor(300)).toBe('text-red-400');
    expect(getLatencyColor(500)).toBe('text-red-400');
    expect(getLatencyColor(1000)).toBe('text-red-400');
  });

  it('should handle edge cases', () => {
    expect(getLatencyColor(0)).toBe('text-emerald-400');
  });
});

describe('formatLatency', () => {
  it('should return dash for null latency', () => {
    expect(formatLatency(null)).toBe('—');
  });

  it('should format latency with ms suffix', () => {
    expect(formatLatency(100)).toBe('100ms');
    expect(formatLatency(0)).toBe('0ms');
    expect(formatLatency(1500)).toBe('1500ms');
  });

  it('should handle various numeric values', () => {
    expect(formatLatency(1)).toBe('1ms');
    expect(formatLatency(999)).toBe('999ms');
    expect(formatLatency(10000)).toBe('10000ms');
  });
});
