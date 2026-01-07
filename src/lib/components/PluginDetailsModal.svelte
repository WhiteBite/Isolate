<script lang="ts">
  import { 
    type PluginInfo, 
    getLevelColor, 
    getLevelLabel, 
    getTypeLabel, 
    getTypeIcon 
  } from '$lib/stores/plugins';
  import { logger } from '$lib/utils/logger';
  import BaseModal from './BaseModal.svelte';

  interface Props {
    open?: boolean;
    plugin: PluginInfo;
    onClose: () => void;
    onInstall?: () => void;
    onSettings?: () => void;
  }

  let { open = $bindable(true), plugin, onClose, onInstall, onSettings }: Props = $props();

  let installing = $state(false);
  let activeTab = $state<'overview' | 'permissions' | 'changelog'>('overview');

  async function handleInstall() {
    if (plugin.installed || installing) return;
    installing = true;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Check if backend ready
      const ready = await invoke<boolean>('is_backend_ready').catch(() => false);
      
      if (ready) {
        // Real installation via backend
        await invoke('install_plugin', { pluginId: plugin.id });
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

  function handleClose() {
    open = false;
    onClose();
  }

  // Get contribution items
  function getContributionItems(): { icon: string; label: string; count: number; description: string }[] {
    if (!plugin.contributes) return [];
    
    const items: { icon: string; label: string; count: number; description: string }[] = [];
    
    if (plugin.contributes.services && plugin.contributes.services > 0) {
      items.push({ 
        icon: '📡', 
        label: 'services', 
        count: plugin.contributes.services,
        description: 'Services for availability checking'
      });
    }
    if (plugin.contributes.hostlists && plugin.contributes.hostlists > 0) {
      items.push({ 
        icon: '📋', 
        label: 'hostlists', 
        count: plugin.contributes.hostlists,
        description: 'Domain lists for strategies'
      });
    }
    if (plugin.contributes.strategies && plugin.contributes.strategies > 0) {
      items.push({ 
        icon: '🎯', 
        label: 'strategies', 
        count: plugin.contributes.strategies,
        description: 'Bypass strategies'
      });
    }
    if (plugin.contributes.widgets && plugin.contributes.widgets > 0) {
      items.push({ 
        icon: '🎨', 
        label: 'widgets', 
        count: plugin.contributes.widgets,
        description: 'UI widgets for Dashboard'
      });
    }
    if (plugin.contributes.scripts && plugin.contributes.scripts > 0) {
      items.push({ 
        icon: '📜', 
        label: 'scripts', 
        count: plugin.contributes.scripts,
        description: 'Lua scripts with custom logic'
      });
    }
    
    return items;
  }

  let contributionItems = $derived(getContributionItems());
</script>

<BaseModal bind:open onclose={handleClose} class="w-full max-w-2xl overflow-hidden animate-scale-in">
    <!-- Header -->
    <div class="relative p-6 border-b border-white/5 bg-gradient-to-b from-zinc-800/50 to-transparent">
      <!-- Close Button -->
      <button
        onclick={onClose}
        class="absolute top-4 right-4 p-2 rounded-lg text-zinc-400 hover:text-zinc-200 
               hover:bg-white/5 transition-colors"
        aria-label="Close"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
      </button>

      <div class="flex items-start gap-4">
        <!-- Icon -->
        <div class="w-16 h-16 rounded-xl bg-zinc-800 flex items-center justify-center text-3xl shrink-0">
          {plugin.icon}
        </div>
        
        <!-- Info -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <h2 id="plugin-details-title" class="text-xl font-bold text-zinc-100">{plugin.name}</h2>
            <span class="text-sm text-zinc-500">v{plugin.version}</span>
          </div>
          
          <div class="flex items-center gap-2 mt-2 flex-wrap">
            <!-- Level Badge -->
            <span class="px-2 py-0.5 text-xs rounded-md border font-medium {getLevelColor(plugin.level)}">
              Level {plugin.level} — {getLevelLabel(plugin.level)}
            </span>
            
            <!-- Type Badge -->
            <span class="px-2 py-0.5 text-xs rounded-md bg-zinc-800 text-zinc-400 flex items-center gap-1">
              <span>{getTypeIcon(plugin.type)}</span>
              <span>{getTypeLabel(plugin.type)}</span>
            </span>
          </div>
          
          <div class="flex items-center gap-3 mt-2 text-sm text-zinc-500">
            <span>by {plugin.author}</span>
            {#if plugin.downloads && plugin.downloads > 0}
              <span>•</span>
              <span class="flex items-center gap-1">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                </svg>
                {plugin.downloads.toLocaleString()} downloads
              </span>
            {/if}
            {#if plugin.rating && plugin.rating > 0}
              <span>•</span>
              <span class="flex items-center gap-1">
                <svg class="w-4 h-4 text-amber-400" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                </svg>
                {plugin.rating.toFixed(1)}
              </span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex border-b border-white/5">
      <button
        onclick={() => activeTab = 'overview'}
        class="flex-1 px-4 py-3 text-sm font-medium transition-colors
               {activeTab === 'overview' 
                 ? 'text-indigo-400 border-b-2 border-indigo-400 bg-indigo-500/5' 
                 : 'text-zinc-400 hover:text-zinc-200'}"
      >
        Overview
      </button>
      <button
        onclick={() => activeTab = 'permissions'}
        class="flex-1 px-4 py-3 text-sm font-medium transition-colors
               {activeTab === 'permissions' 
                 ? 'text-indigo-400 border-b-2 border-indigo-400 bg-indigo-500/5' 
                 : 'text-zinc-400 hover:text-zinc-200'}"
      >
        Permissions
      </button>
      {#if plugin.changelog && plugin.changelog.length > 0}
        <button
          onclick={() => activeTab = 'changelog'}
          class="flex-1 px-4 py-3 text-sm font-medium transition-colors
                 {activeTab === 'changelog' 
                   ? 'text-indigo-400 border-b-2 border-indigo-400 bg-indigo-500/5' 
                   : 'text-zinc-400 hover:text-zinc-200'}"
        >
          Changelog
        </button>
      {/if}
    </div>

    <!-- Content -->
    <div class="p-6 max-h-[400px] overflow-y-auto">
      {#if activeTab === 'overview'}
        <!-- Description -->
        <div class="mb-6">
          <h3 class="text-sm font-medium text-zinc-400 mb-2">Description</h3>
          <p class="text-zinc-200">{plugin.description}</p>
        </div>

        <!-- Contributions -->
        {#if contributionItems.length > 0}
          <div class="mb-6">
            <h3 class="text-sm font-medium text-zinc-400 mb-3">What it adds</h3>
            <div class="grid gap-2">
              {#each contributionItems as item}
                <div class="flex items-center gap-3 p-3 bg-zinc-800/50 rounded-lg">
                  <span class="text-2xl">{item.icon}</span>
                  <div class="flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-zinc-200">{item.count} {item.label}</span>
                    </div>
                    <p class="text-xs text-zinc-500">{item.description}</p>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Source URL for Level 3 -->
        {#if plugin.level === 3 && plugin.sourceUrl}
          <div class="mb-6">
            <h3 class="text-sm font-medium text-zinc-400 mb-2">Source Code</h3>
            <a 
              href={plugin.sourceUrl} 
              target="_blank" 
              rel="noopener noreferrer"
              class="flex items-center gap-2 text-indigo-400 hover:text-indigo-300 transition-colors"
            >
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="16 18 22 12 16 6"/>
                <polyline points="8 6 2 12 8 18"/>
              </svg>
              <span>View code</span>
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3"/>
              </svg>
            </a>
          </div>
        {/if}

      {:else if activeTab === 'permissions'}
        {#if plugin.permissions}
          <div class="space-y-4">
            <!-- HTTP Permissions -->
            {#if plugin.permissions.http && plugin.permissions.http.length > 0}
              <div class="p-4 bg-zinc-800/50 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <svg class="w-5 h-5 text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/>
                    <line x1="2" y1="12" x2="22" y2="12"/>
                    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                  </svg>
                  <span class="font-medium text-zinc-200">HTTP Access</span>
                </div>
                <p class="text-xs text-zinc-500 mb-2">Plugin can make requests to the following domains:</p>
                <div class="flex flex-wrap gap-1">
                  {#each plugin.permissions.http as domain}
                    <span class="px-2 py-0.5 text-xs bg-zinc-700 text-zinc-300 rounded-md font-mono">
                      {domain}
                    </span>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Storage Permission -->
            {#if plugin.permissions.storage}
              <div class="p-4 bg-zinc-800/50 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <svg class="w-5 h-5 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7"/>
                    <ellipse cx="12" cy="7" rx="8" ry="4"/>
                    <path d="M4 12c0 2.21 3.582 4 8 4s8-1.79 8-4"/>
                  </svg>
                  <span class="font-medium text-zinc-200">Storage</span>
                </div>
                <p class="text-xs text-zinc-500">Plugin can save data locally</p>
              </div>
            {/if}

            <!-- Events Permission -->
            {#if plugin.permissions.events && plugin.permissions.events.length > 0}
              <div class="p-4 bg-zinc-800/50 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <svg class="w-5 h-5 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
                  </svg>
                  <span class="font-medium text-zinc-200">Events</span>
                </div>
                <p class="text-xs text-zinc-500 mb-2">Plugin can send/receive events:</p>
                <div class="flex flex-wrap gap-1">
                  {#each plugin.permissions.events as event}
                    <span class="px-2 py-0.5 text-xs bg-zinc-700 text-zinc-300 rounded-md font-mono">
                      {event}
                    </span>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Timeout -->
            {#if plugin.permissions.timeout}
              <div class="p-4 bg-zinc-800/50 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <svg class="w-5 h-5 text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/>
                    <polyline points="12 6 12 12 16 14"/>
                  </svg>
                  <span class="font-medium text-zinc-200">Timeout</span>
                </div>
                <p class="text-xs text-zinc-500">Maximum execution time: <span class="text-zinc-300">{plugin.permissions.timeout} ms</span></p>
              </div>
            {/if}

            <!-- Memory -->
            {#if plugin.permissions.memory}
              <div class="p-4 bg-zinc-800/50 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <svg class="w-5 h-5 text-pink-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="4" y="4" width="16" height="16" rx="2"/>
                    <rect x="9" y="9" width="6" height="6"/>
                    <line x1="9" y1="1" x2="9" y2="4"/>
                    <line x1="15" y1="1" x2="15" y2="4"/>
                    <line x1="9" y1="20" x2="9" y2="23"/>
                    <line x1="15" y1="20" x2="15" y2="23"/>
                    <line x1="20" y1="9" x2="23" y2="9"/>
                    <line x1="20" y1="14" x2="23" y2="14"/>
                    <line x1="1" y1="9" x2="4" y2="9"/>
                    <line x1="1" y1="14" x2="4" y2="14"/>
                  </svg>
                  <span class="font-medium text-zinc-200">Memory Limit</span>
                </div>
                <p class="text-xs text-zinc-500">Memory limit: <span class="text-zinc-300">{(plugin.permissions.memory / 1024 / 1024).toFixed(0)} MB</span></p>
              </div>
            {/if}
          </div>
        {:else}
          <div class="text-center py-8 text-zinc-500">
            <svg class="w-12 h-12 mx-auto mb-3 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            </svg>
            <p>Plugin requires no special permissions</p>
          </div>
        {/if}

      {:else if activeTab === 'changelog'}
        {#if plugin.changelog && plugin.changelog.length > 0}
          <div class="space-y-3">
            {#each plugin.changelog as entry, i}
              <div class="flex gap-3">
                <div class="w-2 h-2 mt-2 rounded-full bg-indigo-400 shrink-0"></div>
                <p class="text-zinc-300">{entry}</p>
              </div>
            {/each}
          </div>
        {:else}
          <div class="text-center py-8 text-zinc-500">
            <p>Changelog not available</p>
          </div>
        {/if}
      {/if}
    </div>

    <!-- Footer -->
    <div class="p-4 border-t border-white/5 bg-zinc-800/30 flex items-center justify-between">
      <div class="text-xs text-zinc-500">
        {#if plugin.installed}
          <span class="flex items-center gap-1 text-emerald-400">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12"/>
            </svg>
            Installed
          </span>
        {:else}
          Not installed
        {/if}
      </div>
      
      <div class="flex items-center gap-2">
        {#if plugin.installed && plugin.settings && plugin.settings.length > 0}
          <button
            onclick={onSettings}
            class="px-4 py-2 rounded-lg text-sm font-medium
                   bg-zinc-700 hover:bg-zinc-600 text-zinc-200 transition-colors"
          >
            Settings
          </button>
        {/if}
        
        <button
          onclick={handleInstall}
          disabled={plugin.installed || installing}
          class="px-4 py-2 rounded-lg text-sm font-medium transition-all
                 {plugin.installed 
                   ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 cursor-default' 
                   : installing
                     ? 'bg-indigo-500/50 text-white cursor-wait'
                     : 'bg-indigo-500 hover:bg-indigo-600 text-white'}"
        >
          {#if installing}
            <span class="flex items-center gap-1.5">
              <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10" stroke-opacity="0.25"/>
                <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
              </svg>
              Installing...
            </span>
          {:else if plugin.installed}
            Installed
          {:else}
            Install
          {/if}
        </button>
      </div>
    </div>
</BaseModal>

<style>
  @keyframes scale-in {
    from { 
      opacity: 0;
      transform: scale(0.95);
    }
    to { 
      opacity: 1;
      transform: scale(1);
    }
  }
  
  .animate-scale-in {
    animation: scale-in 0.2s ease-out;
  }
</style>
