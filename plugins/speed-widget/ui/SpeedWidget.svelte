<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  interface Props {
    context: PluginContext;
  }
  
  let { context }: Props = $props();
  
  // State
  let downloadSpeed = $state(0);
  let uploadSpeed = $state(0);
  let ping = $state(0);
  let isRunning = $state(false);
  let lastUpdate = $state<Date | null>(null);
  let history = $state<{ download: number; upload: number; ping: number; time: Date }[]>([]);
  
  // Load saved history on mount
  $effect(() => {
    loadHistory();
  });
  
  async function loadHistory() {
    const saved = await context.storage.get<typeof history>('speed-history');
    if (saved) {
      history = saved.slice(-10); // Keep last 10 entries
    }
  }
  
  async function saveHistory() {
    await context.storage.set('speed-history', history.slice(-10));
  }
  
  // Simulate speed test (in real implementation, would use actual network test)
  async function runSpeedTest() {
    if (isRunning) return;
    
    isRunning = true;
    context.events.emit('speed-test-started');
    
    try {
      // Simulate download test
      for (let i = 0; i < 10; i++) {
        await new Promise(r => setTimeout(r, 200));
        downloadSpeed = Math.random() * 100 + 50; // 50-150 Mbps
      }
      
      // Simulate upload test
      for (let i = 0; i < 10; i++) {
        await new Promise(r => setTimeout(r, 200));
        uploadSpeed = Math.random() * 50 + 20; // 20-70 Mbps
      }
      
      // Simulate ping
      ping = Math.floor(Math.random() * 30 + 10); // 10-40ms
      
      lastUpdate = new Date();
      
      // Save to history
      history = [...history, {
        download: downloadSpeed,
        upload: uploadSpeed,
        ping,
        time: lastUpdate
      }].slice(-10);
      
      await saveHistory();
      
      context.events.emit('speed-test-completed', {
        download: downloadSpeed,
        upload: uploadSpeed,
        ping
      });
    } catch (error) {
      context.events.emit('speed-test-error', { error: String(error) });
    } finally {
      isRunning = false;
    }
  }
  
  // Format speed
  function formatSpeed(speed: number): string {
    if (speed >= 1000) {
      return `${(speed / 1000).toFixed(1)} Gbps`;
    }
    return `${speed.toFixed(1)} Mbps`;
  }
  
  // Get speed color
  function getSpeedColor(speed: number): string {
    if (speed >= 100) return 'text-emerald-400';
    if (speed >= 50) return 'text-cyan-400';
    if (speed >= 20) return 'text-amber-400';
    return 'text-red-400';
  }
  
  // Get ping color
  function getPingColor(ms: number): string {
    if (ms <= 20) return 'text-emerald-400';
    if (ms <= 50) return 'text-cyan-400';
    if (ms <= 100) return 'text-amber-400';
    return 'text-red-400';
  }
</script>

<div class="speed-widget p-4 bg-zinc-900/40 backdrop-blur-md border border-white/5 rounded-xl">
  <!-- Header -->
  <div class="flex items-center justify-between mb-4">
    <div class="flex items-center gap-2">
      <span class="text-lg">⚡</span>
      <h3 class="text-xs text-zinc-400 uppercase tracking-wider font-semibold">Speed Monitor</h3>
    </div>
    <button
      onclick={runSpeedTest}
      disabled={isRunning}
      class="px-3 py-1.5 text-xs font-medium rounded-lg transition-all
        {isRunning 
          ? 'bg-zinc-800 text-zinc-400 cursor-not-allowed' 
          : 'bg-cyan-500/20 text-cyan-400 hover:bg-cyan-500/30 border border-cyan-500/30'}"
    >
      {isRunning ? 'Testing...' : 'Run Test'}
    </button>
  </div>
  
  <!-- Speed Stats -->
  <div class="grid grid-cols-3 gap-4">
    <!-- Download -->
    <div class="text-center">
      <div class="text-[10px] text-zinc-400 uppercase tracking-wider mb-1">Download</div>
      <div class="text-2xl font-bold {getSpeedColor(downloadSpeed)}">
        {downloadSpeed > 0 ? formatSpeed(downloadSpeed) : '—'}
      </div>
      <div class="flex items-center justify-center gap-1 mt-1">
        <svg class="w-3 h-3 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
        </svg>
        <span class="text-[10px] text-zinc-400">↓</span>
      </div>
    </div>
    
    <!-- Upload -->
    <div class="text-center">
      <div class="text-[10px] text-zinc-400 uppercase tracking-wider mb-1">Upload</div>
      <div class="text-2xl font-bold {getSpeedColor(uploadSpeed)}">
        {uploadSpeed > 0 ? formatSpeed(uploadSpeed) : '—'}
      </div>
      <div class="flex items-center justify-center gap-1 mt-1">
        <svg class="w-3 h-3 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18" />
        </svg>
        <span class="text-[10px] text-zinc-400">↑</span>
      </div>
    </div>
    
    <!-- Ping -->
    <div class="text-center">
      <div class="text-[10px] text-zinc-400 uppercase tracking-wider mb-1">Ping</div>
      <div class="text-2xl font-bold {getPingColor(ping)}">
        {ping > 0 ? `${ping}ms` : '—'}
      </div>
      <div class="flex items-center justify-center gap-1 mt-1">
        <svg class="w-3 h-3 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
        <span class="text-[10px] text-zinc-400">ms</span>
      </div>
    </div>
  </div>
  
  <!-- Progress bar during test -->
  {#if isRunning}
    <div class="mt-4">
      <div class="h-1 bg-zinc-800 rounded-full overflow-hidden">
        <div class="h-full bg-gradient-to-r from-cyan-500 to-emerald-500 animate-pulse" style="width: 100%"></div>
      </div>
      <p class="text-[10px] text-zinc-400 text-center mt-2">Running speed test...</p>
    </div>
  {/if}
  
  <!-- Last update -->
  {#if lastUpdate && !isRunning}
    <div class="mt-4 pt-3 border-t border-white/5">
      <p class="text-[10px] text-zinc-400 text-center">
        Last tested: {lastUpdate.toLocaleTimeString()}
      </p>
    </div>
  {/if}
</div>

<style>
  .speed-widget {
    min-height: 180px;
  }
</style>
