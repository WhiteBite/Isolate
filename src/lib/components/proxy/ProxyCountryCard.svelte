<script lang="ts">
  import CountryFlag from './CountryFlag.svelte';
  import ProtocolBadge from './ProtocolBadge.svelte';
  import LatencyIndicator from './LatencyIndicator.svelte';
  import { getCountryName } from '$lib/utils/countries';
  
  type Protocol = 'vless' | 'vmess' | 'shadowsocks' | 'trojan' | 'ss' | 'http' | 'socks5';
  type Status = 'active' | 'inactive' | 'testing' | 'error';
  
  interface Props {
    id: string;
    name: string;
    countryCode: string;
    protocol: Protocol;
    server: string;
    port: number;
    latency?: number | null;
    status?: Status;
    onTest?: (id: string) => void;
    onConnect?: (id: string) => void;
    onEdit?: (id: string) => void;
    onDelete?: (id: string) => void;
  }
  
  let { 
    id,
    name, 
    countryCode, 
    protocol, 
    server,
    port,
    latency = null,
    status = 'inactive',
    onTest,
    onConnect,
    onEdit,
    onDelete
  }: Props = $props();
  
  let countryName = $derived(getCountryName(countryCode));
  let isTesting = $derived(status === 'testing');
  let isActive = $derived(status === 'active');
  
  let statusBorderClass = $derived.by(() => {
    switch (status) {
      case 'active': return 'border-green-500/50 shadow-green-500/10';
      case 'testing': return 'border-blue-500/50 shadow-blue-500/10';
      case 'error': return 'border-red-500/50 shadow-red-500/10';
      default: return 'border-white/10';
    }
  });
</script>

<div 
  class="group relative bg-white/5 backdrop-blur-sm rounded-xl border {statusBorderClass} 
         p-4 transition-all duration-200 hover:bg-white/8 hover:border-white/20 
         hover:shadow-lg cursor-pointer"
>
  <!-- Status indicator -->
  {#if isActive}
    <div class="absolute top-3 right-3">
      <span class="relative flex h-3 w-3">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
        <span class="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
      </span>
    </div>
  {/if}
  
  <!-- Main content -->
  <div class="flex items-start gap-4">
    <!-- Large flag -->
    <div class="flex-shrink-0">
      <CountryFlag countryCode={countryCode} size="lg" />
    </div>
    
    <!-- Info -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2 mb-1">
        <h3 class="text-white font-medium truncate">{name}</h3>
        <ProtocolBadge protocol={protocol} size="sm" />
      </div>
      
      <p class="text-sm text-gray-400 mb-2">{countryName}</p>
      
      <div class="flex items-center gap-3 text-xs text-gray-500">
        <span class="truncate max-w-[120px]" title="{server}:{port}">
          {server}:{port}
        </span>
        <LatencyIndicator latency={latency} testing={isTesting} />
      </div>
    </div>
  </div>
  
  <!-- Actions -->
  <div class="flex items-center gap-2 mt-4 pt-3 border-t border-white/5">
    <button
      onclick={() => onTest?.(id)}
      disabled={isTesting}
      class="flex-1 px-3 py-1.5 text-xs font-medium rounded-lg
             bg-white/5 text-gray-300 hover:bg-white/10 hover:text-white
             disabled:opacity-50 disabled:cursor-not-allowed
             transition-colors"
    >
      {isTesting ? 'Testing...' : 'Test'}
    </button>
    
    <button
      onclick={() => onConnect?.(id)}
      disabled={isTesting}
      class="flex-1 px-3 py-1.5 text-xs font-medium rounded-lg
             {isActive 
               ? 'bg-green-500/20 text-green-400 hover:bg-green-500/30' 
               : 'bg-blue-500/20 text-blue-400 hover:bg-blue-500/30'}
             disabled:opacity-50 disabled:cursor-not-allowed
             transition-colors"
    >
      {isActive ? 'Disconnect' : 'Connect'}
    </button>
    
    <button
      onclick={() => onEdit?.(id)}
      class="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 transition-colors"
      title="Edit"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
      </svg>
    </button>
    
    <button
      onclick={() => onDelete?.(id)}
      class="p-1.5 rounded-lg text-gray-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
      title="Delete"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
    </button>
  </div>
</div>
