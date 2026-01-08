<script lang="ts">
  import { browser } from '$app/environment';
  import { providerStore } from '$lib/stores/providers';
  import type { ProviderSummary } from '$lib/api/types';

  // Props
  interface Props {
    onSelect?: (providerId: string | null) => void;
  }
  let { onSelect }: Props = $props();

  // State
  let providers = $state<ProviderSummary[]>([]);
  let selectedId = $state<string | null>(null);
  let loading = $state(true);

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
    onSelect?.(providerId);
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
</script>

<div class="flex-1 flex flex-col">
  <div class="text-center mb-6">
    <h2 class="text-2xl font-bold text-white mb-2">Select Your Provider</h2>
    <p class="text-zinc-400">Choose your internet provider for optimized strategy recommendations</p>
  </div>

  {#if loading}
    <div class="flex-1 flex items-center justify-center">
      <div class="animate-spin w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full"></div>
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto">
      <!-- Skip option -->
      <button
        onclick={() => selectProvider(null)}
        class="w-full p-4 mb-3 rounded-xl border-2 transition-all duration-200 text-left flex items-center gap-4
          {selectedId === null 
            ? 'border-indigo-500 bg-indigo-500/10' 
            : 'border-white/10 hover:border-white/20 bg-white/5'}"
      >
        <span class="text-2xl opacity-50">🌐</span>
        <div class="flex-1">
          <p class="text-white font-medium">Skip / Other Provider</p>
          <p class="text-zinc-400 text-sm">I'll choose strategies manually</p>
        </div>
        {#if selectedId === null}
          <svg class="w-5 h-5 text-indigo-400" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
          </svg>
        {/if}
      </button>

      <!-- Provider list -->
      <div class="grid gap-3">
        {#each providers as provider}
          <button
            onclick={() => selectProvider(provider.id)}
            class="w-full p-4 rounded-xl border-2 transition-all duration-200 text-left flex items-center gap-4
              {selectedId === provider.id 
                ? 'border-indigo-500 bg-indigo-500/10' 
                : 'border-white/10 hover:border-white/20 bg-white/5'}"
          >
            <span class="text-3xl">{getProviderIcon(provider.id)}</span>
            <div class="flex-1">
              <p class="text-white font-medium">{provider.name}</p>
              <p class="text-zinc-400 text-sm">{provider.description}</p>
            </div>
            <div class="text-right">
              <span class="px-2 py-1 bg-white/5 rounded text-xs text-zinc-400">
                {provider.dpi_type}
              </span>
            </div>
            {#if selectedId === provider.id}
              <svg class="w-5 h-5 text-indigo-400" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
              </svg>
            {/if}
          </button>
        {/each}
      </div>

      {#if providers.length === 0}
        <div class="text-center py-8 text-zinc-400">
          <p>No providers available</p>
          <p class="text-sm mt-1">You can configure this later in Settings</p>
        </div>
      {/if}
    </div>

    <!-- Info box -->
    <div class="mt-4 p-3 bg-indigo-500/10 rounded-xl border border-indigo-500/20">
      <p class="text-indigo-300 text-sm flex items-start gap-2">
        <svg class="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span>Selecting your provider helps us recommend the best strategies for bypassing DPI blocks.</span>
      </p>
    </div>
  {/if}
</div>
