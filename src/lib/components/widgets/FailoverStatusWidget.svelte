<script lang="ts">
  /**
   * FailoverStatusWidget Component
   * 
   * Displays current failover status with controls for manual intervention.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { browser } from '$app/environment';
  import type { FailoverStatus, FailoverState } from '$lib/api/failover';

  // Props
  interface Props {
    /** Compact mode for smaller displays */
    compact?: boolean;
    /** Optional class for container */
    class?: string;
  }

  let { compact = false, class: className = '' }: Props = $props();

  // State
  let status = $state<FailoverStatus | null>(null);
  let loading = $state(true);
  let switching = $state(false);
  let restoring = $state(false);
  let error = $state<string | null>(null);
  let isTauri = $state(false);

  // Polling
  const POLL_INTERVAL = 3000;
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // Derived
  let stateColor = $derived(getStateColor(status?.state ?? 'normal'));
  let stateLabel = $derived(getStateLabel(status?.state ?? 'normal'));
  let countdownFormatted = $derived(formatCountdown(status?.restore_countdown_secs ?? null));
  let failureProgress = $derived(
    status ? (status.failure_count / status.max_failures) * 100 : 0
  );

  // Initialize
  import { onMount } from 'svelte';
  onMount(() => {
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      startPolling();
    } else {
      loading = false;
    }
    return () => stopPolling();
  });

  function startPolling() {
    fetchStatus();
    pollTimer = setInterval(fetchStatus, POLL_INTERVAL);
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  async function fetchStatus() {
    if (!browser || !isTauri) return;
    
    try {
      const { getFailoverStatus } = await import('$lib/api/failover');
      status = await getFailoverStatus();
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function handleForceSwitch() {
    if (!browser || !isTauri || switching) return;
    
    switching = true;
    try {
      const { forceSwitchToBackup } = await import('$lib/api/failover');
      const result = await forceSwitchToBackup();
      if (!result.success) {
        error = result.message;
      }
      await fetchStatus();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      switching = false;
    }
  }

  async function handleForceRestore() {
    if (!browser || !isTauri || restoring) return;
    
    restoring = true;
    try {
      const { forceRestorePrimary } = await import('$lib/api/failover');
      const result = await forceRestorePrimary();
      if (!result.success) {
        error = result.message;
      }
      await fetchStatus();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      restoring = false;
    }
  }

  function getStateColor(state: FailoverState): { bg: string; text: string; border: string; dot: string } {
    switch (state) {
      case 'normal':
        return { 
          bg: 'bg-emerald-500/10', 
          text: 'text-emerald-400', 
          border: 'border-emerald-500/20',
          dot: 'bg-emerald-500'
        };
      case 'degraded':
        return { 
          bg: 'bg-amber-500/10', 
          text: 'text-amber-400', 
          border: 'border-amber-500/20',
          dot: 'bg-amber-500'
        };
      case 'on_backup':
        return { 
          bg: 'bg-orange-500/10', 
          text: 'text-orange-400', 
          border: 'border-orange-500/20',
          dot: 'bg-orange-500'
        };
      default:
        return { 
          bg: 'bg-zinc-500/10', 
          text: 'text-zinc-400', 
          border: 'border-zinc-500/20',
          dot: 'bg-zinc-500'
        };
    }
  }

  function getStateLabel(state: FailoverState): string {
    switch (state) {
      case 'normal': return 'Normal';
      case 'degraded': return 'Degraded';
      case 'on_backup': return 'On Backup';
      default: return 'Unknown';
    }
  }

  function formatCountdown(secs: number | null): string {
    if (secs === null || secs <= 0) return '—';
    
    const mins = Math.floor(secs / 60);
    const remainingSecs = secs % 60;
    
    if (mins > 0) {
      return `${mins}m ${remainingSecs}s`;
    }
    return `${remainingSecs}s`;
  }

  function getStrategyTypeIcon(isPrimary: boolean): string {
    return isPrimary ? '⭐' : '🔄';
  }
</script>

<div class="flex flex-col h-full {compact ? 'gap-2' : 'gap-3'} {className}">
  {#if loading}
    <!-- Loading skeleton -->
    <div class="flex-1 flex items-center justify-center">
      <div class="animate-pulse flex flex-col items-center gap-2">
        <div class="w-8 h-8 rounded-full bg-zinc-700/50"></div>
        <div class="h-3 w-20 bg-zinc-700/50 rounded"></div>
      </div>
    </div>
  {:else if !status}
    <!-- No failover configured -->
    <div class="flex-1 flex flex-col items-center justify-center text-center p-4">
      <svg class="w-8 h-8 text-zinc-600 mb-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
      </svg>
      <span class="text-xs text-zinc-500">Auto-recovery disabled</span>
      <span class="text-[10px] text-zinc-600 mt-1">Enable in settings</span>
    </div>
  {:else}
    <!-- Status Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="w-2 h-2 rounded-full {stateColor.dot} {status.state !== 'normal' ? 'animate-pulse' : ''}"></span>
        <span class="text-sm font-medium {stateColor.text}">{stateLabel}</span>
      </div>
      <div class="px-2 py-0.5 rounded text-[10px] {stateColor.bg} {stateColor.text} {stateColor.border} border">
        {status.is_primary ? 'Primary' : 'Backup'}
      </div>
    </div>

    <!-- Active Strategy -->
    <div class="p-3 rounded-lg bg-zinc-900/30 border border-white/5">
      <div class="flex items-center gap-2 mb-1">
        <span class="text-lg">{getStrategyTypeIcon(status.is_primary)}</span>
        <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Active Strategy</span>
      </div>
      <div class="text-sm font-medium text-white truncate" title={status.active_strategy_id ?? 'None'}>
        {status.active_strategy_id ?? 'None'}
      </div>
    </div>

    <!-- Failure Counter -->
    <div class="p-3 rounded-lg bg-zinc-900/30 border border-white/5">
      <div class="flex items-center justify-between mb-2">
        <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Failures</span>
        <span class="text-sm font-mono {status.failure_count > 0 ? 'text-amber-400' : 'text-zinc-400'}">
          {status.failure_count} / {status.max_failures}
        </span>
      </div>
      <!-- Progress bar -->
      <div class="h-1.5 bg-zinc-800 rounded-full overflow-hidden">
        <div 
          class="h-full transition-all duration-300 rounded-full {failureProgress >= 100 ? 'bg-red-500' : failureProgress >= 66 ? 'bg-amber-500' : 'bg-emerald-500'}"
          style="width: {failureProgress}%"
        ></div>
      </div>
    </div>

    <!-- Restore Countdown (only when on backup) -->
    {#if status.state === 'on_backup' && status.restore_countdown_secs !== null}
      <div class="p-3 rounded-lg {stateColor.bg} border {stateColor.border}">
        <div class="flex items-center gap-2 mb-1">
          <svg class="w-3.5 h-3.5 {stateColor.text}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <polyline points="12 6 12 12 16 14"/>
          </svg>
          <span class="text-[10px] uppercase tracking-wider {stateColor.text} font-medium">Restore In</span>
        </div>
        <div class="text-lg font-bold font-mono {stateColor.text}">{countdownFormatted}</div>
      </div>
    {/if}

    <!-- Action Buttons -->
    {#if !compact}
      <div class="flex gap-2 mt-auto pt-2">
        <button
          onclick={handleForceSwitch}
          disabled={switching || status.state === 'on_backup'}
          class="flex-1 px-3 py-2 text-xs font-medium rounded-lg transition-colors flex items-center justify-center gap-1.5
            {status.state === 'on_backup' 
              ? 'bg-zinc-800/50 text-zinc-600 cursor-not-allowed' 
              : 'bg-amber-500/10 text-amber-400 hover:bg-amber-500/20 border border-amber-500/20'}"
          title="Force switch to backup strategy"
        >
          {#if switching}
            <svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
            </svg>
          {:else}
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7"/>
            </svg>
          {/if}
          Switch
        </button>
        
        <button
          onclick={handleForceRestore}
          disabled={restoring || status.is_primary}
          class="flex-1 px-3 py-2 text-xs font-medium rounded-lg transition-colors flex items-center justify-center gap-1.5
            {status.is_primary 
              ? 'bg-zinc-800/50 text-zinc-600 cursor-not-allowed' 
              : 'bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 border border-emerald-500/20'}"
          title="Force restore to primary strategy"
        >
          {#if restoring}
            <svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
            </svg>
          {:else}
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
            </svg>
          {/if}
          Restore
        </button>
      </div>
    {/if}

    <!-- Error display -->
    {#if error && !compact}
      <div class="px-2 py-1.5 rounded bg-red-500/10 border border-red-500/20">
        <span class="text-[10px] text-red-400 line-clamp-2">{error}</span>
      </div>
    {/if}

    <!-- Last timestamps (compact footer) -->
    {#if !compact}
      <div class="flex items-center justify-between px-1 pt-1 border-t border-white/5 text-[10px] text-zinc-600">
        {#if status.last_failure_at}
          <span title="Last failure">
            ❌ {new Date(status.last_failure_at).toLocaleTimeString()}
          </span>
        {:else}
          <span>No failures</span>
        {/if}
        {#if status.last_success_at}
          <span title="Last success">
            ✓ {new Date(status.last_success_at).toLocaleTimeString()}
          </span>
        {/if}
      </div>
    {/if}
  {/if}
</div>
