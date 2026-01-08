<script lang="ts">
  import type { ProxyConfig } from '$lib/api';
  
  interface Props {
    proxies?: ProxyConfig[];
    chain?: string[];
    onChainChange?: (chain: string[]) => void;
  }
  
  let { proxies = [], chain = [], onChainChange }: Props = $props();
  
  let localChain = $state<string[]>([...chain]);
  let isDragging = $state(false);
  let dragIndex = $state<number | null>(null);
  
  // Sync with external chain
  $effect(() => {
    if (JSON.stringify(chain) !== JSON.stringify(localChain)) {
      localChain = [...chain];
    }
  });
  
  function addToChain(proxyId: string) {
    if (!localChain.includes(proxyId)) {
      localChain = [...localChain, proxyId];
      onChainChange?.(localChain);
    }
  }
  
  function removeFromChain(index: number) {
    localChain = localChain.filter((_, i) => i !== index);
    onChainChange?.(localChain);
  }
  
  function moveInChain(fromIndex: number, toIndex: number) {
    const newChain = [...localChain];
    const [removed] = newChain.splice(fromIndex, 1);
    newChain.splice(toIndex, 0, removed);
    localChain = newChain;
    onChainChange?.(localChain);
  }
  
  function clearChain() {
    localChain = [];
    onChainChange?.([]);
  }
  
  function getProxyById(id: string): ProxyConfig | undefined {
    return proxies.find(p => p.id === id);
  }
  
  function handleDragStart(index: number) {
    isDragging = true;
    dragIndex = index;
  }
  
  function handleDragEnd() {
    isDragging = false;
    dragIndex = null;
  }
  
  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (dragIndex !== null && dragIndex !== index) {
      moveInChain(dragIndex, index);
      dragIndex = index;
    }
  }
  
  let availableProxies = $derived(
    proxies.filter(p => !localChain.includes(p.id))
  );
  
  let chainProxies = $derived(
    localChain.map(id => getProxyById(id)).filter(Boolean) as ProxyConfig[]
  );
</script>

<div class="bg-zinc-900/50 rounded-xl border border-white/5 overflow-hidden">
  <!-- Header -->
  <div class="flex items-center justify-between px-4 py-3 border-b border-white/5">
    <div class="flex items-center gap-2">
      <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
      </svg>
      <span class="text-sm font-medium text-white">Proxy Chain</span>
      {#if localChain.length > 0}
        <span class="px-2 py-0.5 text-xs rounded-full bg-indigo-500/20 text-indigo-400">
          {localChain.length} hop{localChain.length !== 1 ? 's' : ''}
        </span>
      {/if}
    </div>
    
    {#if localChain.length > 0}
      <button
        onclick={clearChain}
        class="text-xs text-zinc-400 hover:text-white transition-colors"
      >
        Clear
      </button>
    {/if}
  </div>
  
  <!-- Chain visualization -->
  <div class="p-4">
    {#if localChain.length === 0}
      <div class="flex flex-col items-center justify-center py-8 text-center">
        <div class="w-12 h-12 mb-3 rounded-full bg-white/5 flex items-center justify-center">
          <svg class="w-6 h-6 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                  d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
          </svg>
        </div>
        <p class="text-sm text-zinc-400 mb-1">No chain configured</p>
        <p class="text-xs text-zinc-500">Add proxies to create a multi-hop chain</p>
      </div>
    {:else}
      <div class="flex items-center gap-2 flex-wrap">
        <!-- You -->
        <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
          <div class="w-2 h-2 rounded-full bg-emerald-400"></div>
          <span class="text-sm text-emerald-400 font-medium">You</span>
        </div>
        
        {#each chainProxies as proxy, index (proxy.id)}
          <!-- Arrow -->
          <svg class="w-4 h-4 text-zinc-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
          
          <!-- Proxy node -->
          <div
            draggable="true"
            ondragstart={() => handleDragStart(index)}
            ondragend={handleDragEnd}
            ondragover={(e) => handleDragOver(e, index)}
            class="group flex items-center gap-2 px-3 py-2 rounded-lg bg-white/5 border border-white/10 
                   hover:border-indigo-500/30 cursor-move transition-colors
                   {dragIndex === index ? 'opacity-50' : ''}"
          >
            <span class="text-xs text-zinc-500 font-mono">{index + 1}</span>
            <span class="text-sm text-white">{proxy.name}</span>
            <span class="text-xs text-zinc-500 uppercase">{proxy.protocol}</span>
            <button
              onclick={() => removeFromChain(index)}
              class="ml-1 p-0.5 rounded text-zinc-500 hover:text-red-400 hover:bg-red-500/10 
                     opacity-0 group-hover:opacity-100 transition-all"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        {/each}
        
        <!-- Arrow to destination -->
        <svg class="w-4 h-4 text-zinc-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
        
        <!-- Destination -->
        <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-blue-500/10 border border-blue-500/20">
          <svg class="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
          </svg>
          <span class="text-sm text-blue-400 font-medium">Internet</span>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Available proxies to add -->
  {#if availableProxies.length > 0}
    <div class="px-4 pb-4">
      <p class="text-xs text-zinc-500 mb-2">Click to add to chain:</p>
      <div class="flex flex-wrap gap-2">
        {#each availableProxies.slice(0, 6) as proxy (proxy.id)}
          <button
            onclick={() => addToChain(proxy.id)}
            class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-white/5 
                   hover:bg-white/10 text-sm text-zinc-300 hover:text-white transition-colors"
          >
            <span class="w-1.5 h-1.5 rounded-full bg-zinc-500"></span>
            {proxy.name}
          </button>
        {/each}
        {#if availableProxies.length > 6}
          <span class="px-2.5 py-1.5 text-xs text-zinc-500">
            +{availableProxies.length - 6} more
          </span>
        {/if}
      </div>
    </div>
  {/if}
</div>
