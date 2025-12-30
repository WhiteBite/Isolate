<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  
  let status = $state<'idle' | 'active' | 'optimizing' | 'error'>('idle');
  let currentStrategy = $state<string | null>(null);
  let services = $state<{id: string; name: string; critical: boolean}[]>([
    {id: 'discord', name: 'Discord', critical: true},
    {id: 'youtube', name: 'YouTube', critical: true},
    {id: 'telegram', name: 'Telegram', critical: false}
  ]);
  let progress = $state<{
    stage: string;
    percent: number;
    message: string;
    current_strategy: string | null;
    tested_count: number;
    total_count: number;
    best_score: number | null;
  } | null>(null);
  let errorMessage = $state<string | null>(null);

  onMount(async () => {
    if (!browser) return;
    
    const { invoke } = await import('@tauri-apps/api/core');
    const { listen } = await import('@tauri-apps/api/event');
    
    // Load initial state
    try {
      const appStatus = await invoke<{is_active: boolean; current_strategy: string | null}>('get_status');
      status = appStatus.is_active ? 'active' : 'idle';
      currentStrategy = appStatus.current_strategy;
      
      const loadedServices = await invoke<{id: string; name: string; critical: boolean}[]>('get_services');
      if (loadedServices.length > 0) {
        services = loadedServices;
      }
    } catch (e) {
      console.error('Failed to load initial state:', e);
    }
    
    // Subscribe to optimization events
    const unlistenProgress = await listen('optimization:progress', (event) => {
      progress = event.payload as typeof progress;
    });
    
    const unlistenComplete = await listen('optimization:complete', (event) => {
      const result = event.payload as {strategy_id: string; score: number};
      status = 'active';
      currentStrategy = result.strategy_id;
      progress = null;
      errorMessage = null;
    });
    
    const unlistenFailed = await listen('optimization:failed', (event) => {
      status = 'error';
      errorMessage = event.payload as string;
      progress = null;
    });
    
    return () => {
      unlistenProgress();
      unlistenComplete();
      unlistenFailed();
    };
  });

  async function handleOptimize() {
    if (!browser) return;
    
    status = 'optimizing';
    errorMessage = null;
    progress = {
      stage: 'initializing',
      percent: 0,
      message: 'Начинаем оптимизацию...',
      current_strategy: null,
      tested_count: 0,
      total_count: 0,
      best_score: null
    };
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('run_optimization', { mode: 'turbo' });
    } catch (e) {
      console.error('Optimization failed:', e);
      status = 'error';
      errorMessage = String(e);
    }
  }

  async function handleStop() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('stop_strategy');
      status = 'idle';
      currentStrategy = null;
    } catch (e) {
      console.error('Failed to stop strategy:', e);
    }
  }
  
  async function handlePanicReset() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('panic_reset');
      status = 'idle';
      currentStrategy = null;
      progress = null;
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
</script>

<div class="flex flex-col items-center justify-center min-h-screen p-8">
  <div class="w-full max-w-md space-y-8">
    <!-- Header -->
    <div class="text-center">
      <h1 class="text-4xl font-bold text-primary-400">Isolate</h1>
      <p class="mt-2 text-gray-400">Автоматический обход DPI-блокировок</p>
    </div>

    <!-- Status Card -->
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

      {#if currentStrategy}
        <div class="flex items-center justify-between">
          <span class="text-gray-400">Стратегия</span>
          <span class="text-white">{currentStrategy}</span>
        </div>
      {/if}
      
      {#if errorMessage}
        <div class="text-red-400 text-sm">{errorMessage}</div>
      {/if}
    </div>
    
    <!-- Progress -->
    {#if status === 'optimizing' && progress}
      <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
        <div class="flex items-center justify-between text-sm">
          <span class="text-gray-400">{getStageText(progress.stage)}</span>
          <span class="text-primary-400">{progress.percent}%</span>
        </div>
        
        <div class="w-full bg-gray-700 rounded-full h-2">
          <div 
            class="bg-primary-500 h-2 rounded-full transition-all duration-300"
            style="width: {progress.percent}%"
          ></div>
        </div>
        
        <p class="text-sm text-gray-400">{progress.message}</p>
        
        {#if progress.current_strategy}
          <p class="text-sm text-gray-500">
            Тестируем: {progress.current_strategy}
          </p>
        {/if}
        
        {#if progress.tested_count > 0}
          <p class="text-sm text-gray-500">
            Протестировано: {progress.tested_count} / {progress.total_count}
          </p>
        {/if}
      </div>
    {/if}

    <!-- Actions -->
    <div class="space-y-3">
      <button
        onclick={handleOptimize}
        disabled={status === 'optimizing'}
        class="w-full py-4 px-6 bg-primary-600 hover:bg-primary-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-xl font-medium transition-colors"
      >
        {status === 'optimizing' ? 'Оптимизация...' : 'Оптимизировать'}
      </button>

      {#if status === 'active'}
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
        {#each services as service}
          <div class="flex items-center justify-between">
            <span class="flex items-center gap-2">
              {service.name}
              {#if service.critical}
                <span class="text-xs text-yellow-500">★</span>
              {/if}
            </span>
            <span class="w-2 h-2 bg-gray-500 rounded-full"></span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>
