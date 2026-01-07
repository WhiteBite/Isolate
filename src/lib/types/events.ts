/**
 * Tauri Event Type Definitions
 * 
 * Provides type-safe event handling for all Tauri IPC events.
 * 
 * Usage:
 * ```typescript
 * import { listen } from '@tauri-apps/api/event';
 * import type { TauriEvents, OptimizationProgressPayload } from '$lib/types/events';
 * 
 * const unlisten = await listen<OptimizationProgressPayload>('optimization:progress', (event) => {
 *   console.log(event.payload.stage, event.payload.percent);
 * });
 * ```
 */

// ============================================================================
// Optimization Events
// ============================================================================

export interface OptimizationProgressPayload {
  /** Current optimization stage */
  stage: string;
  /** Progress percentage (0-100) */
  percent: number;
  /** Human-readable message */
  message: string;
  /** Currently testing strategy ID */
  current_strategy?: string;
}

export interface OptimizationCompletePayload {
  /** Best strategy ID */
  strategy_id: string;
  /** Best strategy display name */
  strategy_name: string;
  /** Best strategy score */
  score: number;
}

export type OptimizationFailedPayload = string;

// ============================================================================
// Strategy Events
// ============================================================================

export interface StrategyAppliedPayload {
  /** Applied strategy ID */
  strategy_id: string;
  /** Applied strategy display name */
  strategy_name: string;
}

export interface StrategyStoppedPayload {
  // Empty payload
}

export interface StrategyTestResultPayload {
  /** Strategy ID */
  strategy_id: string;
  /** Whether test was successful */
  success: boolean;
  /** Strategy score */
  score: number;
  /** Latency in milliseconds */
  latency_ms: number;
}

export interface StrategyDegradedPayload {
  /** Strategy ID that degraded */
  strategy_id: string;
  /** Reason for degradation */
  reason: string;
  /** Suggested action */
  action?: 'restart' | 'switch' | 'notify';
}

// ============================================================================
// Test Events
// ============================================================================

export type TestStageStatus = 'pending' | 'running' | 'done' | 'error';

export interface TestProgressPayload {
  /** Current test stage */
  stage: string;
  /** Progress percentage (0-100) */
  percent: number;
  /** Human-readable message */
  message: string;
}

export interface TestResultPayload {
  /** Service ID */
  service_id: string;
  /** Service name */
  service_name: string;
  /** Whether test passed */
  success: boolean;
  /** Latency in milliseconds */
  latency_ms?: number;
  /** Error message if failed */
  error?: string;
}

export interface TestCompletePayload {
  /** Total tests run */
  total: number;
  /** Passed tests */
  passed: number;
  /** Failed tests */
  failed: number;
  /** Total duration in milliseconds */
  duration_ms: number;
}

export interface TestStagePayload {
  /** Stage ID */
  stage_id: string;
  /** Stage status */
  status: TestStageStatus;
  /** Error message if status is 'error' */
  error?: string;
}

// ============================================================================
// Download Events
// ============================================================================

export interface DownloadProgressPayload {
  /** File being downloaded */
  file: string;
  /** Downloaded bytes */
  downloaded: number;
  /** Total bytes */
  total: number;
  /** Progress percentage (0-100) */
  percent: number;
}

export interface DownloadCompletePayload {
  /** File that was downloaded */
  file: string;
  /** Path where file was saved */
  path: string;
}

export interface DownloadErrorPayload {
  /** File that failed to download */
  file: string;
  /** Error message */
  error: string;
}

// ============================================================================
// Service Events
// ============================================================================

export interface ServiceStatusPayload {
  /** Service ID */
  service_id: string;
  /** Service status */
  status: 'available' | 'blocked' | 'unknown' | 'checking';
  /** Latency in milliseconds */
  latency_ms?: number;
}

export interface ServiceCheckCompletePayload {
  /** Total services checked */
  total: number;
  /** Available services */
  available: number;
  /** Blocked services */
  blocked: number;
}

// ============================================================================
// System Events
// ============================================================================

export interface SystemErrorPayload {
  /** Error code */
  code: string;
  /** Error message */
  message: string;
  /** Additional context */
  context?: Record<string, unknown>;
}

export interface SystemLogPayload {
  /** Log level */
  level: 'debug' | 'info' | 'warn' | 'error';
  /** Log message */
  message: string;
  /** Timestamp */
  timestamp: string;
  /** Source module */
  source?: string;
}

// ============================================================================
// Event Name Constants
// ============================================================================

export const TAURI_EVENTS = {
  // Optimization
  OPTIMIZATION_PROGRESS: 'optimization:progress',
  OPTIMIZATION_COMPLETE: 'optimization:complete',
  OPTIMIZATION_FAILED: 'optimization:failed',
  
  // Strategy
  STRATEGY_APPLIED: 'strategy:applied',
  STRATEGY_STOPPED: 'strategy:stopped',
  STRATEGY_TEST_RESULT: 'strategy:test_result',
  STRATEGY_DEGRADED: 'strategy:degraded',
  
  // Test
  TEST_PROGRESS: 'test:progress',
  TEST_RESULT: 'test:result',
  TEST_COMPLETE: 'test:complete',
  TEST_STAGE: 'test:stage',
  
  // Download
  DOWNLOAD_PROGRESS: 'download:progress',
  DOWNLOAD_COMPLETE: 'download:complete',
  DOWNLOAD_ERROR: 'download:error',
  
  // Service
  SERVICE_STATUS: 'service:status',
  SERVICE_CHECK_COMPLETE: 'service:check_complete',
  
  // System
  SYSTEM_ERROR: 'system:error',
  SYSTEM_LOG: 'system:log',
} as const;

export type TauriEventName = typeof TAURI_EVENTS[keyof typeof TAURI_EVENTS];

// ============================================================================
// Event Payload Map
// ============================================================================

export interface TauriEventPayloadMap {
  'optimization:progress': OptimizationProgressPayload;
  'optimization:complete': OptimizationCompletePayload;
  'optimization:failed': OptimizationFailedPayload;
  
  'strategy:applied': StrategyAppliedPayload;
  'strategy:stopped': StrategyStoppedPayload;
  'strategy:test_result': StrategyTestResultPayload;
  'strategy:degraded': StrategyDegradedPayload;
  
  'test:progress': TestProgressPayload;
  'test:result': TestResultPayload;
  'test:complete': TestCompletePayload;
  'test:stage': TestStagePayload;
  
  'download:progress': DownloadProgressPayload;
  'download:complete': DownloadCompletePayload;
  'download:error': DownloadErrorPayload;
  
  'service:status': ServiceStatusPayload;
  'service:check_complete': ServiceCheckCompletePayload;
  
  'system:error': SystemErrorPayload;
  'system:log': SystemLogPayload;
}

// ============================================================================
// Type-safe Event Listener Helper
// ============================================================================

import type { Event, UnlistenFn } from '@tauri-apps/api/event';

/**
 * Type-safe event listener
 * 
 * @example
 * ```typescript
 * import { listenTyped } from '$lib/types/events';
 * 
 * const unlisten = await listenTyped('optimization:progress', (payload) => {
 *   // payload is typed as OptimizationProgressPayload
 *   console.log(payload.stage, payload.percent);
 * });
 * ```
 */
export async function listenTyped<K extends keyof TauriEventPayloadMap>(
  event: K,
  handler: (payload: TauriEventPayloadMap[K]) => void
): Promise<UnlistenFn> {
  const { listen } = await import('@tauri-apps/api/event');
  return listen<TauriEventPayloadMap[K]>(event, (e: Event<TauriEventPayloadMap[K]>) => {
    handler(e.payload);
  });
}
