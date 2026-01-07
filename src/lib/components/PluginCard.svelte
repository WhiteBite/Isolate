<script lang="ts">
  import { 
    type PluginType, 
    type PluginLevel, 
    type PluginContributes,
    getLevelColor, 
    getLevelLabel, 
    getTypeLabel, 
    getTypeIcon 
  } from '$lib/stores/plugins';
  import { logger } from '$lib/utils/logger';

  interface Props {
    id: string;
    name: string;
    version: string;
    description: string;
    author: string;
    icon: string;
    type: PluginType;
    level: PluginLevel;
    category?: string;
    installed?: boolean;
    downloads?: number;
    rating?: number;
    featured?: boolean;
    hasSettings?: boolean;
    contributes?: PluginContributes;
    sourceUrl?: string;
    onInstall?: () => void;
    onSettings?: () => void;
    onViewDetails?: () => void;
    onViewSource?: () => void;
    onReload?: () => void;
  }
  
  let { 
    id, 
    name, 
    version, 
    description, 
    author, 
    icon, 
    type,
    level,
    category,
    installed = false, 
    downloads = 0, 
    rating = 0,
    featured = false,
    hasSettings = false,
    contributes,
    sourceUrl,
    onInstall,
    onSettings,
    onViewDetails,
    onViewSource,
    onReload
  }: Props = $props();

  let installing = $state(false);
  let reloadingPlugin = $state(false);

  async function handleInstall() {
    if (installed || installing) return;
    installing = true;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Check if backend ready
      const ready = await invoke<boolean>('is_backend_ready').catch(() => false);
      
      if (ready) {
        // Real installation via backend
        await invoke('install_plugin', { pluginId: id });
      } else {
        // Browser mode - just update local state
        logger.log('Plugin', 'Browser mode - simulating install');
      }
      
      onInstall?.();
    } catch (e) {
      logger.error('Plugin', 'Install failed:', e);
      // Import toasts and show error
      const { toasts } = await import('$lib/stores/toast');
      toasts.error(`Failed to install: ${e}`);
    } finally {
      installing = false;
    }
  }

  async function handleReload() {
    if (reloadingPlugin || !installed) return;
    reloadingPlugin = true;
    try {
      await onReload?.();
    } finally {
      reloadingPlugin = false;
    }
  }

  function getCategoryLabel(cat: string): string {
    const labels: Record<string, string> = {
      strategies: 'Strategy',
      services: 'Service',
      tools: 'Tool'
    };
    return labels[cat] || cat;
  }

  function getCategoryColor(cat: string): string {
    const colors: Record<string, string> = {
      strategies: 'bg-violet-500/10 text-violet-400 border-violet-500/20',
      services: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
      tools: 'bg-amber-500/10 text-amber-400 border-amber-500/20'
    };
    return colors[cat] || 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20';
  }

  function getLevelBadgeInfo(lvl: PluginLevel): { label: string; color: string; tooltip: string } {
    switch (lvl) {
      case 1:
        return { 
          label: 'L1', 
          color: 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30',
          tooltip: 'Declarative — JSON/YAML configs'
        };
      case 2:
        return { 
          label: 'L2', 
          color: 'bg-indigo-500/15 text-indigo-400 border-indigo-500/30',
          tooltip: 'UI Plugin — Svelte components'
        };
      case 3:
        return { 
          label: 'L3', 
          color: 'bg-amber-500/15 text-amber-400 border-amber-500/30',
          tooltip: 'Script — Lua scripts'
        };
      default:
        return { 
          label: 'L?', 
          color: 'bg-zinc-500/15 text-zinc-400 border-zinc-500/30',
          tooltip: 'Unknown level'
        };
    }
  }

  let levelBadge = $derived(getLevelBadgeInfo(level));

  function getContributionItems(): { icon: string; label: string; count: number }[] {
    if (!contributes) return [];
    const items: { icon: string; label: string; count: number }[] = [];
    if (contributes.services && contributes.services > 0) {
      items.push({ icon: '📡', label: 'services', count: contributes.services });
    }
    if (contributes.hostlists && contributes.hostlists > 0) {
      items.push({ icon: '📋', label: 'hostlists', count: contributes.hostlists });
    }
    if (contributes.strategies && contributes.strategies > 0) {
      items.push({ icon: '🎯', label: 'strategies', count: contributes.strategies });
    }
    if (contributes.widgets && contributes.widgets > 0) {
      items.push({ icon: '🎨', label: 'widgets', count: contributes.widgets });
    }
    if (contributes.scripts && contributes.scripts > 0) {
      items.push({ icon: '📜', label: 'scripts', count: contributes.scripts });
    }
    return items;
  }

  let contributionItems = $derived(getContributionItems());
</script>

<div 
  class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl hover:border-white/10 transition-all group flex flex-col h-full
         {featured ? 'ring-1 ring-indigo-500/30 bg-gradient-to-br from-indigo-500/5 to-transparent' : ''}"
  role="article"
>
  <!-- Featured badge -->
  {#if featured}
    <div class="flex items-center gap-1.5 mb-3 text-xs">
      <span class="flex items-center gap-1.5 px-2 py-1 bg-gradient-to-r from-indigo-500/20 to-purple-500/20 rounded-md text-indigo-300 border border-indigo-500/20">
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
        </svg>
        <span class="font-medium">Featured</span>
      </span>
    </div>
  {/if}
  
  <!-- Header: Icon + Info -->
  <div class="flex items-start gap-3 mb-3">
    <div class="w-11 h-11 rounded-xl bg-zinc-800/80 flex items-center justify-center text-2xl shrink-0 group-hover:scale-105 transition-transform">
      {icon}
    </div>
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2 flex-wrap">
        <h3 class="font-semibold text-zinc-100 text-sm leading-tight truncate">{name}</h3>
        <span class="text-[10px] text-zinc-400 font-mono">v{version}</span>
      </div>
      <div class="flex items-center gap-1 mt-1 flex-wrap">
        <span class="px-1.5 py-0.5 text-[10px] rounded border font-bold {levelBadge.color}" title={levelBadge.tooltip}>
          {levelBadge.label}
        </span>
        <span class="px-1.5 py-0.5 text-[10px] rounded bg-zinc-800/80 text-zinc-400 border border-zinc-700/50">
          {getTypeLabel(type)}
        </span>
        {#if category}
          <span class="px-1.5 py-0.5 text-[10px] rounded border {getCategoryColor(category)}">
            {getCategoryLabel(category)}
          </span>
        {/if}
      </div>
    </div>
  </div>
  
  <!-- Description -->
  <p class="text-xs text-zinc-400 line-clamp-2 leading-relaxed mb-3 flex-1">{description}</p>
  
  <!-- Meta row -->
  <div class="flex items-center gap-2 text-[10px] text-zinc-400 mb-3 flex-wrap">
    <span class="flex items-center gap-1">
      <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
        <circle cx="12" cy="7" r="4"/>
      </svg>
      <span class="text-zinc-400">{author}</span>
    </span>
    {#if downloads > 0}
      <span class="text-zinc-600">•</span>
      <span class="flex items-center gap-1">
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
        </svg>
        <span class="tabular-nums">{downloads.toLocaleString()}</span>
      </span>
    {/if}
    {#if rating > 0}
      <span class="text-zinc-600">•</span>
      <span class="flex items-center gap-1">
        <svg class="w-3 h-3 text-amber-400" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
        </svg>
        <span class="text-amber-400/80 tabular-nums">{rating.toFixed(1)}</span>
      </span>
    {/if}
    {#each contributionItems as item}
      <span class="flex items-center gap-0.5" title="{item.count} {item.label}">
        <span class="text-xs">{item.icon}</span>
        <span class="text-zinc-400">{item.count}</span>
      </span>
    {/each}
  </div>
  
  <!-- Actions -->
  <div class="flex items-center justify-between gap-2 pt-3 border-t border-white/5">
    <div class="flex items-center gap-1">
      {#if onViewDetails}
        <button onclick={onViewDetails} class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800 transition-all" title="Details">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>
          </svg>
        </button>
      {/if}
      {#if level === 3 && sourceUrl && onViewSource}
        <button onclick={onViewSource} class="p-1.5 rounded-lg text-amber-500/70 hover:text-amber-400 hover:bg-amber-500/10 transition-all" title="View Source">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/>
          </svg>
        </button>
      {/if}
      {#if installed && hasSettings}
        <button onclick={onSettings} class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800 transition-all" title="Settings">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
        </button>
      {/if}
      {#if installed && onReload}
        <button onclick={handleReload} disabled={reloadingPlugin} class="p-1.5 rounded-lg transition-all {reloadingPlugin ? 'text-zinc-600 cursor-wait' : 'text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800'}" title="Reload">
          {#if reloadingPlugin}
            <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" stroke-opacity="0.25"/><path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
            </svg>
          {:else}
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M1 4v6h6M23 20v-6h-6"/><path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15"/>
            </svg>
          {/if}
        </button>
      {/if}
    </div>
    
    <button
      onclick={handleInstall}
      disabled={installed || installing}
      class="px-3 py-1.5 rounded-lg text-xs font-medium transition-all
             {installed 
               ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' 
               : installing
                 ? 'bg-indigo-500/50 text-white cursor-wait'
                 : 'bg-indigo-500 hover:bg-indigo-600 text-white'}"
    >
      {#if installing}
        <span class="flex items-center gap-1.5">
          <svg class="w-3.5 h-3.5 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" stroke-opacity="0.25"/><path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
          </svg>
          Installing...
        </span>
      {:else if installed}
        <span class="flex items-center gap-1.5">
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          Installed
        </span>
      {:else}
        Install
      {/if}
    </button>
  </div>
</div>
