<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  // Types matching Rust StrategyMetrics
  interface StrategyMetrics {
    strategy_id: string;
    started_at: string;
    uptime_secs: number;
    bytes_sent: number;
    bytes_received: number;
    connection_count: number;
    error_count: number;
    last_error: string | null;
    last_updated: string;
  }

  interface Props {
    compact?: boolean;
  }

  let { compact = false }: Props = $props();

  // State
  let metrics = $state<StrategyMetrics | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Polling interval (5 seconds)
  const POLL_INTERVAL = 5000;
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // Format uptime to human readable
  function formatUptime(seconds: number): string {
    if (seconds < 60) {
      return `${seconds}s`;
    }
    if (seconds < 3600) {
      const mins = Math.floor(seconds / 60);
      const secs = seconds % 60;
      return `${mins}m ${secs}s`;
    }
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
  }

  // Format bytes to human readable
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let unitIndex = 0;
    let value = bytes;
    
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }
    
    return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[unitIndex]}`;
  }

  // Fetch metrics from backend
  async function fetchMetrics() {
    try {
      const result = await invoke<StrategyMetrics | null>('get_strategy_metrics');
      metrics = result;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  // Start polling
  function startPolling() {
    fetchMetrics();
    pollTimer = setInterval(fetchMetrics, POLL_INTERVAL);
  }

  // Stop polling
  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  // Lifecycle
  import { onMount } from 'svelte';
  
  onMount(() => {
    startPolling();
    return () => stopPolling();
  });

  // Derived values
  let hasMetrics = $derived(metrics !== null);
  let uptimeFormatted = $derived(metrics ? formatUptime(metrics.uptime_secs) : '—');
  let bytesSentFormatted = $derived(metrics ? formatBytes(metrics.bytes_sent) : '0 B');
  let bytesReceivedFormatted = $derived(metrics ? formatBytes(metrics.bytes_received) : '0 B');
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
  {:else if !hasMetrics}
    <!-- No active strategy -->
    <div class="flex-1 flex flex-col items-center justify-center text-center p-4">
      <svg class="w-8 h-8 text-zinc-400 mb-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
      </svg>
      <span class="text-xs text-zinc-400">No active strategy</span>
      <span class="text-[10px] text-zinc-400 mt-1">Start a strategy to see metrics</span>
    </div>
  {:else}
    <!-- Metrics display -->
    <div class="grid grid-cols-2 gap-2 flex-1">
      <!-- Uptime -->
      <div class="p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
        <div class="flex items-center gap-2 mb-1">
          <svg class="w-3.5 h-3.5 text-neon-green" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <polyline points="12 6 12 12 16 14"/>
          </svg>
          <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Uptime</span>
        </div>
        <div class="text-lg font-bold text-white font-mono">{uptimeFormatted}</div>
      </div>

      <!-- Connections -->
      <div class="p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
        <div class="flex items-center gap-2 mb-1">
          <svg class="w-3.5 h-3.5 text-neon-cyan" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
            <circle cx="9" cy="7" r="4"/>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
          </svg>
          <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Connections</span>
        </div>
        <div class="text-lg font-bold text-white font-mono">{metrics?.connection_count ?? 0}</div>
      </div>

      <!-- Bytes Sent -->
      <div class="p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
        <div class="flex items-center gap-2 mb-1">
          <svg class="w-3.5 h-3.5 text-indigo-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 21V3M5 10l7-7 7 7"/>
          </svg>
          <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Sent</span>
        </div>
        <div class="text-lg font-bold text-white font-mono">{bytesSentFormatted}</div>
      </div>

      <!-- Bytes Received -->
      <div class="p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
        <div class="flex items-center gap-2 mb-1">
          <svg class="w-3.5 h-3.5 text-neon-cyan" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 3v18M5 12l7 7 7-7"/>
          </svg>
          <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Received</span>
        </div>
        <div class="text-lg font-bold text-white font-mono">{bytesReceivedFormatted}</div>
      </div>
    </div>

    <!-- Footer with errors and strategy info -->
    {#if !compact}
      <div class="flex items-center justify-between px-1 pt-1 border-t border-white/5">
        <div class="flex items-center gap-2">
          {#if metrics && metrics.error_count > 0}
            <div class="flex items-center gap-1">
              <svg class="w-3 h-3 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              <span class="text-[10px] text-red-400">{metrics.error_count} errors</span>
            </div>
          {:else}
            <div class="flex items-center gap-1">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
              <span class="text-[10px] text-zinc-400">Healthy</span>
            </div>
          {/if}
        </div>
        <div class="text-[10px] text-zinc-400 truncate max-w-[120px]" title={metrics?.strategy_id}>
          {metrics?.strategy_id ?? '—'}
        </div>
      </div>
    {/if}

    <!-- Last error tooltip -->
    {#if metrics?.last_error && !compact}
      <div class="px-2 py-1.5 rounded bg-red-500/10 border border-red-500/20">
        <span class="text-[10px] text-red-400 line-clamp-2">{metrics.last_error}</span>
      </div>
    {/if}
  {/if}
</div>
