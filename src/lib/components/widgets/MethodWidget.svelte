<script lang="ts">
  type MethodType = 'direct' | 'zapret' | 'vless' | 'proxy';

  interface Props {
    method?: MethodType;
    methodName?: string;
    proxyLocation?: string;
    proxyName?: string;
    active?: boolean;
  }

  let { 
    method = 'zapret',
    methodName = 'DPI Bypass',
    proxyLocation,
    proxyName,
    active = false
  }: Props = $props();

  const methodConfig = {
    direct: {
      icon: '🌐',
      label: 'Direct',
      description: 'No bypass active',
      color: 'text-zinc-500',
      glowColor: ''
    },
    zapret: {
      icon: '⚡',
      label: 'Zapret',
      description: 'DPI Bypass',
      color: 'text-zinc-100',
      glowColor: ''
    },
    vless: {
      icon: '🔒',
      label: 'VLESS',
      description: 'Encrypted tunnel',
      color: 'text-zinc-100',
      glowColor: ''
    },
    proxy: {
      icon: '🌍',
      label: 'Proxy',
      description: 'External proxy',
      color: 'text-zinc-100',
      glowColor: ''
    }
  };

  const config = $derived(methodConfig[method]);
  const isActive = $derived(active && method !== 'direct');
</script>

<div class="flex flex-col h-full justify-between relative">
  <!-- Method info -->
  <div class="flex items-start gap-3 relative z-10">
    <div 
      class="w-10 h-10 rounded-lg flex items-center justify-center text-lg
             border transition-all duration-200
             bg-zinc-800/50 border-white/5"
    >
      {config.icon}
    </div>
    
    <div class="flex-1">
      <h4 class="text-base font-medium text-zinc-100 flex items-center gap-2">
        {methodName || config.label}
        {#if isActive}
          <span class="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]"></span>
        {/if}
      </h4>
      <p class="text-sm text-zinc-500">
        {config.description}
      </p>
    </div>
  </div>

  <!-- Connection path -->
  <div class="mt-3 pt-3 border-t border-white/5 relative z-10">
    <div class="flex items-center gap-2 text-xs">
      <span class="text-zinc-600">via</span>
      {#if method === 'direct'}
        <span class="text-zinc-500">Direct Connection</span>
      {:else if method === 'proxy' && proxyName}
        <span class="px-2 py-0.5 rounded bg-zinc-800/80 text-zinc-300 text-xs">
          {proxyName}
          {#if proxyLocation}
            <span class="text-zinc-500">({proxyLocation})</span>
          {/if}
        </span>
      {:else if method === 'vless'}
        <span class="px-2 py-0.5 rounded bg-zinc-800/80 text-zinc-300 text-xs">
          VLESS Tunnel
          {#if proxyLocation}
            <span class="text-zinc-500">({proxyLocation})</span>
          {/if}
        </span>
      {:else}
        <span class="px-2 py-0.5 rounded bg-zinc-800/80 text-zinc-300 text-xs">
          Local DPI Bypass
        </span>
      {/if}
    </div>
  </div>
</div>
