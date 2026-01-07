<script lang="ts">
  /**
   * FailoverStatusWidget Component
   * 
   * Компактный виджет для Dashboard, показывающий статус Auto Recovery:
   * - enabled/disabled
   * - current strategy
   * - failures count
   * - Кнопка быстрого переключения
   * - Индикатор состояния (green/yellow/red)
   */
  import { browser } from '$app/environment';
  import { listen } from '@tauri-apps/api/event';
  import type { FailoverStatus } from '$lib/api/failover';

  // Props
  interface Props {
    /** Compact mode for smaller spaces */
    compact?: boolean;
  }

  let { compact = false }: Props = $props();

  // State
  let status = $state<FailoverStatus | null>(null);
  let loading = $state(true);
  let switching = $state(false);
  let isTauri = $state(false);

  // Polling interval (5 seconds)
  const POLL_INTERVAL = 5000;
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let unlistenFailover: (() => void) | null = null;

  // Derived - compute values directly, not functions
  let failoverProgress = $derived(
    status && status.maxFailures > 0 
      ? Math.min(100, (status.failureCount / status.maxFailures) * 100) 
      : 0
  );

  let statusColor = $derived(
    !status?.enabled ? 'gray' :
    failoverProgress === 0 ? 'green' :
    failoverProgress < 66 ? 'yellow' : 'red'
  );

  let statusLabel = $derived(
    !status?.enabled ? 'Disabled' :
    failoverProgress === 0 ? 'Healthy' :
    failoverProgress < 66 ? 'Degraded' : 'Critical'
  );

  let cooldownFormatted = $derived(
    !status?.cooldownRemaining || status.cooldownRemaining <= 0 ? null :
    status.cooldownRemaining < 60 ? `${status.cooldownRemaining}s` :
    `${Math.floor(status.cooldownRemaining / 60)}m`
  );

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    if (isTauri) {
      loadStatus();
      startPolling();
      setupEventListener();
    } else {
      loading = false;
    }
    
    return () => {
      stopPolling();
      if (unlistenFailover) {
        unlistenFailover();
      }
    };
  });

  function startPolling() {
    pollTimer = setInterval(loadStatus, POLL_INTERVAL);
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  async function setupEventListener() {
    try {
      // Listen for failover events
      unlistenFailover = await listen<{ from: string; to: string }>('failover:switched', (event) => {
        console.log('Failover switched:', event.payload);
        loadStatus();
      });
    } catch (e) {
      console.error('Failed to setup failover event listener:', e);
    }
  }

  async function loadStatus() {
    if (!browser || !isTauri) return;
    
    try {
      const { getFailoverStatus } = await import('$lib/api/failover');
      status = await getFailoverStatus();
    } catch (e) {
      console.error('Failed to load failover status:', e);
    } finally {
      loading = false;
    }
  }

  async function handleQuickSwitch() {
    if (!browser || !isTauri || switching || !status?.enabled) return;
    
    switching = true;
    
    try {
      const { triggerManualFailover } = await import('$lib/api/failover');
      await triggerManualFailover();
      await loadStatus();
    } catch (e) {
      console.error('Failed to trigger failover:', e);
    } finally {
      switching = false;
    }
  }

  async function handleToggle() {
    if (!browser || !isTauri || switching) return;
    
    switching = true;
    
    try {
      const { setFailoverEnabled } = await import('$lib/api/failover');
      await setFailoverEnabled(!status?.enabled);
      await loadStatus();
    } catch (e) {
      console.error('Failed to toggle failover:', e);
    } finally {
      switching = false;
    }
  }
</script>

<div class="flex flex-col h-full {compact ? 'gap-2' : 'gap-3'}">
  {#if loading}
    <!-- Loading skeleton -->
    <div class="flex-1 flex items-center justify-center">
      <div class="animate-pulse flex flex-col items-center gap-2">
        <div class="w-8 h-8 rounded-full bg-zinc-700/50"></div>
        <div class="h-3 w-20 bg-zinc-700/50 rounded"></div>
      </div>
    </div>
  {:else}
    <!-- Status Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <!-- Status Indicator -->
        <div 
          class="w-2.5 h-2.5 rounded-full {statusColor === 'green' ? 'bg-emerald-500 animate-pulse' : statusColor === 'yellow' ? 'bg-amber-500' : statusColor === 'red' ? 'bg-red-500 animate-pulse' : 'bg-zinc-500'}"
          role="status"
          aria-label="Failover status: {statusLabel}"
        ></div>
        <span class="text-xs font-medium {statusColor === 'green' ? 'text-emerald-400' : statusColor === 'yellow' ? 'text-amber-400' : statusColor === 'red' ? 'text-red-400' : 'text-zinc-400'}">
          {statusLabel}
        </span>
      </div>
      
      <!-- Quick Toggle -->
      <button
        onclick={handleToggle}
        disabled={switching}
        class="p-1.5 rounded-lg transition-colors {status?.enabled ? 'bg-amber-500/20 text-amber-400 hover:bg-amber-500/30' : 'bg-zinc-700/50 text-zinc-400 hover:bg-zinc-700'}"
        aria-label="{status?.enabled ? 'Disable' : 'Enable'} auto recovery"
        title="{status?.enabled ? 'Disable' : 'Enable'} Auto Recovery"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          {#if status?.enabled}
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
          {:else}
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          {/if}
        </svg>
      </button>
    </div>

    {#if status?.enabled}
      <!-- Failure Progress -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between text-[10px]">
          <span class="text-zinc-400 uppercase tracking-wider">Failures</span>
          <span class="text-zinc-300 font-mono">{status?.failureCount ?? 0}/{status?.maxFailures ?? 0}</span>
        </div>
        <div 
          class="h-1.5 bg-zinc-800 rounded-full overflow-hidden"
          role="progressbar"
          aria-valuenow={failoverProgress}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label="Failure progress"
        >
          <div 
            class="h-full transition-all duration-500 {failoverProgress === 0 ? 'bg-emerald-500' : failoverProgress < 66 ? 'bg-amber-500' : 'bg-red-500'}"
            style="width: {failoverProgress}%"
          ></div>
        </div>
      </div>

      <!-- Current Strategy -->
      {#if !compact}
        <div class="flex-1 flex flex-col justify-center">
          <span class="text-[10px] text-zinc-500 uppercase tracking-wider mb-1">Current</span>
          <p class="text-xs text-zinc-300 font-mono truncate" title={status?.currentStrategy ?? 'None'}>
            {status?.currentStrategy ?? 'None'}
          </p>
          {#if status?.nextBackup}
            <span class="text-[10px] text-zinc-500 mt-2">
              Next: <span class="text-zinc-400 font-mono">{status.nextBackup}</span>
            </span>
          {/if}
        </div>
      {/if}

      <!-- Cooldown or Switch Button -->
      <div class="mt-auto">
        {#if cooldownFormatted}
          <div class="flex items-center justify-center gap-1.5 py-1.5 px-2 bg-zinc-800/50 rounded-lg">
            <svg class="w-3 h-3 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
            <span class="text-[10px] text-zinc-400">Cooldown: {cooldownFormatted}</span>
          </div>
        {:else}
          <button
            onclick={handleQuickSwitch}
            disabled={switching || !status?.currentStrategy}
            class="w-full py-1.5 px-2 bg-amber-500/10 hover:bg-amber-500/20 disabled:opacity-50 disabled:cursor-not-allowed text-amber-400 text-[10px] font-medium rounded-lg transition-colors flex items-center justify-center gap-1.5"
            aria-label="Switch to backup strategy now"
          >
            {#if switching}
              <svg class="w-3 h-3 animate-spin" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            {:else}
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
              </svg>
            {/if}
            Switch Now
          </button>
        {/if}
      </div>
    {:else}
      <!-- Disabled State -->
      <div class="flex-1 flex flex-col items-center justify-center text-center p-2">
        <svg class="w-6 h-6 text-zinc-600 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
        </svg>
        <span class="text-[10px] text-zinc-500">Auto Recovery disabled</span>
        <button
          onclick={handleToggle}
          disabled={switching}
          class="mt-2 text-[10px] text-amber-400 hover:text-amber-300 transition-colors"
          aria-label="Enable auto recovery"
        >
          Enable
        </button>
      </div>
    {/if}

    <!-- Last Error (compact indicator) -->
    {#if status?.lastFailureReason && !compact}
      <div class="px-2 py-1.5 bg-red-500/10 rounded-lg border border-red-500/20">
        <p class="text-[10px] text-red-400 line-clamp-1" title={status.lastFailureReason}>
          {status.lastFailureReason}
        </p>
      </div>
    {/if}
  {/if}
</div>
