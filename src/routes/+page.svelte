<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { 
    appStatus, 
    optimizationProgress, 
    isOptimizing, 
    services,
    hasActiveStrategy,
    optimizationError
  } from '$lib/stores';

  let errorMessage = $state<string | null>(null);

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
    });
    
    return () => {
      unlistenProgress();
      unlistenComplete();
      unlistenFailed();
    };
  });

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
    } catch (e) {
      console.error('Failed to stop strategy:', e);
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
  
  // Computed status for UI
  function getStatus(isActive: boolean, optimizing: boolean, error: string | null): 'idle' | 'active' | 'optimizing' | 'error' {
    if (error) return 'error';
    if (optimizing) return 'optimizing';
    if (isActive) return 'active';
    return 'idle';
  }
</script>

<div class="flex flex-col items-center justify-center min-h-screen p-8">
  <div class="w-full max-w-md space-y-8">
    <!-- Header -->
    <div class="text-center relative">
      <button
        onclick={() => goto('/settings')}
        class="absolute right-0 top-0 p-2 hover:bg-gray-800 rounded-lg transition-colors"
        aria-label="Настройки"
      >
        <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </button>
      <h1 class="text-4xl font-bold text-primary-400">Isolate</h1>
      <p class="mt-2 text-gray-400">Автоматический обход DPI-блокировок</p>
    </div>

    <!-- Status Card -->
    {@const status = getStatus($appStatus.isActive, $isOptimizing, errorMessage)}
    <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
      <div class="flex items-center justify-between">
        <span class="text-gray-400">Статус</span>
        <span class="flex items-center gap-2">
          {#if status === 'active'}
            <span class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></span>
            <span class="text-green-400">Активен</span>
          {:else if status === 'optimizing'}
            <span class="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></span>
            <span class="text-yellow-400">Оптимизация...</span>
          {:else if status === 'error'}
            <span class="w-2 h-2 bg-red-500 rounded-full"></span>
            <span class="text-red-400">Ошибка</span>
          {:else}
            <span class="w-2 h-2 bg-gray-500 rounded-full"></span>
            <span class="text-gray-400">Неактивен</span>
          {/if}
        </span>
      </div>

      {#if $appStatus.currentStrategy}
        <div class="flex items-center justify-between">
          <span class="text-gray-400">Стратегия</span>
          <span class="text-white">{$appStatus.currentStrategyName || $appStatus.currentStrategy}</span>
        </div>
      {/if}
      
      {#if errorMessage}
        <div class="text-red-400 text-sm">{errorMessage}</div>
      {/if}
    </div>
    
    <!-- Progress -->
    {#if $isOptimizing && $optimizationProgress.step}
      <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
        <div class="flex items-center justify-between text-sm">
          <span class="text-gray-400">{getStageText($optimizationProgress.step)}</span>
          <span class="text-primary-400">{$optimizationProgress.progress}%</span>
        </div>
        
        <div class="w-full bg-gray-700 rounded-full h-2">
          <div 
            class="bg-primary-500 h-2 rounded-full transition-all duration-300"
            style="width: {$optimizationProgress.progress}%"
          ></div>
        </div>
        
        <p class="text-sm text-gray-400">{$optimizationProgress.message}</p>
      </div>
    {/if}

    <!-- Actions -->
    <div class="space-y-3">
      <button
        onclick={handleOptimize}
        disabled={$isOptimizing}
        class="w-full py-4 px-6 bg-primary-600 hover:bg-primary-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-xl font-medium transition-colors"
      >
        {$isOptimizing ? 'Оптимизация...' : 'Оптимизировать'}
      </button>

      {#if $hasActiveStrategy}
        <button
          onclick={handleStop}
          class="w-full py-3 px-6 bg-gray-700 hover:bg-gray-600 rounded-xl font-medium transition-colors"
        >
          Отключить
        </button>
      {/if}
      
      {#if status === 'error'}
        <button
          onclick={handlePanicReset}
          class="w-full py-3 px-6 bg-red-700 hover:bg-red-600 rounded-xl font-medium transition-colors"
        >
          Сбросить сеть
        </button>
      {/if}
    </div>

    <!-- Services -->
    <div class="bg-gray-800 rounded-2xl p-6">
      <h2 class="text-lg font-medium mb-4">Сервисы</h2>
      <div class="space-y-3">
        {#each $services as service}
          <div class="flex items-center justify-between">
            <span class="flex items-center gap-2">
              {service.name}
              {#if service.status === 'working'}
                <span class="text-xs text-green-500">✓</span>
              {:else if service.status === 'blocked'}
                <span class="text-xs text-red-500">✗</span>
              {/if}
            </span>
            <span class="w-2 h-2 rounded-full" class:bg-green-500={service.status === 'working'} class:bg-red-500={service.status === 'blocked'} class:bg-gray-500={service.status === 'unknown'}></span>
          </div>
        {:else}
          <div class="flex items-center justify-between">
            <span>Discord</span>
            <span class="w-2 h-2 bg-gray-500 rounded-full"></span>
          </div>
          <div class="flex items-center justify-between">
            <span>YouTube</span>
            <span class="w-2 h-2 bg-gray-500 rounded-full"></span>
          </div>
          <div class="flex items-center justify-between">
            <span>Telegram</span>
            <span class="w-2 h-2 bg-gray-500 rounded-full"></span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>
