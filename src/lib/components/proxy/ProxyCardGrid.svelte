<script lang="ts">
  import ProxyCountryCard from './ProxyCountryCard.svelte';
  
  type Protocol = 'vless' | 'vmess' | 'shadowsocks' | 'trojan' | 'ss' | 'http' | 'socks5';
  type Status = 'active' | 'inactive' | 'testing' | 'error';
  type SortBy = 'name' | 'latency' | 'country';
  
  interface Proxy {
    id: string;
    name: string;
    countryCode: string;
    protocol: Protocol;
    server: string;
    port: number;
    latency?: number | null;
    status?: Status;
  }
  
  interface Props {
    proxies: Proxy[];
    filterCountry?: string | null;
    filterProtocol?: Protocol | null;
    sortBy?: SortBy;
    onTest?: (id: string) => void;
    onConnect?: (id: string) => void;
    onEdit?: (id: string) => void;
    onDelete?: (id: string) => void;
  }
  
  let { 
    proxies,
    filterCountry = null,
    filterProtocol = null,
    sortBy = 'name',
    onTest,
    onConnect,
    onEdit,
    onDelete
  }: Props = $props();
  
  // Filter proxies
  let filteredProxies = $derived.by(() => {
    let result = [...proxies];
    
    if (filterCountry) {
      result = result.filter(p => p.countryCode.toUpperCase() === filterCountry.toUpperCase());
    }
    
    if (filterProtocol) {
      result = result.filter(p => p.protocol === filterProtocol);
    }
    
    return result;
  });
  
  // Sort proxies
  let sortedProxies = $derived.by(() => {
    const sorted = [...filteredProxies];
    
    switch (sortBy) {
      case 'latency':
        return sorted.sort((a, b) => {
          // Null/undefined latencies go to the end
          if ((a.latency === null || a.latency === undefined) && (b.latency === null || b.latency === undefined)) return 0;
          if (a.latency === null || a.latency === undefined) return 1;
          if (b.latency === null || b.latency === undefined) return -1;
          return a.latency - b.latency;
        });
      case 'country':
        return sorted.sort((a, b) => a.countryCode.localeCompare(b.countryCode));
      case 'name':
      default:
        return sorted.sort((a, b) => a.name.localeCompare(b.name));
    }
  });
  
  let isEmpty = $derived(sortedProxies.length === 0);
</script>

{#if isEmpty}
  <div class="flex flex-col items-center justify-center py-16 text-center">
    <div class="w-16 h-16 mb-4 rounded-full bg-white/5 flex items-center justify-center">
      <svg class="w-8 h-8 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
              d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
      </svg>
    </div>
    <h3 class="text-lg font-medium text-white mb-2">No proxies found</h3>
    <p class="text-sm text-gray-400 max-w-sm">
      {#if filterCountry || filterProtocol}
        No proxies match your current filters. Try adjusting your filters or add a new proxy.
      {:else}
        You haven't added any proxies yet. Import a proxy link or add one manually to get started.
      {/if}
    </p>
  </div>
{:else}
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
    {#each sortedProxies as proxy (proxy.id)}
      <ProxyCountryCard
        id={proxy.id}
        name={proxy.name}
        countryCode={proxy.countryCode}
        protocol={proxy.protocol}
        server={proxy.server}
        port={proxy.port}
        latency={proxy.latency}
        status={proxy.status}
        {onTest}
        {onConnect}
        {onEdit}
        {onDelete}
      />
    {/each}
  </div>
{/if}
