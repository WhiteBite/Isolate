/**
 * Failover API — управление автоматическим переключением стратегий
 * 
 * Предоставляет функции для:
 * - Получения и изменения конфигурации failover
 * - Мониторинга статуса failover
 * - Ручного переключения на backup стратегию
 * - Управления списком backup стратегий
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

/**
 * Конфигурация Auto Failover
 */
export interface FailoverConfig {
  /** Максимальное количество сбоев до переключения */
  maxFailures: number;
  /** Время ожидания (cooldown) в секундах перед повторной попыткой */
  cooldownSecs: number;
  /** Список backup стратегий в порядке приоритета */
  backupStrategies: string[];
}

/**
 * Статус Auto Failover
 */
export interface FailoverStatus {
  /** Включен ли auto failover */
  enabled: boolean;
  /** Текущее количество сбоев */
  failureCount: number;
  /** Максимальное количество сбоев до переключения */
  maxFailures: number;
  /** ID текущей стратегии */
  currentStrategy: string | null;
  /** ID следующей backup стратегии (если есть) */
  nextBackup: string | null;
  /** Время до окончания cooldown (в секундах) */
  cooldownRemaining: number | null;
  /** Причина последнего сбоя */
  lastFailureReason: string | null;
}

// ============================================================================
// API Functions
// ============================================================================

/**
 * Получить текущий статус failover
 * 
 * @returns Текущий статус failover
 */
export async function getFailoverStatus(): Promise<FailoverStatus> {
  return invoke<FailoverStatus>('get_failover_status');
}

/**
 * Включить или выключить auto failover
 * 
 * @param enabled - true для включения, false для выключения
 */
export async function setFailoverEnabled(enabled: boolean): Promise<void> {
  return invoke('set_failover_enabled', { enabled });
}

/**
 * Получить конфигурацию failover
 * 
 * @returns Текущая конфигурация
 */
export async function getFailoverConfig(): Promise<FailoverConfig> {
  return invoke<FailoverConfig>('get_failover_config');
}

/**
 * Обновить конфигурацию failover
 * 
 * @param maxFailures - Количество сбоев до переключения (1-10)
 * @param cooldownSecs - Время ожидания в секундах (10-300)
 */
export async function setFailoverConfig(
  maxFailures: number,
  cooldownSecs: number
): Promise<void> {
  return invoke('set_failover_config', { maxFailures, cooldownSecs });
}

/**
 * Выполнить ручное переключение на backup стратегию
 * 
 * Форсирует немедленное переключение на следующую доступную
 * backup стратегию, независимо от количества сбоев.
 * 
 * @returns ID backup стратегии или null если нет доступных
 */
export async function triggerManualFailover(): Promise<string | null> {
  return invoke<string | null>('trigger_manual_failover');
}

/**
 * Получить список learned стратегий
 * 
 * Learned стратегии — это стратегии, которые успешно работали
 * в прошлом и могут использоваться как backup.
 * 
 * @returns Список ID стратегий
 */
export async function getLearnedStrategies(): Promise<string[]> {
  return invoke<string[]>('get_learned_strategies');
}

/**
 * Сбросить состояние failover для стратегии
 * 
 * Очищает счётчик сбоев и список попробованных стратегий.
 * 
 * @param strategyId - ID стратегии для сброса
 */
export async function resetFailoverState(strategyId: string): Promise<void> {
  return invoke('reset_failover_state', { strategyId });
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Проверить, включен ли failover
 */
export async function isFailoverEnabled(): Promise<boolean> {
  const status = await getFailoverStatus();
  return status.enabled;
}

/**
 * Получить прогресс до failover (0-100%)
 */
export function getFailoverProgress(status: FailoverStatus): number {
  if (status.maxFailures === 0) return 0;
  return Math.min(100, (status.failureCount / status.maxFailures) * 100);
}

/**
 * Форматировать cooldown в читаемый вид
 */
export function formatCooldown(seconds: number | null): string {
  if (seconds === null || seconds <= 0) return '';
  
  if (seconds < 60) {
    return `${seconds}s`;
  }
  
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  
  if (secs === 0) {
    return `${mins}m`;
  }
  
  return `${mins}m ${secs}s`;
}

/**
 * Получить цвет статуса failover
 */
export function getFailoverStatusColor(status: FailoverStatus): 'green' | 'yellow' | 'red' | 'gray' {
  if (!status.enabled) return 'gray';
  
  const progress = getFailoverProgress(status);
  
  if (progress === 0) return 'green';
  if (progress < 66) return 'yellow';
  return 'red';
}
