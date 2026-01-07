<script lang="ts">
  import { browser } from '$app/environment';
  import { waitForBackend } from '$lib/utils/backend';

  // Types matching backend StrategyStats
  interface StrategyHistoryStats {
    successes: number;
    failures: number;
    lastSuccess: string | null;
    lastFailure: string | null;
  }

  interface DomainHistory {
    domain: string;
    strategies: Map<string, StrategyHistoryStats>;
  }

  interface Props {
    domain?: string;
    onClose?: () => void;
  }

  let { domain = '', onClose }: Props = $props();

  // State
  let loading = $state(true);
  let error = $state<string | null>(null);
  let history = $state<Record<string, StrategyHistoryStats>>({});
  let domainInput = $state('');
  let sortBy = $state<'successRate' | 'lastActivity' | 'total'>('successRate');
  let sortAsc = $state(false);

  // Derived: sorted history entries
  let sortedHistory = $derived(() => {
    const entries = Object.entries(history).map(([strategyId, stats]) => ({
      strategyId,
      ...stats,
      successRate: calculateSuccessRate(stats),
      totalAttempts: stats.successes + stats.failures,
      lastActivity: getLastActivity(stats)
    }));

    return entries.sort((a, b) => {
      let comparison = 0;
      switch (sortBy) {
        case 'successRate':
          comparison = a.successRate - b.successRate;
          break;
        case 'lastActivity':
          const aTime = a.lastActivity?.getTime() ?? 0;
          const bTime = b.lastActivity?.getTime() ?? 0;
          comparison = aTime - bTime;
          break;
        case 'total':
          comparison = a.totalAttempts - b.totalAttempts;
          break;
      }
      return sortAsc ? comparison : -comparison;
    });
  });

  let hasHistory = $derived(Object.keys(history).length > 0);

  // Initialize domainInput from prop and load history
  $effect(() => {
    if (browser) {
      if (domain && !domainInput) {
        domainInput = domain;
      }
      if (domainInput) {
        loadHistory(domainInput);
      }
    }
  });

  async function loadHistory(targetDomain: string) {
    if (!targetDomain.trim()) return;
    
    loading = true;
    error = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const ready = await waitForBackend(10, 200);
      
      if (!ready) {
        error = 'Backend not ready';
        return;
      }

      const result = await invoke<Record<string, StrategyHistoryStats>>('get_automation_history', {
        domain: targetDomain.trim()
      });
      
      history = result;
    } catch (e) {
      error = `Failed to load history: ${e}`;
      history = {};
    } finally {
      loading = false;
    }
  }

  function calculateSuccessRate(stats: StrategyHistoryStats): number {
    const total = stats.successes + stats.failures;
    if (total === 0) return 0;
    return (stats.successes / total) * 100;
  }

  function getLastActivity(stats: StrategyHistoryStats): Date | null {
    const dates: Date[] = [];
    if (stats.lastSuccess) dates.push(new Date(stats.lastSuccess));
    if (stats.lastFailure) dates.push(new Date(stats.lastFailure));
    if (dates.length === 0) return null;
    return dates.reduce((a, b) => a > b ? a : b);
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return '—';
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      day: 'numeric',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function formatSuccessRate(rate: number): string {
    return `${rate.toFixed(1)}%`;
  }

  function getSuccessRateColor(rate: number): string {
    if (rate >= 80) return 'text-emerald-400';
    if (rate >= 50) return 'text-amber-400';
    return 'text-red-400';
  }

  function getSuccessRateBg(rate: number): string {
    if (rate >= 80) return 'bg-emerald-400/10';
    if (rate >= 50) return 'bg-amber-400/10';
    return 'bg-red-400/10';
  }

  function toggleSort(column: typeof sortBy) {
    if (sortBy === column) {
      sortAsc = !sortAsc;
    } else {
      sortBy = column;
      sortAsc = false;
    }
  }

  function handleSearch() {
    if (domainInput.trim()) {
      loadHistory(domainInput.trim());
    }
  }
</script>

<div class="bg-void-50 rounded-xl border border-glass-border overflow-hidden">
  <!-- Header -->
  <div class="p-4 border-b border-glass-border">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-text-primary font-semibold text-lg flex items-center gap-2">
        <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
        </svg>
        Strategy History
      </h3>
      {#if onClose}
        <button
          onclick={onClose}
          aria-label="Close history panel"
          class="p-1.5 text-text-muted hover:text-text-primary hover:bg-void-100 rounded-lg transition-colors"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      {/if}
    </div>

    <!-- Domain Search -->
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={domainInput}
        placeholder="Enter domain (e.g., youtube.com)"
        onkeydown={(e) => e.key === 'Enter' && handleSearch()}
        class="flex-1 bg-void-100 text-text-primary rounded-lg px-4 py-2 border border-glass-border focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/20 focus:outline-none placeholder-text-muted text-sm"
      />
      <button
        onclick={handleSearch}
        disabled={loading || !domainInput.trim()}
        class="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg text-sm font-medium transition-colors"
      >
        {#if loading}
          <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        {:else}
          Search
        {/if}
      </button>
    </div>
  </div>

  <!-- Content -->
  <div class="p-4">
    {#if error}
      <div class="flex items-center gap-2 p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">
        <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        {error}
      </div>
    {:else if loading && domainInput}
      <div class="flex items-center justify-center py-8">
        <svg class="w-6 h-6 animate-spin text-indigo-400" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>
    {:else if !domainInput}
      <div class="text-center py-8 text-text-muted">
        <svg class="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <p>Enter a domain to view strategy history</p>
      </div>
    {:else if !hasHistory}
      <div class="text-center py-8 text-text-muted">
        <svg class="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>
        </svg>
        <p>No history found for <span class="text-text-primary font-medium">{domainInput}</span></p>
      </div>
    {:else}
      <!-- Sort Controls -->
      <div class="flex items-center gap-2 mb-4 text-sm">
        <span class="text-text-muted">Sort by:</span>
        <button
          onclick={() => toggleSort('successRate')}
          class="px-3 py-1.5 rounded-lg transition-colors {sortBy === 'successRate' ? 'bg-indigo-500/20 text-indigo-400' : 'bg-void-100 text-text-muted hover:text-text-primary'}"
        >
          Success Rate {sortBy === 'successRate' ? (sortAsc ? '↑' : '↓') : ''}
        </button>
        <button
          onclick={() => toggleSort('total')}
          class="px-3 py-1.5 rounded-lg transition-colors {sortBy === 'total' ? 'bg-indigo-500/20 text-indigo-400' : 'bg-void-100 text-text-muted hover:text-text-primary'}"
        >
          Total {sortBy === 'total' ? (sortAsc ? '↑' : '↓') : ''}
        </button>
        <button
          onclick={() => toggleSort('lastActivity')}
          class="px-3 py-1.5 rounded-lg transition-colors {sortBy === 'lastActivity' ? 'bg-indigo-500/20 text-indigo-400' : 'bg-void-100 text-text-muted hover:text-text-primary'}"
        >
          Last Activity {sortBy === 'lastActivity' ? (sortAsc ? '↑' : '↓') : ''}
        </button>
      </div>

      <!-- History Table -->
      <div class="space-y-2">
        {#each sortedHistory() as entry (entry.strategyId)}
          <div class="bg-void-100 rounded-xl p-4 border border-glass-border hover:border-glass-border-active transition-colors">
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-3">
                <span class="text-text-primary font-medium">{entry.strategyId}</span>
                <span class="px-2 py-0.5 rounded-lg text-xs font-medium {getSuccessRateBg(entry.successRate)} {getSuccessRateColor(entry.successRate)}">
                  {formatSuccessRate(entry.successRate)}
                </span>
              </div>
              <span class="text-text-muted text-sm">
                {entry.totalAttempts} attempts
              </span>
            </div>

            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <!-- Successes -->
              <div class="flex items-center gap-2">
                <div class="w-8 h-8 rounded-lg bg-emerald-400/10 flex items-center justify-center">
                  <svg class="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                  </svg>
                </div>
                <div>
                  <p class="text-text-muted text-xs">Successes</p>
                  <p class="text-emerald-400 font-semibold">{entry.successes}</p>
                </div>
              </div>

              <!-- Failures -->
              <div class="flex items-center gap-2">
                <div class="w-8 h-8 rounded-lg bg-red-400/10 flex items-center justify-center">
                  <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                  </svg>
                </div>
                <div>
                  <p class="text-text-muted text-xs">Failures</p>
                  <p class="text-red-400 font-semibold">{entry.failures}</p>
                </div>
              </div>

              <!-- Last Success -->
              <div>
                <p class="text-text-muted text-xs mb-1">Last Success</p>
                <p class="text-text-secondary text-xs">{formatDate(entry.lastSuccess)}</p>
              </div>

              <!-- Last Failure -->
              <div>
                <p class="text-text-muted text-xs mb-1">Last Failure</p>
                <p class="text-text-secondary text-xs">{formatDate(entry.lastFailure)}</p>
              </div>
            </div>

            <!-- Progress Bar -->
            <div class="mt-3 h-1.5 bg-void-200 rounded-full overflow-hidden">
              <div 
                class="h-full rounded-full transition-all duration-300 {entry.successRate >= 80 ? 'bg-emerald-400' : entry.successRate >= 50 ? 'bg-amber-400' : 'bg-red-400'}"
                style="width: {entry.successRate}%"
              ></div>
            </div>
          </div>
        {/each}
      </div>

      <!-- Summary -->
      <div class="mt-4 pt-4 border-t border-glass-border">
        <div class="flex items-center justify-between text-sm">
          <span class="text-text-muted">
            Showing <span class="text-text-primary font-medium">{sortedHistory().length}</span> strategies for <span class="text-indigo-400 font-medium">{domainInput}</span>
          </span>
        </div>
      </div>
    {/if}
  </div>
</div>
