import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { libraryStore, type AccessMethod } from './library.svelte';

export type TroubleshootStep = 'select' | 'testing' | 'results';
export type StrategyTestStatus = 'waiting' | 'testing' | 'success' | 'failed' | 'skipped';

export interface StrategyTestState {
  id: string;
  name: string;
  status: StrategyTestStatus;
  latency: number | null;
  progress: number;
  error?: string;
}

export interface ServiceProblem {
  id: string;
  serviceId: string;
  serviceName: string;
  icon: string;
  description: string;
  category: 'video' | 'social' | 'gaming' | 'other';
}

// Backend event types
interface TroubleshootProgress {
  strategy_id: string;
  strategy_name: string;
  status: string;
  progress: number;
  latency_ms: number | null;
}

interface TroubleshootStrategyResult {
  strategy_id: string;
  strategy_name: string;
  success: boolean;
  latency_ms: number | null;
  error: string | null;
}

interface TroubleshootComplete {
  service_id: string;
  strategies_tested: TroubleshootStrategyResult[];
  best_strategy_id: string | null;
  best_strategy_name: string | null;
  best_latency_ms: number | null;
}

interface BackendServiceProblem {
  service_id: string;
  service_name: string;
  category: string;
}

// Service icons mapping
const serviceIcons: Record<string, string> = {
  youtube: '📺',
  discord: '💬',
  telegram: '✈️',
  twitter: '🐦',
  twitch: '🎮',
  steam: '🎮',
  instagram: '📷',
  facebook: '👤',
  whatsapp: '📱',
  default: '🌐'
};

// Service descriptions
const serviceDescriptions: Record<string, string> = {
  youtube: 'Видео тормозит или не загружается',
  discord: 'Голосовой чат не работает',
  telegram: 'Не подключается',
  twitter: 'Страницы не загружаются',
  twitch: 'Стримы буферизируются',
  steam: 'Медленная загрузка игр',
  instagram: 'Не загружаются фото и видео',
  facebook: 'Страницы не открываются',
  whatsapp: 'Сообщения не отправляются',
  default: 'Проблемы с доступом'
};

class TroubleshootStore {
  step = $state<TroubleshootStep>('select');
  selectedProblem = $state<ServiceProblem | null>(null);
  strategies = $state<StrategyTestState[]>([]);
  bestStrategy = $state<StrategyTestState | null>(null);
  isRunning = $state(false);
  error = $state<string | null>(null);
  applyError = $state<string | null>(null); // Ошибка применения результата
  
  // Available problems (loaded from backend)
  problems = $state<ServiceProblem[]>([]);
  problemsLoading = $state(false);
  
  // Existing method from Library (if any)
  existingMethod = $state<AccessMethod | null>(null);
  
  // Event listeners
  private unlisteners: UnlistenFn[] = [];

  constructor() {
    // Load problems on init
    this.loadProblems();
  }

  async loadProblems() {
    this.problemsLoading = true;
    
    try {
      // Check if Tauri is available
      if (!('__TAURI__' in window) && !('__TAURI_INTERNALS__' in window)) {
        // Demo mode - use mock data
        this.problems = this.getMockProblems();
        return;
      }
      
      const backendProblems = await invoke<BackendServiceProblem[]>('get_troubleshoot_problems');
      
      this.problems = backendProblems.map(p => ({
        id: `${p.service_id}-problem`,
        serviceId: p.service_id,
        serviceName: p.service_name,
        icon: serviceIcons[p.service_id] || serviceIcons.default,
        description: serviceDescriptions[p.service_id] || serviceDescriptions.default,
        category: p.category as ServiceProblem['category']
      }));
    } catch (e) {
      console.error('Failed to load problems:', e);
      // Fallback to mock data
      this.problems = this.getMockProblems();
    } finally {
      this.problemsLoading = false;
    }
  }

  private getMockProblems(): ServiceProblem[] {
    return [
      { id: 'youtube-slow', serviceId: 'youtube', serviceName: 'YouTube', icon: '📺', description: 'Видео тормозит или не загружается', category: 'video' },
      { id: 'discord-voice', serviceId: 'discord', serviceName: 'Discord', icon: '💬', description: 'Голосовой чат не работает', category: 'social' },
      { id: 'telegram-blocked', serviceId: 'telegram', serviceName: 'Telegram', icon: '✈️', description: 'Не подключается', category: 'social' },
      { id: 'twitter-blocked', serviceId: 'twitter', serviceName: 'Twitter/X', icon: '🐦', description: 'Страницы не загружаются', category: 'social' },
      { id: 'steam-slow', serviceId: 'steam', serviceName: 'Steam', icon: '🎮', description: 'Медленная загрузка игр', category: 'gaming' },
    ];
  }

  async selectProblem(problem: ServiceProblem) {
    // Блокируем повторный запуск если тест уже идёт
    if (this.isRunning) {
      console.warn('Test already running, ignoring selectProblem');
      return;
    }
    
    this.selectedProblem = problem;
    this.step = 'testing';
    this.error = null;
    this.applyError = null;
    
    // Подтягиваем существующий метод из Library, если он задан для этого сервиса
    this.existingMethod = this.getExistingMethodFromLibrary(problem.serviceId);
    
    await this.startTesting();
  }

  /**
   * Получает текущий метод доступа из Library для сервиса
   * @param serviceId ID сервиса
   * @returns AccessMethod или null если не найден или метод direct/auto
   */
  private getExistingMethodFromLibrary(serviceId: string): AccessMethod | null {
    const rule = libraryStore.rules.find(r => r.id === serviceId);
    if (!rule) return null;
    
    // Возвращаем только если это конкретная стратегия (не direct/auto)
    if (rule.currentMethod.type === 'strategy' || 
        rule.currentMethod.type === 'vless' || 
        rule.currentMethod.type === 'proxy') {
      return rule.currentMethod;
    }
    
    return null;
  }

  async startTesting() {
    if (!this.selectedProblem) return;
    
    this.isRunning = true;
    this.strategies = [];
    this.bestStrategy = null;
    this.error = null;
    
    // Check if Tauri is available
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    if (!isTauri) {
      // Demo mode - simulate testing
      await this.simulateTesting();
      return;
    }
    
    try {
      // Setup event listeners
      await this.setupEventListeners();
      
      // Call backend
      await invoke('troubleshoot_service', {
        serviceId: this.selectedProblem.serviceId
      });
      
    } catch (e) {
      console.error('Troubleshoot failed:', e);
      this.error = e instanceof Error ? e.message : String(e);
      this.isRunning = false;
      this.step = 'results';
    }
  }

  private async setupEventListeners() {
    // Clean up previous listeners
    await this.cleanupListeners();
    
    // Progress updates
    const unlistenProgress = await listen<TroubleshootProgress>('troubleshoot:progress', (event) => {
      const data = event.payload;
      
      // Find or create strategy entry
      const existingIndex = this.strategies.findIndex(s => s.id === data.strategy_id);
      
      const strategyState: StrategyTestState = {
        id: data.strategy_id,
        name: data.strategy_name,
        status: data.status as StrategyTestStatus,
        latency: data.latency_ms,
        progress: data.progress
      };
      
      if (existingIndex >= 0) {
        this.strategies[existingIndex] = strategyState;
        this.strategies = [...this.strategies]; // Trigger reactivity
      } else {
        this.strategies = [...this.strategies, strategyState];
      }
    });
    this.unlisteners.push(unlistenProgress);
    
    // Strategy result
    const unlistenResult = await listen<TroubleshootStrategyResult>('troubleshoot:strategy_result', (event) => {
      const data = event.payload;
      
      const existingIndex = this.strategies.findIndex(s => s.id === data.strategy_id);
      
      const strategyState: StrategyTestState = {
        id: data.strategy_id,
        name: data.strategy_name,
        status: data.success ? 'success' : 'failed',
        latency: data.latency_ms,
        progress: 100,
        error: data.error || undefined
      };
      
      if (existingIndex >= 0) {
        this.strategies[existingIndex] = strategyState;
        this.strategies = [...this.strategies];
      } else {
        this.strategies = [...this.strategies, strategyState];
      }
    });
    this.unlisteners.push(unlistenResult);
    
    // Complete
    const unlistenComplete = await listen<TroubleshootComplete>('troubleshoot:complete', (event) => {
      const data = event.payload;
      
      // Find best strategy
      if (data.best_strategy_id) {
        this.bestStrategy = this.strategies.find(s => s.id === data.best_strategy_id) || null;
      }
      
      this.isRunning = false;
      this.step = 'results';
      
      // Cleanup listeners
      this.cleanupListeners();
    });
    this.unlisteners.push(unlistenComplete);
  }

  private async cleanupListeners() {
    for (const unlisten of this.unlisteners) {
      unlisten();
    }
    this.unlisteners = [];
  }

  // Demo mode simulation
  private async simulateTesting() {
    const mockStrategies = [
      { id: 'fake_tls', name: 'Fake TLS' },
      { id: 'split_tls', name: 'Split TLS' },
      { id: 'multisplit', name: 'Multisplit' },
      { id: 'disorder', name: 'Disorder' },
    ];
    
    this.strategies = mockStrategies.map(s => ({
      id: s.id,
      name: s.name,
      status: 'waiting' as StrategyTestStatus,
      latency: null,
      progress: 0
    }));

    for (let i = 0; i < this.strategies.length; i++) {
      this.strategies[i].status = 'testing';
      this.strategies = [...this.strategies];
      
      // Simulate progress
      for (let p = 0; p <= 100; p += 25) {
        await new Promise(r => setTimeout(r, 150));
        this.strategies[i].progress = p;
        this.strategies = [...this.strategies];
      }
      
      // Random result (biased toward success for demo)
      const success = Math.random() > 0.25;
      this.strategies[i].status = success ? 'success' : 'failed';
      this.strategies[i].latency = success ? Math.floor(Math.random() * 150) + 30 : null;
      this.strategies = [...this.strategies];
      
      await new Promise(r => setTimeout(r, 200));
    }

    // Find best strategy
    const successful = this.strategies.filter(s => s.status === 'success');
    if (successful.length > 0) {
      this.bestStrategy = successful.reduce((a, b) => 
        (a.latency || 999) < (b.latency || 999) ? a : b
      );
    }

    this.isRunning = false;
    this.step = 'results';
  }

  async applyResult() {
    if (!this.bestStrategy || !this.selectedProblem) {
      console.warn('No best strategy or problem selected');
      return;
    }
    
    this.applyError = null;
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    if (isTauri) {
      try {
        await invoke('apply_troubleshoot_result', {
          serviceId: this.selectedProblem.serviceId,
          strategyId: this.bestStrategy.id
        });
        console.log('Applied strategy:', this.bestStrategy.id, 'for service:', this.selectedProblem.serviceId);
        
        // Обновляем правило в Library
        await this.updateLibraryRule();
        
        this.reset();
      } catch (e) {
        console.error('Failed to apply result:', e);
        this.applyError = e instanceof Error ? e.message : String(e);
        // НЕ сбрасываем состояние - пользователь может попробовать снова
      }
    } else {
      // Demo mode - обновляем Library и сбрасываем
      console.log('Demo: Applied strategy:', this.bestStrategy.id, 'for service:', this.selectedProblem.serviceId);
      await this.updateLibraryRule();
      this.reset();
    }
  }

  /**
   * Обновляет правило в Library после успешного применения стратегии
   */
  private async updateLibraryRule() {
    if (!this.bestStrategy || !this.selectedProblem) return;
    
    const newMethod: AccessMethod = {
      type: 'strategy',
      strategyId: this.bestStrategy.id,
      strategyName: this.bestStrategy.name
    };
    
    try {
      await libraryStore.setRuleMethod(this.selectedProblem.serviceId, newMethod);
      console.log('[Troubleshoot] Updated Library rule:', this.selectedProblem.serviceId, 'with method:', newMethod);
    } catch (e) {
      // Логируем ошибку, но не прерываем основной flow
      // Правило в backend уже применено через apply_troubleshoot_result
      console.warn('[Troubleshoot] Failed to update Library rule (non-critical):', e);
    }
  }

  reset() {
    this.cleanupListeners();
    this.step = 'select';
    this.selectedProblem = null;
    this.strategies = [];
    this.bestStrategy = null;
    this.isRunning = false;
    this.error = null;
    this.applyError = null;
    this.existingMethod = null;
  }

  // Cleanup on destroy
  destroy() {
    this.cleanupListeners();
  }
}

export const troubleshootStore = new TroubleshootStore();
