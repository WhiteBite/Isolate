<script lang="ts">
  import type { ProxyConfig } from '$lib/api/types';
  import { testProxy } from '$lib/api/proxy';
  import { 
    proxyTester, 
    type ProxyTestState, 
    type ProxyTesterProgress,
    getLatencyColor,
    formatLatency,
    sortByTestResults
  } from '$lib/utils/proxyTester';
  
  // Props
  interface Props {
    /** Proxies to test */
    proxies: ProxyConfig[];
    /** Maximum concurrent tests */
    concurrency?: number;
    /** Callback when testing completes */
    oncomplete?: (results: ProxyTestState[]) => void;
    /** Callback to update proxies with sorted results */
    onsort?: (sortedProxies: ProxyConfig[]) => void;
  }
  
  let { 
    proxies, 
    concurrency = 5, 
    oncomplete,
    onsort
  }: Props = $props();
  
  // State
  let testing = $state(false);
  let progress = $state<ProxyTesterProgress | null>(null);
  let results = $state<Map<string, ProxyTestState>>(new Map());
  let showResults = $state(false);
  
  // Derived
  let progressPercent = $derived(progress?.percent ?? 0);
  let hasResults = $derived(results.size > 0);
  let canTest = $derived(proxies.length > 0 && !testing);
  
  // Demo mode detection
  let isDemoMode = $derived(
    typeof window !== 'undefined' && 
    !('__TAURI__' in window) && 
    !('__TAURI_INTERNALS__' in window)
  );
  
  // Test function (real or mock)
  async function testProxyFn(id: string) {
    if (isDemoMode) {
      // Demo mode: simulate random latency
      await new Promise(r => setTimeout(r, 200 + Math.random() * 500));
      const success = Math.random() > 0.2; // 80% success rate
      return {
        success,
        latency: success ? Math.floor(50 + Math.random() * 250) : undefined,
        error: success ? undefined : 'Connection timeout'
      };
    }
    return testProxy(id);
  }
  
  async function startTesting() {
    if (testing || proxies.length === 0) return;
    
    testing = true;
    showResults = true;
    results = new Map();
    
    // Initialize all as pending
    for (const proxy of proxies) {
      results.set(proxy.id, {
        proxyId: proxy.id,
        proxyName: proxy.name,
        latency: null,
        status: 'pending'
      });
    }
    results = new Map(results); // Trigger reactivity
    
    try {
      const testResults = await proxyTester.testAll(
        proxies,
        testProxyFn,
        {
          concurrency,
          onResult: (result) => {
            results.set(result.proxyId, result);
            results = new Map(results); // Trigger reactivity
          },
          onProgress: (p) => {
            progress = p;
          }
        }
      );
      
      // Sort proxies by results
      const sorted = sortByTestResults(proxies, testResults);
      onsort?.(sorted);
      oncomplete?.(testResults);
      
    } catch (err) {
      console.error('Proxy testing failed:', err);
    } finally {
      testing = false;
    }
  }
  
  function cancelTesting() {
    proxyTester.cancel();
    testing = false;
  }
  
  function clearResults() {
    results = new Map();
    progress = null;
    showResults = false;
  }
  
  // Get status icon
  function getStatusIcon(status: ProxyTestState['status']): string {
    switch (status) {
      case 'pending': return '○';
      case 'testing': return '◐';
      case 'success': return '●';
      case 'failed': return '✕';
      case 'cancelled': return '◌';
      default: return '○';
    }
  }
  
  // Get status color
  function getStatusColor(status: ProxyTestState['status']): string {
    switch (status) {
      case 'pending': return 'text-zinc-400';
      case 'testing': return 'text-blue-400 animate-pulse';
      case 'success': return 'text-emerald-400';
      case 'failed': return 'text-red-400';
      case 'cancelled': return 'text-zinc-400';
      default: return 'text-zinc-400';
    }
  }
</script>

<div class="flex flex-col gap-3">
  <!-- Control Bar -->
  <div class="flex items-center gap-3">
    {#if !testing}
      <button
        type="button"
        onclick={startTesting}
        disabled={!canTest}
        class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200
               {canTest 
                 ? 'bg-blue-500/20 text-blue-400 hover:bg-blue-500/30 border border-blue-500/30' 
                 : 'bg-zinc-800/50 text-zinc-400 cursor-not-allowed border border-zinc-700/30'}"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
        Test All ({proxies.length})
      </button>
    {:else}
      <button
        type="button"
        onclick={cancelTesting}
        class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200
               bg-red-500/20 text-red-400 hover:bg-red-500/30 border border-red-500/30"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M6 18L18 6M6 6l12 12" />
        </svg>
        Cancel
      </button>
    {/if}
    
    {#if hasResults && !testing}
      <button
        type="button"
        onclick={clearResults}
        class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200
               bg-zinc-800/50 text-zinc-400 hover:bg-zinc-700/50 hover:text-zinc-300 border border-zinc-700/30"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
        Clear
      </button>
    {/if}
    
    <!-- Concurrency indicator -->
    <span class="text-xs text-zinc-400 ml-auto">
      Max {concurrency} parallel
    </span>
  </div>
  
  <!-- Progress Bar -->
  {#if testing || (progress && progress.completed > 0)}
    <div class="space-y-2">
      <!-- Progress bar -->
      <div class="relative h-2 bg-zinc-800/50 rounded-full overflow-hidden">
        <div 
          class="absolute inset-y-0 left-0 bg-gradient-to-r from-blue-500 to-blue-400 rounded-full transition-all duration-300 ease-out"
          style="width: {progressPercent}%"
        ></div>
        {#if testing}
          <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent animate-shimmer"></div>
        {/if}
      </div>
      
      <!-- Progress stats -->
      <div class="flex items-center justify-between text-xs">
        <div class="flex items-center gap-4">
          <span class="text-zinc-400">
            {progress?.completed ?? 0} / {progress?.total ?? proxies.length}
          </span>
          {#if progress}
            <span class="text-emerald-400">
              ✓ {progress.successful}
            </span>
            <span class="text-red-400">
              ✕ {progress.failed}
            </span>
          {/if}
        </div>
        <span class="text-zinc-400 font-mono">
          {progressPercent}%
        </span>
      </div>
    </div>
  {/if}
  
  <!-- Results List -->
  {#if showResults && results.size > 0}
    <div class="mt-2 max-h-64 overflow-y-auto rounded-lg border border-white/5 bg-zinc-900/50">
      <div class="divide-y divide-white/5">
        {#each Array.from(results.values()) as result (result.proxyId)}
          <div class="flex items-center justify-between px-3 py-2 hover:bg-white/5 transition-colors">
            <div class="flex items-center gap-3 min-w-0">
              <!-- Status icon -->
              <span class="text-sm {getStatusColor(result.status)}">
                {getStatusIcon(result.status)}
              </span>
              
              <!-- Proxy name -->
              <span class="text-sm text-zinc-300 truncate">
                {result.proxyName}
              </span>
            </div>
            
            <div class="flex items-center gap-3 flex-shrink-0">
              <!-- Latency -->
              {#if result.status === 'success' && result.latency !== null}
                <span class="text-sm font-mono {getLatencyColor(result.latency)}">
                  {formatLatency(result.latency)}
                </span>
              {:else if result.status === 'failed'}
                <span class="text-xs text-red-400 truncate max-w-32" title={result.error}>
                  {result.error || 'Failed'}
                </span>
              {:else if result.status === 'testing'}
                <span class="text-xs text-blue-400 animate-pulse">
                  Testing...
                </span>
              {:else if result.status === 'cancelled'}
                <span class="text-xs text-zinc-400">
                  Cancelled
                </span>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes shimmer {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }
  
  .animate-shimmer {
    animation: shimmer 1.5s infinite;
  }
</style>
