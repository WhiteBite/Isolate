<script lang="ts">
  import { proxyChainStore, type ActiveChainInfo } from '$lib/stores/proxyChain.svelte';

  interface Props {
    ondeactivate?: () => void;
  }

  let { ondeactivate }: Props = $props();

  // Get active chain info from store
  let activeInfo = $derived(proxyChainStore.activeChainInfo);
  let deactivating = $state(false);

  // Format activation time
  let activatedTimeAgo = $derived.by(() => {
    if (!activeInfo) return '';
    
    const now = Date.now();
    const diff = now - activeInfo.activatedAt;
    
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) return `${hours}h ${minutes % 60}m ago`;
    if (minutes > 0) return `${minutes}m ago`;
    return 'just now';
  });

  async function handleDeactivate() {
    if (deactivating || !activeInfo) return;
    
    deactivating = true;
    try {
      await proxyChainStore.stopChain(activeInfo.chain.id);
      ondeactivate?.();
    } finally {
      deactivating = false;
    }
  }
</script>

{#if activeInfo}
  <div class="flex items-center gap-3 px-3 py-2 bg-emerald-500/10 border border-emerald-500/20 rounded-xl">
    <!-- Status indicator -->
    <div class="flex items-center gap-2">
      <span class="relative flex h-2 w-2">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-500 opacity-75"></span>
        <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
      </span>
    </div>

    <!-- Chain info -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-emerald-400 truncate">
          {activeInfo.chain.name}
        </span>
        <span class="text-xs text-zinc-500">•</span>
        <span class="text-xs text-zinc-400">
          {activeInfo.chain.proxies.length} {activeInfo.chain.proxies.length === 1 ? 'proxy' : 'proxies'}
        </span>
      </div>
      <p class="text-xs text-zinc-500">
        Active {activatedTimeAgo}
      </p>
    </div>

    <!-- Deactivate button -->
    <button
      type="button"
      onclick={handleDeactivate}
      disabled={deactivating}
      class="px-3 py-1.5 text-xs font-medium rounded-lg
             bg-zinc-800/60 border border-white/10
             text-zinc-300 hover:text-white hover:bg-zinc-700/60
             disabled:opacity-50 disabled:cursor-not-allowed
             transition-colors flex items-center gap-1.5"
    >
      {#if deactivating}
        <svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      {:else}
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      {/if}
      <span>Deactivate</span>
    </button>
  </div>
{/if}
