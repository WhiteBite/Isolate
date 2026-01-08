<script lang="ts">
  import { browser } from '$app/environment';
  import { waitForBackend } from '$lib/utils/backend';
  import { toasts } from '$lib/stores/toast';
  import { ProxyCardGrid, ImportZone, ChainBuilder } from '$lib/components/proxy';
  import type { ProxyConfig } from '$lib/api';
  import { mockGateways } from '$lib/mocks';
  
  // State
  let proxies = $state<ProxyConfig[]>([]);
  let loading = $state(true);
  let isDemoMode = $state(false);
  let showImport = $state(false);
  let proxyChain = $state<string[]>([]);
  
  // Filters
  let filterCountry = $state<string | null>(null);
  let filterProtocol = $state<string | null>(null);
  let sortBy = $state<'name' | 'latency' | 'country'>('name');
  
  // Derived
  let activeProxy = $derived(proxies.find(p => p.active));
  let countries = $derived([...new Set(proxies.map(p => p.country).filter(Boolean))]);
  let protocols = $derived([...new Set(proxies.map(p => p.protocol))]);
  
  // Initialize
  $effect(() => {
    if (browser) {
      loadProxies();
    }
  });
  
  async function loadProxies() {
    loading = true;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    isDemoMode = !isTauri;
    
    if (!isTauri) {
      // Demo mode
      await new Promise(r => setTimeout(r, 500));
      proxies = mockGateways;
      loading = false;
      return;
    }
    
    try {
      const ready = await waitForBackend(30, 300);
      if (!ready) {
        proxies = mockGateways;
        loading = false;
        return;
      }
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<ProxyConfig[]>('get_proxies').catch(() => []);
      proxies = result.length > 0 ? result : mockGateways;
    } catch (e) {
      console.error('Failed to load proxies:', e);
      proxies = mockGateways;
    } finally {
      loading = false;
    }
  }
  
  async function handleTest(id: string) {
    const proxy = proxies.find(p => p.id === id);
    if (!proxy) return;
    
    toasts.info(`Testing ${proxy.name}...`);
    
    if (isDemoMode) {
      await new Promise(r => setTimeout(r, 1000));
      proxies = proxies.map(p => 
        p.id === id ? { ...p, ping: Math.floor(Math.random() * 200) + 50 } : p
      );
      toasts.success(`${proxy.name}: ${proxies.find(p => p.id === id)?.ping}ms`);
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<{ success: boolean; latency?: number; error?: string }>('test_proxy', { id });
      if (result.success && result.latency) {
        proxies = proxies.map(p => p.id === id ? { ...p, ping: result.latency } : p);
        toasts.success(`${proxy.name}: ${result.latency}ms`);
      } else {
        toasts.error(`${proxy.name}: ${result.error || 'Failed'}`);
      }
    } catch (e) {
      toasts.error(`Test failed: ${e}`);
    }
  }
  
  async function handleConnect(id: string) {
    const proxy = proxies.find(p => p.id === id);
    if (!proxy) return;
    
    if (isDemoMode) {
      proxies = proxies.map(p => ({ ...p, active: p.id === id }));
      toasts.success(`Connected to ${proxy.name}`);
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Deactivate current
      const current = proxies.find(p => p.active);
      if (current && current.id !== id) {
        await invoke('deactivate_proxy', { id: current.id });
      }
      
      await invoke('apply_proxy', { id });
      proxies = proxies.map(p => ({ ...p, active: p.id === id }));
      toasts.success(`Connected to ${proxy.name}`);
    } catch (e) {
      toasts.error(`Connection failed: ${e}`);
    }
  }
  
  async function handleEdit(id: string) {
    toasts.info('Edit functionality coming soon');
  }
  
  async function handleDelete(id: string) {
    const proxy = proxies.find(p => p.id === id);
    if (!proxy) return;
    
    if (isDemoMode) {
      proxies = proxies.filter(p => p.id !== id);
      toasts.success(`Deleted ${proxy.name}`);
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_proxy', { id });
      proxies = proxies.filter(p => p.id !== id);
      toasts.success(`Deleted ${proxy.name}`);
    } catch (e) {
      toasts.error(`Delete failed: ${e}`);
    }
  }
  
  function handleImport(parsed: any) {
    const newProxy: ProxyConfig = {
      id: `proxy-${Date.now()}`,
      name: parsed.name || `${parsed.protocol.toUpperCase()} Proxy`,
      protocol: parsed.protocol,
      server: parsed.server,
      port: parsed.port,
      tls: parsed.tls ?? false,
      active: false,
      custom_fields: {}
    };
    
    proxies = [...proxies, newProxy];
    showImport = false;
    toasts.success(`Imported ${newProxy.name}`);
  }
  
  function handleChainChange(chain: string[]) {
    proxyChain = chain;
  }
  
  // Map proxies to card format
  let cardProxies = $derived(proxies.map(p => ({
    id: p.id,
    name: p.name,
    countryCode: p.country || 'XX',
    protocol: p.protocol as any,
    server: p.server,
    port: p.port,
    latency: p.ping ?? null,
    status: p.active ? 'active' as const : 'inactive' as const
  })));
</script>

<div class="h-full flex flex-col bg-[#09090b]">
  <!-- Header -->
  <header class="sticky top-0 z-10 flex items-center justify-between px-6 py-4 bg-[#09090b]/80 backdrop-blur-xl border-b border-white/5">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-3">
        <h1 class="text-xl font-semibold text-white">Proxy & VPN</h1>
        {#if isDemoMode}
          <span class="px-2 py-0.5 text-[10px] uppercase tracking-wider bg-amber-500/20 text-amber-400 rounded font-medium border border-amber-500/30">Demo</span>
        {/if}
      </div>
      
      <!-- Active proxy indicator -->
      {#if activeProxy}
        <div class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
          <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
          <span class="text-sm text-emerald-400">{activeProxy.name}</span>
        </div>
      {/if}
    </div>
    
    <div class="flex items-center gap-3">
      <!-- Filters -->
      <select
        bind:value={filterCountry}
        class="px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-sm text-white 
               focus:outline-none focus:border-indigo-500/50"
      >
        <option value={null}>All Countries</option>
        {#each countries as country}
          <option value={country}>{country}</option>
        {/each}
      </select>
      
      <select
        bind:value={sortBy}
        class="px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-sm text-white 
               focus:outline-none focus:border-indigo-500/50"
      >
        <option value="name">Sort by Name</option>
        <option value="latency">Sort by Latency</option>
        <option value="country">Sort by Country</option>
      </select>
      
      <!-- Import button -->
      <button
        onclick={() => showImport = !showImport}
        class="flex items-center gap-2 px-4 py-2 rounded-lg bg-indigo-500 hover:bg-indigo-600 
               text-white text-sm font-medium transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Import
      </button>
    </div>
  </header>
  
  <!-- Content -->
  <div class="flex-1 overflow-auto p-6 space-y-6">
    <!-- Import Zone (collapsible) -->
    {#if showImport}
      <div class="max-w-xl">
        <ImportZone 
          onImport={handleImport}
          onCancel={() => showImport = false}
        />
      </div>
    {/if}
    
    <!-- Chain Builder -->
    <div class="max-w-3xl">
      <ChainBuilder 
        {proxies}
        chain={proxyChain}
        onChainChange={handleChainChange}
      />
    </div>
    
    <!-- Proxy Grid -->
    <div>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-medium text-white">
          Your Proxies
          <span class="ml-2 text-sm text-zinc-400">({proxies.length})</span>
        </h2>
        
        <button
          onclick={loadProxies}
          disabled={loading}
          class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 
                 text-sm text-zinc-400 hover:text-white transition-colors disabled:opacity-50"
        >
          <svg class="w-4 h-4 {loading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Refresh
        </button>
      </div>
      
      {#if loading}
        <div class="flex items-center justify-center py-16">
          <div class="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else}
        <ProxyCardGrid
          proxies={cardProxies}
          filterCountry={filterCountry}
          filterProtocol={filterProtocol as any}
          {sortBy}
          onTest={handleTest}
          onConnect={handleConnect}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      {/if}
    </div>
  </div>
</div>
