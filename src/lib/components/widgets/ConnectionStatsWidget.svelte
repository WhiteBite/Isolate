<script lang="ts">
  import { connectionStats, type ConnectionHistoryPoint } from '$lib/stores/connectionStats.svelte';

  interface Props {
    compact?: boolean;
  }

  let { compact = false }: Props = $props();

  // Get reactive stats from store
  let stats = $derived(connectionStats.stats);
  let history = $derived(connectionStats.history);

  // Initialize store on mount
  import { onMount } from 'svelte';
  onMount(() => {
    connectionStats.init();
    
    return () => {
      connectionStats.cleanup();
    };
  });

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

  // Format duration to human readable
  function formatDuration(ms: number): string {
    if (ms < 1000) return `${Math.round(ms)}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${Math.round(ms / 60000)}m`;
  }

  // Generate sparkline path for connections over time
  function generateSparkline(data: ConnectionHistoryPoint[]): string {
    if (data.length < 2) return '';
    
    const values = data.map(p => p.connections);
    const max = Math.max(...values, 1);
    const width = 100;
    const height = 32;
    
    const points = values.map((value, i) => {
      const x = (i / (values.length - 1)) * width;
      const y = height - (value / max) * (height - 4) - 2; // Leave padding
      return `${x},${y}`;
    });
    
    return `M ${points.join(' L ')}`;
  }

  let sparklinePath = $derived(generateSparkline(history));

  // Time since last update
  let timeSinceUpdate = $derived(() => {
    if (!stats.lastUpdate) return 'Never';
    const diff = Date.now() - stats.lastUpdate;
    if (diff < 1000) return 'Just now';
    if (diff < 60000) return `${Math.round(diff / 1000)}s ago`;
    return `${Math.round(diff / 60000)}m ago`;
  });
</script>

<div class="flex flex-col h-full {compact ? 'gap-2' : 'gap-3'}">
  {#if !stats.isReady}
    <!-- Waiting for data state -->
    <div class="flex-1 flex flex-col items-center justify-center p-4 rounded-lg bg-zinc-900/30 border border-white/5">
      <div class="flex items-center gap-2 mb-2">
        <div class="w-2 h-2 rounded-full bg-zinc-500"></div>
        <span class="text-sm text-zinc-400">Ожидание данных</span>
      </div>
      <p class="text-[10px] text-zinc-400 text-center">
        Статистика появится после включения защиты
      </p>
    </div>
  {:else}
    <!-- Active Connections -->
    <div class="flex-1 flex flex-col justify-center p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <svg class="w-4 h-4 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/>
          </svg>
          <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Active Connections</span>
        </div>
        <div class="flex items-baseline gap-1">
          <span class="text-2xl font-bold text-white font-mono">{stats.activeConnections}</span>
          <span class="text-[10px] text-zinc-400">/ {stats.peakConnections} peak</span>
        </div>
      </div>
      
      <!-- Connections over time chart -->
      {#if history.length > 1}
        <div class="h-8 w-full overflow-hidden">
          <svg class="w-full h-full" viewBox="0 0 100 32" preserveAspectRatio="none">
            <defs>
              <linearGradient id="connGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stop-color="rgb(52, 211, 153)" stop-opacity="0.3"/>
                <stop offset="100%" stop-color="rgb(52, 211, 153)" stop-opacity="0"/>
              </linearGradient>
            </defs>
            <path 
              d="{sparklinePath} L 100,32 L 0,32 Z" 
              fill="url(#connGradient)"
            />
            <path 
              d={sparklinePath} 
              fill="none" 
              stroke="rgb(52, 211, 153)" 
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </div>
        <div class="flex justify-between text-[10px] text-zinc-400 mt-1">
          <span>5 min ago</span>
          <span>now</span>
        </div>
      {/if}
    </div>

    <!-- Data Transfer Stats -->
    <div class="grid grid-cols-2 gap-2">
      <!-- Bytes In -->
      <div class="p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
        <div class="flex items-center gap-1.5 mb-1">
          <svg class="w-3 h-3 text-neon-cyan" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 3v18M5 12l7 7 7-7"/>
          </svg>
          <span class="text-[10px] uppercase tracking-wider text-zinc-400">Received</span>
        </div>
        <span class="text-sm font-bold text-white font-mono">{formatBytes(stats.totalBytesIn)}</span>
      </div>
      
      <!-- Bytes Out -->
      <div class="p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
        <div class="flex items-center gap-1.5 mb-1">
          <svg class="w-3 h-3 text-indigo-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 21V3M5 10l7-7 7 7"/>
          </svg>
          <span class="text-[10px] uppercase tracking-wider text-zinc-400">Sent</span>
        </div>
        <span class="text-sm font-bold text-white font-mono">{formatBytes(stats.totalBytesOut)}</span>
      </div>
    </div>

    <!-- Additional Stats -->
    {#if !compact}
      <div class="flex items-center justify-between px-1 pt-1 text-[10px]">
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1">
            <svg class="w-3 h-3 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 6v6l4 2"/>
            </svg>
            <span class="text-zinc-400">Avg: {formatDuration(stats.avgDuration)}</span>
          </div>
          <div class="flex items-center gap-1">
            <svg class="w-3 h-3 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
            </svg>
            <span class="text-zinc-400">{stats.connectionsPerMinute.toFixed(1)}/min</span>
          </div>
        </div>
        <div class="flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
          <span class="text-zinc-400">Updated {timeSinceUpdate()}</span>
        </div>
      </div>
    {/if}
  {/if}
</div>
