<script lang="ts">
  import { browser } from '$app/environment';
  import Spinner from '$lib/components/Spinner.svelte';
  import { toasts } from '$lib/stores/toast';

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
    item_type: 'proxy' | 'strategy';
    success_rate: number;
    latency_ms: number;
    score: number;
    services_tested: string[];
    services_passed: string[];
  }

  interface TestProgress {
    current_item: string;
    current_type: string;
    tested_count: number;
    total_count: number;
    percent: number;
  }

  // Testing stage types
  type StageStatus = 'pending' | 'running' | 'done' | 'failed';
  
  interface TestStage {
    id: string;
    name: string;
    description: string;
    status: StageStatus;
    duration?: number;
    error?: string;
  }

  // State
  let proxies = $state<ProxyConfig[]>([]);
  let strategies = $state<Strategy[]>([]);
  let isTauri = $state(false);
  let services = $state<Service[]>([
    { id: 'discord', name: 'Discord', critical: true },
    { id: 'youtube', name: 'YouTube', critical: true },
    { id: 'telegram', name: 'Telegram', critical: false },
    { id: 'twitch', name: 'Twitch', critical: false },
    { id: 'spotify', name: 'Spotify', critical: false }
  ]);

  // Demo data for browser mode
  function getDemoStrategies(): Strategy[] {
    return [
      { id: 'discord-zapret-1', name: 'Discord Zapret Basic', family: 'zapret', engine: 'zapret' },
      { id: 'youtube-zapret-1', name: 'YouTube Zapret', family: 'zapret', engine: 'zapret' },
      { id: 'universal-vless-1', name: 'Universal VLESS', family: 'vless', engine: 'singbox' }
    ];
  }

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

  // Testing stages for visualization
  let testStages = $state<TestStage[]>([]);
  let currentStageIndex = $state(-1);
  let testStartTime = $state<number | null>(null);
  let elapsedTime = $state(0);
  let elapsedInterval: ReturnType<typeof setInterval> | null = null;

  // Live results animation
  let lastResultId = $state<string | null>(null);

  let unlistenProgress: (() => void) | null = null;
  let unlistenResult: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;
  let unlistenStage: (() => void) | null = null;

  // Initialize test stages based on selection
  function initTestStages() {
    const stages: TestStage[] = [
      { id: 'init', name: 'Инициализация', description: 'Подготовка тестового окружения', status: 'pending' },
    ];
    
    if (selectedStrategies.size > 0) {
      stages.push({ id: 'strategies', name: 'Стратегии', description: `Тестирование ${selectedStrategies.size} стратегий`, status: 'pending' });
    }
    
    if (selectedProxies.size > 0) {
      stages.push({ id: 'proxies', name: 'Прокси', description: `Тестирование ${selectedProxies.size} прокси`, status: 'pending' });
    }
    
    stages.push({ id: 'scoring', name: 'Оценка', description: 'Расчёт итоговых баллов', status: 'pending' });
    stages.push({ id: 'complete', name: 'Завершение', description: 'Формирование отчёта', status: 'pending' });
    
    testStages = stages;
    currentStageIndex = -1;
  }

  // Update stage status
  function updateStage(stageId: string, status: StageStatus, error?: string) {
    testStages = testStages.map((stage, idx) => {
      if (stage.id === stageId) {
        currentStageIndex = idx;
        return { ...stage, status, error };
      }
      return stage;
    });
  }

  // Start elapsed time counter
  function startElapsedTimer() {
    testStartTime = Date.now();
    elapsedTime = 0;
    elapsedInterval = setInterval(() => {
      if (testStartTime) {
        elapsedTime = Math.floor((Date.now() - testStartTime) / 1000);
      }
    }, 1000);
  }

  // Stop elapsed time counter
  function stopElapsedTimer() {
    if (elapsedInterval) {
      clearInterval(elapsedInterval);
      elapsedInterval = null;
    }
  }

  // Format elapsed time
  function formatElapsedTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return mins > 0 ? `${mins}м ${secs}с` : `${secs}с`;
  }

  // Load data with backend ready check and retry logic
  async function loadData(retries = 10) {
    if (!browser) return;
    
    console.log('[Testing] loadData started');
    
    // Check if we're in Tauri environment
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    console.log('[Testing] isTauri:', isTauri);
    
    if (!isTauri) {
      console.log('[Testing] Not in Tauri, using demo data');
      strategies = getDemoStrategies();
      return;
    }

    try {
      console.log('[Testing] Importing @tauri-apps/api/core...');
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');
      console.log('[Testing] Tauri API imported');

      // Wait for backend to be ready with retry logic
      for (let i = 0; i < retries; i++) {
        try {
          const ready = await invoke<boolean>('is_backend_ready');
          if (ready) {
            console.log('[Testing] Backend is ready');
            break;
          }
          console.log(`[Testing] Backend not ready, retry ${i + 1}/${retries}`);
          await new Promise(r => setTimeout(r, 200));
        } catch {
          console.log(`[Testing] Backend check failed, retry ${i + 1}/${retries}`);
          await new Promise(r => setTimeout(r, 200));
        }
        
        // Fallback to demo data after all retries
        if (i === retries - 1) {
          console.warn('[Testing] Backend not ready after retries, using demo data');
          strategies = getDemoStrategies();
          return;
        }
      }

      // Load proxies (VLESS configs)
      try {
        const loadedProxies = await invoke<any[]>('get_vless_configs');
        proxies = loadedProxies.map(p => ({
          id: p.id,
          name: p.name,
          protocol: 'VLESS',
          server: p.server,
          port: p.port
        }));
        console.log('[Testing] Loaded proxies:', proxies.length);
      } catch (e) {
        console.warn('[Testing] Failed to load proxies:', e);
      }

      // Load strategies
      try {
        const loadedStrategies = await invoke<Strategy[]>('get_strategies');
        strategies = loadedStrategies;
        console.log('[Testing] Loaded strategies:', strategies.length);
      } catch (e) {
        console.warn('[Testing] Failed to load strategies:', e);
        strategies = getDemoStrategies();
      }

      // Load services
      try {
        const loadedServices = await invoke<Service[]>('get_services');
        if (loadedServices && loadedServices.length > 0) {
          services = loadedServices;
        }
        console.log('[Testing] Loaded services:', services.length);
      } catch (e) {
        console.warn('[Testing] Failed to load services:', e);
      }

      // Listen for test progress
      unlistenProgress = await listen('test:progress', (event) => {
        const payload = event.payload as TestProgress;
        progress = payload;
        
        // Update stages based on progress
        if (payload.current_type === 'strategy') {
          updateStage('init', 'done');
          updateStage('strategies', 'running');
        } else if (payload.current_type === 'proxy') {
          updateStage('strategies', 'done');
          updateStage('proxies', 'running');
        }
      });

      // Listen for test results
      unlistenResult = await listen('test:result', (event) => {
        const result = event.payload as TestResult;
        results = [...results, result];
        lastResultId = result.id;
        
        // Clear highlight after animation
        setTimeout(() => {
          if (lastResultId === result.id) {
            lastResultId = null;
          }
        }, 1500);
      });

      // Listen for test complete
      unlistenComplete = await listen('test:complete', () => {
        // Mark all stages as done
        testStages = testStages.map(stage => ({ ...stage, status: 'done' as StageStatus }));
        stopElapsedTimer();
        isTesting = false;
        progress = null;
        toasts.success('Тестирование завершено');
      });
      
      // Listen for stage updates
      unlistenStage = await listen('test:stage', (event) => {
        const { stage_id, status, error } = event.payload as { stage_id: string; status: StageStatus; error?: string };
        updateStage(stage_id, status, error);
      });
    } catch (e) {
      console.error('[Testing] Failed to load data:', e);
      toasts.error(`Ошибка загрузки данных: ${e}`);
      strategies = getDemoStrategies();
    }
  }

  // Initialize on component mount using $effect
  $effect(() => {
    loadData();
    
    // Cleanup function
    return () => {
      unlistenProgress?.();
      unlistenResult?.();
      unlistenComplete?.();
      unlistenStage?.();
      stopElapsedTimer();
    };
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
      toasts.error('Выберите хотя бы один прокси или стратегию');
      return;
    }
    if (selectedServices.size === 0) {
      toasts.error('Выберите хотя бы один сервис для проверки');
      return;
    }

    isTesting = true;
    results = [];
    progress = { current_item: 'Инициализация...', current_type: '', tested_count: 0, total_count: 0, percent: 0 };
    
    // Initialize stages and timer
    initTestStages();
    startElapsedTimer();
    updateStage('init', 'running');

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Check backend ready before running tests
      const ready = await invoke<boolean>('is_backend_ready');
      if (!ready) {
        toasts.error('Бэкенд не готов, попробуйте позже');
        updateStage('init', 'failed', 'Backend not ready');
        stopElapsedTimer();
        isTesting = false;
        progress = null;
        return;
      }
      
      await invoke('run_tests', {
        proxyIds: Array.from(selectedProxies),
        strategyIds: Array.from(selectedStrategies),
        serviceIds: Array.from(selectedServices),
        mode: testMode,
        interactive: isInteractive
      });
    } catch (e) {
      console.error('Testing failed:', e);
      toasts.error(`Ошибка тестирования: ${e}`);
      // Mark current stage as failed
      const currentStage = testStages[currentStageIndex];
      if (currentStage) {
        updateStage(currentStage.id, 'failed', String(e));
      }
      stopElapsedTimer();
      isTesting = false;
      progress = null;
    }
  }

  async function cancelTesting() {
    if (!browser) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('cancel_tests');
      toasts.info('Тестирование отменено');
    } catch (e) {
      console.error('Failed to cancel:', e);
      toasts.error(`Ошибка отмены: ${e}`);
    }
    
    // Mark remaining stages as failed
    testStages = testStages.map(stage => 
      stage.status === 'pending' || stage.status === 'running' 
        ? { ...stage, status: 'failed' as StageStatus, error: 'Отменено пользователем' }
        : stage
    );
    stopElapsedTimer();
    isTesting = false;
    progress = null;
  }

  async function applyBest() {
    if (!browser || sortedResults.length === 0) return;

    const best = sortedResults[0];
    if (!best) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (best.item_type === 'strategy') {
        await invoke('apply_strategy', { strategyId: best.id });
        toasts.success(`Стратегия "${best.name}" применена`);
      } else {
        await invoke('apply_proxy', { id: best.id });
        toasts.success(`Прокси "${best.name}" запущен`);
      }
    } catch (e) {
      console.error('Failed to apply:', e);
      toasts.error(`Ошибка применения: ${e}`);
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
          const aLat = a.latency_ms >= 9999 ? 99999 : a.latency_ms;
          const bLat = b.latency_ms >= 9999 ? 99999 : b.latency_ms;
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

  function getResultStatus(result: TestResult): string {
    if (result.success_rate >= 80) return 'success';
    if (result.success_rate >= 50) return 'partial';
    if (result.success_rate > 0) return 'partial';
    return 'failed';
  }

  // Stage icon and color helpers
  function getStageIcon(status: StageStatus): string {
    switch (status) {
      case 'done': return '✓';
      case 'running': return '●';
      case 'failed': return '✕';
      default: return '○';
    }
  }

  function getStageColor(status: StageStatus): string {
    switch (status) {
      case 'done': return 'text-[#00ff88]';
      case 'running': return 'text-[#00d4ff]';
      case 'failed': return 'text-[#ff3333]';
      default: return 'text-[#4a4f6a]';
    }
  }

  function getStageBgColor(status: StageStatus): string {
    switch (status) {
      case 'done': return 'bg-[#00ff88]/10 border-[#00ff88]/30';
      case 'running': return 'bg-[#00d4ff]/10 border-[#00d4ff]/30';
      case 'failed': return 'bg-[#ff3333]/10 border-[#ff3333]/30';
      default: return 'bg-[#1a1f3a] border-[#2a2f4a]';
    }
  }

  function getStageGlow(status: StageStatus): string {
    switch (status) {
      case 'done': return 'shadow-[0_0_15px_rgba(0,255,136,0.3)]';
      case 'running': return 'shadow-[0_0_20px_rgba(0,212,255,0.4)] animate-pulse';
      case 'failed': return 'shadow-[0_0_15px_rgba(255,51,51,0.3)]';
      default: return '';
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

<div class="p-8 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-white">Тестирование</h1>
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
      {#if isTesting || testStages.length > 0}
        <div class="bg-[#1a1f3a]/80 backdrop-blur-sm rounded-xl p-5 border border-[#2a2f4a] space-y-4">
          <!-- Header with timer -->
          <div class="flex items-center justify-between">
            <h3 class="text-white font-semibold flex items-center gap-2">
              {#if isTesting}
                <span class="relative flex h-3 w-3">
                  <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#00d4ff] opacity-75"></span>
                  <span class="relative inline-flex rounded-full h-3 w-3 bg-[#00d4ff]"></span>
                </span>
              {/if}
              Прогресс тестирования
            </h3>
            {#if testStartTime}
              <span class="text-[#a0a0a0] text-sm font-mono">
                ⏱ {formatElapsedTime(elapsedTime)}
              </span>
            {/if}
          </div>

          <!-- Enhanced Progress Bar -->
          {#if progress}
            <div class="space-y-2">
              <div class="flex items-center justify-between text-sm">
                <span class="text-[#a0a0a0]">{progress.tested_count} из {progress.total_count}</span>
                <span class="text-[#00d4ff] font-bold text-lg">{progress.percent}%</span>
              </div>
              
              <!-- Glowing Progress Bar -->
              <div class="relative h-4 bg-[#0a0e27] rounded-full overflow-hidden">
                <!-- Background glow -->
                <div 
                  class="absolute inset-0 bg-gradient-to-r from-[#00d4ff]/20 to-[#00ff88]/20 blur-sm transition-all duration-500"
                  style="width: {progress.percent}%"
                ></div>
                <!-- Main bar -->
                <div 
                  class="relative h-full bg-gradient-to-r from-[#00d4ff] via-[#00a8cc] to-[#00ff88] rounded-full transition-all duration-300 ease-out"
                  style="width: {progress.percent}%"
                >
                  <!-- Shimmer effect -->
                  <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer"></div>
                </div>
                <!-- Glow on edge -->
                {#if progress.percent > 0 && progress.percent < 100}
                  <div 
                    class="absolute top-0 h-full w-2 bg-white/50 blur-sm transition-all duration-300"
                    style="left: calc({progress.percent}% - 4px)"
                  ></div>
                {/if}
              </div>
              
              <!-- Current item -->
              <div class="flex items-center gap-2 mt-2">
                <span class="px-2 py-0.5 text-xs rounded-full {progress.current_type === 'proxy' ? 'bg-purple-500/20 text-purple-400' : 'bg-blue-500/20 text-blue-400'}">
                  {progress.current_type === 'proxy' ? 'Прокси' : 'Стратегия'}
                </span>
                <span class="text-white text-sm truncate">{progress.current_item}</span>
              </div>
            </div>
          {/if}

          <!-- Test Stages Visualization -->
          <div class="space-y-2 pt-2 border-t border-[#2a2f4a]">
            <span class="text-[#a0a0a0] text-xs uppercase tracking-wider">Этапы</span>
            <div class="space-y-2">
              {#each testStages as stage, idx}
                <div 
                  class="flex items-center gap-3 p-3 rounded-lg border transition-all duration-300 {getStageBgColor(stage.status)} {getStageGlow(stage.status)}"
                >
                  <!-- Stage indicator -->
                  <div class="flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center {stage.status === 'running' ? 'bg-[#00d4ff]/20' : stage.status === 'done' ? 'bg-[#00ff88]/20' : stage.status === 'failed' ? 'bg-[#ff3333]/20' : 'bg-[#2a2f4a]'}">
                    {#if stage.status === 'running'}
                      <Spinner size="sm" color="cyan" />
                    {:else}
                      <span class="{getStageColor(stage.status)} text-sm font-bold">{getStageIcon(stage.status)}</span>
                    {/if}
                  </div>
                  
                  <!-- Stage info -->
                  <div class="flex-1 min-w-0">
                    <p class="text-white text-sm font-medium">{stage.name}</p>
                    <p class="text-[#a0a0a0] text-xs truncate">
                      {#if stage.error}
                        <span class="text-[#ff3333]">{stage.error}</span>
                      {:else}
                        {stage.description}
                      {/if}
                    </p>
                  </div>
                  
                  <!-- Connection line to next stage -->
                  {#if idx < testStages.length - 1}
                    <div class="absolute left-[2.25rem] mt-12 w-0.5 h-4 {stage.status === 'done' ? 'bg-[#00ff88]/50' : 'bg-[#2a2f4a]'}"></div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        </div>
      {:else if progress}
        <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
          <div class="flex items-center justify-between mb-2">
            <span class="text-[#a0a0a0] text-sm">Прогресс</span>
            <span class="text-white font-medium">{progress.tested_count} из {progress.total_count}</span>
          </div>
          
          <!-- Progress Bar -->
          <div class="h-3 bg-[#0a0e27] rounded-full overflow-hidden mb-3">
            <div 
              class="h-full bg-gradient-to-r from-[#00d4ff] to-[#00a8cc] rounded-full transition-all duration-300"
              style="width: {progress.percent}%"
            ></div>
          </div>
          
          <p class="text-white text-sm truncate">{progress.current_item}</p>
          <p class="text-[#a0a0a0] text-xs mt-1">{progress.current_type === 'proxy' ? 'Прокси' : 'Стратегия'}</p>
          <p class="text-[#00d4ff] text-lg font-bold mt-1">{progress.percent}%</p>
        </div>
      {/if}

      <!-- Quick Stats -->
      {#if results.length > 0 && !isTesting}
        <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
          <h3 class="text-white font-medium mb-3">Статистика</h3>
          <div class="grid grid-cols-2 gap-4 text-center">
            <div class="bg-[#0a0e27] rounded-lg p-3">
              <p class="text-[#00ff88] text-2xl font-bold">{results.filter(r => r.success_rate >= 80).length}</p>
              <p class="text-[#a0a0a0] text-xs">Успешно</p>
            </div>
            <div class="bg-[#0a0e27] rounded-lg p-3">
              <p class="text-[#ff3333] text-2xl font-bold">{results.filter(r => r.success_rate < 50).length}</p>
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
                  Успешность
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
                  Задержка
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
                  Оценка
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
              <tr 
                class="transition-all duration-500 {i === 0 ? 'bg-[#00d4ff]/5' : ''} {lastResultId === result.id ? 'bg-[#00ff88]/10 animate-pulse' : 'hover:bg-[#2a2f4a]/30'}"
              >
                <td class="px-5 py-4">
                  <div class="flex items-center gap-2">
                    {#if i === 0 && !isTesting}
                      <span class="text-yellow-400 animate-bounce">🏆</span>
                    {:else if lastResultId === result.id}
                      <span class="text-[#00ff88] animate-ping">●</span>
                    {/if}
                    <span class="text-white font-medium">{result.name}</span>
                  </div>
                </td>
                <td class="px-5 py-4">
                  <span class="px-2 py-1 text-xs rounded-full {result.item_type === 'proxy' ? 'bg-purple-500/20 text-purple-400' : 'bg-blue-500/20 text-blue-400'}">
                    {result.item_type === 'proxy' ? 'Прокси' : 'Стратегия'}
                  </span>
                </td>
                <td class="px-5 py-4">
                  <div class="flex items-center gap-2">
                    <!-- Mini progress bar for success rate -->
                    <div class="w-16 h-2 bg-[#0a0e27] rounded-full overflow-hidden">
                      <div 
                        class="h-full rounded-full transition-all duration-500 {result.success_rate >= 80 ? 'bg-[#00ff88]' : result.success_rate >= 50 ? 'bg-[#ffaa00]' : 'bg-[#ff3333]'}"
                        style="width: {result.success_rate}%"
                      ></div>
                    </div>
                    <span class="{getSuccessRateColor(result.success_rate)} font-medium">
                      {result.success_rate.toFixed(1)}%
                    </span>
                  </div>
                </td>
                <td class="px-5 py-4 text-[#a0a0a0]">
                  {result.latency_ms < 9999 ? `${result.latency_ms}ms` : '—'}
                </td>
                <td class="px-5 py-4">
                  <span class="text-[#00d4ff] font-bold {lastResultId === result.id ? 'animate-pulse text-lg' : ''}">{result.score.toFixed(1)}</span>
                </td>
                <td class="px-5 py-4">
                  <span class="px-2 py-1 text-xs rounded-full {getStatusBadge(getResultStatus(result))}">
                    {getStatusText(getResultStatus(result))}
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

<style>
  @keyframes shimmer {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(100%);
    }
  }
  
  .animate-shimmer {
    animation: shimmer 2s infinite;
  }
</style>