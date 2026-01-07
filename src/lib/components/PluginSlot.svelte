<script lang="ts">
  import { installedPlugins, type PluginSlotLocation } from '$lib/stores/plugins';
  import { pluginSlots, type PluginSlots } from '$lib/stores/pluginSlots';
  import { widgetsBySlot, type LoadedComponent } from '$lib/plugins/loader';
  import { logger } from '$lib/utils/logger';
  import type { Snippet } from 'svelte';
  
  interface Props {
    location: PluginSlotLocation;
    fallback?: Snippet;
    context?: Record<string, unknown>;
  }
  
  let { location, fallback, context }: Props = $props();
  
  // Get plugins that have UI for this location (legacy support)
  let legacyPlugins = $derived(
    $installedPlugins.filter(p => p.enabled && p.ui?.locations?.includes(location))
  );
  
  // Get dynamically registered components for this slot (legacy)
  let slotName = $derived(location as keyof PluginSlots);
  let dynamicEntries = $derived($pluginSlots[slotName] || []);
  
  // Get Level 2 UI plugin widgets for this slot
  let pluginWidgets = $derived($widgetsBySlot[location] || []);
  
  // Track component errors for error boundary
  let componentErrors = $state<Map<string, string>>(new Map());
  
  // Handle component error
  function handleComponentError(widgetId: string, error: unknown) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    componentErrors.set(widgetId, errorMessage);
    componentErrors = new Map(componentErrors);
    logger.error('PluginSlot', `Widget "${widgetId}" error:`, error);
  }
  
  // Check if we have any content to render
  let hasContent = $derived(
    legacyPlugins.length > 0 || 
    dynamicEntries.length > 0 || 
    pluginWidgets.length > 0
  );
</script>

{#if hasContent}
  <!-- Render Level 2 UI plugin widgets -->
  {#each pluginWidgets as widget (widget.pluginId + ':' + widget.widgetDef.id)}
    {@const widgetId = `${widget.pluginId}:${widget.widgetDef.id}`}
    {@const hasError = componentErrors.has(widgetId)}
    
    <div 
      class="plugin-widget" 
      data-plugin={widget.pluginId} 
      data-widget={widget.widgetDef.id}
      data-location={location}
    >
      {#if hasError}
        <!-- Error boundary fallback -->
        <div class="p-3 bg-red-950/40 border border-red-500/20 rounded-lg">
          <div class="flex items-center gap-2 text-red-400">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <span class="text-xs font-medium">Plugin Error</span>
          </div>
          <p class="text-[10px] text-red-400/70 mt-1.5 font-mono">
            {widget.widgetDef.name}: {componentErrors.get(widgetId)}
          </p>
          <button 
            class="mt-2 text-[10px] text-red-400 hover:text-red-300 underline"
            onclick={() => {
              componentErrors.delete(widgetId);
              componentErrors = new Map(componentErrors);
            }}
          >
            Retry
          </button>
        </div>
      {:else}
        <!-- Render the plugin component -->
        {#key widgetId}
          {@const Component = widget.component}
          <Component 
            context={widget.context} 
            {...context}
          />
        {/key}
      {/if}
    </div>
  {/each}
  
  <!-- Render legacy plugins with UI for this location -->
  {#each legacyPlugins as plugin (plugin.id)}
    <div class="plugin-slot" data-plugin={plugin.id} data-location={location}>
      <div class="p-2 bg-zinc-900/40 border border-white/5 rounded-lg hover:bg-zinc-900/60 hover:border-white/10 transition-all">
        <div class="flex items-center gap-2">
          <span class="text-sm">{plugin.icon}</span>
          <span class="text-xs text-zinc-400 font-medium">{plugin.name}</span>
        </div>
        {#if plugin.description}
          <p class="text-[10px] text-zinc-400 mt-1 line-clamp-1">{plugin.description}</p>
        {/if}
      </div>
    </div>
  {/each}
  
  <!-- Render dynamically registered components (legacy) -->
  {#each dynamicEntries as entry (entry.pluginId)}
    {@const Component = entry.component}
    <Component {...entry.props} {context} />
  {/each}
{:else if fallback}
  {@render fallback()}
{/if}

<style>
  .plugin-widget {
    width: 100%;
  }
  
  .plugin-slot {
    width: 100%;
  }
</style>
