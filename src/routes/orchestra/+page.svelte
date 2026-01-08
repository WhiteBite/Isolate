<script lang="ts">
  import { browser } from '$app/environment';
  import { BentoGrid } from '$lib/components';
  import { toasts } from '$lib/stores/toast';
  import { logs } from '$lib/stores/logs';
  import { waitForBackend } from '$lib/utils/backend';
  
  // Orchestra components
  import {
    OrchestraStatus,
    OptimizationProgress,
    OrchestraControls,
    ServiceGrid,
    OrchestraStats,
    StrategyQueue,
    ActivityLog,
    type Strategy,
    type QueueItem,
    type OrchestraState,
    type ServiceInfo,
    type OptimizationMode,
    getServiceIcon
  } from '$lib/components/orchestra';
  import { mockOrchestraServices } from '$lib/mocks';

  // State
  let orchestraState = $state<OrchestraState>({
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
  
  // Demo mode flag (browser preview without Tauri)
  let isDemoMode = $state(false);

  let queue = $state<QueueItem[]>([]);
  let strategies = $state<Strategy[]>([]);
  let logLines = $state<string[]>([]);
  let isTauri = $state(false);
  let initialized = $state(false);

  // Services for testing
  let availableServices = $state<ServiceInfo[]>([]);
  let selectedServices = $state<Set<string>>(new Set(['youtube', 'discord']));

  // Settings
  let mode = $state<OptimizationMode>('turbo');
  let autoApply = $state(true);

  // Cleanup
  let cleanupFns: (() => void)[] = [];
  let timerInterval: ReturnType<typeof setInterval> | null = null;

  function getDefaultServices(): ServiceInfo[] {
    return [...mockOrchestraServices];
  }

  async function loadServices() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      const ready = await invoke<boolean>('is_backend_ready').catch(() => false);
      if (!ready) {
        availableServices = getDefaultServices();
        return;
      }
      
      const services = await invoke<any[]>('get_services');
      availableServices = services.map(s => ({
        id: s.id,
        name: s.name,
        icon: getServiceIcon(s.id)
      }));
      
      const validIds = new Set(services.map(s => s.id));
      selectedServices = new Set([...selectedServices].filter(id => validIds.has(id)));
      
      if (selectedServices.size === 0 && availableServices.length > 0) {
        const defaultSelected = availableServices.slice(0, 2).map(s => s.id);
        selectedServices = new Set(defaultSelected);
      }
    } catch (e) {
      console.warn('Failed to load services:', e);
      availableServices = getDefaultServices();
    }
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

  async function initialize() {
    if (!browser || initialized) return;
    initialized = true;

    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    isDemoMode = !isTauri;

    if (!isTauri) {
      addLog('[INFO] Running in Demo mode (browser preview)');
      return;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      const backendReady = await waitForBackend();
      if (!backendReady) {
        addLog('[ERROR] Backend not ready');
        return;
      }

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
      }

      const unlistenProgress = await listen('automation:progress', (event) => {
        const payload = event.payload as { stage: string; percent: number; message: string; current_strategy?: string };
        orchestraState.progress = payload.percent;
        orchestraState.currentItem = payload.current_strategy || payload.stage;
        orchestraState.status = 'learning';
        addLog(`[PROGRESS] ${payload.message}`);

        if (payload.current_strategy) {
          updateQueueItem(payload.current_strategy, 'testing');
        }
      });

      const unlistenComplete = await listen('automation:complete', (event) => {
        const result = event.payload as { strategy_id: string; strategy_name: string; score: number };
        orchestraState.status = 'completed';
        orchestraState.bestStrategy = result.strategy_name;
        orchestraState.bestScore = result.score;
        orchestraState.progress = 100;
        addLog(`[SUCCESS] Best strategy: ${result.strategy_name} (score: ${result.score.toFixed(1)})`);
        
        queue = queue.map(item => {
          if (item.id === result.strategy_id) {
            return { ...item, status: 'success' as const, score: result.score };
          } else if (item.status === 'pending' || item.status === 'testing') {
            return { ...item, status: 'skipped' as const };
          }
          return item;
        });
        orchestraState.testedItems = queue.length;
        
        stopTimer();
        toasts.success(`Optimization complete: ${result.strategy_name}`);
      });

      const unlistenFailed = await listen('automation:error', (event) => {
        orchestraState.status = 'error';
        addLog(`[ERROR] Optimization failed: ${event.payload}`);
        stopTimer();
        toasts.error(`Optimization failed: ${event.payload}`);
      });

      const unlistenResult = await listen('strategy:test_result', (event) => {
        const result = event.payload as { strategy_id: string; success: boolean; score: number; latency_ms: number };
        const status = result.success ? 'success' : 'failed';
        updateQueueItem(result.strategy_id, status, result.score, result.latency_ms);
        orchestraState.testedItems++;
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
    }
  }

  import { onMount } from 'svelte';
  
  onMount(() => {
    initialize();
    loadServices();
    return () => {
      cleanupFns.forEach(fn => fn());
      cleanupFns = [];
      stopTimer();
    };
  });

  function startTimer() {
    orchestraState.startTime = Date.now();
    orchestraState.elapsedTime = 0;
    timerInterval = setInterval(() => {
      if (orchestraState.startTime) {
        orchestraState.elapsedTime = Date.now() - orchestraState.startTime;
      }
    }, 1000);
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  function updateQueueItem(id: string, status: QueueItem['status'], score?: number, latency?: number) {
    queue = queue.map(item => 
      item.id === id ? { ...item, status, score, latency } : item
    );
  }

  function resetQueue() {
    queue = queue.map(item => ({ ...item, status: 'pending' as const, score: undefined, latency: undefined }));
  }

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    logLines = [...logLines.slice(-99), `[${timestamp}] ${message}`];
    logs.debug('orchestra', message);
  }

  function clearLogs() {
    logLines = [];
  }

  async function startOptimization() {
    const canStart = (orchestraState.status === 'idle' || orchestraState.status === 'completed' || orchestraState.status === 'error') && selectedServices.size > 0;
    if (!canStart) return;

    resetQueue();
    orchestraState = {
      ...orchestraState,
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
      simulateOptimization();
      return;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('run_optimization_v2');
    } catch (e) {
      orchestraState.status = 'error';
      addLog(`[ERROR] Failed to start: ${e}`);
      stopTimer();
      toasts.error(`Failed to start optimization: ${e}`);
    }
  }

  async function pauseOptimization() {
    const canPause = orchestraState.status === 'running' || orchestraState.status === 'learning';
    if (!canPause) return;
    orchestraState.status = 'paused';
    addLog('[PAUSE] Optimization paused');
  }

  async function stopOptimization() {
    const canStop = orchestraState.status !== 'idle';
    if (!canStop) return;
    orchestraState.status = 'idle';
    orchestraState.progress = 0;
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

  // Demo mode simulation - generates random test results for browser preview
  async function simulateOptimization() {
    addLog('[DEMO] Running simulated optimization (no real network tests)');
    
    for (let i = 0; i < queue.length; i++) {
      if (orchestraState.status !== 'learning') break;

      const item = queue[i];
      orchestraState.currentItem = item.name;
      updateQueueItem(item.id, 'testing');
      addLog(`[DEMO] Testing ${item.name}...`);

      // Simulated delay (800-1200ms)
      await new Promise(r => setTimeout(r, 800 + Math.random() * 400));

      // Simulated random results
      const success = Math.random() > 0.3;
      const score = success ? 50 + Math.random() * 50 : Math.random() * 30;
      const latency = 50 + Math.random() * 200;

      updateQueueItem(item.id, success ? 'success' : 'failed', score, latency);
      orchestraState.testedItems = i + 1;
      orchestraState.progress = ((i + 1) / queue.length) * 100;

      if (score > orchestraState.bestScore) {
        orchestraState.bestScore = score;
        orchestraState.bestStrategy = item.name;
      }

      addLog(`[DEMO] ${item.name}: ${success ? '✓' : '✗'} score=${score.toFixed(1)} latency=${latency.toFixed(0)}ms`);
    }

    if (orchestraState.status === 'learning') {
      orchestraState.status = 'completed';
      orchestraState.progress = 100;
      stopTimer();
      addLog(`[DEMO] Complete! Best: ${orchestraState.bestStrategy} (score: ${orchestraState.bestScore.toFixed(1)})`);
      toasts.success(`Demo complete: ${orchestraState.bestStrategy}`);
    }
  }
</script>

<div class="h-full p-8 overflow-auto bg-gradient-to-br from-zinc-950 to-black">
  <!-- Page Header -->
  <div class="mb-8">
    <div class="flex items-center gap-3">
      <h1 class="text-3xl font-bold text-white tracking-tight">Orchestra</h1>
      {#if isDemoMode}
        <span class="px-2 py-1 text-xs uppercase tracking-wider bg-amber-500/20 text-amber-400 rounded-md font-medium border border-amber-500/30">Demo</span>
      {/if}
    </div>
    <p class="text-sm text-zinc-400 mt-2">Automatic strategy optimization and learning center</p>
  </div>

  <BentoGrid columns={4} gap={4}>
    <!-- Status Widget (2x1) -->
    <OrchestraStatus state={orchestraState} />

    <!-- Progress Widget (2x1) -->
    <OptimizationProgress state={orchestraState} queueLength={queue.length} />

    <!-- Controls Widget (1x1) -->
    <OrchestraControls 
      state={orchestraState}
      {mode}
      {autoApply}
      selectedServicesCount={selectedServices.size}
      onModeChange={(m) => mode = m}
      onAutoApplyChange={(v) => autoApply = v}
      onStart={startOptimization}
      onPause={pauseOptimization}
      onStop={stopOptimization}
    />

    <!-- Services Widget (1x1) -->
    <ServiceGrid 
      services={availableServices}
      {selectedServices}
      isLearning={orchestraState.status === 'learning'}
      onToggle={toggleService}
    />

    <!-- Stats Widget (1x1) -->
    <OrchestraStats {queue} />

    <!-- Queue Widget (2x2) -->
    <StrategyQueue {queue} />

    <!-- Log Widget (2x2) -->
    <ActivityLog {logLines} onClear={clearLogs} />
  </BentoGrid>
</div>
