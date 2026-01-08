<script lang="ts">
  import { browser } from '$app/environment';
  import BaseModal from '../BaseModal.svelte';
  import Button from '../Button.svelte';
  import ProgressBar from '../ProgressBar.svelte';
  import {
    startABTest,
    cancelABTest,
    getABTestResults,
    onABTestStarted,
    onABTestCompleted,
    onABTestError,
    type ABTestResult,
    type ABTestStrategyResult
  } from '$lib/api';
  import { waitForBackend } from '$lib/utils/backend';

  interface Strategy {
    id: string;
    name: string;
    family: string;
  }

  interface Service {
    id: string;
    name: string;
  }

  interface Props {
    open: boolean;
    onclose: () => void;
    strategies: Strategy[];
    services: Service[];
  }

  let { open = $bindable(), onclose, strategies, services }: Props = $props();

  // Form state
  let strategyA = $state('');
  let strategyB = $state('');
  let serviceId = $state('');
  let iterations = $state(5);

  // Test state
  let testId = $state<string | null>(null);
  let isRunning = $state(false);
  let progress = $state(0);
  let currentStrategy = $state('');
  let result = $state<ABTestResult | null>(null);
  let error = $state<string | null>(null);

  // Unsubscribe functions
  let unsubscribers: (() => void)[] = [];

  // Derived
  let canStart = $derived(
    strategyA && strategyB && serviceId && 
    strategyA !== strategyB && 
    iterations >= 1 && iterations <= 20 &&
    !isRunning
  );

  let strategyAName = $derived(strategies.find(s => s.id === strategyA)?.name ?? '');
  let strategyBName = $derived(strategies.find(s => s.id === strategyB)?.name ?? '');

  // Setup event listeners
  $effect(() => {
    if (!browser || !open) return;

    const setupListeners = async () => {
      const unsub1 = await onABTestStarted((id) => {
        testId = id;
        isRunning = true;
        progress = 0;
      });

      const unsub2 = await onABTestCompleted((res) => {
        result = res;
        isRunning = false;
        progress = 100;
      });

      const unsub3 = await onABTestError((err) => {
        error = err;
        isRunning = false;
      });

      unsubscribers = [unsub1, unsub2, unsub3];
    };

    setupListeners();

    return () => {
      unsubscribers.forEach(unsub => unsub());
      unsubscribers = [];
    };
  });

  async function handleStart() {
    if (!canStart) return;
    
    error = null;
    result = null;
    
    try {
      const ready = await waitForBackend(20, 300);
      if (!ready) {
        error = 'Backend not ready';
        return;
      }

      testId = await startABTest(strategyA, strategyB, serviceId, iterations);
      isRunning = true;
    } catch (e) {
      error = `Failed to start test: ${e}`;
    }
  }

  async function handleCancel() {
    if (!testId) return;
    
    try {
      await cancelABTest(testId);
      isRunning = false;
      testId = null;
    } catch (e) {
      error = `Failed to cancel: ${e}`;
    }
  }

  function handleClose() {
    if (isRunning) return; // Don't close while running
    
    // Reset state
    strategyA = '';
    strategyB = '';
    serviceId = '';
    iterations = 5;
    testId = null;
    result = null;
    error = null;
    progress = 0;
    
    onclose();
  }

  function formatLatency(ms: number): string {
    return ms > 0 ? `${Math.round(ms)}ms` : '-';
  }

  function formatSuccessRate(rate: number): string {
    return `${rate.toFixed(1)}%`;
  }

  function getWinnerClass(strategyResult: ABTestStrategyResult, winnerId: string | null): string {
    if (!winnerId) return '';
    return strategyResult.strategy_id === winnerId 
      ? 'ring-2 ring-green-500/50 bg-green-500/10' 
      : 'opacity-75';
  }
</script>

<BaseModal {open} onclose={handleClose} class="w-[600px] max-h-[80vh] overflow-y-auto">
  <div class="p-6 space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold text-text-primary flex items-center gap-2">
        <svg class="w-6 h-6 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
        </svg>
        A/B Testing
      </h2>
      {#if !isRunning}
        <button onclick={handleClose} class="text-text-muted hover:text-text-primary">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      {/if}
    </div>

    {#if !result}
      <!-- Configuration Form -->
      <div class="space-y-4">
        <!-- Strategy A -->
        <div>
          <label class="block text-sm font-medium text-text-muted mb-2">Strategy A</label>
          <select 
            bind:value={strategyA}
            disabled={isRunning}
            class="w-full px-3 py-2 bg-void-50 border border-glass-border rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-indigo-500/50 disabled:opacity-50"
          >
            <option value="">Select strategy...</option>
            {#each strategies as strategy}
              <option value={strategy.id} disabled={strategy.id === strategyB}>
                {strategy.name} ({strategy.family})
              </option>
            {/each}
          </select>
        </div>

        <!-- Strategy B -->
        <div>
          <label class="block text-sm font-medium text-text-muted mb-2">Strategy B</label>
          <select 
            bind:value={strategyB}
            disabled={isRunning}
            class="w-full px-3 py-2 bg-void-50 border border-glass-border rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-indigo-500/50 disabled:opacity-50"
          >
            <option value="">Select strategy...</option>
            {#each strategies as strategy}
              <option value={strategy.id} disabled={strategy.id === strategyA}>
                {strategy.name} ({strategy.family})
              </option>
            {/each}
          </select>
        </div>

        <!-- Service -->
        <div>
          <label class="block text-sm font-medium text-text-muted mb-2">Test Service</label>
          <select 
            bind:value={serviceId}
            disabled={isRunning}
            class="w-full px-3 py-2 bg-void-50 border border-glass-border rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-indigo-500/50 disabled:opacity-50"
          >
            <option value="">Select service...</option>
            {#each services as service}
              <option value={service.id}>{service.name}</option>
            {/each}
          </select>
        </div>

        <!-- Iterations -->
        <div>
          <label class="block text-sm font-medium text-text-muted mb-2">
            Iterations per strategy: {iterations}
          </label>
          <input 
            type="range" 
            bind:value={iterations}
            min="1" 
            max="20" 
            disabled={isRunning}
            class="w-full accent-indigo-500"
          />
          <div class="flex justify-between text-xs text-text-muted mt-1">
            <span>1 (fast)</span>
            <span>20 (thorough)</span>
          </div>
        </div>
      </div>

      <!-- Progress -->
      {#if isRunning}
        <div class="space-y-3 p-4 bg-void-50 rounded-lg border border-glass-border">
          <div class="flex items-center justify-between text-sm">
            <span class="text-text-muted">Testing...</span>
            <span class="text-indigo-400 font-medium">{currentStrategy || 'Preparing'}</span>
          </div>
          <ProgressBar value={progress} showPercent />
          <p class="text-xs text-text-muted text-center">
            This may take a few minutes. Each strategy is tested {iterations} times.
          </p>
        </div>
      {/if}

      <!-- Error -->
      {#if error}
        <div class="p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
          <p class="text-red-400 text-sm">{error}</p>
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex justify-end gap-3">
        {#if isRunning}
          <Button variant="secondary" onclick={handleCancel}>
            Cancel Test
          </Button>
        {:else}
          <Button variant="secondary" onclick={handleClose}>
            Close
          </Button>
          <Button variant="primary" onclick={handleStart} disabled={!canStart}>
            Start A/B Test
          </Button>
        {/if}
      </div>

    {:else}
      <!-- Results -->
      <div class="space-y-6">
        <!-- Winner Banner -->
        {#if result.winner_id}
          <div class="p-4 bg-green-500/10 border border-green-500/30 rounded-lg text-center">
            <p class="text-green-400 font-medium">
              🏆 Winner: {result.winner_id === result.strategy_a_result.strategy_id 
                ? result.strategy_a_result.strategy_name 
                : result.strategy_b_result.strategy_name}
            </p>
            <p class="text-xs text-text-muted mt-1">
              {Math.abs(result.success_rate_diff).toFixed(1)}% better success rate
              {result.latency_diff_ms !== 0 ? `, ${Math.abs(result.latency_diff_ms).toFixed(0)}ms ${result.latency_diff_ms < 0 ? 'faster' : 'slower'}` : ''}
            </p>
          </div>
        {:else}
          <div class="p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg text-center">
            <p class="text-yellow-400 font-medium">🤝 It's a tie!</p>
            <p class="text-xs text-text-muted mt-1">Both strategies performed similarly</p>
          </div>
        {/if}

        <!-- Comparison Cards -->
        <div class="grid grid-cols-2 gap-4">
          <!-- Strategy A Result -->
          <div class="p-4 bg-void-50 rounded-lg border border-glass-border {getWinnerClass(result.strategy_a_result, result.winner_id)}">
            <h4 class="font-medium text-text-primary mb-3 flex items-center gap-2">
              <span class="w-6 h-6 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center text-xs font-bold">A</span>
              {result.strategy_a_result.strategy_name}
            </h4>
            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-text-muted">Success Rate</span>
                <span class="text-text-primary font-medium">{formatSuccessRate(result.strategy_a_result.success_rate)}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-text-muted">Avg Latency</span>
                <span class="text-text-primary">{formatLatency(result.strategy_a_result.avg_latency_ms)}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-text-muted">Min / Max</span>
                <span class="text-text-primary">
                  {formatLatency(result.strategy_a_result.min_latency_ms)} / {formatLatency(result.strategy_a_result.max_latency_ms)}
                </span>
              </div>
              <div class="flex justify-between">
                <span class="text-text-muted">Tests</span>
                <span class="text-text-primary">
                  {result.strategy_a_result.successful_tests}/{result.strategy_a_result.total_tests}
                </span>
              </div>
            </div>
          </div>

          <!-- Strategy B Result -->
          <div class="p-4 bg-void-50 rounded-lg border border-glass-border {getWinnerClass(result.strategy_b_result, result.winner_id)}">
            <h4 class="font-medium text-text-primary mb-3 flex items-center gap-2">
              <span class="w-6 h-6 rounded-full bg-purple-500/20 text-purple-400 flex items-center justify-center text-xs font-bold">B</span>
              {result.strategy_b_result.strategy_name}
            </h4>
            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-text-muted">Success Rate</span>
                <span class="text-text-primary font-medium">{formatSuccessRate(result.strategy_b_result.success_rate)}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-text-muted">Avg Latency</span>
                <span class="text-text-primary">{formatLatency(result.strategy_b_result.avg_latency_ms)}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-text-muted">Min / Max</span>
                <span class="text-text-primary">
                  {formatLatency(result.strategy_b_result.min_latency_ms)} / {formatLatency(result.strategy_b_result.max_latency_ms)}
                </span>
              </div>
              <div class="flex justify-between">
                <span class="text-text-muted">Tests</span>
                <span class="text-text-primary">
                  {result.strategy_b_result.successful_tests}/{result.strategy_b_result.total_tests}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Test Info -->
        <div class="text-xs text-text-muted text-center">
          Tested on: {result.service_name} • Completed: {new Date(result.completed_at).toLocaleString()}
        </div>

        <!-- Actions -->
        <div class="flex justify-end gap-3">
          <Button variant="secondary" onclick={() => { result = null; }}>
            New Test
          </Button>
          <Button variant="primary" onclick={handleClose}>
            Done
          </Button>
        </div>
      </div>
    {/if}
  </div>
</BaseModal>
