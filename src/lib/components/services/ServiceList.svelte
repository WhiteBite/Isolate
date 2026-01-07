<script lang="ts">
  import { ScanningIndicator } from '$lib/components';
  import { getServiceIconSvg } from '$lib/utils/icons';
  import { ServiceHealthChart } from '$lib/components/services';

  interface ServiceWithStatus {
    id: string;
    name: string;
    status: 'working' | 'blocked' | 'unknown' | 'checking' | 'error';
    ping?: number;
    category: string;
    isCustom?: boolean;
    error?: string | null;
  }

  type StatusFilter = 'all' | 'working' | 'blocked' | 'unknown';

  interface Props {
    services: ServiceWithStatus[];
    totalCount: number;
    selectedId: string | null;
    scanning: boolean;
    searchQuery: string;
    statusFilter: StatusFilter;
    onsearchchange: (query: string) => void;
    onfilterchange: (filter: StatusFilter) => void;
    onselect: (id: string) => void;
    oncontextmenu: (event: MouseEvent, service: ServiceWithStatus) => void;
    oncheckall: () => void;
    onadd: () => void;
  }

  let { 
    services, 
    totalCount,
    selectedId, 
    scanning, 
    searchQuery,
    statusFilter,
    onsearchchange,
    onfilterchange,
    onselect, 
    oncontextmenu, 
    oncheckall, 
    onadd 
  }: Props = $props();

  function getIcon(id: string) {
    return getServiceIconSvg(id);
  }

  function getStatusColor(status: string): string {
    if (status === 'working') return 'bg-emerald-500';
    if (status === 'blocked') return 'bg-red-500';
    if (status === 'error') return 'bg-rose-600';
    if (status === 'checking') return 'bg-indigo-500';
    return 'bg-amber-500';
  }

  function getPingColor(ping?: number): string {
    if (!ping) return 'text-zinc-400';
    if (ping < 50) return 'text-emerald-400';
    if (ping < 150) return 'text-amber-400';
    return 'text-red-400';
  }

  let workingCount = $derived(services.filter(s => s.status === 'working').length);
</script>

<div class="w-[320px] flex-shrink-0 border-r border-white/5 flex flex-col bg-zinc-900/30">
  <!-- Header -->
  <div class="p-4 border-b border-white/5">
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-lg font-semibold text-zinc-100">Services</h2>
      <button
        onclick={onadd}
        class="p-2 rounded-lg bg-zinc-800/60 border border-white/5 
               hover:bg-zinc-700/60 hover:border-white/10 transition-colors"
        title="Add custom service"
      >
        <svg class="w-4 h-4 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 5v14M5 12h14"/>
        </svg>
      </button>
    </div>
    
    <!-- Search Input -->
    <div class="relative mb-3">
      <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        type="text"
        placeholder="Search services..."
        value={searchQuery}
        oninput={(e) => onsearchchange(e.currentTarget.value)}
        class="w-full pl-10 pr-4 py-2 bg-zinc-800/60 border border-white/5 rounded-xl
               text-sm text-zinc-100 placeholder-zinc-500
               focus:outline-none focus:border-indigo-500/30 focus:ring-1 focus:ring-indigo-500/20
               transition-all duration-200"
      />
      {#if searchQuery}
        <button
          onclick={() => onsearchchange('')}
          class="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 rounded-full
                 text-zinc-400 hover:text-zinc-300 hover:bg-zinc-700/50 transition-colors"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      {/if}
    </div>
    
    <!-- Status Filter -->
    <div class="flex gap-1 mb-3">
      <button
        onclick={() => onfilterchange('all')}
        class="flex-1 px-2 py-1.5 rounded-lg text-xs font-medium transition-all duration-200
               {statusFilter === 'all' 
                 ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' 
                 : 'bg-zinc-800/40 text-zinc-400 border border-white/5 hover:bg-zinc-800/60'}"
      >
        All
      </button>
      <button
        onclick={() => onfilterchange('working')}
        class="flex-1 px-2 py-1.5 rounded-lg text-xs font-medium transition-all duration-200
               {statusFilter === 'working' 
                 ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' 
                 : 'bg-zinc-800/40 text-zinc-400 border border-white/5 hover:bg-zinc-800/60'}"
      >
        Working
      </button>
      <button
        onclick={() => onfilterchange('blocked')}
        class="flex-1 px-2 py-1.5 rounded-lg text-xs font-medium transition-all duration-200
               {statusFilter === 'blocked' 
                 ? 'bg-red-500/20 text-red-400 border border-red-500/30' 
                 : 'bg-zinc-800/40 text-zinc-400 border border-white/5 hover:bg-zinc-800/60'}"
      >
        Blocked
      </button>
      <button
        onclick={() => onfilterchange('unknown')}
        class="flex-1 px-2 py-1.5 rounded-lg text-xs font-medium transition-all duration-200
               {statusFilter === 'unknown' 
                 ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' 
                 : 'bg-zinc-800/40 text-zinc-400 border border-white/5 hover:bg-zinc-800/60'}"
      >
        Unknown
      </button>
    </div>
    
    <button
      onclick={oncheckall}
      disabled={scanning}
      class="w-full flex items-center justify-center gap-2 px-4 py-2.5
             bg-indigo-500/10 border border-indigo-500/20 rounded-xl
             text-indigo-400 text-sm font-medium
             hover:bg-indigo-500/20 hover:border-indigo-500/30
             disabled:opacity-50 disabled:cursor-not-allowed
             transition-all duration-200"
    >
      {#if scanning}
        <ScanningIndicator active={true} text="Checking..." variant="dots" />
      {:else}
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
          <path d="M3 3v5h5"/>
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
          <path d="M16 16h5v5"/>
        </svg>
        Check All Services
      {/if}
    </button>
  </div>

  <!-- Service List -->
  <div class="flex-1 overflow-y-auto p-2 space-y-1">
    {#if services.length === 0}
      <div class="flex flex-col items-center justify-center py-12 text-center">
        <svg class="w-12 h-12 text-zinc-600 mb-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <p class="text-sm text-zinc-400">No services found</p>
        <p class="text-xs text-zinc-600 mt-1">Try adjusting your search or filter</p>
      </div>
    {:else}
      {#each services as service (service.id)}
      {@const icon = getIcon(service.id)}
      <button
        onclick={() => onselect(service.id)}
        oncontextmenu={(e) => oncontextmenu(e, service)}
        class="group w-full flex items-center gap-3 p-3 rounded-xl transition-all duration-200
               {selectedId === service.id 
                 ? 'bg-white/5 border border-indigo-500/30 shadow-lg shadow-indigo-500/5' 
                 : 'bg-zinc-900/40 border border-white/5 hover:bg-zinc-900/60 hover:border-white/10'}"
      >
        <!-- Icon -->
        <div class="w-10 h-10 flex-shrink-0 flex items-center justify-center rounded-lg bg-zinc-800/60 group-hover:bg-zinc-800 transition-colors">
          <svg class="w-5 h-5 {icon.color}" viewBox="0 0 24 24" fill="currentColor">
            <path d={icon.path}/>
          </svg>
        </div>
        
        <!-- Info -->
        <div class="flex-1 text-left min-w-0">
          <div class="text-sm font-medium text-zinc-100 truncate">{service.name}</div>
          <div class="text-xs text-zinc-400 capitalize">{service.category}</div>
        </div>

        <!-- Health Mini Chart -->
        <div class="flex-shrink-0 hidden sm:block">
          <ServiceHealthChart serviceId={service.id} hours={24} compact={true} />
        </div>

        <!-- Status -->
        <div class="flex items-center gap-2 flex-shrink-0">
          {#if service.status === 'checking'}
            <svg class="w-4 h-4 animate-spin text-indigo-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
          {:else}
            {#if service.ping}
              <span class="text-xs font-mono {getPingColor(service.ping)}">{service.ping}ms</span>
            {/if}
            {#if service.status === 'error'}
              <div class="relative group">
                <svg class="w-4 h-4 text-rose-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="12" y1="8" x2="12" y2="12"/>
                  <line x1="12" y1="16" x2="12.01" y2="16"/>
                </svg>
                {#if service.error}
                  <div class="absolute right-0 bottom-full mb-2 px-2 py-1 bg-zinc-800 border border-white/10 rounded-lg text-xs text-zinc-300 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity z-10 pointer-events-none">
                    {service.error}
                  </div>
                {/if}
              </div>
            {:else}
              <div class="w-2.5 h-2.5 rounded-full {getStatusColor(service.status)} 
                          {service.status === 'working' ? 'animate-pulse' : ''}"></div>
            {/if}
          {/if}
        </div>
      </button>
    {/each}
    {/if}
  </div>

  <!-- Footer -->
  <div class="p-3 border-t border-white/5">
    <div class="text-xs text-zinc-400 text-center">
      {#if services.length === totalCount}
        {workingCount} / {totalCount} services available
      {:else}
        Showing {services.length} of {totalCount} services ({workingCount} available)
      {/if}
    </div>
  </div>
</div>
