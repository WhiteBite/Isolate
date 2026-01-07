<script lang="ts">
  import { browser } from '$app/environment';

  // Types
  interface HealthRecord {
    id: number;
    serviceId: string;
    timestamp: number;
    accessible: boolean;
    latencyMs: number | null;
    error: string | null;
  }

  interface HealthStats {
    serviceId: string;
    totalChecks: number;
    successfulChecks: number;
    failedChecks: number;
    uptimePercent: number;
    avgLatencyMs: number | null;
    minLatencyMs: number | null;
    maxLatencyMs: number | null;
    lastCheckAt: number | null;
    lastSuccessAt: number | null;
    lastFailureAt: number | null;
  }

  interface Props {
    serviceId: string;
    hours?: number;
    compact?: boolean;
    showStats?: boolean;
    onexpand?: () => void;
  }

  let { 
    serviceId, 
    hours = 24, 
    compact = false,
    showStats = true,
    onexpand
  }: Props = $props();

  // State
  let history = $state<HealthRecord[]>([]);
  let stats = $state<HealthStats | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Load data when serviceId changes
  $effect(() => {
    if (browser && serviceId) {
      loadHealthData();
    }
  });

  async function loadHealthData() {
    loading = true;
    error = null;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      const [historyResult, statsResult] = await Promise.all([
        invoke<HealthRecord[]>('get_service_health_history', { serviceId, hours }),
        invoke<HealthStats>('get_service_health_stats', { serviceId, hours })
      ]);

      history = historyResult;
      stats = statsResult;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      console.warn('Failed to load health data:', e);
    }

    loading = false;
  }

  // Derived values
  let chartWidth = $derived(compact ? 120 : 200);
  let chartHeight = $derived(compact ? 24 : 40);
  let barCount = $derived(compact ? 24 : 48);

  // Aggregate history into time buckets for visualization
  let buckets = $derived.by(() => {
    if (history.length === 0) return [];

    const now = Date.now() / 1000;
    const bucketDuration = (hours * 3600) / barCount;
    const result: { accessible: boolean; count: number; avgLatency: number | null }[] = [];

    for (let i = 0; i < barCount; i++) {
      const bucketStart = now - (barCount - i) * bucketDuration;
      const bucketEnd = bucketStart + bucketDuration;

      const bucketRecords = history.filter(
        r => r.timestamp >= bucketStart && r.timestamp < bucketEnd
      );

      if (bucketRecords.length === 0) {
        result.push({ accessible: true, count: 0, avgLatency: null });
      } else {
        const successCount = bucketRecords.filter(r => r.accessible).length;
        const accessible = successCount >= bucketRecords.length / 2;
        const latencies = bucketRecords
          .filter(r => r.latencyMs !== null)
          .map(r => r.latencyMs!);
        const avgLatency = latencies.length > 0
          ? latencies.reduce((a, b) => a + b, 0) / latencies.length
          : null;

        result.push({ accessible, count: bucketRecords.length, avgLatency });
      }
    }

    return result;
  });

  // Helper functions
  function getUptimeColor(uptime: number): string {
    if (uptime >= 99) return 'text-emerald-400';
    if (uptime >= 95) return 'text-lime-400';
    if (uptime >= 90) return 'text-amber-400';
    if (uptime >= 80) return 'text-orange-400';
    return 'text-red-400';
  }

  function getLatencyColor(latency: number | null): string {
    if (latency === null) return 'text-zinc-500';
    if (latency < 50) return 'text-emerald-400';
    if (latency < 150) return 'text-amber-400';
    return 'text-red-400';
  }

  function formatTime(timestamp: number | null): string {
    if (!timestamp) return '—';
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString('en-US', { 
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false 
    });
  }

  function formatDuration(timestamp: number | null): string {
    if (!timestamp) return '—';
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }
</script>

{#if loading}
  <div class="flex items-center justify-center {compact ? 'h-6' : 'h-16'}">
    <div class="w-4 h-4 border-2 border-zinc-600 border-t-indigo-500 rounded-full animate-spin"></div>
  </div>
{:else if error}
  <div class="text-xs text-red-400 {compact ? '' : 'p-2'}">
    {compact ? '!' : `Error: ${error}`}
  </div>
{:else if compact}
  <!-- Compact Mode: Mini chart for list view -->
  <button
    onclick={() => onexpand?.()}
    class="flex items-center gap-2 group cursor-pointer"
    title="Click to view details"
  >
    <!-- Mini Status Bar Chart -->
    <svg width={chartWidth} height={chartHeight} class="flex-shrink-0">
      {#each buckets as bucket, i}
        {@const barWidth = chartWidth / barCount - 1}
        {@const x = i * (chartWidth / barCount)}
        <rect
          {x}
          y="0"
          width={barWidth}
          height={chartHeight}
          rx="1"
          class="transition-colors duration-200"
          fill={bucket.count === 0 
            ? '#3f3f46' 
            : bucket.accessible 
              ? '#22c55e' 
              : '#ef4444'}
          opacity={bucket.count === 0 ? 0.3 : 1}
        />
      {/each}
    </svg>

    <!-- Uptime Badge -->
    {#if stats}
      <span class="text-xs font-mono {getUptimeColor(stats.uptimePercent)} opacity-80 group-hover:opacity-100 transition-opacity">
        {stats.uptimePercent.toFixed(0)}%
      </span>
    {/if}
  </button>
{:else}
  <!-- Full Mode: Detailed chart with stats -->
  <div class="space-y-3">
    <!-- Chart Header -->
    <div class="flex items-center justify-between">
      <h4 class="text-sm font-medium text-zinc-100">Health History</h4>
      <span class="text-xs text-zinc-400">Last {hours}h</span>
    </div>

    <!-- Status Bar Chart -->
    <div class="relative">
      <svg width="100%" height={chartHeight} viewBox="0 0 {chartWidth} {chartHeight}" preserveAspectRatio="none" class="rounded-lg overflow-hidden">
        {#each buckets as bucket, i}
          {@const barWidth = chartWidth / barCount - 0.5}
          {@const x = i * (chartWidth / barCount)}
          <rect
            {x}
            y="0"
            width={barWidth}
            height={chartHeight}
            class="transition-colors duration-200"
            fill={bucket.count === 0 
              ? '#27272a' 
              : bucket.accessible 
                ? '#22c55e' 
                : '#ef4444'}
            opacity={bucket.count === 0 ? 0.5 : 1}
          >
            <title>
              {bucket.count === 0 
                ? 'No data' 
                : bucket.accessible 
                  ? `OK${bucket.avgLatency ? ` (${Math.round(bucket.avgLatency)}ms)` : ''}` 
                  : 'Failed'}
            </title>
          </rect>
        {/each}
      </svg>
      
      <!-- Time labels -->
      <div class="flex justify-between mt-1 text-[10px] text-zinc-500">
        <span>{hours}h ago</span>
        <span>now</span>
      </div>
    </div>

    <!-- Stats Grid -->
    {#if showStats && stats}
      <div class="grid grid-cols-3 gap-2">
        <!-- Uptime -->
        <div class="p-2 bg-zinc-900/40 rounded-lg">
          <div class="text-[10px] text-zinc-500 uppercase tracking-wider mb-0.5">Uptime</div>
          <div class="text-sm font-semibold {getUptimeColor(stats.uptimePercent)}">
            {stats.uptimePercent.toFixed(1)}%
          </div>
        </div>

        <!-- Avg Latency -->
        <div class="p-2 bg-zinc-900/40 rounded-lg">
          <div class="text-[10px] text-zinc-500 uppercase tracking-wider mb-0.5">Avg Latency</div>
          <div class="text-sm font-semibold {getLatencyColor(stats.avgLatencyMs)}">
            {stats.avgLatencyMs !== null ? `${Math.round(stats.avgLatencyMs)}ms` : '—'}
          </div>
        </div>

        <!-- Checks -->
        <div class="p-2 bg-zinc-900/40 rounded-lg">
          <div class="text-[10px] text-zinc-500 uppercase tracking-wider mb-0.5">Checks</div>
          <div class="text-sm font-semibold text-zinc-100">
            {stats.totalChecks}
          </div>
        </div>
      </div>

      <!-- Additional Stats Row -->
      <div class="flex items-center justify-between text-xs text-zinc-400 px-1">
        <div class="flex items-center gap-3">
          <span>
            <span class="text-emerald-400">{stats.successfulChecks}</span> ok
          </span>
          <span>
            <span class="text-red-400">{stats.failedChecks}</span> failed
          </span>
        </div>
        <div>
          Last check: <span class="text-zinc-300">{formatDuration(stats.lastCheckAt)}</span>
        </div>
      </div>

      <!-- Latency Range -->
      {#if stats.minLatencyMs !== null && stats.maxLatencyMs !== null}
        <div class="flex items-center gap-2 text-xs text-zinc-500 px-1">
          <span>Latency range:</span>
          <span class="text-zinc-400">
            {stats.minLatencyMs}ms — {stats.maxLatencyMs}ms
          </span>
        </div>
      {/if}
    {/if}
  </div>
{/if}
