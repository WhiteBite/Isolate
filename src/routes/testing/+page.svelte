<script lang="ts">
  import { browser } from '$app/environment';
  import { toasts } from '$lib/stores/toast';
  import { waitForBackend } from '$lib/utils/backend';
  import { 
    TestServiceList, 
    TestResultsPanel, 
    TestProgressBar, 
    TestControls,
    ABTestModal
  } from '$lib/components/testing';
  import type { ProxyConfig, Strategy, Service } from '$lib/api';
  import { mockTestingServices } from '$lib/mocks';

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
  let services = $state<Service[]>([]);

  function getDefaultServices(): Service[] {
    return [...mockTestingServices];
  }

  let selectedProxies = $state<Set<string>>(new Set());
  let selectedStrategies = $state<Set<string>>(new Set());
  let selectedServices = $state<Set<string>>(new Set(['discord', 'youtube']));

  let testMode = $state<'turbo' | 'deep'>('turbo');
  let isInteractive = $state(false);

  let isTesting = $state(false);
  let progress = $state<TestProgress | null>(null);
  let results = $state<TestResult[]>([]);

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
      { id: 'init', name: 'Initialization', description: 'Preparing test environment', status: 'pending' },
    ];
    
    if (selectedStrategies.size > 0) {
      stages.push({ id: 'strategies', name: 'Strategies', description: `Testing ${selectedStrategies.size} strategies`, status: 'pending' });
    }
    
    if (selectedProxies.size > 0) {
      stages.push({ id: 'proxies', name: 'Proxies', description: `Testing ${selectedProxies.size} proxies`, status: 'pending' });
    }
    
    stages.push({ id: 'scoring', name: 'Scoring', description: 'Calculating final scores', status: 'pending' });
    stages.push({ id: 'complete', name: 'Complete', description: 'Generating report', status: 'pending' });
    
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

  // Timer functions
  function startElapsedTimer() {
    testStartTime = Date.now();
    elapsedTime = 0;
    elapsedInterval = setInterval(() => {
      if (testStartTime) {
        elapsedTime = Math.floor((Date.now() - testStartTime) / 1000);
      }
    }, 1000);
  }

  function stopElapsedTimer() {
    if (elapsedInterval) {
      clearInterval(elapsedInterval);
      elapsedInterval = null;
    }
  }

  // Load data with backend ready check
  async function loadData() {
    if (!browser) return;
    
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    if (!isTauri) {
      services = getDefaultServices();
      return;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      const ready = await waitForBackend(30, 300);
      if (!ready) {
        console.warn('[Testing] Backend not ready after retries');
        return;
      }

      // Load proxies
      try {
        const loadedProxies = await invoke<any[]>('get_vless_configs');
        proxies = loadedProxies.map(p => ({
          id: p.id,
          name: p.name,
          protocol: 'vless' as const,
          server: p.server,
          port: p.port,
          tls: p.tls ?? true,
          custom_fields: p.custom_fields ?? {},
          active: p.active ?? false
        }));
      } catch (e) { console.warn('[Testing] Failed to load proxies:', e); }

      // Load strategies
      try {
        strategies = await invoke<Strategy[]>('get_strategies');
      } catch (e) { console.warn('[Testing] Failed to load strategies:', e); }

      // Load services
      try {
        const loadedServices = await invoke<any[]>('get_services');
        services = loadedServices?.length > 0 
          ? loadedServices.map(s => ({ id: s.id, name: s.name, critical: s.critical || false }))
          : getDefaultServices();
      } catch (e) {
        console.warn('[Testing] Failed to load services:', e);
        services = getDefaultServices();
      }

      // Event listeners
      unlistenProgress = await listen('test:progress', (event) => {
        const payload = event.payload as TestProgress;
        progress = payload;
        if (payload.current_type === 'strategy') {
          updateStage('init', 'done');
          updateStage('strategies', 'running');
        } else if (payload.current_type === 'proxy') {
          updateStage('strategies', 'done');
          updateStage('proxies', 'running');
        }
      });

      unlistenResult = await listen('test:result', (event) => {
        const result = event.payload as TestResult;
        results = [...results, result];
        lastResultId = result.id;
        setTimeout(() => { if (lastResultId === result.id) lastResultId = null; }, 1500);
      });

      unlistenComplete = await listen('test:complete', () => {
        testStages = testStages.map(stage => ({ ...stage, status: 'done' as StageStatus }));
        stopElapsedTimer();
        isTesting = false;
        progress = null;
        toasts.success('Testing completed');
      });
      
      unlistenStage = await listen('test:stage', (event) => {
        const { stage_id, status, error } = event.payload as { stage_id: string; status: StageStatus; error?: string };
        updateStage(stage_id, status, error);
      });
    } catch (e) {
      console.error('[Testing] Failed to load data:', e);
      toasts.error(`Data load error: ${e}`);
    }
  }

  // Initialize on mount
  import { onMount } from 'svelte';
  
  onMount(() => {
    loadData();
    return () => {
      unlistenProgress?.();
      unlistenResult?.();
      unlistenComplete?.();
      unlistenStage?.();
      stopElapsedTimer();
    };
  });

  // Testing functions
  async function startTesting() {
    if (!browser) return;
    if (selectedProxies.size === 0 && selectedStrategies.size === 0) {
      toasts.error('Select at least one proxy or strategy');
      return;
    }
    if (selectedServices.size === 0) {
      toasts.error('Select at least one service to check');
      return;
    }

    isTesting = true;
    results = [];
    progress = { current_item: 'Initializing...', current_type: '', tested_count: 0, total_count: 0, percent: 0 };
    
    initTestStages();
    startElapsedTimer();
    updateStage('init', 'running');

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      const ready = await invoke<boolean>('is_backend_ready');
      if (!ready) {
        toasts.error('Backend not ready, try again later');
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
      toasts.error(`Testing error: ${e}`);
      const currentStage = testStages[currentStageIndex];
      if (currentStage) updateStage(currentStage.id, 'failed', String(e));
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
      toasts.info('Testing cancelled');
    } catch (e) {
      console.error('Failed to cancel:', e);
      toasts.error(`Cancel error: ${e}`);
    }
    
    testStages = testStages.map(stage => 
      stage.status === 'pending' || stage.status === 'running' 
        ? { ...stage, status: 'failed' as StageStatus, error: 'Cancelled by user' }
        : stage
    );
    stopElapsedTimer();
    isTesting = false;
    progress = null;
  }

  async function applyBest() {
    if (!browser || results.length === 0) return;

    // Sort by score to get best
    const sorted = [...results].sort((a, b) => b.score - a.score);
    const best = sorted[0];
    if (!best) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (best.item_type === 'strategy') {
        await invoke('apply_strategy', { strategyId: best.id });
        toasts.success(`Strategy "${best.name}" applied`);
      } else {
        await invoke('apply_proxy', { id: best.id });
        toasts.success(`Proxy "${best.name}" started`);
      }
    } catch (e) {
      console.error('Failed to apply:', e);
      toasts.error(`Apply error: ${e}`);
    }
  }

  let canStartTest = $derived(
    (selectedProxies.size > 0 || selectedStrategies.size > 0) && selectedServices.size > 0
  );

  // A/B Testing modal state
  let showABTestModal = $state(false);

  // Prepare strategies for A/B test modal
  let abTestStrategies = $derived(
    strategies.map(s => ({
      id: s.id,
      name: s.name,
      family: s.family || 'unknown'
    }))
  );

  // Prepare services for A/B test modal
  let abTestServices = $derived(
    services.map(s => ({
      id: s.id,
      name: s.name
    }))
  );

</script>

<div class="p-8 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-white">Testing</h1>
      <p class="text-[#a0a0a0] mt-1">Check proxies and strategies for functionality</p>
    </div>
    <button
      onclick={() => showABTestModal = true}
      disabled={strategies.length < 2}
      class="flex items-center gap-2 px-4 py-2 bg-indigo-500/20 hover:bg-indigo-500/30 text-indigo-400 rounded-lg border border-indigo-500/30 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
      </svg>
      A/B Test
    </button>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Selection Panel -->
    <div class="lg:col-span-2">
      <TestServiceList
        {proxies}
        {strategies}
        {services}
        bind:selectedProxies
        bind:selectedStrategies
        bind:selectedServices
        disabled={isTesting}
      />
    </div>

    <!-- Control Panel -->
    <div class="space-y-6">
      <TestControls
        bind:testMode
        bind:isInteractive
        {isTesting}
        {canStartTest}
        onStart={startTesting}
        onCancel={cancelTesting}
      />

      <!-- Progress -->
      <TestProgressBar
        {progress}
        stages={testStages}
        {elapsedTime}
        {isTesting}
      />

      <!-- Quick Stats -->
      {#if results.length > 0 && !isTesting}
        <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
          <h3 class="text-white font-medium mb-3">Statistics</h3>
          <div class="grid grid-cols-2 gap-4 text-center">
            <div class="bg-[#0a0e27] rounded-lg p-3">
              <p class="text-[#00ff88] text-2xl font-bold">{results.filter(r => r.success_rate >= 80).length}</p>
              <p class="text-[#a0a0a0] text-xs">Passed</p>
            </div>
            <div class="bg-[#0a0e27] rounded-lg p-3">
              <p class="text-[#ff3333] text-2xl font-bold">{results.filter(r => r.success_rate < 50).length}</p>
              <p class="text-[#a0a0a0] text-xs">Failed</p>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Results Table -->
  <TestResultsPanel
    {results}
    {isTesting}
    {lastResultId}
    onApplyBest={applyBest}
  />

  <!-- A/B Test Modal -->
  <ABTestModal
    bind:open={showABTestModal}
    onclose={() => showABTestModal = false}
    strategies={abTestStrategies}
    services={abTestServices}
  />
</div>
