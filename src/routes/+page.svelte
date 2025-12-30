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
  }

  let errorMessage = $state<string | null>(null);
  let recentActivity = $state<ActivityEntry[]>([]);
  let activeProxies = $state<ActiveProxy[]>([]);

  onMount(async () => {
    if (!browser) return;
    
    const { invoke } = await import('@tauri-apps/api/core');
    const { listen } = await import('@tauri-apps/api/event');
    
    // Load initial state
    try {
      const status = await invoke<{is_active: boolean; current_strategy: string | null; current_strategy_name: string | null}>('get_status');
      appStatus.set({
        isActive: status.is_active,
        currentStrategy: status.current_strategy,
        currentStrategyName: status.current_strategy_name ?? null
      });
      
      const loadedServices = await invoke<{id: string; name: string; critical: boolean}[]>('get_services');
      if (loadedServices.length > 0) {
        services.set(loadedServices.map(s => ({
          id: s.id,
          name: s.name,
          icon: '',
          enabled: true,
          status: 'unknown' as const
        })));
      }

      // Load active proxies
      const vlessStatus = await invoke<{config_id: string; config_name: string; status: string; socks_port: number}[]>('get_all_vless_status').catch(() => []);
      activeProxies = vlessStatus.map(v => ({
        id: v.config_id,
        name: v.config_name,
        type: 'vless' as const,
        status: v.status === 'running' ? 'active' as const : 'connecting' as const
      }));

      // Add initial activity
      addActivity('info', 'Приложение запущено');
      if (status.is_active && status.current_strategy_name) {
        addActivity('success', `Активна стратегия: ${status.current_strategy_name}`);
      }
    } catch (e) {
      console.error('Failed to load initial state:', e);
    }
    
    // Subscribe to optimization events
    const unlistenProgress = await listen('optimization:progress', (event) => {
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
    
    const unlistenComplete = await listen('optimization:complete', (event) => {
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
      addActivity('success', `Оптимизация завершена: ${result.strategy_name}`);
    });
    
    const unlistenFailed = await listen('optimization:failed', (event) => {
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
    
    return () => {
      unlistenProgress();
      unlistenComplete();
      unlistenFailed();
    };
  });

  function addActivity(type: ActivityEntry['type'], message: string) {
    const entry: ActivityEntry = {
      id: crypto.randomUUID(),
      timestamp: new Date(),
      type,
      message
    };
    recentActivity = [entry, ...recentActivity].slice(0, 10);
  }

  async function handleOptimize() {
    if (!browser) return;
    
    isOptimizing.set(true);
    errorMessage = null;
    optimizationProgress.set({
      step: 'initializing',
      progress: 0,
      message: 'Начинаем оптимизацию...',
      isComplete: false,
      error: null
    });
    addActivity('info', 'Запуск оптимизации...');
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('run_optimization', { mode: 'turbo' });
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
      addActivity('warning', 'Выполнен сброс сети');
    } catch (e) {
      console.error('Panic reset failed:', e);
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
</script>

<div class="p-8 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-white">Dashboard</h1>
      <p class="text-[#a0a0a0] mt-1">Управление обходом блокировок</p>
    </div>
  </div>

  <!-- Status Card -->
  {@const status = getStatus($appStatus.isActive, $isOptimizing, errorMessage)}
  <div class="bg-[#1a1f3a] rounded-2xl p-6 border border-[#2a2f4a]">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-4">
        <!-- Status Indicator -->
        <div class="relative">
          {#if status === 'active'}
            <div class="w-16 h-16 rounded-full bg-[#00ff88]/20 flex items-center justify-center">
              <div class="w-10 h-10 rounded-full bg-[#00ff88] animate-pulse"></div>
            </div>
          {:else if status === 'optimizing'}
            <div class="w-16 h-16 rounded-full bg-[#ffaa00]/20 flex items-center justify-center">
              <div class="w-10 h-10 rounded-full bg-[#ffaa00] animate-pulse"></div>
            </div>
          {:else if status === 'error'}
            <div class="w-16 h-16 rounded-full bg-[#ff3333]/20 flex items-center justify-center">
              <div class="w-10 h-10 rounded-full bg-[#ff3333]"></div>
            </div>
          {:else}
            <div class="w-16 h-16 rounded-full bg-[#a0a0a0]/20 flex items-center justify-center">
              <div class="w-10 h-10 rounded-full bg-[#a0a0a0]"></div>
            </div>
          {/if}
        </div>

        <div>
          <div class="flex items-center gap-2">
            {#if status === 'active'}
              <span class="text-xl font-semibold text-[#00ff88]">Активен</span>
            {:else if status === 'optimizing'}
              <span class="text-xl font-semibold text-[#ffaa00]">Оптимизация...</span>
            {:else if status === 'error'}
              <span class="text-xl font-semibold text-[#ff3333]">Ошибка</span>
            {:else}
              <span class="text-xl font-semibold text-[#a0a0a0]">Неактивен</span>
            {/if}
          </div>
          
          {#if $appStatus.currentStrategy}
            <p class="text-[#a0a0a0] mt-1">
              Стратегия: <span class="text-white">{$appStatus.currentStrategyName || $appStatus.currentStrategy}</span>
            </p>
          {:else if status === 'idle'}
            <p class="text-[#a0a0a0] mt-1">Нажмите "Оптимизировать" для начала</p>
          {/if}
          
          {#if errorMessage}
            <p class="text-[#ff3333] text-sm mt-1">{errorMessage}</p>
          {/if}
        </div>
      </div>

      <!-- Action Button -->
      <div class="flex gap-3">
        {#if $hasActiveStrategy}
          <button
            onclick={handleStop}
            class="px-6 py-3 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-xl font-medium transition-all duration-200"
          >
            Отключить
          </button>
        {/if}
        
        {#if status === 'error'}
          <button
            onclick={handlePanicReset}
            class="px-6 py-3 bg-[#ff3333] hover:bg-[#ff4444] text-white rounded-xl font-medium transition-all duration-200"
          >
            Сбросить сеть
          </button>
        {/if}
      </div>
    </div>

    <!-- Progress Bar -->
    {#if $isOptimizing && $optimizationProgress.step}
      <div class="mt-6 pt-6 border-t border-[#2a2f4a]">
        <div class="flex items-center justify-between text-sm mb-2">
          <span class="text-[#a0a0a0]">{getStageText($optimizationProgress.step)}</span>
          <span class="text-[#00d4ff]">{$optimizationProgress.progress}%</span>
        </div>
        
        <div class="w-full bg-[#2a2f4a] rounded-full h-2">
          <div 
            class="bg-[#00d4ff] h-2 rounded-full transition-all duration-300"
            style="width: {$optimizationProgress.progress}%"
          ></div>
        </div>
        
        <p class="text-sm text-[#a0a0a0] mt-2">{$optimizationProgress.message}</p>
      </div>
    {/if}
  </div>

  <!-- Quick Actions -->
  <div class="grid grid-cols-3 gap-4">
    <button
      onclick={handleOptimize}
      disabled={$isOptimizing}
      class="bg-[#1a1f3a] hover:bg-[#2a2f4a] disabled:opacity-50 disabled:cursor-not-allowed border border-[#2a2f4a] rounded-xl p-6 text-left transition-all duration-200 group"
    >
      <div class="w-12 h-12 rounded-lg bg-[#00d4ff]/20 flex items-center justify-center mb-4 group-hover:bg-[#00d4ff]/30 transition-all duration-200">
        <svg class="w-6 h-6 text-[#00d4ff]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      </div>
      <h3 class="text-white font-semibold">Оптимизировать</h3>
      <p class="text-[#a0a0a0] text-sm mt-1">Найти лучшую стратегию</p>
    </button>

    <a
      href="/proxies"
      class="bg-[#1a1f3a] hover:bg-[#2a2f4a] border border-[#2a2f4a] rounded-xl p-6 text-left transition-all duration-200 group"
    >
      <div class="w-12 h-12 rounded-lg bg-[#00ff88]/20 flex items-center justify-center mb-4 group-hover:bg-[#00ff88]/30 transition-all duration-200">
        <svg class="w-6 h-6 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </div>
      <h3 class="text-white font-semibold">Добавить прокси</h3>
      <p class="text-[#a0a0a0] text-sm mt-1">Импортировать VLESS</p>
    </a>

    <a
      href="/settings"
      class="bg-[#1a1f3a] hover:bg-[#2a2f4a] border border-[#2a2f4a] rounded-xl p-6 text-left transition-all duration-200 group"
    >
      <div class="w-12 h-12 rounded-lg bg-[#ffaa00]/20 flex items-center justify-center mb-4 group-hover:bg-[#ffaa00]/30 transition-all duration-200">
        <svg class="w-6 h-6 text-[#ffaa00]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
        </svg>
      </div>
      <h3 class="text-white font-semibold">Импорт подписки</h3>
      <p class="text-[#a0a0a0] text-sm mt-1">Добавить из URL</p>
    </a>
  </div>

  <!-- Two Column Layout -->
  <div class="grid grid-cols-2 gap-6">
    <!-- Active Proxies -->
    <div class="bg-[#1a1f3a] rounded-2xl p-6 border border-[#2a2f4a]">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-white">Активные прокси</h2>
        <a href="/proxies" class="text-[#00d4ff] text-sm hover:underline">Все</a>
      </div>

      <div class="space-y-3">
        {#if activeProxies.length > 0}
          {#each activeProxies as proxy}
            <div class="flex items-center justify-between bg-[#2a2f4a]/50 rounded-lg p-4">
              <div class="flex items-center gap-3">
                <div class="w-2 h-2 rounded-full {proxy.status === 'active' ? 'bg-[#00ff88]' : proxy.status === 'connecting' ? 'bg-[#ffaa00] animate-pulse' : 'bg-[#ff3333]'}"></div>
                <div>
                  <p class="text-white font-medium">{proxy.name}</p>
                  <p class="text-[#a0a0a0] text-xs uppercase">{proxy.type}</p>
                </div>
              </div>
              {#if proxy.latency}
                <span class="text-[#a0a0a0] text-sm">{proxy.latency}ms</span>
              {/if}
            </div>
          {/each}
        {:else}
          <div class="text-center py-8">
            <svg class="w-12 h-12 text-[#a0a0a0]/50 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2" />
            </svg>
            <p class="text-[#a0a0a0]">Нет активных прокси</p>
            <a href="/proxies" class="text-[#00d4ff] text-sm hover:underline mt-2 inline-block">Добавить прокси</a>
          </div>
        {/if}
      </div>
    </div>

    <!-- Recent Activity -->
    <div class="bg-[#1a1f3a] rounded-2xl p-6 border border-[#2a2f4a]">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-white">Последние действия</h2>
        <a href="/logs" class="text-[#00d4ff] text-sm hover:underline">Все логи</a>
      </div>

      <div class="space-y-3">
        {#if recentActivity.length > 0}
          {#each recentActivity as activity}
            <div class="flex items-start gap-3 text-sm">
              <span class="text-[#a0a0a0] shrink-0">{formatTime(activity.timestamp)}</span>
              <span class="w-2 h-2 rounded-full mt-1.5 shrink-0 {activity.type === 'success' ? 'bg-[#00ff88]' : activity.type === 'warning' ? 'bg-[#ffaa00]' : activity.type === 'error' ? 'bg-[#ff3333]' : 'bg-[#00d4ff]'}"></span>
              <span class="text-[#a0a0a0]">{activity.message}</span>
            </div>
          {/each}
        {:else}
          <div class="text-center py-8">
            <svg class="w-12 h-12 text-[#a0a0a0]/50 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p class="text-[#a0a0a0]">Нет активности</p>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Services Status -->
  <div class="bg-[#1a1f3a] rounded-2xl p-6 border border-[#2a2f4a]">
    <h2 class="text-lg font-semibold text-white mb-4">Статус сервисов</h2>
    
    <div class="grid grid-cols-4 gap-4">
      {#each $services as service}
        <div class="bg-[#2a2f4a]/50 rounded-lg p-4 flex items-center gap-3">
          <div class="w-3 h-3 rounded-full {service.status === 'working' ? 'bg-[#00ff88]' : service.status === 'blocked' ? 'bg-[#ff3333]' : 'bg-[#a0a0a0]'}"></div>
          <div>
            <p class="text-white font-medium">{service.name}</p>
            <p class="text-xs {service.status === 'working' ? 'text-[#00ff88]' : service.status === 'blocked' ? 'text-[#ff3333]' : 'text-[#a0a0a0]'}">
              {service.status === 'working' ? 'Работает' : service.status === 'blocked' ? 'Заблокирован' : 'Неизвестно'}
            </p>
          </div>
        </div>
      {:else}
        <div class="bg-[#2a2f4a]/50 rounded-lg p-4 flex items-center gap-3">
          <div class="w-3 h-3 rounded-full bg-[#a0a0a0]"></div>
          <div>
            <p class="text-white font-medium">Discord</p>
            <p class="text-xs text-[#a0a0a0]">Неизвестно</p>
          </div>
        </div>
        <div class="bg-[#2a2f4a]/50 rounded-lg p-4 flex items-center gap-3">
          <div class="w-3 h-3 rounded-full bg-[#a0a0a0]"></div>
          <div>
            <p class="text-white font-medium">YouTube</p>
            <p class="text-xs text-[#a0a0a0]">Неизвестно</p>
          </div>
        </div>
        <div class="bg-[#2a2f4a]/50 rounded-lg p-4 flex items-center gap-3">
          <div class="w-3 h-3 rounded-full bg-[#a0a0a0]"></div>
          <div>
            <p class="text-white font-medium">Telegram</p>
            <p class="text-xs text-[#a0a0a0]">Неизвестно</p>
          </div>
        </div>
        <div class="bg-[#2a2f4a]/50 rounded-lg p-4 flex items-center gap-3">
          <div class="w-3 h-3 rounded-full bg-[#a0a0a0]"></div>
          <div>
            <p class="text-white font-medium">Google</p>
            <p class="text-xs text-[#a0a0a0]">Неизвестно</p>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
