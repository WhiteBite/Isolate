/**
 * A/B Testing API
 * 
 * API для сравнения эффективности двух стратегий.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ============================================================================
// Types
// ============================================================================

/** Статус A/B теста */
export type ABTestStatus = 'pending' | 'running' | 'completed' | 'cancelled' | 'failed';

/** Результат тестирования одной стратегии */
export interface ABTestStrategyResult {
    strategy_id: string;
    strategy_name: string;
    success_rate: number;
    avg_latency_ms: number;
    min_latency_ms: number;
    max_latency_ms: number;
    total_tests: number;
    successful_tests: number;
    failed_tests: number;
    latencies: number[];
    errors: string[];
}

/** Конфигурация A/B теста */
export interface ABTest {
    id: string;
    strategy_a: string;
    strategy_b: string;
    service_id: string;
    iterations: number;
    status: ABTestStatus;
    progress: number;
    current_iteration: number;
    current_strategy: string;
    started_at: string | null;
    completed_at: string | null;
    error_message: string | null;
}

/** Полный результат A/B теста */
export interface ABTestResult {
    test_id: string;
    strategy_a_result: ABTestStrategyResult;
    strategy_b_result: ABTestStrategyResult;
    service_id: string;
    service_name: string;
    winner_id: string | null;
    success_rate_diff: number;
    latency_diff_ms: number;
    completed_at: string;
}

/** Прогресс A/B теста */
export interface ABTestProgress {
    test_id: string;
    status: ABTestStatus;
    progress: number;
    current_iteration: number;
    total_iterations: number;
    current_strategy: string;
    current_strategy_name: string;
}

// ============================================================================
// API Functions
// ============================================================================

/**
 * Запускает A/B тест двух стратегий
 * @param strategyAId - ID первой стратегии
 * @param strategyBId - ID второй стратегии
 * @param serviceId - ID сервиса для тестирования
 * @param iterations - Количество итераций (1-20)
 * @returns ID созданного теста
 */
export async function startABTest(
    strategyAId: string,
    strategyBId: string,
    serviceId: string,
    iterations: number
): Promise<string> {
    return invoke('start_ab_test', {
        strategyAId,
        strategyBId,
        serviceId,
        iterations
    });
}

/**
 * Получает статус A/B теста
 */
export async function getABTestStatus(testId: string): Promise<ABTest | null> {
    return invoke('get_ab_test_status', { testId });
}

/**
 * Получает прогресс A/B теста
 */
export async function getABTestProgress(testId: string): Promise<ABTestProgress | null> {
    return invoke('get_ab_test_progress', { testId });
}

/**
 * Получает результаты A/B теста
 */
export async function getABTestResults(testId: string): Promise<ABTestResult | null> {
    return invoke('get_ab_test_results', { testId });
}

/**
 * Отменяет A/B тест
 */
export async function cancelABTest(testId: string): Promise<void> {
    return invoke('cancel_ab_test', { testId });
}

/**
 * Получает список активных A/B тестов
 */
export async function getActiveABTests(): Promise<ABTest[]> {
    return invoke('get_active_ab_tests');
}

// ============================================================================
// Event Listeners
// ============================================================================

/**
 * Подписка на событие начала теста
 */
export function onABTestStarted(callback: (testId: string) => void): Promise<UnlistenFn> {
    return listen('ab_test:started', (event) => {
        callback(event.payload as string);
    });
}

/**
 * Подписка на событие завершения теста
 */
export function onABTestCompleted(callback: (result: ABTestResult) => void): Promise<UnlistenFn> {
    return listen('ab_test:completed', (event) => {
        callback(event.payload as ABTestResult);
    });
}

/**
 * Подписка на событие ошибки теста
 */
export function onABTestError(callback: (error: string) => void): Promise<UnlistenFn> {
    return listen('ab_test:error', (event) => {
        callback(event.payload as string);
    });
}
