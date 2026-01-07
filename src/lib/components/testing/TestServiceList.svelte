<script lang="ts">
  import type { ProxyConfig, Strategy, Service } from '$lib/api';
  import { getProxyFlag, getProxyCountryName } from '$lib/utils/countries';

  // Props
  let {
    proxies = [],
    strategies = [],
    services = [],
    selectedProxies = $bindable(new Set<string>()),
    selectedStrategies = $bindable(new Set<string>()),
    selectedServices = $bindable(new Set<string>()),
    disabled = false
  }: {
    proxies: ProxyConfig[];
    strategies: Strategy[];
    services: Service[];
    selectedProxies: Set<string>;
    selectedStrategies: Set<string>;
    selectedServices: Set<string>;
    disabled?: boolean;
  } = $props();

  // Toggle functions
  function toggleProxy(id: string) {
    const newSet = new Set(selectedProxies);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedProxies = newSet;
  }

  function toggleStrategy(id: string) {
    const newSet = new Set(selectedStrategies);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedStrategies = newSet;
  }

  function toggleService(id: string) {
    const newSet = new Set(selectedServices);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedServices = newSet;
  }

  // Select all / clear all
  function selectAllProxies() {
    selectedProxies = new Set(proxies.map(p => p.id));
  }

  function clearAllProxies() {
    selectedProxies = new Set();
  }

  function selectAllStrategies() {
    selectedStrategies = new Set(strategies.map(s => s.id));
  }

  function clearAllStrategies() {
    selectedStrategies = new Set();
  }

  function selectAllServices() {
    selectedServices = new Set(services.map(s => s.id));
  }

  function clearAllServices() {
    selectedServices = new Set();
  }
</script>

<div class="space-y-6">
  <!-- Proxies Selection -->
  <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold text-white">Proxies</h2>
      {#if proxies.length > 0}
        <div class="flex gap-2">
          <button
            onclick={selectAllProxies}
            {disabled}
            class="text-sm text-[#00d4ff] hover:text-[#00b8e6] transition-colors disabled:opacity-50"
          >
            Select all
          </button>
          <span class="text-[#2a2f4a]">|</span>
          <button
            onclick={clearAllProxies}
            {disabled}
            class="text-sm text-[#a0a0a0] hover:text-white transition-colors disabled:opacity-50"
          >
            Clear all
          </button>
        </div>
      {/if}
    </div>
    
    {#if proxies.length === 0}
      <p class="text-[#a0a0a0] text-sm">No proxies available</p>
    {:else}
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {#each proxies as proxy}
          <label class="flex items-center gap-3 p-3 bg-[#0a0e27] rounded-lg cursor-pointer hover:bg-[#2a2f4a]/50 transition-colors {selectedProxies.has(proxy.id) ? 'ring-1 ring-[#00d4ff]/50' : ''} {disabled ? 'opacity-50 cursor-not-allowed' : ''}">
            <input
              type="checkbox"
              checked={selectedProxies.has(proxy.id)}
              onchange={() => toggleProxy(proxy.id)}
              {disabled}
              class="w-4 h-4 rounded bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff] focus:ring-offset-[#1a1f3a]"
            />
            <span class="text-lg flex-shrink-0" title={getProxyCountryName(proxy.country, proxy.server)}>{getProxyFlag(proxy.country, proxy.server)}</span>
            <div class="flex-1 min-w-0">
              <p class="text-white text-sm truncate">{proxy.name}</p>
              <p class="text-[#a0a0a0] text-xs">{proxy.protocol} • {proxy.server}:{proxy.port}</p>
            </div>
          </label>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Strategies Selection -->
  <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold text-white">Strategies</h2>
      {#if strategies.length > 0}
        <div class="flex gap-2">
          <button
            onclick={selectAllStrategies}
            {disabled}
            class="text-sm text-[#00d4ff] hover:text-[#00b8e6] transition-colors disabled:opacity-50"
          >
            Select all
          </button>
          <span class="text-[#2a2f4a]">|</span>
          <button
            onclick={clearAllStrategies}
            {disabled}
            class="text-sm text-[#a0a0a0] hover:text-white transition-colors disabled:opacity-50"
          >
            Clear all
          </button>
        </div>
      {/if}
    </div>
    
    {#if strategies.length === 0}
      <p class="text-[#a0a0a0] text-sm">No strategies available</p>
    {:else}
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {#each strategies as strategy}
          <label class="flex items-center gap-3 p-3 bg-[#0a0e27] rounded-lg cursor-pointer hover:bg-[#2a2f4a]/50 transition-colors {selectedStrategies.has(strategy.id) ? 'ring-1 ring-[#00d4ff]/50' : ''} {disabled ? 'opacity-50 cursor-not-allowed' : ''}">
            <input
              type="checkbox"
              checked={selectedStrategies.has(strategy.id)}
              onchange={() => toggleStrategy(strategy.id)}
              {disabled}
              class="w-4 h-4 rounded bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff] focus:ring-offset-[#1a1f3a]"
            />
            <div class="flex-1 min-w-0">
              <p class="text-white text-sm truncate">{strategy.name}</p>
              <p class="text-[#a0a0a0] text-xs">{strategy.family} • {strategy.engine}</p>
            </div>
          </label>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Services Selection -->
  <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold text-white">Services to check</h2>
      <div class="flex gap-2">
        <button
          onclick={selectAllServices}
          {disabled}
          class="text-sm text-[#00d4ff] hover:text-[#00b8e6] transition-colors disabled:opacity-50"
        >
          Select all
        </button>
        <span class="text-[#2a2f4a]">|</span>
        <button
          onclick={clearAllServices}
          {disabled}
          class="text-sm text-[#a0a0a0] hover:text-white transition-colors disabled:opacity-50"
        >
          Clear all
        </button>
      </div>
    </div>
    <div class="flex flex-wrap gap-3">
      {#each services as service}
        <label class="flex items-center gap-2 px-4 py-2 bg-[#0a0e27] rounded-lg cursor-pointer hover:bg-[#2a2f4a]/50 transition-colors {selectedServices.has(service.id) ? 'ring-2 ring-[#00d4ff]' : ''} {disabled ? 'opacity-50 cursor-not-allowed' : ''}">
          <input
            type="checkbox"
            checked={selectedServices.has(service.id)}
            onchange={() => toggleService(service.id)}
            {disabled}
            class="w-4 h-4 rounded bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff] focus:ring-offset-[#1a1f3a]"
          />
          <span class="text-white text-sm">{service.name}</span>
          {#if service.critical}
            <span class="text-[#ff3333] text-xs">●</span>
          {/if}
        </label>
      {/each}
    </div>
  </div>
</div>
