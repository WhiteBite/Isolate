import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { TestProgress, TestResult } from './types';

// ============================================================================
// Testing API Functions
// ============================================================================

/**
 * Run tests on proxies and/or strategies.
 * @param proxyIds - Array of proxy IDs to test
 * @param strategyIds - Array of strategy IDs to test
 * @param serviceIds - Array of service IDs to test against
 * @param mode - Testing mode: 'turbo' (fast) or 'deep' (thorough)
 */
export async function runTests(
    proxyIds: string[],
    strategyIds: string[],
    serviceIds: string[],
    mode: 'turbo' | 'deep'
): Promise<void> {
    return invoke('run_tests', { proxyIds, strategyIds, serviceIds, mode });
}

/**
 * Cancel running tests.
 */
export async function cancelTests(): Promise<void> {
    return invoke('cancel_tests');
}

// ============================================================================
// Testing Event Listeners
// ============================================================================

/**
 * Subscribe to test progress events.
 * @param callback - Function to call on progress update
 * @returns Unsubscribe function
 */
export function onTestProgress(callback: (progress: TestProgress) => void): Promise<UnlistenFn> {
    return listen('test:progress', (event) => {
        callback(event.payload as TestProgress);
    });
}

/**
 * Subscribe to individual test result events.
 * @param callback - Function to call when a test completes
 * @returns Unsubscribe function
 */
export function onTestResult(callback: (result: TestResult) => void): Promise<UnlistenFn> {
    return listen('test:result', (event) => {
        callback(event.payload as TestResult);
    });
}

/**
 * Subscribe to test completion event.
 * @param callback - Function to call when all tests complete
 * @returns Unsubscribe function
 */
export function onTestComplete(callback: (results: TestResult[]) => void): Promise<UnlistenFn> {
    return listen('test:complete', (event) => {
        callback(event.payload as TestResult[]);
    });
}
