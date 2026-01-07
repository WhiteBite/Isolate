<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  interface Props {
    context?: PluginContext;
  }
  
  let { context }: Props = $props();
  
  // State
  let isTesting = $state(false);
  let downloadSpeed = $state<number | null>(null);
  let uploadSpeed = $state<number | null>(null);
  let progress = $state(0);
  let currentPhase = $state<'idle' | 'download' | 'upload' | 'done'>('idle');
  
  interface TestResult {
    download: number;
    upload: number;
    timestamp: number;
  }
  
  let history = $state<TestResult[]>([]);
  
  // Load history on mount
  $effect(() => {
    loadHistory();
  });
  
  async function loadHistory() {
    try {
      const saved = localStorage.getItem('speed-test-history');
      if (saved) {
        history = JSON.parse(saved).slice(-5);
      }
    } catch {
      history = [];
    }
  }
  
  function saveHistory(result: TestResult) {
    history = [...history.slice(-4), result];
    localStorage.setItem('speed-test-history', JSON.stringify(history));
  }
  
  // Simulate speed test (in real implementation, would use actual speed test API)
  async function runSpeedTest() {
    if (isTesting) return;
    
    isTesting = true;
    progress = 0;
    downloadSpeed = null;
    uploadSpeed = null;
    currentPhase = 'download';
    
    // Simulate download test
    for (let i = 0; i <= 100; i += 5) {
      await sleep(50);
      progress = i / 2;
    }
    downloadSpeed = Math.round(50 + Math.random() * 150);
    
    // Simulate upload test
    currentPhase = 'upload';
    for (let i = 0; i <= 100; i += 5) {
      await sleep(50);
      progress = 50 + i / 2;
    }
    uploadSpeed = Math.round(20 + Math.random() * 80);
    
    currentPhase = 'done';
    isTesting = false;
    progress = 100;
    
    // Save to history
    saveHistory({
      download: downloadSpeed,
      upload: uploadSpeed,
      timestamp: Date.now()
    });
  }
  
  function sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  function formatSpeed(speed: number | null): string {
    if (speed === null) return '--';
    return speed.toFixed(1);
  }
  
  function formatTime(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  }
  
  // Derived states
  let statusText = $derived(
    currentPhase === 'idle' ? 'Ready to test' :
    currentPhase === 'download' ? 'Testing download...' :
    currentPhase === 'upload' ? 'Testing upload...' :
    'Complete'
  );
</script>

<div class="flex flex-col h-full gap-4">
  <!-- Main Speed Display -->
  <div class="flex items-center justify-between gap-4">
    <!-- Download -->
    <div class="flex-1 text-center">
      <div class="text-[10px] uppercase tracking-wider text-zinc-500 mb-1">Download</div>
      <div class="flex items-baseline justify-center gap-1">
        <span class="text-2xl font-bold text-cyan-400 tabular-nums transition-all duration-300"
              class:animate-pulse={currentPhase === 'download'}>
          {formatSpeed(downloadSpeed)}
        </span>
        <span class="text-xs text-zinc-500">Mbps</span>
      </div>
    </div>
    
    <!-- Test Button -->
    <button
      onclick={runSpeedTest}
      disabled={isTesting}
      class="
        relative w-16 h-16 rounded-full
        bg-gradient-to-br from-cyan-500/20 to-purple-500/20
        border border-white/10
        flex items-center justify-center
        transition-all duration-300
        hover:border-cyan-400/50 hover:shadow-lg hover:shadow-cyan-500/20
        disabled:opacity-50 disabled:cursor-not-allowed
        group
      "
    >
      {#if isTesting}
        <!-- Progress Ring -->
        <svg class="absolute inset-0 w-full h-full -rotate-90" viewBox="0 0 64 64">
          <circle
            cx="32" cy="32" r="28"
            fill="none"
            stroke="rgba(34, 211, 238, 0.2)"
            stroke-width="3"
          />
          <circle
            cx="32" cy="32" r="28"
            fill="none"
            stroke="rgb(34, 211, 238)"
            stroke-width="3"
            stroke-linecap="round"
            stroke-dasharray={2 * Math.PI * 28}
            stroke-dashoffset={2 * Math.PI * 28 * (1 - progress / 100)}
            class="transition-all duration-200"
          />
        </svg>
        <span class="text-lg">⚡</span>
      {:else}
        <span class="text-2xl group-hover:scale-110 transition-transform">⚡</span>
      {/if}
    </button>
    
    <!-- Upload -->
    <div class="flex-1 text-center">
      <div class="text-[10px] uppercase tracking-wider text-zinc-500 mb-1">Upload</div>
      <div class="flex items-baseline justify-center gap-1">
        <span class="text-2xl font-bold text-purple-400 tabular-nums transition-all duration-300"
              class:animate-pulse={currentPhase === 'upload'}>
          {formatSpeed(uploadSpeed)}
        </span>
        <span class="text-xs text-zinc-500">Mbps</span>
      </div>
    </div>
  </div>
  
  <!-- Status -->
  <div class="text-center text-xs text-zinc-500">
    {statusText}
  </div>
  
  <!-- History -->
  {#if history.length > 0}
    <div class="mt-auto pt-3 border-t border-white/5">
      <div class="text-[10px] uppercase tracking-wider text-zinc-600 mb-2">History</div>
      <div class="flex gap-2 overflow-x-auto pb-1">
        {#each history.slice().reverse() as result}
          <div class="flex-shrink-0 px-2 py-1 rounded bg-white/5 text-[10px]">
            <div class="text-zinc-500">{formatTime(result.timestamp)}</div>
            <div class="flex gap-2 mt-0.5">
              <span class="text-cyan-400">↓{result.download}</span>
              <span class="text-purple-400">↑{result.upload}</span>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .animate-pulse {
    animation: pulse 1s ease-in-out infinite;
  }
</style>
