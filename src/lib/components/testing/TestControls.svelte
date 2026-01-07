<script lang="ts">
  import Spinner from '$lib/components/Spinner.svelte';

  // Props
  let {
    testMode = $bindable('turbo'),
    isInteractive = $bindable(false),
    isTesting = false,
    canStartTest = false,
    onStart,
    onCancel
  }: {
    testMode: 'turbo' | 'deep';
    isInteractive: boolean;
    isTesting: boolean;
    canStartTest: boolean;
    onStart: () => void;
    onCancel: () => void;
  } = $props();
</script>

<div class="space-y-6">
  <!-- Test Mode -->
  <div class="bg-zinc-900/60 rounded-xl p-5 border border-white/5">
    <h2 class="text-lg font-semibold text-white mb-4">Test Mode</h2>
    
    <div class="space-y-3">
      <label class="flex items-center gap-3 p-3 bg-zinc-950/60 rounded-lg cursor-pointer hover:bg-zinc-800/50 transition-colors {testMode === 'turbo' ? 'ring-2 ring-cyan-500' : ''} {isTesting ? 'opacity-50 cursor-not-allowed' : ''}">
        <input
          type="radio"
          name="testMode"
          value="turbo"
          bind:group={testMode}
          disabled={isTesting}
          class="w-4 h-4 bg-zinc-800 border-zinc-600 text-cyan-500 focus:ring-cyan-500"
        />
        <div>
          <p class="text-white font-medium">Turbo (fast)</p>
          <p class="text-zinc-400 text-xs">1 check per service, ~5 sec</p>
        </div>
      </label>
      
      <label class="flex items-center gap-3 p-3 bg-zinc-950/60 rounded-lg cursor-pointer hover:bg-zinc-800/50 transition-colors {testMode === 'deep' ? 'ring-2 ring-cyan-500' : ''} {isTesting ? 'opacity-50 cursor-not-allowed' : ''}">
        <input
          type="radio"
          name="testMode"
          value="deep"
          bind:group={testMode}
          disabled={isTesting}
          class="w-4 h-4 bg-zinc-800 border-zinc-600 text-cyan-500 focus:ring-cyan-500"
        />
        <div>
          <p class="text-white font-medium">Deep (thorough)</p>
          <p class="text-zinc-400 text-xs">3 checks, averaging, ~15 sec</p>
        </div>
      </label>
    </div>

    <label class="flex items-center gap-3 mt-4 cursor-pointer {isTesting ? 'opacity-50 cursor-not-allowed' : ''}">
      <input
        type="checkbox"
        bind:checked={isInteractive}
        disabled={isTesting}
        class="w-4 h-4 rounded bg-zinc-800 border-zinc-600 text-cyan-500 focus:ring-cyan-500 focus:ring-offset-zinc-900"
      />
      <div>
        <span class="text-white text-sm">Interactive testing</span>
        <p class="text-zinc-400 text-xs">Open browser for verification</p>
      </div>
    </label>
  </div>

  <!-- Start Button -->
  <button
    onclick={isTesting ? onCancel : onStart}
    disabled={!canStartTest && !isTesting}
    class="w-full py-4 px-6 rounded-xl font-semibold text-lg transition-all duration-300 {isTesting 
      ? 'bg-red-500 hover:bg-red-600 text-white' 
      : canStartTest 
        ? 'bg-gradient-to-r from-cyan-500 to-cyan-600 hover:from-cyan-400 hover:to-cyan-500 text-zinc-950' 
        : 'bg-zinc-800 text-zinc-400 cursor-not-allowed'}"
  >
    {#if isTesting}
      <span class="flex items-center justify-center gap-2">
        <Spinner size="sm" color="white" />
        Cancel
      </span>
    {:else}
      Start Testing
    {/if}
  </button>
</div>
