<script lang="ts">
  import type { TrafficPoint, ActiveConnection } from '$lib/stores/dashboard.svelte';
  import TrafficChart from './TrafficChart.svelte';
  import ActiveConnectionItem from './ActiveConnectionItem.svelte';

  interface Props {
    trafficHistory: TrafficPoint[];
    connections: ActiveConnection[];
    maxConnections?: number;
  }

  let { trafficHistory, connections, maxConnections = 5 }: Props = $props();

  const displayedConnections = $derived(
    connections.slice(0, maxConnections)
  );

  const hasMoreConnections = $derived(connections.length > maxConnections);
  const hiddenCount = $derived(connections.length - maxConnections);
</script>

<div class="flex flex-col gap-4 p-4 rounded-2xl bg-slate-800/30 border border-slate-700/50">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-semibold text-slate-200">Активность</h3>
    <span class="text-xs text-slate-500">
      {connections.length} {connections.length === 1 ? 'соединение' : 'соединений'}
    </span>
  </div>

  <!-- Traffic Chart -->
  <div class="w-full">
    <TrafficChart data={trafficHistory} height={100} />
  </div>

  <!-- Connections List -->
  {#if connections.length > 0}
    <div class="flex flex-col gap-1">
      <h4 class="text-xs font-medium text-slate-400 uppercase tracking-wider mb-1">
        Активные соединения
      </h4>
      
      <div class="flex flex-col gap-1 max-h-48 overflow-y-auto">
        {#each displayedConnections as connection (connection.domain + connection.method)}
          <ActiveConnectionItem {connection} />
        {/each}
      </div>

      {#if hasMoreConnections}
        <button 
          type="button"
          class="mt-2 text-xs text-slate-500 hover:text-slate-300 transition-colors text-center"
        >
          + ещё {hiddenCount}
        </button>
      {/if}
    </div>
  {:else}
    <div class="py-6 text-center">
      <p class="text-sm text-slate-500">Нет активных соединений</p>
    </div>
  {/if}
</div>
