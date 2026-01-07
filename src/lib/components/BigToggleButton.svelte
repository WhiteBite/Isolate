<script lang="ts">
  interface Props {
    active?: boolean;
    loading?: boolean;
    disabled?: boolean;
    onclick?: () => void;
  }

  let { active = false, loading = false, disabled = false, onclick }: Props = $props();

  function handleClick() {
    if (!disabled && !loading) {
      onclick?.();
    }
  }
</script>

<button
  class="group relative w-[180px] h-[180px] rounded-full cursor-pointer
         font-semibold text-base outline-none
         transition-all duration-500 ease-out
         flex items-center justify-center
         {active 
           ? 'bg-gradient-to-br from-indigo-500 to-purple-600 text-white border-indigo-400/50 shadow-[0_0_50px_-10px_rgba(79,70,229,0.5)]' 
           : 'bg-zinc-900 text-zinc-400 border-white/5 shadow-inner hover:text-zinc-300 hover:border-white/10'}
         {loading ? 'animate-pulse cursor-wait' : ''}
         {disabled ? 'opacity-50 cursor-not-allowed' : ''}
         border"
  disabled={disabled || loading}
  onclick={handleClick}
>
  <!-- Active glow -->
  {#if active && !loading}
    <div class="absolute -inset-4 rounded-full bg-indigo-500/30 blur-2xl animate-pulse-slow pointer-events-none"></div>
  {/if}
  
  <!-- Hover glow -->
  {#if !active && !loading && !disabled}
    <div class="absolute -inset-3 rounded-full bg-indigo-500/0 group-hover:bg-indigo-500/10 blur-xl transition-all duration-500 pointer-events-none"></div>
  {/if}

  <!-- Content -->
  <div class="relative z-10 flex flex-col items-center justify-center gap-3">
    {#if loading}
      <div class="w-12 h-12 border-3 border-white/20 border-t-white rounded-full animate-spin"></div>
      <span class="text-xs tracking-widest uppercase font-medium">Connecting</span>
    {:else if active}
      <svg class="w-14 h-14 animate-pulse" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm-2 16l-4-4 1.41-1.41L10 14.17l6.59-6.59L18 9l-8 8z"/>
      </svg>
      <span class="text-xs tracking-widest uppercase font-semibold">Protected</span>
    {:else}
      <svg class="w-14 h-14 opacity-60 group-hover:opacity-100 transition-opacity duration-300" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
      </svg>
      <span class="text-xs tracking-widest uppercase font-medium opacity-60 group-hover:opacity-100 transition-opacity">Start</span>
    {/if}
  </div>
</button>

<style>
  @keyframes pulse-slow {
    0%, 100% { opacity: 0.3; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(1.05); }
  }
  .animate-pulse-slow { animation: pulse-slow 2s ease-in-out infinite; }
  .border-3 { border-width: 3px; }
</style>
