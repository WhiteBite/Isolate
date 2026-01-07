<script lang="ts">
  interface Props {
    mode: 'system' | 'tun';
    onchange: (mode: 'system' | 'tun') => void;
    systemActive?: boolean;
    tunActive?: boolean;
    disabled?: boolean;
  }

  let { 
    mode, 
    onchange, 
    systemActive = false, 
    tunActive = false, 
    disabled = false 
  }: Props = $props();
</script>

<div class="flex items-center gap-1 p-1 bg-zinc-900/50 rounded-xl border border-white/5">
  <button
    type="button"
    onclick={() => onchange('system')}
    disabled={disabled}
    class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 {mode === 'system' 
      ? 'bg-zinc-800 text-white shadow-sm' 
      : 'bg-transparent text-zinc-500 hover:text-zinc-300 hover:bg-white/5'} {disabled ? 'opacity-50 cursor-not-allowed' : ''}"
  >
    <span class="text-base">🌍</span>
    <span>System Proxy</span>
    {#if systemActive && mode === 'system'}
      <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
    {/if}
  </button>
  
  <button
    type="button"
    onclick={() => onchange('tun')}
    disabled={disabled}
    class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 {mode === 'tun' 
      ? 'bg-zinc-800 text-white shadow-sm' 
      : 'bg-transparent text-zinc-500 hover:text-zinc-300 hover:bg-white/5'} {disabled ? 'opacity-50 cursor-not-allowed' : ''}"
  >
    <span class="text-base">🛡️</span>
    <span>TUN Driver</span>
    {#if tunActive && mode === 'tun'}
      <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
    {/if}
  </button>
</div>
