/**
 * AI Pilot Store - управление фоновой оптимизацией стратегий
 * 
 * Поддерживает два режима работы:
 * - Demo режим (браузер): симуляция для разработки
 * - Real режим (Tauri): интеграция с Rust backend
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { isTauriEnv, waitForBackend } from '$lib/utils/backend';
import { toasts } from '$lib/stores/toast';

// ============================================================================
// Types
// ============================================================================

export interface AIPilotAction {
  id: string;
  timestamp: Date;
  serviceId: string;
  serviceName: string;
  oldStrategy: string;
  newStrategy: string;
  reason: string;
  actionType: string;
  success: boolean;
}

export interface AIPilotState {
  enabled: boolean;
  interval: 30 | 60 | 120; // минуты
  lastCheck: Date | null;
  history: AIPilotAction[];
  isChecking: boolean;
}

// ============================================================================
// Backend Response Types (соответствуют Rust структурам в ai_pilot.rs)
// ============================================================================

/**
 * Rust: AiPilotStatus in commands/ai_pilot.rs
 */
interface BackendAIPilotStatus {
  is_running: boolean;
  started_at: number | null; // Unix timestamp ms
  checks_count: number;
  actions_count: number;
  last_check_at: number | null; // Unix timestamp ms
}

/**
 * Rust: AiPilotAction in commands/ai_pilot.rs
 */
interface BackendAIPilotAction {
  id: string;
  action_type: string; // "switch_strategy" | "restart" | "fallback"
  service_id: string;
  from_strategy: string | null;
  to_strategy: string | null;
  reason: string;
  timestamp: number; // Unix timestamp ms
  success: boolean;
}

/**
 * Rust: AiPilotHistory in commands/ai_pilot.rs
 */
interface BackendAIPilotHistory {
  actions: BackendAIPilotAction[];
  total_count: number;
}

/**
 * Event payload for ai_pilot:started
 */
interface AiPilotStartedEvent {
  started_at: number;
}

/**
 * Event payload for ai_pilot:stopped
 */
interface AiPilotStoppedEvent {
  stopped_at: number;
  total_checks: number;
  total_actions: number;
}

/**
 * Event payload for ai_pilot:action
 */
interface AiPilotActionEvent {
  action: BackendAIPilotAction;
}

// ============================================================================
// Tauri Detection (используем стандартные утилиты из backend.ts)
// ============================================================================

/**
 * Проверяет готовность backend перед вызовом команд
 * Использует retry логику для ожидания инициализации AppState
 */
async function isBackendReady(retries = 10): Promise<boolean> {
  return waitForBackend(retries, 200);
}

// ============================================================================
// Helpers
// ============================================================================

/**
 * Конвертирует backend action в frontend формат
 */
function convertBackendAction(action: BackendAIPilotAction): AIPilotAction {
  return {
    id: action.id,
    timestamp: new Date(action.timestamp),
    serviceId: action.service_id,
    // Используем service_id как имя, пока backend не передаёт имя
    serviceName: action.service_id,
    oldStrategy: action.from_strategy ?? 'Unknown',
    newStrategy: action.to_strategy ?? 'Unknown',
    reason: action.reason,
    actionType: action.action_type,
    success: action.success,
  };
}

// ============================================================================
// Store
// ============================================================================

class AIPilotStore {
  enabled = $state(false);
  interval = $state<30 | 60 | 120>(60);
  lastCheck = $state<Date | null>(null);
  history = $state<AIPilotAction[]>([]);
  isChecking = $state(false);
  
  // Backend stats
  checksCount = $state(0);
  actionsCount = $state(0);
  startedAt = $state<Date | null>(null);
  
  // Текущее уведомление (если есть)
  currentNotification = $state<AIPilotAction | null>(null);
  
  // Режим работы
  private _isTauriMode = $state(false);
  private checkTimer: ReturnType<typeof setInterval> | null = null;
  
  // Event listeners
  private unlisteners: UnlistenFn[] = [];

  constructor() {
    // Определяем режим при создании store
    this._isTauriMode = isTauriEnv();
  }

  // ============================================================================
  // Derived состояния
  // ============================================================================

  get isTauriMode() {
    return this._isTauriMode;
  }

  get statusText() {
    if (!this.enabled) return 'Выключен';
    if (this.isChecking) return 'Проверка...';
    return 'Активен';
  }

  get intervalText() {
    switch (this.interval) {
      case 30: return '30 минут';
      case 60: return '1 час';
      case 120: return '2 часа';
    }
  }

  get recentHistory() {
    return this.history.slice(0, 10);
  }

  // ============================================================================
  // Public API
  // ============================================================================

  /**
   * Запустить AI Pilot
   */
  async startAiPilot(): Promise<void> {
    if (this.enabled) return;
    
    try {
      if (this._isTauriMode) {
        const ready = await isBackendReady();
        if (!ready) {
          const errorMsg = 'Backend не готов. Попробуйте позже.';
          toasts.error(errorMsg);
          throw new Error('Backend not ready');
        }
        
        const status = await invoke<BackendAIPilotStatus>('start_ai_pilot');
        this.updateFromStatus(status);
        toasts.success('AI Pilot запущен');
      } else {
        // Demo mode
        console.log('[Demo] start_ai_pilot called with interval:', this.interval);
        this.enabled = true;
        this.startedAt = new Date();
        this.startDemoMonitoring();
        toasts.success('AI Pilot запущен (Demo режим)');
      }
    } catch (error) {
      console.error('Failed to start AI Pilot:', error);
      const errorMsg = error instanceof Error ? error.message : 'Неизвестная ошибка';
      if (!errorMsg.includes('Backend не готов')) {
        toasts.error(`Не удалось запустить AI Pilot: ${errorMsg}`);
      }
      throw error;
    }
  }

  /**
   * Остановить AI Pilot
   */
  async stopAiPilot(): Promise<void> {
    if (!this.enabled) return;
    
    try {
      if (this._isTauriMode) {
        const ready = await isBackendReady();
        if (!ready) {
          const errorMsg = 'Backend не готов. Попробуйте позже.';
          toasts.error(errorMsg);
          throw new Error('Backend not ready');
        }
        
        const status = await invoke<BackendAIPilotStatus>('stop_ai_pilot');
        this.updateFromStatus(status);
        toasts.success('AI Pilot остановлен');
      } else {
        // Demo mode
        console.log('[Demo] stop_ai_pilot called');
        this.enabled = false;
        this.stopDemoMonitoring();
        toasts.success('AI Pilot остановлен (Demo режим)');
      }
    } catch (error) {
      console.error('Failed to stop AI Pilot:', error);
      const errorMsg = error instanceof Error ? error.message : 'Неизвестная ошибка';
      if (!errorMsg.includes('Backend не готов')) {
        toasts.error(`Не удалось остановить AI Pilot: ${errorMsg}`);
      }
      throw error;
    }
  }

  /**
   * Переключить состояние AI Pilot
   */
  async toggle(): Promise<void> {
    if (this.enabled) {
      await this.stopAiPilot();
    } else {
      await this.startAiPilot();
    }
  }

  /**
   * Загрузить историю действий с backend
   */
  async fetchHistory(limit: number = 50): Promise<void> {
    try {
      if (this._isTauriMode) {
        const ready = await isBackendReady();
        if (!ready) {
          console.warn('[AI Pilot] Backend not ready for fetchHistory');
          return;
        }
        
        const result = await invoke<BackendAIPilotHistory>('get_ai_pilot_history', { limit });
        this.history = result.actions.map(convertBackendAction);
      } else {
        console.log('[Demo] get_ai_pilot_history called with limit:', limit);
        // Demo mode: keep existing history
      }
    } catch (error) {
      console.error('Failed to fetch AI Pilot history:', error);
      toasts.error('Не удалось загрузить историю AI Pilot');
    }
  }

  /**
   * Синхронизировать состояние с backend
   */
  async syncWithBackend(): Promise<void> {
    if (!this._isTauriMode) {
      console.log('[Demo] syncWithBackend called');
      return;
    }
    
    try {
      const ready = await isBackendReady();
      if (!ready) {
        console.warn('[AI Pilot] Backend not ready for sync');
        return;
      }
      
      const status = await invoke<BackendAIPilotStatus>('get_ai_pilot_status');
      this.updateFromStatus(status);
      
      // Загружаем историю
      await this.fetchHistory();
    } catch (error) {
      console.error('Failed to sync with backend:', error);
      toasts.error('Не удалось синхронизировать состояние AI Pilot');
    }
  }

  /**
   * Обновить состояние из backend status
   */
  private updateFromStatus(status: BackendAIPilotStatus) {
    this.enabled = status.is_running;
    this.checksCount = status.checks_count;
    this.actionsCount = status.actions_count;
    this.startedAt = status.started_at ? new Date(status.started_at) : null;
    this.lastCheck = status.last_check_at ? new Date(status.last_check_at) : null;
  }

  setInterval(value: 30 | 60 | 120) {
    this.interval = value;
    // Note: Backend currently doesn't support changing interval while running
    // If needed, restart AI Pilot with new interval
    if (this.enabled && !this._isTauriMode) {
      this.stopDemoMonitoring();
      this.startDemoMonitoring();
    }
  }

  /**
   * Запустить проверку вручную
   */
  async runCheck(): Promise<void> {
    if (this.isChecking) return;
    
    this.isChecking = true;
    this.lastCheck = new Date();
    
    try {
      if (this._isTauriMode) {
        // В Tauri режиме обновляем историю с backend
        // Backend сам выполняет проверки по расписанию
        await this.fetchHistory();
        
        // Также синхронизируем статус
        const ready = await isBackendReady();
        if (ready) {
          const status = await invoke<BackendAIPilotStatus>('get_ai_pilot_status');
          this.checksCount = status.checks_count;
          this.actionsCount = status.actions_count;
        }
        toasts.info('Проверка завершена');
      } else {
        // Demo режим: симуляция проверки
        await this.runDemoCheck();
      }
    } catch (error) {
      console.error('AI Pilot check failed:', error);
      toasts.error('Ошибка при проверке AI Pilot');
    } finally {
      this.isChecking = false;
    }
  }

  // ============================================================================
  // Event Listeners (Tauri mode)
  // ============================================================================

  /**
   * Настройка подписок на Tauri events
   */
  private async setupEventListeners() {
    if (!isTauriEnv()) return;
    
    try {
      // AI Pilot started event
      const unlistenStarted = await listen<AiPilotStartedEvent>('ai_pilot:started', (event) => {
        console.log('[AI Pilot] Received ai_pilot:started event');
        this.enabled = true;
        this.startedAt = new Date(event.payload.started_at);
        this.checksCount = 0;
        this.actionsCount = 0;
      });
      this.unlisteners.push(unlistenStarted);
      
      // AI Pilot stopped event
      const unlistenStopped = await listen<AiPilotStoppedEvent>('ai_pilot:stopped', (event) => {
        console.log('[AI Pilot] Received ai_pilot:stopped event');
        this.enabled = false;
        this.checksCount = event.payload.total_checks;
        this.actionsCount = event.payload.total_actions;
      });
      this.unlisteners.push(unlistenStopped);
      
      // AI Pilot action event
      const unlistenAction = await listen<AiPilotActionEvent>('ai_pilot:action', (event) => {
        console.log('[AI Pilot] Received ai_pilot:action event', event.payload);
        const action = convertBackendAction(event.payload.action);
        this.addAction(action);
        this.showNotification(action);
      });
      this.unlisteners.push(unlistenAction);
      
      console.log('[AI Pilot] Event listeners set up');
    } catch (e) {
      console.error('[AI Pilot] Failed to setup event listeners:', e);
    }
  }

  /**
   * Очистка подписок
   */
  private async cleanupEventListeners() {
    for (const unlisten of this.unlisteners) {
      unlisten();
    }
    this.unlisteners = [];
    console.log('[AI Pilot] Event listeners cleaned up');
  }

  // ============================================================================
  // Demo Monitoring (Demo mode only)
  // ============================================================================

  private startDemoMonitoring() {
    if (this.checkTimer) {
      clearInterval(this.checkTimer);
    }
    
    // Demo mode: симуляция проверок
    this.checkTimer = setInterval(() => {
      this.runDemoCheck();
    }, this.interval * 60 * 1000);
    
    // Первая проверка через 5 секунд после включения
    setTimeout(() => {
      if (this.enabled && !this._isTauriMode) {
        this.runDemoCheck();
      }
    }, 5000);
  }

  private stopDemoMonitoring() {
    if (this.checkTimer) {
      clearInterval(this.checkTimer);
      this.checkTimer = null;
    }
  }

  /**
   * Demo режим: симуляция проверки для разработки
   * Вызывается из runCheck() или из таймера
   */
  private async runDemoCheck() {
    // Если вызвано из таймера, устанавливаем флаги
    const wasChecking = this.isChecking;
    if (!wasChecking) {
      this.isChecking = true;
      this.lastCheck = new Date();
    }
    this.checksCount++;
    
    try {
      // Симуляция проверки
      await new Promise(r => setTimeout(r, 2000));
      
      // Симуляция случайного переключения (для демо)
      if (Math.random() > 0.7) {
        const demoServices = [
          { id: 'youtube', name: 'YouTube' },
          { id: 'discord', name: 'Discord' },
          { id: 'twitch', name: 'Twitch' },
        ];
        const demoStrategies = ['Fake TLS', 'Split TLS', 'Disorder', 'VLESS'];
        const demoReasons = [
          'Обнаружено снижение скорости на 40%',
          'Нестабильное соединение',
          'Обнаружена новая блокировка',
          'Оптимизация по результатам тестов',
        ];
        
        const service = demoServices[Math.floor(Math.random() * demoServices.length)];
        const oldStrategy = demoStrategies[Math.floor(Math.random() * demoStrategies.length)];
        let newStrategy = demoStrategies[Math.floor(Math.random() * demoStrategies.length)];
        while (newStrategy === oldStrategy) {
          newStrategy = demoStrategies[Math.floor(Math.random() * demoStrategies.length)];
        }
        
        const action: AIPilotAction = {
          id: crypto.randomUUID(),
          timestamp: new Date(),
          serviceId: service.id,
          serviceName: service.name,
          oldStrategy,
          newStrategy,
          reason: demoReasons[Math.floor(Math.random() * demoReasons.length)],
          actionType: 'switch_strategy',
          success: true,
        };
        
        this.addAction(action);
        this.showNotification(action);
        this.actionsCount++;
      }
    } catch (error) {
      console.error('AI Pilot demo check failed:', error);
    } finally {
      // Сбрасываем флаг только если мы его установили
      if (!wasChecking) {
        this.isChecking = false;
      }
    }
  }

  // ============================================================================
  // Actions
  // ============================================================================

  private addAction(action: AIPilotAction) {
    this.history = [action, ...this.history].slice(0, 50);
  }

  showNotification(action: AIPilotAction) {
    this.currentNotification = action;
  }

  dismissNotification() {
    this.currentNotification = null;
  }

  /**
   * Откатить действие AI Pilot
   */
  async undoAction(actionId: string): Promise<void> {
    const action = this.history.find(a => a.id === actionId);
    if (!action) return;
    
    try {
      if (this._isTauriMode) {
        const ready = await isBackendReady();
        if (!ready) {
          toasts.error('Backend не готов. Попробуйте позже.');
          throw new Error('Backend not ready');
        }
        
        const success = await invoke<boolean>('undo_ai_pilot_action', { actionId });
        if (!success) {
          throw new Error('Undo failed');
        }
        
        // Backend добавит undo action через событие ai_pilot:action
        // Но мы можем удалить оригинальное действие из UI
        this.history = this.history.filter(a => a.id !== actionId);
        toasts.success('Действие отменено');
      } else {
        // Demo mode
        console.log('[Demo] undo_ai_pilot_action called with id:', actionId);
        this.history = this.history.filter(a => a.id !== actionId);
        toasts.success('Действие отменено (Demo режим)');
      }
      
      this.dismissNotification();
      console.log('Action undone:', actionId);
    } catch (error) {
      console.error('Failed to undo action:', error);
      const errorMsg = error instanceof Error ? error.message : 'Неизвестная ошибка';
      if (!errorMsg.includes('Backend не готов')) {
        toasts.error(`Не удалось отменить действие: ${errorMsg}`);
      }
      throw error;
    }
  }

  clearHistory() {
    this.history = [];
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  /**
   * Инициализация store - вызвать при загрузке приложения
   */
  async init(): Promise<void> {
    this._isTauriMode = isTauriEnv();
    
    if (this._isTauriMode) {
      // Подписываемся на события
      await this.setupEventListeners();
      
      // Синхронизируем состояние с backend
      await this.syncWithBackend();
    }
    
    console.log(`[AI Pilot] Initialized in ${this._isTauriMode ? 'REAL' : 'DEMO'} mode`);
  }

  /**
   * Cleanup при уничтожении
   */
  async destroy() {
    this.stopDemoMonitoring();
    await this.cleanupEventListeners();
  }
}

export const aiPilotStore = new AIPilotStore();
