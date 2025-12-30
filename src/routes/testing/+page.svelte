<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import Spinner from '$lib/components/Spinner.svelte';

  // Types
  interface ProxyConfig {
    id: string;
    name: string;
    protocol: string;
    server: string;
    port: number;
  }

  interface Strategy {
    id: string;
    name: string;
    family: string;
    engine: string;
  }

  interface Service {
    id: string;
    name: string;
    critical: boolean;
  }

  interface TestResult {
    id: string;
    name: string;
    type: 'proxy' | 'strategy';
    success_rate: number;
    latency: number | null;
    score: number;
    status: 'pending' | 'testing' | 'success' | 'partial' | 'failed';
  }

  interface TestProgress {
    current: string;
    currentIndex: number;
    total: number;
    percent: number;
  }

  // State
  let proxies = $state<ProxyConfig[]>([]);
  let strategies = $state<Strategy[]>([]);
  let services = $state<Service[]>([
    { id: 'discord', name: 'Discord', critical: true },
    { id: 'youtube', name: 'YouTube', critical: true },
    { id: 'telegram', name: 'Telegram', critical: false },
    { id: 'twitch', name: 'Twitch', critical: false },
    { id: 'spotify', name: 'Spotify', critical: false }
  ]);

  let selectedProxies = $state<Set<string>>(new Set());
  let selectedStrategies = $state<Set<string>>(new Set());
  let selectedServices = $state<Set<string>>(new Set(['discord', 'youtube']));

  let testMode = $state<'turbo' | 'deep'>('turbo');
  let isInteractive = $state(false);

  let isTesting = $state(false);
  let progress = $state<TestProgress | null>(null);
  let results = $state<TestResult[]>([]);
  let sortBy = $state<'score' | 'latency' | 'success_rate'>('score');
  let sortDesc = $state(true);

  // Toast state
  let toasts = $state<{id: number; type: 'success' | 'error' | 'info'; message: string}[]>([]);
  let toastId = 0;

  let unlistenProgress: (() => void) | null = null;
  let unlistenResult: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;

  function showToast(type: 'success' | 'error' | 'info', message: string) {
    const id = ++toastId;
    toasts = [...toasts, { id, type, message }];
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id);
    }, 4000);
  }

  onMount(async () => {
    if (!browser) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      // Load proxies (VLESS configs)
      const loadedProxies = await invoke<any[]>('get_vless_configs').catch(() => []);
      proxies = loadedProxies.map(p => ({
        id: p.id,
        name: p.name,
        protocol: 'VLESS',
        server: p.server,
        port: p.port
      }));

      // Load strategies
      const loadedStrategies = await invoke<Strategy[]>('get_strategies').catch(() => []);
      strategies = loadedStrategies;

      // Load services
      const loadedServices = await invoke<Service[]>('get_services').catch(() => null);
      if (loadedServices && loadedServices.length > 0) {
        services = loadedServices;
      }

      // Listen for test progress
      unlistenProgress = await listen('test:progress', (event) => {
        const payload = event.payload as TestProgress;
        progress = payload;
      });

      // Listen for test results
      unlistenResult = await listen('test:result', (event) => {
        const result = event.payload as TestResult;
        results = [...results, result];
      });

      // Listen for test complete
      unlistenComplete = await listen('test:complete', () => {
        isTesting = false;
        progress = null;
        showToast('success', 'Тестирование завершено');
      });
    } catch (e) {
      console.error('Failed to load data:', e);
      showToast('error', `Ошибка загрузки данных: ${e}`);
    }
  });

  onDestroy(() => {
    unlistenProgress?.();
    unlistenResult?.();
    unlistenComplete?.();
  });

  // Toggle functions
  function toggleProxy(id: string) {
    const newSet = new Set(selectedProxies);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedProxies = newSet;
  }

  function toggleStrategy(id: string) {
    const newSet = new Set(selectedStrategies);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedStrategies = newSet;
  }

  function toggleService(id: string) {
    const newSet = new Set(selectedServices);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedServices = newSet;
  }

  // Select all / clear all
  function selectAllProxies() {
    selectedProxies = new Set(proxies.map(p => p.id));
  }

  function clearAllProxies() {
    selectedProxies = new Set();
  }

  function selectAllStrategies() {
    selectedStrategies = new Set(strategies.map(s => s.id));
  }

  function clearAllStrategies() {
    selectedStrategies = new Set();
  }

  function selectAllServices() {
    selectedServices = new Set(services.map(s => s.id));
  }

  function clearAllServices() {
    selectedServices = new Set();
  }

  // Testing functions
  async function startTesting() {
    if (!browser) return;
    if (selectedProxies.size === 0 && selectedStrategies.size === 0) {
      showToast('error', 'Выберите хотя бы один прокси или стратегию');
      return;
    }
    if (selectedServices.size === 0) {
      showToast('error', 'Выберите хотя бы один сервис для проверки');
      return;
    }

    isTesting = true;
    results = [];
    progress = { current: 'Инициализация...', currentIndex: 0, total: 0, percent: 0 };

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('run_tests', {
        proxyIds: Array.from(selectedProxies),
        strategyIds: Array.from(selectedStrategies),
        serviceIds: Array.from(selectedServices),
        mode: testMode,
        interactive: isInteractive
      });
    } catch (e) {
      console.error('Testing failed:', e);
      showToast('error', `Ошибка тестирования: ${e}`);
      isTesting = false;
      progress = null;
    }
  }

  async function cancelTesting() {
    if (!browser) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('cancel_tests');
      showToast('info', 'Тестирование отменено');
    } catch (e) {
      console.error('Failed to cancel:', e);
      showToast('error', `Ошибка отмены: ${e}`);
    }
    
    isTesting = false;
    progress = null;
  }

  async function applyBest() {
    if (!browser || sortedResults.length === 0) return;

    const best = sortedResults[0];
    if (!best) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (best.type === 'strategy') {
        await invoke('apply_strategy', { strategyId: best.id });
        showToast('success', `Стратегия "${best.name}" применена`);
      } else {
        await invoke('start_vless_proxy', { configId: best.id });
        showToast('success', `Прокси "${best.name}" запущен`);
      }
    } catch (e) {
      console.error('Failed to apply:', e);
      showToast('error', `Ошибка применения: ${e}`);
    }
  }

  // Sorting
  function toggleSort(column: 'score' | 'latency' | 'success_rate') {
    if (sortBy === column) {
      sortDesc = !sortDesc;
    } else {
      sortBy = column;
      sortDesc = true;
    }
  }

  // Computed sorted results
  let sortedResults = $derived.by(() => {
    return [...results].sort((a, b) => {
      let diff = 0;
      switch (sortBy) {
        case 'score':
          diff = b.score - a.score;
          break;
        case 'latency':
          const aLat = a.latency ?? 9999;
          const bLat = b.latency ?? 9999;
          diff = aLat - bLat;
          break;
        case 'success_rate':
          diff = b.success_rate - a.success_rate;
          break;
      }
      return sortDesc ? diff : -diff;
    });
  });

  let canStartTest = $derived(
    (selectedProxies.size > 0 || selectedStrategies.size > 0) && selectedServices.size > 0
  );

  // Helper functions
  function getStatusBadge(status: string): string {
    switch (status) {
      case 'success': return 'bg-[#00ff88]/20 text-[#00ff88]';
      case 'partial': return 'bg-[#ffaa00]/20 text-[#ffaa00]';
      case 'failed': return 'bg-[#ff3333]/20 text-[#ff3333]';
      case 'testing': return 'bg-[#00d4ff]/20 text-[#00d4ff]';
      default: return 'bg-[#a0a0a0]/20 text-[#a0a0a0]';
    }
  }

  function getStatusText(status: string): string {
    switch (status) {
      case 'success': return 'Успешно';
      case 'partial': return 'Частично';
      case 'failed': return 'Ошибка';
      case 'testing': return 'Тестируется';
      case 'pending': return 'Ожидание';
      default: return status;
    }
  }

  function getSuccessRateColor(rate: number): string {
    if (rate >= 80) return 'text-[#00ff88]';
    if (rate >= 50) return 'text-[#ffaa00]';
    return 'text-[#ff3333]';
  }

  function getToastClass(type: string): string {
    switch (type) {
      case 'success': return 'bg-[#00ff88]/10 border-[#00ff88]/50 text-[#00ff88]';
      case 'error': return 'bg-[#ff3333]/10 border-[#ff3333]/50 text-[#ff3333]';
      case 'info': return 'bg-[#00d4ff]/10 border-[#00d4ff]/50 text-[#00d4ff]';
      default: return 'bg-[#a0a0a0]/10 border-[#a0a0a0]/50 text-[#a0a0a0]';
    }
  }
</script>

<!-- Toast notifications -->
<div class="fixed top-4 right-4 z-50 space-y-2">
  {#each toasts as toast (toast.id)}
    <div class="flex items-center gap-3 px-4 py-3 rounded-lg border shadow-lg backdrop-blur-sm {getToastClass(toast.type)} animate-in slide-in-from-right">
      {#if toast.type === 'success'}
        <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
      {:else if toast.type === 'error'}
        <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      {:else}
        <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      {/if}
      <span class="text-sm">{toast.message}</span>
    </div>
  {/each}
</div>

<div class="p-6 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold text-white">Тестирование</h1>
      <p class="text-[#a0a0a0] mt-1">Проверка прокси и стратегий на работоспособность</p>
    </div>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Selection Panel -->
    <div class="lg:col-span-2 space-y-6">

      <!-- Proxies Selection -->
      <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-white">Прокси</h2>
          {#if proxies.length > 0}
            <div class="flex gap-2">
              <button
                onclick={selectAllProxies}
                class="text-sm text-[#00d4ff] hover:text-[#00b8e6] transition-colors"
              >
                Выбрать все
              </button>
              <span class="text-[#2a2f4a]">|</span>
              <button
                onclick={clearAllProxies}
                class="text-sm text-[#a0a0a0] hover:text-white transition-colors"
              >
                Снять все
              </button>
            </div>
          {/if}
        </div>
        
        {#if proxies.length === 0}
          <p class="text-[#a0a0a0] text-sm">Нет доступных прокси</p>
        {:else}
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {#each proxies as proxy}
              <label class="flex items-center gap-3 p-3 bg-[#0a0e27] rounded-lg cursor-pointer hover:bg-[#2a2f4a]/50 transition-colors {selectedProxies.has(proxy.id) ? 'ring-1 ring-[#00d4ff]/50' : ''}">
                <input
                  type="checkbox"
                  checked={selectedProxies.has(proxy.id)}
                  onchange={() => toggleProxy(proxy.id)}
                  class="w-4 h-4 rounded bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff] focus:ring-offset-[#1a1f3a]"
                />
                <div class="flex-1 min-w-0">
                  <p class="text-white text-sm truncate">{proxy.name}</p>
                  <p class="text-[#a0a0a0] text-xs">{proxy.protocol} • {proxy.server}:{proxy.port}</p>
                </div>
              </label>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Strategies Selection -->
      <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-white">Стратегии</h2>
          {#if strategies.length > 0}
            <div class="flex gap-2">
              <button
                onclick={selectAllStrategies}
                class="text-sm text-[#00d4ff] hover:text-[#00b8e6] transition-colors"
              >
                Выбрать все
              </button>
              <span class="text-[#2a2f4a]">|</span>
              <button
                onclick={clearAllStrategies}
                class="text-sm text-[#a0a0a0] hover:text-white transition-colors"
              >
                Снять все
              </button>
            </div>
          {/if}
        </div>
        
        {#if strategies.length === 0}
          <p class="text-[#a0a0a0] text-sm">Нет доступных стратегий</p>
        {:else}
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {#each strategies as strategy}
              <label class="flex items-center gap-3 p-3 bg-[#0a0e27] rounded-lg cursor-pointer hover:bg-[#2a2f4a]/50 transition-colors {selectedStrategies.has(strategy.id) ? 'ring-1 ring-[#00d4ff]/50' : ''}">
                <input
                  type="checkbox"
                  checked={selectedStrategies.has(strategy.id)}
                  onchange={() => toggleStrategy(strategy.id)}
                  class="w-4 h-4 rounded bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff] focus:ring-offset-[#1a1f3a]"
                />
                <div class="flex-1 min-w-0">
                  <p class="text-white text-sm truncate">{strategy.name}</p>
                  <p class="text-[#a0a0a0] text-xs">{strategy.family} • {strategy.engine}</p>
                </div>
              </label>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Services Selection -->
      <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-white">Сервисы для проверки</h2>
          <div class="flex gap-2">
            <button
              onclick={selectAllServices}
              class="text-sm text-[#00d4ff] hover:text-[#00b8e6] transition-colors"
            >
              Выбрать все
            </button>
            <span class="text-[#2a2f4a]">|</span>
            <button
              onclick={clearAllServices}
              class="text-sm text-[#a0a0a0] hover:text-white transition-colors"
            >
              Снять все
            </button>
          </div>
        </div>
        <div class="flex flex-wrap gap-3">
          {#each services as service}
            <label class="flex items-center gap-2 px-4 py-2 bg-[#0a0e27] rounded-lg cursor-pointer hover:bg-[#2a2f4a]/50 transition-colors {selectedServices.has(service.id) ? 'ring-2 ring-[#00d4ff]' : ''}">
              <input
                type="checkbox"
                checked={selectedServices.has(service.id)}
                onchange={() => toggleService(service.id)}
                class="w-4 h-4 rounded bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff] focus:ring-offset-[#1a1f3a]"
              />
              <span class="text-white text-sm">{service.name}</span>
              {#if service.critical}
                <span class="text-[#ff3333] text-xs">●</span>
              {/if}
            </label>
          {/each}
        </div>
      </div>
    </div>

    <!-- Control Panel -->
    <div class="space-y-6">
      <!-- Test Mode -->
      <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
        <h2 class="text-lg font-semibold text-white mb-4">Режим тестирования</h2>
        
        <div class="space-y-3">
          <label class="flex items-center gap-3 p-3 bg-[#0a0e27] rounded-lg cursor-pointer hover:bg-[#2a2f4a]/50 transition-colors {testMode === 'turbo' ? 'ring-2 ring-[#00d4ff]' : ''}">
            <input
              type="radio"
              name="testMode"
              value="turbo"
              bind:group={testMode}
              class="w-4 h-4 bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff]"
            />
            <div>
              <p class="text-white font-medium">Turbo (быстро)</p>
              <p class="text-[#a0a0a0] text-xs">1 проверка на сервис, ~5 сек</p>
            </div>
          </label>
          
          <label class="flex items-center gap-3 p-3 bg-[#0a0e27] rounded-lg cursor-pointer hover:bg-[#2a2f4a]/50 transition-colors {testMode === 'deep' ? 'ring-2 ring-[#00d4ff]' : ''}">
            <input
              type="radio"
              name="testMode"
              value="deep"
              bind:group={testMode}
              class="w-4 h-4 bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff]"
            />
            <div>
              <p class="text-white font-medium">Deep (тщательно)</p>
              <p class="text-[#a0a0a0] text-xs">3 проверки, усреднение, ~15 сек</p>
            </div>
          </label>
        </div>

        <label class="flex items-center gap-3 mt-4 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={isInteractive}
            class="w-4 h-4 rounded bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff] focus:ring-offset-[#1a1f3a]"
          />
          <div>
            <span class="text-white text-sm">Интерактивное тестирование</span>
            <p class="text-[#a0a0a0] text-xs">Открывать браузер для проверки</p>
          </div>
        </label>
      </div>

      <!-- Start Button -->
      <button
        onclick={isTesting ? cancelTesting : startTesting}
        disabled={!canStartTest && !isTesting}
        class="w-full py-4 px-6 rounded-xl font-semibold text-lg transition-all duration-300 {isTesting 
          ? 'bg-[#ff3333] hover:bg-[#ff4444] text-white' 
          : canStartTest 
            ? 'bg-gradient-to-r from-[#00d4ff] to-[#00a8cc] hover:from-[#00b8e6] hover:to-[#0090b3] text-[#0a0e27]' 
            : 'bg-[#2a2f4a] text-[#a0a0a0] cursor-not-allowed'}"
      >
        {#if isTesting}
          <span class="flex items-center justify-center gap-2">
            <Spinner size="sm" color="white" />
            Отменить
          </span>
        {:else}
          Начать тестирование
        {/if}
      </button>

      <!-- Progress -->
      {#if progress}
        <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
          <div class="flex items-center justify-between mb-2">
            <span class="text-[#a0a0a0] text-sm">Прогресс</span>
            <span class="text-white font-medium">{progress.currentIndex} из {progress.total}</span>
          </div>
          
          <!-- Progress Bar -->
          <div class="h-3 bg-[#0a0e27] rounded-full overflow-hidden mb-3">
            <div 
              class="h-full bg-gradient-to-r from-[#00d4ff] to-[#00a8cc] rounded-full transition-all duration-300"
              style="width: {progress.percent}%"
            ></div>
          </div>
          
          <p class="text-white text-sm truncate">{progress.current}</p>
          <p class="text-[#00d4ff] text-lg font-bold mt-1">{progress.percent}%</p>
        </div>
      {/if}

      <!-- Quick Stats -->
      {#if results.length > 0 && !isTesting}
        <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
          <h3 class="text-white font-medium mb-3">Статистика</h3>
          <div class="grid grid-cols-2 gap-4 text-center">
            <div class="bg-[#0a0e27] rounded-lg p-3">
              <p class="text-[#00ff88] text-2xl font-bold">{results.filter(r => r.status === 'success').length}</p>
              <p class="text-[#a0a0a0] text-xs">Успешно</p>
            </div>
            <div class="bg-[#0a0e27] rounded-lg p-3">
              <p class="text-[#ff3333] text-2xl font-bold">{results.filter(r => r.status === 'failed').length}</p>
              <p class="text-[#a0a0a0] text-xs">Ошибки</p>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Results Table -->
  {#if results.length > 0}
    <div class="bg-[#1a1f3a] rounded-xl border border-[#2a2f4a] overflow-hidden">
      <div class="flex items-center justify-between p-5 border-b border-[#2a2f4a]">
        <h2 class="text-lg font-semibold text-white">Результаты</h2>
        <button
          onclick={applyBest}
          class="px-4 py-2 bg-gradient-to-r from-[#00d4ff] to-[#00a8cc] hover:from-[#00b8e6] hover:to-[#0090b3] text-[#0a0e27] rounded-lg font-medium transition-all flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          Применить лучший
        </button>
      </div>
      
      <div class="overflow-x-auto">
        <table class="w-full">
          <thead class="bg-[#0a0e27]">
            <tr>
              <th class="px-5 py-3 text-left text-xs font-medium text-[#a0a0a0] uppercase tracking-wider">
                Название
              </th>
              <th class="px-5 py-3 text-left text-xs font-medium text-[#a0a0a0] uppercase tracking-wider">
                Тип
              </th>
              <th 
                class="px-5 py-3 text-left text-xs font-medium text-[#a0a0a0] uppercase tracking-wider cursor-pointer hover:text-white transition-colors"
                onclick={() => toggleSort('success_rate')}
              >
                <span class="flex items-center gap-1">
                  Success Rate
                  {#if sortBy === 'success_rate'}
                    <span class="text-[#00d4ff]">{sortDesc ? '↓' : '↑'}</span>
                  {/if}
                </span>
              </th>
              <th 
                class="px-5 py-3 text-left text-xs font-medium text-[#a0a0a0] uppercase tracking-wider cursor-pointer hover:text-white transition-colors"
                onclick={() => toggleSort('latency')}
              >
                <span class="flex items-center gap-1">
                  Latency
                  {#if sortBy === 'latency'}
                    <span class="text-[#00d4ff]">{sortDesc ? '↓' : '↑'}</span>
                  {/if}
                </span>
              </th>
              <th 
                class="px-5 py-3 text-left text-xs font-medium text-[#a0a0a0] uppercase tracking-wider cursor-pointer hover:text-white transition-colors"
                onclick={() => toggleSort('score')}
              >
                <span class="flex items-center gap-1">
                  Score
                  {#if sortBy === 'score'}
                    <span class="text-[#00d4ff]">{sortDesc ? '↓' : '↑'}</span>
                  {/if}
                </span>
              </th>
              <th class="px-5 py-3 text-left text-xs font-medium text-[#a0a0a0] uppercase tracking-wider">
                Статус
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-[#2a2f4a]">
            {#each sortedResults as result, i}
              <tr class="hover:bg-[#2a2f4a]/30 transition-colors {i === 0 ? 'bg-[#00d4ff]/5' : ''}">
                <td class="px-5 py-4">
                  <div class="flex items-center gap-2">
                    {#if i === 0}
                      <span class="text-yellow-400">🏆</span>
                    {/if}
                    <span class="text-white font-medium">{result.name}</span>
                  </div>
                </td>
                <td class="px-5 py-4">
                  <span class="px-2 py-1 text-xs rounded-full {result.type === 'proxy' ? 'bg-purple-500/20 text-purple-400' : 'bg-blue-500/20 text-blue-400'}">
                    {result.type === 'proxy' ? 'Прокси' : 'Стратегия'}
                  </span>
                </td>
                <td class="px-5 py-4">
                  <span class="{getSuccessRateColor(result.success_rate)} font-medium">
                    {result.success_rate}%
                  </span>
                </td>
                <td class="px-5 py-4 text-[#a0a0a0]">
                  {result.latency !== null ? `${result.latency}ms` : '—'}
                </td>
                <td class="px-5 py-4">
                  <span class="text-[#00d4ff] font-bold">{result.score}</span>
                </td>
                <td class="px-5 py-4">
                  <span class="px-2 py-1 text-xs rounded-full {getStatusBadge(result.status)}">
                    {getStatusText(result.status)}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>
