<script lang="ts">
  import { browser } from '$app/environment';
  import { BentoGrid, BentoWidget, ProgressBar, Spinner } from '$lib/components';
  import { toasts } from '$lib/stores/toast';
  import { logs } from '$lib/stores/logs';

  // Types
  interface Strategy {
    id: string;
    name: string;
    family: string;
    engine: string;
  }

  interface QueueItem {
    id: string;
    name: string;
    type: 'strategy' | 'service';
    status: 'pending' | 'testing' | 'success' | 'failed' | 'skipped';
    score?: number;
    latency?: number;
  }

  interface OrchestraState {
    status: 'idle' | 'running' | 'learning' | 'paused' | 'completed' | 'error';
    currentItem: string | null;
    progress: number;
    totalItems: number;
    testedItems: number;
    bestStrategy: string | null;
    bestScore: number;
    startTime: number | null;
    elapsedTime: number;
  }

  // State
  let state = $state<OrchestraState>({
    status: 'idle',
    currentItem: null,
    progress: 0,
    totalItems: 0,
    testedItems: 0,
    bestStrategy: null,
    bestScore: 0,
    startTime: null,
    elapsedTime: 0
  });

  let queue = $state<QueueItem[]>([]);
  let strategies = $state<Strategy[]>([]);
  let logLines = $state<string[]>([]);
  let isTauri = $state(false);
  let initialized = $state(false);

  // Services for testing
  interface Service {
    id: string;
    name: string;
    icon: string;
  }
  
  let availableServices = $state<Service[]>([
    { id: 'youtube', name: 'YouTube', icon: '📺' },
    { id: 'discord', name: 'Discord', icon: '💬' },
    { id: 'telegram', name: 'Telegram', icon: '✈️' },
    { id: 'twitch', name: 'Twitch', icon: '🎮' },
    { id: 'spotify', name: 'Spotify', icon: '🎵' },
    { id: 'instagram', name: 'Instagram', icon: '📷' },
  ]);
  let selectedServices = $state<Set<string>>(new Set(['youtube', 'discord']));

  // Settings
  let mode = $state<'turbo' | 'deep'>('turbo');
  let autoApply = $state(true);

  // Cleanup
  let cleanupFns: (() => void)[] = [];
  let timerInterval: ReturnType<typeof setInterval> | null = null;

  // Derived
  let statusColor = $derived(
    state.status === 'running' ? 'text-emerald-400' :
    state.status === 'learning' ? 'text-amber-400' :
    state.status === 'paused' ? 'text-blue-400' :
    state.status === 'completed' ? 'text-cyan-400' :
    state.status === 'error' ? 'text-red-400' :
    'text-zinc-500'
  );

  let statusIcon = $derived(
    state.status === 'running' ? '🟢' :
    state.status === 'learning' ? '🔄' :
    state.status === 'paused' ? '⏸️' :
    state.status === 'completed' ? '✅' :
    state.status === 'error' ? '❌' :
    '⏹️'
  );

  let statusText = $derived(
    state.status === 'running' ? 'Running' :
    state.status === 'learning' ? 'Learning' :
    state.status === 'paused' ? 'Paused' :
    state.status === 'completed' ? 'Completed' :
    state.status === 'error' ? 'Error' :
    'Idle'
  );

  let canStart = $derived((state.status === 'idle' || state.status === 'completed' || state.status === 'error') && selectedServices.size > 0);
  let canPause = $derived(state.status === 'running' || state.status === 'learning');
  let canStop = $derived(state.status !== 'idle');

  let formattedTime = $derived(() => {
    const seconds = Math.floor(state.elapsedTime / 1000);
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  });

  // Toggle service selection
  function toggleService(id: string) {
    const newSet = new Set(selectedServices);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedServices = newSet;
  }

  // Demo data
  function getDemoStrategies(): Strategy[] {
    return [
      { id: 'discord-zapret-1', name: 'Discord Zapret Basic', family: 'zapret', engine: 'zapret' },
      { id: 'discord-zapret-2', name: 'Discord Zapret Advanced', family: 'zapret', engine: 'zapret' },
      { id: 'youtube-zapret-1', name: 'YouTube Zapret', family: 'zapret', engine: 'zapret' },
      { id: 'youtube-zapret-2', name: 'YouTube Zapret v2', family: 'zapret', engine: 'zapret' },
      { id: 'universal-vless-1', name: 'Universal VLESS', family: 'vless', engine: 'singbox' },
      { id: 'telegram-zapret-1', name: 'Telegram Zapret', family: 'zapret', engine: 'zapret' },
    ];
  }

  function getDemoQueue(): QueueItem[] {
    return getDemoStrategies().map(s => ({
      id: s.id,
      name: s.name,
      type: 'strategy' as const,
      status: 'pending' as const
    }));
  }

  // Wait for backend
  async function waitForBackend(retries = 10): Promise<boolean> {
    const { invoke } = await import('@tauri-apps/api/core');
    for (let i = 0; i < retries; i++) {
      try {
        const ready = await invoke<boolean>('is_backend_ready');
        if (ready) return true;
      } catch { /* Backend not ready */ }
      await new Promise(r => setTimeout(r, 200));
    }
    return false;
  }

  // Initialize
  async function initialize() {
    if (!browser || initialized) return;
    initialized = true;

    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;

    if (!isTauri) {
      // Demo mode
      strategies = getDemoStrategies();
      queue = getDemoQueue();
      addLog('[INFO] Demo mode - Orchestra simulation');
      return;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      const backendReady = await waitForBackend();
      if (!backendReady) {
        addLog('[ERROR] Backend not ready');
        strategies = getDemoStrategies();
        queue = getDemoQueue();
        return;
      }

      // Load strategies
      try {
        const loaded = await invoke<Strategy[]>('get_strategies');
        strategies = loaded;
        queue = loaded.map(s => ({
          id: s.id,
          name: s.name,
          type: 'strategy' as const,
          status: 'pending' as const
        }));
        addLog(`[INFO] Loaded ${loaded.length} strategies`);
      } catch (e) {
        addLog(`[ERROR] Failed to load strategies: ${e}`);
        strategies = getDemoStrategies();
        queue = getDemoQueue();
      }

      // Listen for optimization events
      const unlistenProgress = await listen('optimization:progress', (event) => {
        const payload = event.payload as { stage: string; percent: number; message: string; current_strategy?: string };
        state.progress = payload.percent;
        state.currentItem = payload.current_strategy || payload.stage;
        state.status = 'learning';
        addLog(`[PROGRESS] ${payload.message}`);

        // Update queue item status
        if (payload.current_strategy) {
          updateQueueItem(payload.current_strategy, 'testing');
        }
      });

      const unlistenComplete = await listen('optimization:complete', (event) => {
        const result = event.payload as { strategy_id: string; strategy_name: string; score: number };
        state.status = 'completed';
        state.bestStrategy = result.strategy_name;
        state.bestScore = result.score;
        state.progress = 100;
        addLog(`[SUCCESS] Best strategy: ${result.strategy_name} (score: ${result.score.toFixed(1)})`);
        
        // Mark best strategy as success, others as skipped (if backend used cache)
        queue = queue.map(item => {
          if (item.id === result.strategy_id) {
            return { ...item, status: 'success' as const, score: result.score };
          } else if (item.status === 'pending' || item.status === 'testing') {
            // Backend used cache, mark untested as skipped
            return { ...item, status: 'skipped' as const };
          }
          return item;
        });
        state.testedItems = queue.length;
        
        stopTimer();
        toasts.success(`Optimization complete: ${result.strategy_name}`);
      });

      const unlistenFailed = await listen('optimization:failed', (event) => {
        state.status = 'error';
        addLog(`[ERROR] Optimization failed: ${event.payload}`);
        stopTimer();
        toasts.error(`Optimization failed: ${event.payload}`);
      });

      const unlistenResult = await listen('strategy:test_result', (event) => {
        const result = event.payload as { strategy_id: string; success: boolean; score: number; latency_ms: number };
        const status = result.success ? 'success' : 'failed';
        updateQueueItem(result.strategy_id, status, result.score, result.latency_ms);
        state.testedItems++;
        addLog(`[TEST] ${result.strategy_id}: ${result.success ? '✓' : '✗'} score=${result.score.toFixed(1)}`);
      });

      cleanupFns.push(
        () => unlistenProgress(),
        () => unlistenComplete(),
        () => unlistenFailed(),
        () => unlistenResult()
      );

    } catch (e) {
      addLog(`[ERROR] Initialization failed: ${e}`);
      strategies = getDemoStrategies();
      queue = getDemoQueue();
    }
  }

  // Lifecycle
  $effect(() => {
    initialize();
    return () => {
      cleanupFns.forEach(fn => fn());
      cleanupFns = [];
      stopTimer();
    };
  });

  // Timer
  function startTimer() {
    state.startTime = Date.now();
    state.elapsedTime = 0;
    timerInterval = setInterval(() => {
      if (state.startTime) {
        state.elapsedTime = Date.now() - state.startTime;
      }
    }, 1000);
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  // Queue management
  function updateQueueItem(id: string, status: QueueItem['status'], score?: number, latency?: number) {
    queue = queue.map(item => 
      item.id === id ? { ...item, status, score, latency } : item
    );
  }

  function resetQueue() {
    queue = queue.map(item => ({ ...item, status: 'pending' as const, score: undefined, latency: undefined }));
  }

  // Logging
  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    logLines = [...logLines.slice(-99), `[${timestamp}] ${message}`];
    logs.debug('orchestra', message);
  }

  // Actions
  async function startOptimization() {
    if (!canStart) return;

    resetQueue();
    state = {
      ...state,
      status: 'learning',
      progress: 0,
      testedItems: 0,
      totalItems: queue.length,
      currentItem: null,
      bestStrategy: null,
      bestScore: 0
    };
    startTimer();
    const servicesStr = Array.from(selectedServices).join(', ');
    addLog(`[START] Optimization started (mode: ${mode}, services: ${servicesStr})`);

    if (!isTauri) {
      // Demo simulation
      simulateOptimization();
      return;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('run_optimization', { 
        mode,
        services: Array.from(selectedServices),
        ignoreCache: mode === 'deep' // Deep mode = full retest
      });
    } catch (e) {
      state.status = 'error';
      addLog(`[ERROR] Failed to start: ${e}`);
      stopTimer();
      toasts.error(`Failed to start optimization: ${e}`);
    }
  }

  async function pauseOptimization() {
    if (!canPause) return;
    state.status = 'paused';
    addLog('[PAUSE] Optimization paused');
    // TODO: Implement actual pause in backend
  }

  async function stopOptimization() {
    if (!canStop) return;
    state.status = 'idle';
    state.progress = 0;
    stopTimer();
    addLog('[STOP] Optimization stopped');

    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('cancel_optimization');
      } catch (e) {
        addLog(`[ERROR] Failed to stop: ${e}`);
      }
    }
  }

  function clearLogs() {
    logLines = [];
  }

  // Demo simulation
  async function simulateOptimization() {
    for (let i = 0; i < queue.length; i++) {
      if (state.status !== 'learning') break;

      const item = queue[i];
      state.currentItem = item.name;
      updateQueueItem(item.id, 'testing');
      addLog(`[TEST] Testing ${item.name}...`);

      await new Promise(r => setTimeout(r, 800 + Math.random() * 400));

      const success = Math.random() > 0.3;
      const score = success ? 50 + Math.random() * 50 : Math.random() * 30;
      const latency = 50 + Math.random() * 200;

      updateQueueItem(item.id, success ? 'success' : 'failed', score, latency);
      state.testedItems = i + 1;
      state.progress = ((i + 1) / queue.length) * 100;

      if (score > state.bestScore) {
        state.bestScore = score;
        state.bestStrategy = item.name;
      }

      addLog(`[RESULT] ${item.name}: ${success ? '✓' : '✗'} score=${score.toFixed(1)} latency=${latency.toFixed(0)}ms`);
    }

    if (state.status === 'learning') {
      state.status = 'completed';
      state.progress = 100;
      stopTimer();
      addLog(`[COMPLETE] Best: ${state.bestStrategy} (score: ${state.bestScore.toFixed(1)})`);
      toasts.success(`Optimization complete: ${state.bestStrategy}`);
    }
  }

  // Status badge helper
  function getStatusBadgeClass(status: QueueItem['status']): string {
    switch (status) {
      case 'testing': return 'bg-amber-500/20 text-amber-400 border-amber-500/30';
      case 'success': return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30';
      case 'failed': return 'bg-red-500/20 text-red-400 border-red-500/30';
      case 'skipped': return 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30';
      default: return 'bg-zinc-800/50 text-zinc-500 border-zinc-700/50';
    }
  }

  function getStatusIcon(status: QueueItem['status']): string {
    switch (status) {
      case 'testing': return '⏳';
      case 'success': return '✓';
      case 'failed': return '✗';
      case 'skipped': return '⏭';
      default: return '○';
    }
  }
</script>

<div class="h-full p-8 overflow-auto bg-gradient-to-br from-zinc-950 to-black">
  <!-- Page Header -->
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-white tracking-tight">Orchestra</h1>
    <p class="text-sm text-zinc-500 mt-2">Automatic strategy optimization and learning center</p>
  </div>

  <BentoGrid columns={4} gap={4}>
    <!-- Status Widget (2x1) -->
    <BentoWidget colspan={2} title="Status" icon="🎭">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <!-- Status indicator -->
          <div class="relative">
            <div class="w-16 h-16 rounded-2xl bg-zinc-800/50 border border-white/5 flex items-center justify-center">
              <span class="text-3xl">{statusIcon}</span>
            </div>
            {#if state.status === 'learning' || state.status === 'running'}
              <div class="absolute inset-0 rounded-2xl border-2 border-amber-500/50 animate-pulse"></div>
            {/if}
          </div>
          
          <div>
            <div class="flex items-center gap-2">
              <span class="text-xl font-semibold {statusColor}">{statusText}</span>
              {#if state.status === 'learning'}
                <Spinner size="sm" />
              {/if}
            </div>
            <p class="text-sm text-zinc-500 mt-1">
              {#if state.currentItem}
                Testing: {state.currentItem}
              {:else if state.bestStrategy}
                Best: {state.bestStrategy}
              {:else}
                Ready to optimize
              {/if}
            </p>
          </div>
        </div>

        <!-- Timer -->
        <div class="text-right">
          <div class="text-2xl font-mono text-zinc-300">{formattedTime()}</div>
          <p class="text-xs text-zinc-500 mt-1">Elapsed time</p>
        </div>
      </div>
    </BentoWidget>

    <!-- Progress Widget (2x1) -->
    <BentoWidget colspan={2} title="Progress" icon="📊">
      <div class="space-y-4">
        <!-- Progress bar -->
        <div>
          <div class="flex justify-between items-center mb-2">
            <span class="text-sm text-zinc-400">
              {state.testedItems} / {state.totalItems || queue.length} strategies
            </span>
            <span class="text-sm font-mono text-cyan-400">{state.progress.toFixed(0)}%</span>
          </div>
          <div class="h-3 bg-zinc-800/50 rounded-full overflow-hidden border border-white/5">
            <div 
              class="h-full bg-gradient-to-r from-cyan-500 to-indigo-500 rounded-full transition-all duration-500 ease-out"
              class:animate-pulse={state.status === 'learning'}
              style="width: {state.progress}%"
            ></div>
          </div>
        </div>

        <!-- Best score -->
        {#if state.bestScore > 0}
          <div class="flex items-center justify-between p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-lg">
            <div class="flex items-center gap-2">
              <span class="text-emerald-400">🏆</span>
              <span class="text-sm text-emerald-300">{state.bestStrategy}</span>
            </div>
            <span class="text-lg font-bold text-emerald-400">{state.bestScore.toFixed(1)}</span>
          </div>
        {/if}
      </div>
    </BentoWidget>

    <!-- Controls Widget (1x1) -->
    <BentoWidget title="Controls" icon="🎮">
      <div class="space-y-3">
        <!-- Mode selector -->
        <div class="flex gap-2">
          <button
            onclick={() => mode = 'turbo'}
            class="flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-all
              {mode === 'turbo' 
                ? 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30' 
                : 'bg-zinc-800/50 text-zinc-400 border border-white/5 hover:bg-zinc-800'}"
            title="Быстрый режим с кэшем"
          >
            ⚡ Turbo
          </button>
          <button
            onclick={() => mode = 'deep'}
            class="flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-all
              {mode === 'deep' 
                ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' 
                : 'bg-zinc-800/50 text-zinc-400 border border-white/5 hover:bg-zinc-800'}"
            title="Полный ретест всех стратегий"
          >
            🔬 Deep
          </button>
        </div>
        
        <!-- Mode description -->
        <p class="text-xs text-zinc-500 px-1">
          {mode === 'turbo' ? 'Использует кэш, быстрый результат' : 'Тестирует ВСЕ стратегии заново'}
        </p>

        <!-- Auto-apply toggle -->
        <label class="flex items-center justify-between p-3 bg-zinc-800/30 rounded-lg cursor-pointer hover:bg-zinc-800/50 transition-colors">
          <span class="text-sm text-zinc-300">Auto-apply best</span>
          <input 
            type="checkbox" 
            bind:checked={autoApply}
            class="w-5 h-5 rounded bg-zinc-700 border-zinc-600 text-cyan-500 focus:ring-cyan-500 focus:ring-offset-zinc-900"
          />
        </label>

        <!-- Action buttons -->
        <div class="flex gap-2">
          {#if canStart}
            <button
              onclick={startOptimization}
              disabled={selectedServices.size === 0}
              class="flex-1 px-4 py-3 rounded-xl font-semibold text-sm
                {selectedServices.size > 0 
                  ? 'bg-gradient-to-r from-cyan-500 to-indigo-500 text-white hover:from-cyan-400 hover:to-indigo-400 shadow-lg shadow-cyan-500/20'
                  : 'bg-zinc-800/50 text-zinc-600 cursor-not-allowed'}
                transition-all"
            >
              ▶ Start
            </button>
          {:else}
            <button
              onclick={pauseOptimization}
              disabled={!canPause}
              class="flex-1 px-4 py-3 rounded-xl font-semibold text-sm
                {canPause 
                  ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30 hover:bg-amber-500/30' 
                  : 'bg-zinc-800/50 text-zinc-600 cursor-not-allowed'}"
            >
              ⏸ Pause
            </button>
            <button
              onclick={stopOptimization}
              disabled={!canStop}
              class="flex-1 px-4 py-3 rounded-xl font-semibold text-sm
                {canStop 
                  ? 'bg-red-500/20 text-red-400 border border-red-500/30 hover:bg-red-500/30' 
                  : 'bg-zinc-800/50 text-zinc-600 cursor-not-allowed'}"
            >
              ⏹ Stop
            </button>
          {/if}
        </div>
      </div>
    </BentoWidget>

    <!-- Services Widget (1x1) -->
    <BentoWidget title="Test Services" icon="🎯">
      <div class="space-y-2 max-h-[180px] overflow-y-auto pr-1">
        {#each availableServices as service}
          <button
            onclick={() => toggleService(service.id)}
            disabled={state.status === 'learning'}
            class="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-all
              {selectedServices.has(service.id) 
                ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30' 
                : 'bg-zinc-800/30 text-zinc-400 border border-white/5 hover:bg-zinc-800/50'}
              {state.status === 'learning' ? 'opacity-50 cursor-not-allowed' : ''}"
          >
            <span>{service.icon}</span>
            <span class="flex-1 text-left">{service.name}</span>
            {#if selectedServices.has(service.id)}
              <svg class="w-4 h-4 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            {/if}
          </button>
        {/each}
      </div>
      <div class="mt-2 pt-2 border-t border-white/5 text-xs text-zinc-500 text-center">
        {selectedServices.size} selected
      </div>
    </BentoWidget>

    <!-- Stats Widget (1x1) - moved after Services -->
    <BentoWidget title="Statistics" icon="📈">
      <div class="grid grid-cols-2 gap-3">
        <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
          <div class="text-2xl font-bold text-emerald-400">
            {queue.filter(q => q.status === 'success').length}
          </div>
          <div class="text-xs text-zinc-500 mt-1">Passed</div>
        </div>
        <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
          <div class="text-2xl font-bold text-red-400">
            {queue.filter(q => q.status === 'failed').length}
          </div>
          <div class="text-xs text-zinc-500 mt-1">Failed</div>
        </div>
        <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
          <div class="text-2xl font-bold text-amber-400">
            {queue.filter(q => q.status === 'testing').length}
          </div>
          <div class="text-xs text-zinc-500 mt-1">Testing</div>
        </div>
        <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
          <div class="text-2xl font-bold text-zinc-400">
            {queue.filter(q => q.status === 'pending').length}
          </div>
          <div class="text-xs text-zinc-500 mt-1">Pending</div>
        </div>
      </div>
    </BentoWidget>

    <!-- Queue Widget (2x2) -->
    <BentoWidget colspan={2} rowspan={2} title="Strategy Queue" icon="📋">
      <div class="h-full flex flex-col">
        <div class="flex-1 overflow-auto space-y-2 pr-1 -mr-1">
          {#each queue as item, i (item.id)}
            <div 
              class="flex items-center gap-3 p-3 rounded-lg transition-all duration-200
                {item.status === 'testing' 
                  ? 'bg-amber-500/10 border border-amber-500/20' 
                  : 'bg-zinc-800/30 border border-white/5 hover:bg-zinc-800/50'}"
            >
              <!-- Index -->
              <div class="w-6 h-6 rounded-full bg-zinc-800 flex items-center justify-center text-xs text-zinc-500 font-mono">
                {i + 1}
              </div>

              <!-- Info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-zinc-200 truncate">{item.name}</span>
                  {#if item.status === 'testing'}
                    <Spinner size="xs" />
                  {/if}
                </div>
                <div class="flex items-center gap-2 mt-0.5">
                  {#if item.score !== undefined}
                    <span class="text-xs text-zinc-500">Score: <span class="text-cyan-400">{item.score.toFixed(1)}</span></span>
                  {/if}
                  {#if item.latency !== undefined}
                    <span class="text-xs text-zinc-500">Latency: <span class="text-zinc-400">{item.latency.toFixed(0)}ms</span></span>
                  {/if}
                </div>
              </div>

              <!-- Status badge -->
              <div class="px-2 py-1 rounded-md text-xs font-medium border {getStatusBadgeClass(item.status)}">
                <span class="mr-1">{getStatusIcon(item.status)}</span>
                {item.status}
              </div>
            </div>
          {/each}

          {#if queue.length === 0}
            <div class="flex flex-col items-center justify-center h-32 text-zinc-500">
              <span class="text-3xl mb-2">📭</span>
              <span class="text-sm">No strategies in queue</span>
            </div>
          {/if}
        </div>
      </div>
    </BentoWidget>

    <!-- Log Widget (2x2) -->
    <BentoWidget colspan={2} rowspan={2} title="Activity Log" icon="📜">
      <div class="h-full flex flex-col">
        <!-- Log header -->
        <div class="flex items-center justify-between mb-3">
          <span class="text-xs text-zinc-500">{logLines.length} entries</span>
          <button
            onclick={clearLogs}
            class="px-2 py-1 text-xs text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50 rounded transition-colors"
          >
            Clear
          </button>
        </div>

        <!-- Log content -->
        <div class="flex-1 overflow-auto font-mono text-xs bg-black/30 rounded-lg border border-white/5 p-3">
          {#each logLines as line}
            <div class="py-0.5 
              {line.includes('[ERROR]') ? 'text-red-400' : 
               line.includes('[SUCCESS]') || line.includes('[COMPLETE]') ? 'text-emerald-400' : 
               line.includes('[PROGRESS]') || line.includes('[TEST]') ? 'text-amber-400' : 
               line.includes('[START]') ? 'text-cyan-400' : 
               'text-zinc-400'}">
              {line}
            </div>
          {/each}

          {#if logLines.length === 0}
            <div class="text-zinc-600 italic">Waiting for activity...</div>
          {/if}
        </div>
      </div>
    </BentoWidget>
  </BentoGrid>
</div>
