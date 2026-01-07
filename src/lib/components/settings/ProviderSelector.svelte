<script lang="ts">
  import { browser } from '$app/environment';
  import { providerStore } from '$lib/stores/providers';
  import type { ProviderSummary } from '$lib/api/types';

  // Props
  interface Props {
    onProviderChange?: (providerId: string | null) => void;
  }
  let { onProviderChange }: Props = $props();

  // State
  let providers = $state<ProviderSummary[]>([]);
  let selectedId = $state<string | null>(null);
  let loading = $state(true);
  let expanded = $state(false);

  // Subscribe to store
  $effect(() => {
    const unsubscribe = providerStore.subscribe(() => {
      const state = providerStore.getState();
      providers = state.providers;
      selectedId = state.selectedProviderId;
      loading = state.loading;
    });
    return unsubscribe;
  });

  // Load providers on mount
  $effect(() => {
    if (browser) {
      providerStore.loadProviders();
    }
  });

  function selectProvider(providerId: string | null) {
    providerStore.setProvider(providerId);
    expanded = false;
    onProviderChange?.(providerId);
  }

  function getProviderIcon(providerId: string): string {
    const icons: Record<string, string> = {
      rostelecom: '🏢',
      mts: '📱',
      beeline: '🐝',
      megafon: '📶',
      ttk: '🌐',
      dom_ru: '🏠'
    };
    return icons[providerId] || '🌐';
  }

  let selectedProvider = $derived(providers.find(p => p.id === selectedId));
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between">
    <div>
      <p class="text-text-primary font-medium">ISP Profile</p>
      <p class="text-text-secondary text-sm">Select your internet provider for optimized recommendations</p>
    </div>
  </div>

  {#if loading}
    <div class="p-4 bg-void-100 rounded-xl border border-glass-border animate-pulse">
      <div class="h-6 bg-void-200 rounded w-1/3"></div>
    </div>
  {:else}
    <!-- Selected Provider Display / Dropdown Trigger -->
    <button
      onclick={() => expanded = !expanded}
      class="w-full p-4 bg-void-100 rounded-xl border border-glass-border hover:border-indigo-500/50 transition-all duration-200 text-left flex items-center justify-between group"
    >
      {#if selectedProvider}
        <div class="flex items-center gap-3">
          <span class="text-2xl">{getProviderIcon(selectedProvider.id)}</span>
          <div>
            <p class="text-text-primary font-medium">{selectedProvider.name}</p>
            <p class="text-text-muted text-sm">{selectedProvider.dpi_type}</p>
          </div>
        </div>
      {:else}
        <div class="flex items-center gap-3">
          <span class="text-2xl opacity-50">🌐</span>
          <div>
            <p class="text-text-secondary">Not selected</p>
            <p class="text-text-muted text-sm">Click to choose your provider</p>
          </div>
        </div>
      {/if}
      
      <svg 
        class="w-5 h-5 text-text-muted transition-transform duration-200 {expanded ? 'rotate-180' : ''}"
        fill="none" 
        stroke="currentColor" 
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
      </svg>
    </button>

    <!-- Dropdown -->
    {#if expanded}
      <div class="bg-void-100 rounded-xl border border-glass-border overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200">
        <!-- Clear selection option -->
        {#if selectedId}
          <button
            onclick={() => selectProvider(null)}
            class="w-full p-3 text-left hover:bg-void-200 transition-colors flex items-center gap-3 border-b border-glass-border"
          >
            <span class="text-xl opacity-50">✕</span>
            <span class="text-text-secondary">Clear selection</span>
          </button>
        {/if}

        <!-- Provider list -->
        {#each providers as provider}
          <button
            onclick={() => selectProvider(provider.id)}
            class="w-full p-4 text-left hover:bg-void-200 transition-colors flex items-center gap-3 {provider.id === selectedId ? 'bg-indigo-500/10 border-l-2 border-indigo-500' : ''}"
          >
            <span class="text-2xl">{getProviderIcon(provider.id)}</span>
            <div class="flex-1">
              <p class="text-text-primary font-medium">{provider.name}</p>
              <p class="text-text-muted text-sm">{provider.description}</p>
            </div>
            <div class="text-right">
              <span class="px-2 py-1 bg-void-200 rounded text-xs text-text-muted">
                {provider.dpi_type}
              </span>
              <p class="text-text-muted text-xs mt-1">
                {provider.strategy_count} strategies
              </p>
            </div>
          </button>
        {/each}

        {#if providers.length === 0}
          <div class="p-4 text-center text-text-muted">
            No providers available
          </div>
        {/if}
      </div>
    {/if}

    <!-- Recommendations preview -->
    {#if selectedProvider}
      {@const state = providerStore.getState()}
      {#if state.recommendations}
        <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
          <div class="flex items-center gap-2 mb-2">
            <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
            <span class="text-indigo-400 text-sm font-medium">Recommended strategies</span>
          </div>
          <div class="flex flex-wrap gap-2">
            {#each state.recommendations.strategies.slice(0, 4) as strategyId, i}
              <span class="px-2 py-1 bg-indigo-500/20 text-indigo-300 rounded text-xs">
                {i === 0 ? '⭐ ' : ''}{strategyId.replace('zapret_', '').replace(/_/g, ' ')}
              </span>
            {/each}
            {#if state.recommendations.strategies.length > 4}
              <span class="px-2 py-1 text-text-muted text-xs">
                +{state.recommendations.strategies.length - 4} more
              </span>
            {/if}
          </div>
        </div>
      {/if}
    {/if}
  {/if}
</div>
