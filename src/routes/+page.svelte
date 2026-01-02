<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { 
    appStatus, 
    optimizationProgress, 
    isOptimizing, 
    services,
    hasActiveStrategy
  } from '$lib/stores';

  // Types
  interface ActivityEntry {
    id: string;
    timestamp: Date;
    type: 'success' | 'warning' | 'error' | 'info';
    message: string;
  }

  interface ActiveProxy {
    id: string;
    name: string;
    type: 'vless' | 'zapret';
    status: 'active' | 'connecting' | 'error';
    latency?: number;
    socksPort?: number;
  }

  interface SystemStatus {
    tunMode: boolean;
    systemProxy: boolean;
    quicBlocked: boolean;
  }

  // State using Svelte 5 runes
  let errorMessage = $state<string | null>(null);
  let recentActivity = $state<ActivityEntry[]>([]);
  let activeProxies = $state<ActiveProxy[]>([]);
  let systemStatus = $state<SystemStatus>({
    tunMode: false,
    systemProxy: false,
    quicBlocked: false
  });
  let uptime = $state<number>(0);
  let startTime = $state<Date | null>(null);
  let optimizationMode = $state<'turbo' | 'deep'>('turbo');
  let uptimeInterval: ReturnType<typeof setInterval> | null = null;

  // Local reactive copies of store values
  let appStatusValue = $state<{isActive: boolean; currentStrategy: string | null; currentStrategyName: string | null}>({
    isActive: false,
    currentStrategy: null,
    currentStrategyName: null
  });
  let isOptimizingValue = $state(false);
  let optimizationProgressValue = $state<{step: string; progress: number; message: string; isComplete: boolean; error: string | null}>({
    step: '',
    progress: 0,
    message: '',
    isComplete: false,
    error: null
  });
  let servicesValue = $state<{id: string; name: string; icon: string; enabled: boolean; status: 'unknown' | 'working' | 'blocked'}[]>([]);
  let hasActiveStrategyValue = $state(false);

  // Derived state
  let statusType = $derived(getStatus(appStatusValue.isActive, isOptimizingValue, errorMessage));
  let formattedUptime = $derived(formatUptime(uptime));

  onMount(async () => {
    if (!browser) return;
    
    // Force hot reload trigger - v2
    console.log('[Dashboard] onMount started');
    
    // Check if we're in Tauri environment
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    console.log('[Dashboard] isTauri:', isTauri);
    
    // Subscribe to stores
    const unsubAppStatus = appStatus.subscribe(v => { appStatusValue = v; });
    const unsubIsOptimizing = isOptimizing.subscribe(v => { isOptimizingValue = v; });
    const unsubProgress = optimizationProgress.subscribe(v => { optimizationProgressValue = v; });
    const unsubServices = services.subscribe(v => { servicesValue = v; });
    const unsubHasActive = hasActiveStrategy.subscribe(v => { hasActiveStrategyValue = v; });
    
    if (!isTauri) {
      console.log('[Dashboard] Not in Tauri, using default state');
      addActivity('info', 'Приложение запущено (режим браузера)');
      return () => {
        unsubAppStatus();
        unsubIsOptimizing();
        unsubProgress();
        unsubServices();
        unsubHasActive();
        if (uptimeInterval) clearInterval(uptimeInterval);
      };
    }
    
    // Force timeout - GUARANTEED to fire regardless of Promise state (no clearTimeout!)
    setTimeout(() => {
      console.warn('[Dashboard] Force timeout triggered after 5s');
      addActivity('warning', 'Таймаут загрузки данных');
    }, 5000);
    
    let unlistenProgress: (() => void) | undefined;
    let unlistenComplete: (() => void) | undefined;
    let unlistenFailed: (() => void) | undefined;
    let unlistenHealthCheck: (() => void) | undefined;
    let unlistenApplied: (() => void) | undefined;
    let unlistenStopped: (() => void) | undefined;
    let refreshInterval: ReturnType<typeof setInterval> | undefined;
    
    try {
      console.log('[Dashboard] Importing @tauri-apps/api/core...');
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');
      console.log('[Dashboard] Tauri API imported');
      
      // Helper for invoke with timeout
      async function invokeWithTimeout(cmd: string, args: Record<string, unknown> | undefined, fallback: unknown): Promise<unknown> {
        const timeout = new Promise((resolve) => 
          setTimeout(() => {
            console.warn(`[Dashboard] ${cmd} timeout after 2s`);
            resolve(fallback);
          }, 2000)
        );
        return Promise.race([
          invoke(cmd, args).then((r) => { console.log(`[Dashboard] ${cmd} returned`); return r; }).catch(() => fallback),
          timeout
        ]);
      }
    
      // Load initial state
      console.log('[Dashboard] Loading initial state...');
      
      // Load status without timeout - it's critical for correct display
      let status: {is_active: boolean; current_strategy: string | null; current_strategy_name: string | null} | null = null;
      try {
        const rawStatus = await invoke('get_status') as {is_active: boolean; current_strategy: string | null; current_strategy_name: string | null};
        status = rawStatus;
        console.log('[Dashboard] get_status returned:', status);
      } catch (e) {
        console.error('[Dashboard] get_status failed:', e);
        status = {is_active: false, current_strategy: null, current_strategy_name: null};
      }
      
      if (status) {
        appStatus.set({
          isActive: status.is_active,
          currentStrategy: status.current_strategy,
          currentStrategyName: status.current_strategy_name ?? null
        });
        
        if (status.is_active) {
          startTime = new Date();
          startUptimeCounter();
        }
      }
      
      const loadedServices = await invokeWithTimeout('get_services', undefined, []) as {id: string; name: string; critical: boolean}[];
      if (loadedServices && loadedServices.length > 0) {
        services.set(loadedServices.map(s => ({
          id: s.id,
          name: s.name,
          icon: '',
          enabled: true,
          status: 'unknown' as const
        })));
      }

      // Load active proxies
      await refreshProxies();
      
      // Load system status
      await refreshSystemStatus();

      // Add initial activity
      addActivity('info', 'Приложение запущено');
      if (status?.is_active && status?.current_strategy_name) {
        addActivity('success', `Активна стратегия: ${status.current_strategy_name}`);
      }
      
      console.log('[Dashboard] Initial state loaded');

      // Subscribe to optimization events
      unlistenProgress = await listen('optimization:progress', (event) => {
        const payload = event.payload as {
          stage: string;
          percent: number;
          message: string;
          current_strategy: string | null;
          tested_count: number;
          total_count: number;
          best_score: number | null;
        };
        optimizationProgress.set({
          step: payload.stage,
          progress: payload.percent,
          message: payload.message,
          isComplete: false,
          error: null
        });
      });
      
      unlistenComplete = await listen('optimization:complete', (event) => {
        const result = event.payload as {strategy_id: string; strategy_name: string; score: number};
        appStatus.set({
          isActive: true,
          currentStrategy: result.strategy_id,
          currentStrategyName: result.strategy_name ?? null
        });
        isOptimizing.set(false);
        optimizationProgress.set({
          step: 'completed',
          progress: 100,
          message: 'Оптимизация завершена',
          isComplete: true,
          error: null
        });
        errorMessage = null;
        startTime = new Date();
        startUptimeCounter();
        addActivity('success', `Оптимизация завершена: ${result.strategy_name}`);
      });
      
      unlistenFailed = await listen('optimization:failed', (event) => {
        isOptimizing.set(false);
        errorMessage = event.payload as string;
        optimizationProgress.set({
          step: 'failed',
          progress: 0,
          message: '',
          isComplete: false,
          error: event.payload as string
        });
        addActivity('error', `Ошибка оптимизации: ${event.payload}`);
      });

      // Subscribe to strategy events (from other pages like Strategies)
      unlistenApplied = await listen('strategy:applied', (event) => {
        const payload = event.payload as {strategy_id: string; strategy_name: string};
        appStatus.set({
          isActive: true,
          currentStrategy: payload.strategy_id,
          currentStrategyName: payload.strategy_name ?? null
        });
        errorMessage = null;
        startTime = new Date();
        startUptimeCounter();
        addActivity('success', `Стратегия применена: ${payload.strategy_name}`);
      });

      unlistenStopped = await listen('strategy:stopped', () => {
        appStatus.set({
          isActive: false,
          currentStrategy: null,
          currentStrategyName: null
        });
        stopUptimeCounter();
        addActivity('info', 'Стратегия отключена');
      });

      // Subscribe to health check events
      unlistenHealthCheck = await listen('monitor:health_check', async () => {
        await refreshProxies();
        await refreshSystemStatus();
      });

      // Periodic refresh
      refreshInterval = setInterval(async () => {
        await refreshProxies();
        await refreshSystemStatus();
      }, 10000);
    } catch (e) {
      console.error('[Dashboard] Failed to load initial state:', e);
    }
    
    return () => {
      // Unsubscribe from stores
      unsubAppStatus();
      unsubIsOptimizing();
      unsubProgress();
      unsubServices();
      unsubHasActive();
      // Unlisten events
      unlistenProgress?.();
      unlistenComplete?.();
      unlistenFailed?.();
      unlistenHealthCheck?.();
      unlistenApplied?.();
      unlistenStopped?.();
      if (refreshInterval) clearInterval(refreshInterval);
      if (uptimeInterval) clearInterval(uptimeInterval);
    };
  });

  function startUptimeCounter() {
    if (uptimeInterval) clearInterval(uptimeInterval);
    uptimeInterval = setInterval(() => {
      if (startTime) {
        uptime = Math.floor((Date.now() - startTime.getTime()) / 1000);
      }
    }, 1000);
  }

  function stopUptimeCounter() {
    if (uptimeInterval) {
      clearInterval(uptimeInterval);
      uptimeInterval = null;
    }
    uptime = 0;
    startTime = null;
  }

  async function refreshProxies() {
    if (!browser) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const vlessStatus = await invoke<{config_id: string; config_name: string; status: string; socks_port: number}[]>('get_all_vless_status').catch(() => []);
      activeProxies = vlessStatus.map(v => ({
        id: v.config_id,
        name: v.config_name,
        type: 'vless' as const,
        status: v.status === 'running' ? 'active' as const : v.status === 'starting' ? 'connecting' as const : 'error' as const,
        socksPort: v.socks_port
      }));
    } catch (e) {
      console.error('Failed to refresh proxies:', e);
    }
  }

  async function refreshSystemStatus() {
    if (!browser) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const [tunRunning, proxySet, quicBlocked] = await Promise.all([
        invoke<boolean>('is_tun_running').catch(() => false),
        invoke<boolean>('is_system_proxy_set').catch(() => false),
        invoke<boolean>('is_quic_blocked').catch(() => false)
      ]);
      systemStatus = {
        tunMode: tunRunning,
        systemProxy: proxySet,
        quicBlocked: quicBlocked
      };
    } catch (e) {
      console.error('Failed to refresh system status:', e);
    }
  }

  function addActivity(type: ActivityEntry['type'], message: string) {
    const entry: ActivityEntry = {
      id: crypto.randomUUID(),
      timestamp: new Date(),
      type,
      message
    };
    recentActivity = [entry, ...recentActivity].slice(0, 10);
  }

  async function handleOptimize(mode: 'turbo' | 'deep') {
    if (!browser) return;
    
    optimizationMode = mode;
    isOptimizing.set(true);
    errorMessage = null;
    optimizationProgress.set({
      step: 'initializing',
      progress: 0,
      message: mode === 'turbo' ? 'Быстрая оптимизация...' : 'Глубокая оптимизация...',
      isComplete: false,
      error: null
    });
    addActivity('info', `Запуск ${mode === 'turbo' ? 'быстрой' : 'глубокой'} оптимизации...`);
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('run_optimization', { mode });
    } catch (e) {
      console.error('Optimization failed:', e);
      isOptimizing.set(false);
      errorMessage = String(e);
      optimizationProgress.update(p => ({ ...p, error: String(e) }));
    }
  }

  async function handleStop() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('stop_strategy');
      appStatus.set({
        isActive: false,
        currentStrategy: null,
        currentStrategyName: null
      });
      stopUptimeCounter();
      addActivity('info', 'Стратегия отключена');
    } catch (e) {
      console.error('Failed to stop strategy:', e);
      addActivity('error', `Ошибка отключения: ${e}`);
    }
  }
  
  async function handlePanicReset() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('panic_reset');
      appStatus.set({
        isActive: false,
        currentStrategy: null,
        currentStrategyName: null
      });
      isOptimizing.set(false);
      optimizationProgress.set({
        step: '',
        progress: 0,
        message: '',
        isComplete: false,
        error: null
      });
      errorMessage = null;
      stopUptimeCounter();
      addActivity('warning', 'Выполнен сброс сети');
      await refreshSystemStatus();
    } catch (e) {
      console.error('Panic reset failed:', e);
    }
  }

  async function toggleQuicBlock() {
    if (!browser) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (systemStatus.quicBlocked) {
        await invoke('disable_quic_block');
        addActivity('info', 'QUIC разблокирован');
      } else {
        await invoke('enable_quic_block');
        addActivity('success', 'QUIC заблокирован');
      }
      await refreshSystemStatus();
    } catch (e) {
      addActivity('error', `Ошибка QUIC: ${e}`);
    }
  }

  function getStageText(stage: string): string {
    const stages: Record<string, string> = {
      'initializing': 'Инициализация',
      'checking_cache': 'Проверка кэша',
      'diagnosing': 'Диагностика DPI',
      'selecting_candidates': 'Выбор стратегий',
      'testing_vless': 'Тестирование VLESS',
      'testing_zapret': 'Тестирование Zapret',
      'selecting_best': 'Выбор лучшей',
      'applying': 'Применение',
      'completed': 'Завершено',
      'failed': 'Ошибка',
      'cancelled': 'Отменено'
    };
    return stages[stage] || stage;
  }
  
  function getStatus(isActive: boolean, optimizing: boolean, error: string | null): 'idle' | 'active' | 'optimizing' | 'error' {
    if (error) return 'error';
    if (optimizing) return 'optimizing';
    if (isActive) return 'active';
    return 'idle';
  }

  function formatTime(date: Date): string {
    return date.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  function formatUptime(seconds: number): string {
    if (seconds === 0) return '—';
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    if (h > 0) return `${h}ч ${m}м ${s}с`;
    if (m > 0) return `${m}м ${s}с`;
    return `${s}с`;
  }

  function getStatusColor(status: 'idle' | 'active' | 'optimizing' | 'error'): string {
    switch (status) {
      case 'active': return '#00ff88';
      case 'optimizing': return '#ffaa00';
      case 'error': return '#ff3333';
      default: return '#a0a0a0';
    }
  }

  function getStatusText(status: 'idle' | 'active' | 'optimizing' | 'error'): string {
    switch (status) {
      case 'active': return 'Активен';
      case 'optimizing': return 'Оптимизация...';
      case 'error': return 'Ошибка';
      default: return 'Неактивен';
    }
  }
</script>

<div class="p-8 space-y-6 min-h-screen bg-[#0a0e27]">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-white">Dashboard</h1>
      <p class="text-[#a0a0a0] mt-1">Управление обходом блокировок</p>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-[#a0a0a0] text-sm">Время работы:</span>
      <span class="text-white font-mono">{formattedUptime}</span>
    </div>
  </div>

  <!-- Main Status Card -->
  <div class="bg-[#1a1f3a] rounded-2xl p-6 border border-[#2a2f4a] relative overflow-hidden">
    <!-- Background glow effect -->
    {#if statusType === 'active'}
      <div class="absolute inset-0 bg-gradient-to-r from-[#00ff88]/5 to-transparent pointer-events-none"></div>
    {:else if statusType === 'optimizing'}
      <div class="absolute inset-0 bg-gradient-to-r from-[#ffaa00]/5 to-transparent pointer-events-none"></div>
    {/if}
    
    <div class="relative flex items-center justify-between">
      <div class="flex items-center gap-6">
        <!-- Status Indicator -->
        <div class="relative">
          <div class="w-20 h-20 rounded-full flex items-center justify-center"
               style="background: {getStatusColor(statusType)}20">
            <div class="w-12 h-12 rounded-full transition-all duration-300"
                 class:animate-pulse={statusType === 'active' || statusType === 'optimizing'}
                 style="background: {getStatusColor(statusType)}"></div>
          </div>
          {#if statusType === 'active'}
            <div class="absolute -bottom-1 -right-1 w-6 h-6 bg-[#1a1f3a] rounded-full flex items-center justify-center">
              <svg class="w-4 h-4 text-[#00ff88]" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
              </svg>
            </div>
          {/if}
        </div>

        <div>
          <div class="flex items-center gap-3">
            <span class="text-2xl font-bold" style="color: {getStatusColor(statusType)}">{getStatusText(statusType)}</span>
            {#if statusType === 'active' && uptime > 0}
              <span class="px-2 py-1 bg-[#00ff88]/10 text-[#00ff88] text-xs rounded-full font-medium">
                {formattedUptime}
              </span>
            {/if}
          </div>
          
          {#if appStatusValue.currentStrategy}
            <p class="text-[#a0a0a0] mt-2">
              Стратегия: <span class="text-white font-medium">{appStatusValue.currentStrategyName || appStatusValue.currentStrategy}</span>
            </p>
          {:else if statusType === 'idle'}
            <p class="text-[#a0a0a0] mt-2">Выберите режим оптимизации для начала</p>
          {/if}
          
          {#if errorMessage}
            <p class="text-[#ff3333] text-sm mt-2 max-w-md">{errorMessage}</p>
          {/if}
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex gap-3">
        {#if hasActiveStrategyValue}
          <button
            onclick={handleStop}
            class="px-5 py-2.5 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-xl font-medium transition-all duration-200 flex items-center gap-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"/>
            </svg>
            Отключить
          </button>
        {/if}
        
        {#if statusType === 'error'}
          <button
            onclick={handlePanicReset}
            class="px-5 py-2.5 bg-[#ff3333] hover:bg-[#ff4444] text-white rounded-xl font-medium transition-all duration-200 flex items-center gap-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
            </svg>
            Сбросить сеть
          </button>
        {/if}
      </div>
    </div>

    <!-- Progress Bar -->
    {#if isOptimizingValue && optimizationProgressValue.step}
      <div class="mt-6 pt-6 border-t border-[#2a2f4a]">
        <div class="flex items-center justify-between text-sm mb-3">
          <div class="flex items-center gap-2">
            <div class="w-2 h-2 rounded-full bg-[#ffaa00] animate-pulse"></div>
            <span class="text-white font-medium">{getStageText(optimizationProgressValue.step)}</span>
          </div>
          <span class="text-[#00d4ff] font-mono">{optimizationProgressValue.progress}%</span>
        </div>
        
        <div class="w-full bg-[#2a2f4a] rounded-full h-2 overflow-hidden">
          <div 
            class="bg-gradient-to-r from-[#00d4ff] to-[#00ff88] h-2 rounded-full transition-all duration-300"
            style="width: {optimizationProgressValue.progress}%"
          ></div>
        </div>
        
        <p class="text-sm text-[#a0a0a0] mt-3">{optimizationProgressValue.message}</p>
      </div>
    {/if}
  </div>

  <!-- Quick Actions -->
  <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
    <!-- Turbo Optimization -->
    <button
      onclick={() => handleOptimize('turbo')}
      disabled={isOptimizingValue}
      class="bg-[#1a1f3a] hover:bg-[#2a2f4a] disabled:opacity-50 disabled:cursor-not-allowed border border-[#2a2f4a] hover:border-[#00d4ff]/50 rounded-xl p-5 text-left transition-all duration-200 group"
    >
      <div class="w-12 h-12 rounded-lg bg-[#00d4ff]/20 flex items-center justify-center mb-3 group-hover:bg-[#00d4ff]/30 transition-all duration-200">
        <svg class="w-6 h-6 text-[#00d4ff]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
        </svg>
      </div>
      <h3 class="text-white font-semibold">Turbo</h3>
      <p class="text-[#a0a0a0] text-sm mt-1">Быстрая оптимизация</p>
      {#if isOptimizingValue && optimizationMode === 'turbo'}
        <div class="mt-2 flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-[#00d4ff] animate-pulse"></div>
          <span class="text-[#00d4ff] text-xs">{optimizationProgressValue.progress}%</span>
        </div>
      {/if}
    </button>

    <!-- Deep Optimization -->
    <button
      onclick={() => handleOptimize('deep')}
      disabled={isOptimizingValue}
      class="bg-[#1a1f3a] hover:bg-[#2a2f4a] disabled:opacity-50 disabled:cursor-not-allowed border border-[#2a2f4a] hover:border-[#00ff88]/50 rounded-xl p-5 text-left transition-all duration-200 group"
    >
      <div class="w-12 h-12 rounded-lg bg-[#00ff88]/20 flex items-center justify-center mb-3 group-hover:bg-[#00ff88]/30 transition-all duration-200">
        <svg class="w-6 h-6 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
        </svg>
      </div>
      <h3 class="text-white font-semibold">Deep</h3>
      <p class="text-[#a0a0a0] text-sm mt-1">Глубокий анализ</p>
      {#if isOptimizingValue && optimizationMode === 'deep'}
        <div class="mt-2 flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-[#00ff88] animate-pulse"></div>
          <span class="text-[#00ff88] text-xs">{optimizationProgressValue.progress}%</span>
        </div>
      {/if}
    </button>

    <!-- Add Proxy -->
    <a
      href="/proxies"
      class="bg-[#1a1f3a] hover:bg-[#2a2f4a] border border-[#2a2f4a] hover:border-[#ffaa00]/50 rounded-xl p-5 text-left transition-all duration-200 group"
    >
      <div class="w-12 h-12 rounded-lg bg-[#ffaa00]/20 flex items-center justify-center mb-3 group-hover:bg-[#ffaa00]/30 transition-all duration-200">
        <svg class="w-6 h-6 text-[#ffaa00]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
        </svg>
      </div>
      <h3 class="text-white font-semibold">Добавить прокси</h3>
      <p class="text-[#a0a0a0] text-sm mt-1">Импортировать VLESS</p>
    </a>

    <!-- Settings -->
    <a
      href="/settings"
      class="bg-[#1a1f3a] hover:bg-[#2a2f4a] border border-[#2a2f4a] hover:border-[#a0a0a0]/50 rounded-xl p-5 text-left transition-all duration-200 group"
    >
      <div class="w-12 h-12 rounded-lg bg-[#a0a0a0]/20 flex items-center justify-center mb-3 group-hover:bg-[#a0a0a0]/30 transition-all duration-200">
        <svg class="w-6 h-6 text-[#a0a0a0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
        </svg>
      </div>
      <h3 class="text-white font-semibold">Настройки</h3>
      <p class="text-[#a0a0a0] text-sm mt-1">Конфигурация</p>
    </a>
  </div>

  <!-- System Status Bar -->
  <div class="bg-[#1a1f3a] rounded-xl p-4 border border-[#2a2f4a]">
    <div class="flex items-center justify-between">
      <h3 class="text-white font-medium">Системный статус</h3>
      <div class="flex items-center gap-6">
        <!-- TUN Mode -->
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full {systemStatus.tunMode ? 'bg-[#00ff88]' : 'bg-[#a0a0a0]'}"></div>
          <span class="text-sm {systemStatus.tunMode ? 'text-[#00ff88]' : 'text-[#a0a0a0]'}">TUN Mode</span>
        </div>
        
        <!-- System Proxy -->
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full {systemStatus.systemProxy ? 'bg-[#00ff88]' : 'bg-[#a0a0a0]'}"></div>
          <span class="text-sm {systemStatus.systemProxy ? 'text-[#00ff88]' : 'text-[#a0a0a0]'}">System Proxy</span>
        </div>
        
        <!-- QUIC Block -->
        <button 
          onclick={toggleQuicBlock}
          class="flex items-center gap-2 px-3 py-1 rounded-lg transition-all duration-200 {systemStatus.quicBlocked ? 'bg-[#00ff88]/10 hover:bg-[#00ff88]/20' : 'bg-[#2a2f4a] hover:bg-[#3a3f5a]'}"
        >
          <div class="w-2 h-2 rounded-full {systemStatus.quicBlocked ? 'bg-[#00ff88]' : 'bg-[#a0a0a0]'}"></div>
          <span class="text-sm {systemStatus.quicBlocked ? 'text-[#00ff88]' : 'text-[#a0a0a0]'}">QUIC Block</span>
          <svg class="w-3 h-3 {systemStatus.quicBlocked ? 'text-[#00ff88]' : 'text-[#a0a0a0]'}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l4-4 4 4m0 6l-4 4-4-4"/>
          </svg>
        </button>
      </div>
    </div>
  </div>

  <!-- Two Column Layout -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- Active Proxies -->
    <div class="bg-[#1a1f3a] rounded-2xl p-6 border border-[#2a2f4a]">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-white flex items-center gap-2">
          <svg class="w-5 h-5 text-[#00d4ff]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2"/>
          </svg>
          Активные прокси
        </h2>
        <a href="/proxies" class="text-[#00d4ff] text-sm hover:underline flex items-center gap-1">
          Все
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
        </a>
      </div>

      <div class="space-y-3">
        {#if activeProxies.length > 0}
          {#each activeProxies as proxy}
            <div class="flex items-center justify-between bg-[#2a2f4a]/50 hover:bg-[#2a2f4a] rounded-lg p-4 transition-all duration-200">
              <div class="flex items-center gap-3">
                <div class="relative">
                  <div class="w-10 h-10 rounded-lg bg-[#00d4ff]/20 flex items-center justify-center">
                    <svg class="w-5 h-5 text-[#00d4ff]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0"/>
                    </svg>
                  </div>
                  <div class="absolute -bottom-1 -right-1 w-3 h-3 rounded-full border-2 border-[#1a1f3a] {proxy.status === 'active' ? 'bg-[#00ff88]' : proxy.status === 'connecting' ? 'bg-[#ffaa00] animate-pulse' : 'bg-[#ff3333]'}"></div>
                </div>
                <div>
                  <p class="text-white font-medium">{proxy.name}</p>
                  <p class="text-[#a0a0a0] text-xs">
                    {proxy.type.toUpperCase()} {proxy.socksPort ? `• :${proxy.socksPort}` : ''}
                  </p>
                </div>
              </div>
              <div class="text-right">
                {#if proxy.latency}
                  <span class="text-[#00ff88] font-mono text-sm">{proxy.latency}ms</span>
                {:else}
                  <span class="text-[#a0a0a0] text-xs">
                    {proxy.status === 'active' ? 'Подключен' : proxy.status === 'connecting' ? 'Подключение...' : 'Ошибка'}
                  </span>
                {/if}
              </div>
            </div>
          {/each}
        {:else}
          <div class="text-center py-8">
            <div class="w-16 h-16 rounded-full bg-[#2a2f4a] flex items-center justify-center mx-auto mb-4">
              <svg class="w-8 h-8 text-[#a0a0a0]/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2"/>
              </svg>
            </div>
            <p class="text-[#a0a0a0] mb-2">Нет активных прокси</p>
            <a href="/proxies" class="text-[#00d4ff] text-sm hover:underline">Добавить прокси →</a>
          </div>
        {/if}
      </div>
    </div>

    <!-- Recent Activity -->
    <div class="bg-[#1a1f3a] rounded-2xl p-6 border border-[#2a2f4a]">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-white flex items-center gap-2">
          <svg class="w-5 h-5 text-[#ffaa00]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          Последние действия
        </h2>
        <a href="/logs" class="text-[#00d4ff] text-sm hover:underline flex items-center gap-1">
          Все логи
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
        </a>
      </div>

      <div class="space-y-2 max-h-[300px] overflow-y-auto">
        {#if recentActivity.length > 0}
          {#each recentActivity as activity}
            <div class="flex items-start gap-3 p-3 rounded-lg hover:bg-[#2a2f4a]/30 transition-all duration-200">
              <span class="text-[#a0a0a0] text-xs font-mono shrink-0 mt-0.5">{formatTime(activity.timestamp)}</span>
              <div class="w-2 h-2 rounded-full mt-1.5 shrink-0 {activity.type === 'success' ? 'bg-[#00ff88]' : activity.type === 'warning' ? 'bg-[#ffaa00]' : activity.type === 'error' ? 'bg-[#ff3333]' : 'bg-[#00d4ff]'}"></div>
              <span class="text-sm {activity.type === 'error' ? 'text-[#ff3333]' : activity.type === 'success' ? 'text-[#00ff88]' : activity.type === 'warning' ? 'text-[#ffaa00]' : 'text-[#a0a0a0]'}">{activity.message}</span>
            </div>
          {/each}
        {:else}
          <div class="text-center py-8">
            <div class="w-16 h-16 rounded-full bg-[#2a2f4a] flex items-center justify-center mx-auto mb-4">
              <svg class="w-8 h-8 text-[#a0a0a0]/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
              </svg>
            </div>
            <p class="text-[#a0a0a0]">Нет активности</p>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Services Status -->
  <div class="bg-[#1a1f3a] rounded-2xl p-6 border border-[#2a2f4a]">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold text-white flex items-center gap-2">
        <svg class="w-5 h-5 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        Статус сервисов
      </h2>
      <a href="/testing" class="text-[#00d4ff] text-sm hover:underline flex items-center gap-1">
        Тестировать
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
        </svg>
      </a>
    </div>
    
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      {#each servicesValue as service}
        <div class="bg-[#2a2f4a]/50 hover:bg-[#2a2f4a] rounded-xl p-4 transition-all duration-200">
          <div class="flex items-center gap-3">
            <div class="w-3 h-3 rounded-full {service.status === 'working' ? 'bg-[#00ff88]' : service.status === 'blocked' ? 'bg-[#ff3333]' : 'bg-[#a0a0a0]'}"></div>
            <div>
              <p class="text-white font-medium">{service.name}</p>
              <p class="text-xs {service.status === 'working' ? 'text-[#00ff88]' : service.status === 'blocked' ? 'text-[#ff3333]' : 'text-[#a0a0a0]'}">
                {service.status === 'working' ? 'Работает' : service.status === 'blocked' ? 'Заблокирован' : 'Неизвестно'}
              </p>
            </div>
          </div>
        </div>
      {:else}
        <!-- Default services when none loaded -->
        <div class="bg-[#2a2f4a]/50 rounded-xl p-4">
          <div class="flex items-center gap-3">
            <div class="w-3 h-3 rounded-full bg-[#a0a0a0]"></div>
            <div>
              <p class="text-white font-medium">Discord</p>
              <p class="text-xs text-[#a0a0a0]">Неизвестно</p>
            </div>
          </div>
        </div>
        <div class="bg-[#2a2f4a]/50 rounded-xl p-4">
          <div class="flex items-center gap-3">
            <div class="w-3 h-3 rounded-full bg-[#a0a0a0]"></div>
            <div>
              <p class="text-white font-medium">YouTube</p>
              <p class="text-xs text-[#a0a0a0]">Неизвестно</p>
            </div>
          </div>
        </div>
        <div class="bg-[#2a2f4a]/50 rounded-xl p-4">
          <div class="flex items-center gap-3">
            <div class="w-3 h-3 rounded-full bg-[#a0a0a0]"></div>
            <div>
              <p class="text-white font-medium">Telegram</p>
              <p class="text-xs text-[#a0a0a0]">Неизвестно</p>
            </div>
          </div>
        </div>
        <div class="bg-[#2a2f4a]/50 rounded-xl p-4">
          <div class="flex items-center gap-3">
            <div class="w-3 h-3 rounded-full bg-[#a0a0a0]"></div>
            <div>
              <p class="text-white font-medium">Google</p>
              <p class="text-xs text-[#a0a0a0]">Неизвестно</p>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
