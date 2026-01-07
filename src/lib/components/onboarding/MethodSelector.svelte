<script lang="ts">
  type ConnectionMode = 'auto' | 'proxy';
  
  interface Props {
    connectionMode: ConnectionMode;
    onSelect: (mode: ConnectionMode) => void;
  }
  
  let { connectionMode, onSelect }: Props = $props();
</script>

<div class="flex-1 flex flex-col animate-fade-in">
  <div class="text-center mb-6">
    <h2 class="text-2xl font-bold text-white mb-2">Connection Method</h2>
    <p class="text-zinc-400">How do you want to bypass blocks?</p>
  </div>
  
  <div class="flex-1 flex flex-col gap-4 justify-center">
    <!-- Auto mode -->
    <button
      onclick={() => onSelect('auto')}
      class="group p-5 rounded-2xl border-2 transition-all duration-200 text-left
             {connectionMode === 'auto' 
               ? 'border-indigo-500/50 bg-indigo-500/10 shadow-lg shadow-indigo-500/10' 
               : 'border-white/5 bg-zinc-800/30 hover:border-white/10'}"
    >
      <div class="flex items-start gap-4">
        <div class="w-14 h-14 rounded-xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 
                    border border-indigo-500/20 flex items-center justify-center flex-shrink-0">
          <span class="text-2xl">🔧</span>
        </div>
        <div class="flex-1">
          <h3 class="text-white font-semibold text-lg mb-1">Automatic</h3>
          <p class="text-zinc-400 text-sm">
            Isolate will find the best DPI bypass method for your ISP
          </p>
          <div class="flex items-center gap-2 mt-2">
            <span class="px-2 py-0.5 text-xs font-medium bg-emerald-500/10 text-emerald-400 rounded">Recommended</span>
          </div>
        </div>
        <div class="w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all
                    {connectionMode === 'auto' ? 'border-indigo-500 bg-indigo-500' : 'border-zinc-600'}">
          {#if connectionMode === 'auto'}
            <div class="w-2 h-2 rounded-full bg-white"></div>
          {/if}
        </div>
      </div>
    </button>
    
    <!-- Proxy mode -->
    <button
      onclick={() => onSelect('proxy')}
      class="group p-5 rounded-2xl border-2 transition-all duration-200 text-left
             {connectionMode === 'proxy' 
               ? 'border-indigo-500/50 bg-indigo-500/10 shadow-lg shadow-indigo-500/10' 
               : 'border-white/5 bg-zinc-800/30 hover:border-white/10'}"
    >
      <div class="flex items-start gap-4">
        <div class="w-14 h-14 rounded-xl bg-gradient-to-br from-emerald-500/20 to-cyan-500/20 
                    border border-emerald-500/20 flex items-center justify-center flex-shrink-0">
          <span class="text-2xl">🌐</span>
        </div>
        <div class="flex-1">
          <h3 class="text-white font-semibold text-lg mb-1">Custom Proxy</h3>
          <p class="text-zinc-400 text-sm">
            Use VLESS, Shadowsocks, SOCKS5 or another proxy
          </p>
          <div class="flex items-center gap-2 mt-2">
            <span class="px-2 py-0.5 text-xs font-medium bg-zinc-700 text-zinc-400 rounded">Advanced</span>
          </div>
        </div>
        <div class="w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all
                    {connectionMode === 'proxy' ? 'border-indigo-500 bg-indigo-500' : 'border-zinc-600'}">
          {#if connectionMode === 'proxy'}
            <div class="w-2 h-2 rounded-full bg-white"></div>
          {/if}
        </div>
      </div>
    </button>
  </div>
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .animate-fade-in {
    animation: fade-in 0.4s ease-out forwards;
  }
</style>
