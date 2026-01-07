<script lang="ts">
  // Types
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

  // Props
  let {
    results = [],
    isTesting = false,
    lastResultId = null,
    onApplyBest
  }: {
    results: TestResult[];
    isTesting: boolean;
    lastResultId: string | null;
    onApplyBest: () => void;
  } = $props();

  // Sorting state
  let sortBy = $state<'score' | 'name' | 'latency' | 'success_rate'>('score');
  let sortDesc = $state(true);

  // Sorting
  function toggleSort(column: 'score' | 'name' | 'latency' | 'success_rate') {
    if (sortBy === column) {
      sortDesc = !sortDesc;
    } else {
      sortBy = column;
      // Default to descending for numeric values, ascending for name
      sortDesc = column !== 'name';
    }
  }

  // Computed sorted results
  let sortedResults = $derived.by(() => {
    return [...results].sort((a, b) => {
      let diff = 0;
      switch (sortBy) {
        case 'score': diff = b.score - a.score; break;
        case 'name': diff = a.name.localeCompare(b.name); break;
        case 'latency':
          const aLat = a.latency_ms >= 9999 ? 99999 : a.latency_ms;
          const bLat = b.latency_ms >= 9999 ? 99999 : b.latency_ms;
          diff = aLat - bLat; break;
        case 'success_rate': diff = b.success_rate - a.success_rate; break;
      }
      return sortDesc ? diff : -diff;
    });
  });

  // Helper functions
  function getStatusBadge(status: string): string {
    switch (status) {
      case 'success': return 'bg-emerald-500/20 text-emerald-400';
      case 'partial': return 'bg-amber-500/20 text-amber-400';
      case 'failed': return 'bg-red-500/20 text-red-400';
      default: return 'bg-zinc-500/20 text-zinc-400';
    }
  }

  function getStatusText(status: string): string {
    switch (status) {
      case 'success': return 'Success';
      case 'partial': return 'Partial';
      case 'failed': return 'Failed';
      default: return status;
    }
  }

  function getSuccessRateColor(rate: number): string {
    if (rate >= 80) return 'text-emerald-400';
    if (rate >= 50) return 'text-amber-400';
    return 'text-red-400';
  }

  function getResultStatus(result: TestResult): string {
    if (result.success_rate >= 80) return 'success';
    if (result.success_rate >= 50) return 'partial';
    if (result.success_rate > 0) return 'partial';
    return 'failed';
  }

  function getProgressBarColor(rate: number): string {
    if (rate >= 80) return 'bg-emerald-500';
    if (rate >= 50) return 'bg-amber-500';
    return 'bg-red-500';
  }
</script>

{#if results.length > 0}
  <div class="bg-zinc-900/60 rounded-xl border border-white/5 overflow-hidden">
    <div class="flex items-center justify-between p-5 border-b border-white/5">
      <h2 class="text-lg font-semibold text-white">Results</h2>
      <button
        onclick={onApplyBest}
        disabled={isTesting || sortedResults.length === 0}
        class="px-4 py-2 bg-gradient-to-r from-cyan-500 to-cyan-600 hover:from-cyan-400 hover:to-cyan-500 text-zinc-950 rounded-lg font-medium transition-all flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        Apply Best
      </button>
    </div>
    
    <div class="overflow-x-auto">
      <table class="w-full">
        <thead class="bg-zinc-950/60">
          <tr>
            <th class="px-5 py-3 text-left text-xs font-medium text-zinc-400 uppercase tracking-wider cursor-pointer hover:text-white transition-colors" onclick={() => toggleSort('name')}>
              <span class="flex items-center gap-1">
                Name
                {#if sortBy === 'name'}<span class="text-cyan-400">{sortDesc ? '↓' : '↑'}</span>{/if}
              </span>
            </th>
            <th class="px-5 py-3 text-left text-xs font-medium text-zinc-400 uppercase tracking-wider">Type</th>
            <th class="px-5 py-3 text-left text-xs font-medium text-zinc-400 uppercase tracking-wider cursor-pointer hover:text-white transition-colors" onclick={() => toggleSort('success_rate')}>
              <span class="flex items-center gap-1">
                Success Rate
                {#if sortBy === 'success_rate'}<span class="text-cyan-400">{sortDesc ? '↓' : '↑'}</span>{/if}
              </span>
            </th>
            <th class="px-5 py-3 text-left text-xs font-medium text-zinc-400 uppercase tracking-wider cursor-pointer hover:text-white transition-colors" onclick={() => toggleSort('latency')}>
              <span class="flex items-center gap-1">
                Latency
                {#if sortBy === 'latency'}<span class="text-cyan-400">{sortDesc ? '↓' : '↑'}</span>{/if}
              </span>
            </th>
            <th class="px-5 py-3 text-left text-xs font-medium text-zinc-400 uppercase tracking-wider cursor-pointer hover:text-white transition-colors" onclick={() => toggleSort('score')}>
              <span class="flex items-center gap-1">
                Score
                {#if sortBy === 'score'}<span class="text-cyan-400">{sortDesc ? '↓' : '↑'}</span>{/if}
              </span>
            </th>
            <th class="px-5 py-3 text-left text-xs font-medium text-zinc-400 uppercase tracking-wider">Status</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-white/5">
          {#each sortedResults as result, i}
            <tr class="transition-all duration-500 {i === 0 ? 'bg-cyan-500/5' : ''} {lastResultId === result.id ? 'bg-emerald-500/10 animate-pulse' : 'hover:bg-zinc-800/30'}">
              <td class="px-5 py-4">
                <div class="flex items-center gap-2">
                  {#if i === 0 && !isTesting}<span class="text-yellow-400 animate-bounce">🏆</span>
                  {:else if lastResultId === result.id}<span class="text-emerald-400 animate-ping">●</span>{/if}
                  <span class="text-white font-medium">{result.name}</span>
                </div>
              </td>
              <td class="px-5 py-4">
                <span class="px-2 py-1 text-xs rounded-full {result.item_type === 'proxy' ? 'bg-purple-500/20 text-purple-400' : 'bg-blue-500/20 text-blue-400'}">
                  {result.item_type === 'proxy' ? 'Proxy' : 'Strategy'}
                </span>
              </td>
              <td class="px-5 py-4">
                <div class="flex items-center gap-2">
                  <div class="w-16 h-2 bg-zinc-950 rounded-full overflow-hidden">
                    <div class="h-full rounded-full transition-all duration-500 {getProgressBarColor(result.success_rate)}" style="width: {result.success_rate}%"></div>
                  </div>
                  <span class="{getSuccessRateColor(result.success_rate)} font-medium">{result.success_rate.toFixed(1)}%</span>
                </div>
              </td>
              <td class="px-5 py-4 text-zinc-400">{result.latency_ms < 9999 ? `${result.latency_ms}ms` : '—'}</td>
              <td class="px-5 py-4">
                <span class="text-cyan-400 font-bold {lastResultId === result.id ? 'animate-pulse text-lg' : ''}">{result.score.toFixed(1)}</span>
              </td>
              <td class="px-5 py-4">
                <span class="px-2 py-1 text-xs rounded-full {getStatusBadge(getResultStatus(result))}">{getStatusText(getResultStatus(result))}</span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
{/if}
