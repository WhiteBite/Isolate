<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  interface Props {
    context: PluginContext;
  }
  
  let { context }: Props = $props();
  
  // Settings state
  let autoTest = $state(false);
  let testInterval = $state(30);
  let testServer = $state('cloudflare');
  let showNotifications = $state(true);
  
  // Load settings on mount
  $effect(() => {
    loadSettings();
  });
  
  async function loadSettings() {
    const settings = await context.storage.get<{
      autoTest: boolean;
      testInterval: number;
      testServer: string;
      showNotifications: boolean;
    }>('settings');
    
    if (settings) {
      autoTest = settings.autoTest;
      testInterval = settings.testInterval;
      testServer = settings.testServer;
      showNotifications = settings.showNotifications;
    }
  }
  
  async function saveSettings() {
    await context.storage.set('settings', {
      autoTest,
      testInterval,
      testServer,
      showNotifications
    });
    
    context.events.emit('speed-settings-changed', {
      autoTest,
      testInterval,
      testServer,
      showNotifications
    });
  }
  
  // Auto-save on changes
  $effect(() => {
    // Debounce save
    const timeout = setTimeout(saveSettings, 500);
    return () => clearTimeout(timeout);
  });
</script>

<div class="speed-settings space-y-6">
  <div>
    <h3 class="text-sm font-medium text-white mb-4">Speed Monitor Settings</h3>
    <p class="text-xs text-zinc-500 mb-6">
      Configure how the speed monitor widget behaves.
    </p>
  </div>
  
  <!-- Auto Test Toggle -->
  <div class="flex items-center justify-between p-3 bg-zinc-900/40 rounded-lg border border-white/5">
    <div>
      <div class="text-sm text-white">Auto Test</div>
      <div class="text-xs text-zinc-500 mt-0.5">Automatically run speed tests periodically</div>
    </div>
    <button
      onclick={() => autoTest = !autoTest}
      class="relative w-11 h-6 rounded-full transition-colors
        {autoTest ? 'bg-cyan-500' : 'bg-zinc-700'}"
    >
      <span 
        class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform
          {autoTest ? 'translate-x-5' : 'translate-x-0'}"
      ></span>
    </button>
  </div>
  
  <!-- Test Interval -->
  <div class="p-3 bg-zinc-900/40 rounded-lg border border-white/5">
    <div class="flex items-center justify-between mb-3">
      <div>
        <div class="text-sm text-white">Test Interval</div>
        <div class="text-xs text-zinc-500 mt-0.5">How often to run automatic tests</div>
      </div>
      <span class="text-sm text-cyan-400 font-medium">{testInterval} min</span>
    </div>
    <input
      type="range"
      min="5"
      max="120"
      step="5"
      bind:value={testInterval}
      disabled={!autoTest}
      class="w-full h-1.5 bg-zinc-700 rounded-full appearance-none cursor-pointer
        disabled:opacity-50 disabled:cursor-not-allowed
        [&::-webkit-slider-thumb]:appearance-none
        [&::-webkit-slider-thumb]:w-4
        [&::-webkit-slider-thumb]:h-4
        [&::-webkit-slider-thumb]:bg-cyan-500
        [&::-webkit-slider-thumb]:rounded-full
        [&::-webkit-slider-thumb]:cursor-pointer"
    />
    <div class="flex justify-between text-[10px] text-zinc-600 mt-1">
      <span>5 min</span>
      <span>120 min</span>
    </div>
  </div>
  
  <!-- Test Server -->
  <div class="p-3 bg-zinc-900/40 rounded-lg border border-white/5">
    <div class="text-sm text-white mb-2">Test Server</div>
    <div class="text-xs text-zinc-500 mb-3">Select the server to use for speed tests</div>
    <div class="grid grid-cols-2 gap-2">
      {#each [
        { id: 'cloudflare', name: 'Cloudflare', icon: '☁️' },
        { id: 'google', name: 'Google', icon: '🔍' },
        { id: 'fast', name: 'Fast.com', icon: '🎬' },
        { id: 'custom', name: 'Custom', icon: '⚙️' }
      ] as server}
        <button
          onclick={() => testServer = server.id}
          class="flex items-center gap-2 p-2.5 rounded-lg border transition-all
            {testServer === server.id 
              ? 'bg-cyan-500/20 border-cyan-500/30 text-cyan-400' 
              : 'bg-zinc-800/50 border-white/5 text-zinc-400 hover:border-white/10'}"
        >
          <span>{server.icon}</span>
          <span class="text-xs font-medium">{server.name}</span>
        </button>
      {/each}
    </div>
  </div>
  
  <!-- Notifications -->
  <div class="flex items-center justify-between p-3 bg-zinc-900/40 rounded-lg border border-white/5">
    <div>
      <div class="text-sm text-white">Notifications</div>
      <div class="text-xs text-zinc-500 mt-0.5">Show notifications when tests complete</div>
    </div>
    <button
      onclick={() => showNotifications = !showNotifications}
      class="relative w-11 h-6 rounded-full transition-colors
        {showNotifications ? 'bg-cyan-500' : 'bg-zinc-700'}"
    >
      <span 
        class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform
          {showNotifications ? 'translate-x-5' : 'translate-x-0'}"
      ></span>
    </button>
  </div>
  
  <!-- Clear History -->
  <div class="pt-4 border-t border-white/5">
    <button
      onclick={async () => {
        await context.storage.delete('speed-history');
        context.events.emit('speed-history-cleared');
      }}
      class="w-full px-4 py-2.5 text-sm text-red-400 bg-red-500/10 hover:bg-red-500/20 
        border border-red-500/20 rounded-lg transition-colors"
    >
      Clear Speed History
    </button>
  </div>
</div>
