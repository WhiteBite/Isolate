<script lang="ts">
  import { PluginCard } from '$lib/components';
  import type { MarketplacePlugin } from './types';
  import type { PluginInfo } from '$lib/stores/plugins';

  interface Props {
    plugins: MarketplacePlugin[];
    installedPlugins: PluginInfo[];
    viewMode: 'grid' | 'list';
    loading: boolean;
    hasActiveFilters: boolean;
    onInstall: (id: string) => void;
    onSettings: (id: string) => void;
    onViewDetails: (id: string) => void;
    onViewSource: (url: string) => void;
    onReload: (id: string) => void;
    onResetFilters: () => void;
  }

  let { 
    plugins, installedPlugins, viewMode, loading, hasActiveFilters,
    onInstall, onSettings, onViewDetails, onViewSource, onReload, onResetFilters
  }: Props = $props();

  function hasPluginSettings(pluginId: string): boolean {
    const plugin = installedPlugins.find(p => p.id === pluginId);
    return !!(plugin?.settings && plugin.settings.length > 0);
  }
</script>

{#if loading}
  <div class="flex items-center justify-center py-12">
    <div class="flex flex-col items-center gap-3">
      <svg class="w-10 h-10 animate-spin text-indigo-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" stroke-opacity="0.25"/>
        <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
      </svg>
      <span class="text-text-secondary text-sm">Loading plugins...</span>
    </div>
  </div>
{:else}
  <div class="{viewMode === 'grid' ? 'grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4' : 'flex flex-col gap-3'}">
    {#each plugins as plugin (plugin.id)}
      <PluginCard
        {...plugin}
        featured={false}
        hasSettings={hasPluginSettings(plugin.id)}
        onInstall={() => onInstall(plugin.id)}
        onSettings={() => onSettings(plugin.id)}
        onViewDetails={() => onViewDetails(plugin.id)}
        onViewSource={plugin.sourceUrl ? () => onViewSource(plugin.sourceUrl!) : undefined}
        onReload={plugin.installed ? () => onReload(plugin.id) : undefined}
      />
    {/each}
  </div>

  {#if plugins.length === 0}
    <div class="text-center py-16">
      <div class="w-20 h-20 mx-auto mb-4 bg-void-50 border border-glass-border rounded-2xl flex items-center justify-center">
        <span class="text-4xl">🔍</span>
      </div>
      <p class="text-text-primary text-lg font-medium">No plugins found</p>
      <p class="text-text-secondary text-sm mt-2 max-w-md mx-auto">
        Try changing filter settings or reset filters.
      </p>
      {#if hasActiveFilters}
        <button
          onclick={onResetFilters}
          class="mt-6 px-5 py-2.5 bg-indigo-500 hover:bg-indigo-600 text-white rounded-xl text-sm font-medium transition-colors inline-flex items-center gap-2"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
            <path d="M3 3v5h5"/>
          </svg>
          Reset all filters
        </button>
      {/if}
    </div>
  {/if}
{/if}
