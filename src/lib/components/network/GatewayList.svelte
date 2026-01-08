<script lang="ts">
  import type { ProxyConfig } from '$lib/api';
  import GatewayCard from './GatewayCard.svelte';
  import { getCountryFlag, getCountryName, detectCountryFromServer } from '$lib/utils/countries';
  import { browser } from '$app/environment';
  
  interface Props {
    gateways: ProxyConfig[];
    selectedId: string | null;
    loading: boolean;
    testingAll?: boolean;
    testProgress?: { current: number; total: number } | null;
    importing?: boolean;
    onselect: (id: string) => void;
    onadd: () => void;
    ontest: (id: string) => void;
    ondelete: (id: string) => void;
    onedit: (id: string) => void;
    onactivate?: (id: string) => void;
    ondeactivate?: (id: string) => void;
    onshare?: (id: string) => void;
    ontestall?: () => void;
    onimport?: () => void;
  }

  let {
    gateways,
    selectedId,
    loading,
    testingAll = false,
    testProgress = null,
    importing = false,
    onselect,
    onadd,
    ontest,
    ondelete,
    onedit,
    onactivate,
    ondeactivate,
    onshare,
    ontestall,
    onimport,
  }: Props = $props();

  // Group by country toggle state
  const STORAGE_KEY = 'gateway-group-by-country';
  const COLLAPSED_KEY = 'gateway-collapsed-groups';
  const SORT_STORAGE_KEY = 'gateway-sort';
  
  let groupByCountry = $state(
    browser ? localStorage.getItem(STORAGE_KEY) === 'true' : false
  );
  
  let collapsedGroups = $state<Set<string>>(
    browser 
      ? new Set(JSON.parse(localStorage.getItem(COLLAPSED_KEY) || '[]'))
      : new Set()
  );

  // Sorting state
  type SortField = 'name' | 'ping-asc' | 'ping-desc' | 'protocol' | 'country';
  
  let sortField = $state<SortField>(
    browser 
      ? (localStorage.getItem(SORT_STORAGE_KEY) as SortField) || 'name'
      : 'name'
  );
  let showSortDropdown = $state(false);

  // Sort options
  const sortOptions: { value: SortField; label: string }[] = [
    { value: 'name', label: 'Name' },
    { value: 'ping-asc', label: 'Ping (fastest)' },
    { value: 'ping-desc', label: 'Ping (slowest)' },
    { value: 'protocol', label: 'Protocol' },
    { value: 'country', label: 'Country' },
  ];

  function getSortLabel(field: SortField): string {
    return sortOptions.find(o => o.value === field)?.label || 'Name';
  }

  function getSortIcon(field: SortField): string {
    if (field === 'ping-asc') return '↑';
    if (field === 'ping-desc') return '↓';
    return '';
  }

  function setSortField(field: SortField) {
    sortField = field;
    showSortDropdown = false;
    if (browser) {
      localStorage.setItem(SORT_STORAGE_KEY, field);
    }
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.sort-dropdown')) {
      showSortDropdown = false;
    }
  }

  // Save groupByCountry to localStorage
  $effect(() => {
    if (browser) {
      localStorage.setItem(STORAGE_KEY, String(groupByCountry));
    }
  });

  // Save collapsed groups to localStorage
  $effect(() => {
    if (browser) {
      localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...collapsedGroups]));
    }
  });

  // Sort function for proxies
  function sortProxies(proxies: ProxyConfig[]): ProxyConfig[] {
    const sorted = [...proxies];
    switch (sortField) {
      case 'name':
        sorted.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'ping-asc':
        sorted.sort((a, b) => {
          if (a.ping == null && b.ping == null) return 0;
          if (a.ping == null) return 1;
          if (b.ping == null) return -1;
          return a.ping - b.ping;
        });
        break;
      case 'ping-desc':
        sorted.sort((a, b) => {
          if (a.ping == null && b.ping == null) return 0;
          if (a.ping == null) return 1;
          if (b.ping == null) return -1;
          return b.ping - a.ping;
        });
        break;
      case 'protocol':
        sorted.sort((a, b) => a.protocol.localeCompare(b.protocol));
        break;
      case 'country':
        sorted.sort((a, b) => {
          const countryA = a.country || '';
          const countryB = b.country || '';
          if (!countryA && !countryB) return 0;
          if (!countryA) return 1;
          if (!countryB) return -1;
          return countryA.localeCompare(countryB);
        });
        break;
    }
    return sorted;
  }

  // Sorted gateways (for flat view)
  let sortedGateways = $derived(sortProxies(gateways));

  // Group gateways by country
  let groupedGateways = $derived.by(() => {
    if (!groupByCountry) return null;
    
    const groups = new Map<string, ProxyConfig[]>();
    
    for (const gateway of gateways) {
      // Use explicit country or detect from server
      let country = gateway.country?.toUpperCase();
      if (!country) {
        country = detectCountryFromServer(gateway.server)?.toUpperCase() || 'UNKNOWN';
      }
      
      if (!groups.has(country)) {
        groups.set(country, []);
      }
      groups.get(country)!.push(gateway);
    }
    
    // Sort groups: known countries first (alphabetically), then Unknown
    const sortedEntries = [...groups.entries()].sort((a, b) => {
      if (a[0] === 'UNKNOWN') return 1;
      if (b[0] === 'UNKNOWN') return -1;
      const nameA = getCountryName(a[0]);
      const nameB = getCountryName(b[0]);
      return nameA.localeCompare(nameB);
    });
    
    return new Map(sortedEntries);
  });

  // Sort proxies within groups
  let sortedGroupedGateways = $derived.by(() => {
    if (!groupedGateways) return null;
    const result = new Map<string, ProxyConfig[]>();
    for (const [country, proxies] of groupedGateways) {
      result.set(country, sortProxies(proxies));
    }
    return result;
  });

  function toggleGroup(country: string) {
    const newCollapsed = new Set(collapsedGroups);
    if (newCollapsed.has(country)) {
      newCollapsed.delete(country);
    } else {
      newCollapsed.add(country);
    }
    collapsedGroups = newCollapsed;
  }

  function toggleGroupByCountry() {
    groupByCountry = !groupByCountry;
  }
</script>

<svelte:window onclick={handleClickOutside} />

{#snippet skeletonLoader()}
  <div class="space-y-1 p-2">
    {#each Array(4) as _}
      <div class="animate-pulse flex items-center gap-3 px-3 py-2.5 rounded-lg">
        <div class="w-6 h-6 bg-zinc-800 rounded"></div>
        <div class="flex-1 space-y-1.5">
          <div class="h-3.5 bg-zinc-800 rounded w-2/3"></div>
          <div class="h-2.5 bg-zinc-800 rounded w-1/2"></div>
        </div>
        <div class="w-12 h-4 bg-zinc-800 rounded"></div>
      </div>
    {/each}
  </div>
{/snippet}

{#snippet emptyState()}
  <div class="flex flex-col items-center justify-center p-6 h-full">
    <div class="w-full max-w-sm bg-gradient-to-br from-zinc-800/50 to-zinc-900/50 rounded-2xl border border-white/5 p-6 text-center">
      <!-- Icon -->
      <div class="w-16 h-16 mx-auto rounded-2xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 flex items-center justify-center mb-4 ring-1 ring-white/10">
        <svg class="w-8 h-8 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
            d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
        </svg>
      </div>
      
      <!-- Title -->
      <h3 class="text-lg font-semibold text-white mb-2">
        Добавьте первый прокси-сервер
      </h3>
      
      <!-- Description -->
      <p class="text-sm text-zinc-400 mb-6 leading-relaxed">
        Gateway — это прокси-сервер для обхода блокировок. 
        Поддерживаются протоколы VLESS, Shadowsocks, Trojan и другие. 
        Трафик шифруется и направляется через удалённый сервер.
      </p>
      
      <!-- Actions -->
      <div class="flex flex-col gap-2">
        <button
          onclick={onadd}
          class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-indigo-500 hover:bg-indigo-400 text-white text-sm font-medium rounded-xl transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Добавить прокси
        </button>
        
        {#if onimport}
          <button
            onclick={onimport}
            disabled={importing}
            class="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white text-sm font-medium rounded-xl transition-colors border border-white/5 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {#if importing}
              <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Импорт...
            {:else}
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
                <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
              </svg>
              Импортировать из буфера
            {/if}
          </button>
        {/if}
      </div>
      
      <!-- Hint -->
      <p class="text-xs text-zinc-400 mt-4">
        Скопируйте ссылку vless:// или ss:// и нажмите «Импортировать»
      </p>
    </div>
  </div>
{/snippet}

{#snippet countryGroup(country: string, proxies: ProxyConfig[])}
  {@const isCollapsed = collapsedGroups.has(country)}
  {@const flag = country === 'UNKNOWN' ? '🌐' : getCountryFlag(country)}
  {@const name = country === 'UNKNOWN' ? 'Unknown' : getCountryName(country)}
  
  <div class="mb-1">
    <button
      class="w-full flex items-center gap-2 px-3 py-2 hover:bg-white/5 rounded-lg transition-colors"
      onclick={() => toggleGroup(country)}
    >
      <svg 
        class="w-4 h-4 text-zinc-400 transition-transform duration-200 {isCollapsed ? '' : 'rotate-90'}" 
        fill="none" 
        stroke="currentColor" 
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
      </svg>
      <span class="text-base">{flag}</span>
      <span class="text-sm font-medium text-zinc-300">{name}</span>
      <span class="text-xs text-zinc-400 bg-zinc-800 px-1.5 py-0.5 rounded ml-auto">
        {proxies.length}
      </span>
    </button>
    
    {#if !isCollapsed}
      <div class="pl-4 space-y-0.5 overflow-hidden transition-all duration-200">
        {#each proxies as gateway (gateway.id)}
          <GatewayCard 
            {gateway}
            selected={selectedId === gateway.id}
            onclick={() => onselect(gateway.id)}
            ontest={() => ontest(gateway.id)}
            ondelete={() => ondelete(gateway.id)}
            onedit={() => onedit(gateway.id)}
            onactivate={onactivate ? () => onactivate(gateway.id) : undefined}
            ondeactivate={ondeactivate ? () => ondeactivate(gateway.id) : undefined}
            onshare={onshare ? () => onshare(gateway.id) : undefined}
          />
        {/each}
      </div>
    {/if}
  </div>
{/snippet}

<div class="flex flex-col h-full bg-zinc-900/30 rounded-xl border border-white/5">
  <div class="flex items-center justify-between px-4 py-3 border-b border-white/5">
    <div class="flex items-center gap-2">
      <h2 class="text-sm font-semibold text-white">Gateways</h2>
      {#if gateways.length > 0}
        <span class="text-xs text-zinc-400 bg-zinc-800 px-1.5 py-0.5 rounded">
          {gateways.length}
        </span>
      {/if}
    </div>
    <div class="flex items-center gap-1">
      <!-- Sort Dropdown -->
      {#if gateways.length > 1}
        <div class="relative sort-dropdown">
          <button
            type="button"
            onclick={(e) => { e.stopPropagation(); showSortDropdown = !showSortDropdown; }}
            class="flex items-center gap-1 px-2 py-1.5 text-xs text-zinc-400 hover:text-white hover:bg-white/5 rounded-lg transition-colors"
            title="Sort gateways"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4h13M3 8h9m-9 4h6m4 0l4-4m0 0l4 4m-4-4v12" />
            </svg>
            <span class="hidden sm:inline">{getSortLabel(sortField)}</span>
            {#if getSortIcon(sortField)}
              <span class="text-[10px]">{getSortIcon(sortField)}</span>
            {/if}
          </button>
          {#if showSortDropdown}
            <div class="absolute right-0 top-full mt-1 w-40 bg-zinc-900 border border-white/10 rounded-lg shadow-xl z-50 py-1 overflow-hidden">
              {#each sortOptions as option}
                <button
                  type="button"
                  onclick={(e) => { e.stopPropagation(); setSortField(option.value); }}
                  class="w-full flex items-center justify-between px-3 py-2 text-xs text-left transition-colors {sortField === option.value ? 'bg-white/10 text-white' : 'text-zinc-400 hover:bg-white/5 hover:text-white'}"
                >
                  <span>{option.label}</span>
                  {#if sortField === option.value}
                    <svg class="w-3.5 h-3.5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
      
      <!-- Group by Country Toggle -->
      {#if gateways.length > 0}
        <button 
          onclick={toggleGroupByCountry}
          class="flex items-center gap-1.5 px-2 py-1.5 text-xs font-medium rounded-lg transition-colors
            {groupByCountry 
              ? 'bg-indigo-500/20 text-indigo-400' 
              : 'hover:bg-white/5 text-zinc-400 hover:text-white'}"
          title={groupByCountry ? 'Show flat list' : 'Group by country'}
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
        </button>
      {/if}
      
      {#if ontestall && gateways.length > 0}
        <button 
          onclick={ontestall}
          disabled={testingAll || loading}
          class="flex items-center gap-1.5 px-2 py-1.5 text-xs font-medium rounded-lg transition-colors
            {testingAll 
              ? 'bg-blue-500/20 text-blue-400 cursor-not-allowed' 
              : 'hover:bg-white/5 text-zinc-400 hover:text-white'}"
          title={testingAll ? 'Testing...' : 'Test all'}
        >
          {#if testingAll}
            <svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {#if testProgress}
              <span>{testProgress.current}/{testProgress.total}</span>
            {/if}
          {:else}
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          {/if}
        </button>
      {/if}
      
      {#if onimport}
        <button 
          onclick={onimport}
          disabled={importing || loading}
          class="flex items-center gap-1.5 px-2 py-1.5 text-xs font-medium rounded-lg transition-colors
            {importing 
              ? 'bg-emerald-500/20 text-emerald-400 cursor-not-allowed' 
              : 'hover:bg-white/5 text-zinc-400 hover:text-white'}"
          title={importing ? 'Importing...' : 'Import from clipboard'}
        >
          {#if importing}
            <svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          {:else}
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
              <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
            </svg>
          {/if}
        </button>
      {/if}
      
      <button 
        onclick={onadd} 
        class="p-1.5 hover:bg-white/5 rounded-lg transition-colors text-zinc-400 hover:text-white"
        title="Add gateway"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>
  </div>
  
  <div class="flex-1 overflow-y-auto">
    {#if loading}
      {@render skeletonLoader()}
    {:else if gateways.length === 0}
      {@render emptyState()}
    {:else if groupByCountry && sortedGroupedGateways}
      <div class="p-2">
        {#each sortedGroupedGateways as [country, proxies] (country)}
          {@render countryGroup(country, proxies)}
        {/each}
      </div>
    {:else}
      <div class="p-2 space-y-0.5">
        {#each sortedGateways as gateway (gateway.id)}
          <GatewayCard 
            {gateway}
            selected={selectedId === gateway.id}
            onclick={() => onselect(gateway.id)}
            ontest={() => ontest(gateway.id)}
            ondelete={() => ondelete(gateway.id)}
            onedit={() => onedit(gateway.id)}
            onactivate={onactivate ? () => onactivate(gateway.id) : undefined}
            ondeactivate={ondeactivate ? () => ondeactivate(gateway.id) : undefined}
            onshare={onshare ? () => onshare(gateway.id) : undefined}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>
