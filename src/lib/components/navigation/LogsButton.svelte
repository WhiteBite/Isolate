<script lang="ts">
  import { bottomDrawerStore } from '$lib/stores/bottomDrawer.svelte';

  interface Props {
    collapsed?: boolean;
  }

  let { collapsed = false }: Props = $props();

  function handleClick() {
    bottomDrawerStore.toggle();
  }
</script>

<button
  onclick={handleClick}
  class="group relative flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-150 w-full
    {bottomDrawerStore.isOpen 
      ? 'bg-white/5 text-white' 
      : 'text-zinc-400 hover:bg-white/5 hover:text-zinc-200'}"
  aria-expanded={bottomDrawerStore.isOpen}
  aria-label={collapsed ? 'Toggle Logs' : undefined}
  title={collapsed ? 'Toggle Logs' : undefined}
>
  <!-- Active indicator -->
  {#if bottomDrawerStore.isOpen}
    <div class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-4 bg-indigo-500 rounded-r-full shadow-[0_0_10px_rgba(99,102,241,0.5)]"></div>
  {/if}
  
  <!-- Icon -->
  <span class="w-5 h-5 flex-shrink-0">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="4 17 10 11 4 5"/>
      <line x1="12" y1="19" x2="20" y2="19"/>
    </svg>
  </span>
  
  <!-- Name (hidden when collapsed) -->
  {#if !collapsed}
    <span class="text-sm font-medium whitespace-nowrap overflow-hidden flex-1 text-left">Logs</span>
    
    <!-- Keyboard shortcut hint -->
    <kbd class="px-1.5 py-0.5 text-[10px] bg-zinc-800 rounded border border-white/5 text-zinc-400">
      Ctrl+`
    </kbd>
  {/if}
  
  <!-- Tooltip when collapsed -->
  {#if collapsed}
    <div class="absolute left-full ml-2 px-2 py-1 bg-zinc-800 text-zinc-200 text-xs rounded-md opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-150 whitespace-nowrap z-50 pointer-events-none shadow-lg border border-white/10">
      Logs
      <span class="ml-1 text-zinc-400">Ctrl+`</span>
    </div>
  {/if}
</button>
