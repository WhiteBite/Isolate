<script lang="ts">
  import { page } from '$app/stores';
  import { installedPlugins, type PluginInfo } from '$lib/stores/plugins';
  import { browser } from '$app/environment';

  interface Props {
    collapsed?: boolean;
    onToggle?: (collapsed: boolean) => void;
  }

  let { collapsed = $bindable(false), onToggle }: Props = $props();

  let plugins: PluginInfo[] = $state([]);
  
  $effect(() => {
    const unsub = installedPlugins.subscribe((value) => {
      plugins = value;
    });
    return unsub;
  });

  let currentPath = $derived($page.url.pathname);

  // Badge counts - загружаются из backend
  let badgeCounts = $state<Record<string, number>>({
    services: 0,
    network: 0,
    marketplace: 0
  });

  // Состояние загрузки для badge
  let badgeLoading = $state(true);

  // Загрузка badge counts
  async function loadBadgeCounts() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Проверяем готовность backend
      const ready = await invoke<boolean>('is_backend_ready').catch(() => false);
      if (!ready) return;

      // Загружаем counts параллельно
      const [services, proxies] = await Promise.all([
        invoke<unknown[]>('get_services').catch(() => []),
        invoke<unknown[]>('get_proxies').catch(() => [])
      ]);

      badgeCounts = {
        services: services.length,
        network: proxies.length,
        marketplace: 0
      };
    } catch {
      // Игнорируем ошибки
    } finally {
      badgeLoading = false;
    }
  }

  // Загружаем при монтировании
  import { onMount } from 'svelte';
  
  onMount(() => {
    loadBadgeCounts();
  });

  function toggleSidebar() {
    collapsed = !collapsed;
    onToggle?.(collapsed);
  }

  interface NavItem {
    id: string;
    name: string;
    icon: string;
    route: string;
    badgeKey?: string;
  }

  const navItems: NavItem[] = [
    { id: 'dashboard', name: 'Dashboard', icon: 'layout-dashboard', route: '/' },
    { id: 'services', name: 'Services', icon: 'server', route: '/services', badgeKey: 'services' },
    { id: 'network', name: 'Network', icon: 'network', route: '/network', badgeKey: 'network' },
    { id: 'orchestra', name: 'Orchestra', icon: 'wand', route: '/orchestra' },
  ];

  const systemItems: NavItem[] = [
    { id: 'plugins', name: 'Plugins', icon: 'puzzle', route: '/plugins' },
    { id: 'settings', name: 'Settings', icon: 'settings', route: '/settings' },
    { id: 'logs', name: 'Logs', icon: 'terminal', route: '/logs' },
  ];

  function isActive(route: string): boolean {
    if (route === '/') return currentPath === '/';
    return currentPath.startsWith(route);
  }

  const icons: Record<string, string> = {
    'layout-dashboard': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="9" rx="1"/><rect x="14" y="3" width="7" height="5" rx="1"/><rect x="14" y="12" width="7" height="9" rx="1"/><rect x="3" y="16" width="7" height="5" rx="1"/></svg>`,
    'server': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><circle cx="6" cy="6" r="1" fill="currentColor"/><circle cx="6" cy="18" r="1" fill="currentColor"/></svg>`,
    'wand': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M15 4V2"/><path d="M15 16v-2"/><path d="M8 9h2"/><path d="M20 9h2"/><path d="M17.8 11.8L19 13"/><path d="M15 9h0"/><path d="M17.8 6.2L19 5"/><path d="m3 21 9-9"/><path d="M12.2 6.2L11 5"/></svg>`,
    'network': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="5" r="3"/><circle cx="5" cy="19" r="3"/><circle cx="19" cy="19" r="3"/><path d="M12 8v4"/><path d="M8.5 14.5L12 12l3.5 2.5"/><path d="M6.5 16.5L12 12"/><path d="M17.5 16.5L12 12"/></svg>`,
    'git-branch': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg>`,
    'globe': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>`,
    'puzzle': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M19.439 7.85c-.049.322.059.648.289.878l1.568 1.568c.47.47.706 1.087.706 1.704s-.235 1.233-.706 1.704l-1.611 1.611a.98.98 0 0 1-.837.276c-.47-.07-.802-.48-.968-.925a2.501 2.501 0 1 0-3.214 3.214c.446.166.855.497.925.968a.979.979 0 0 1-.276.837l-1.61 1.61a2.404 2.404 0 0 1-1.705.707 2.402 2.402 0 0 1-1.704-.706l-1.568-1.568a1.026 1.026 0 0 0-.877-.29c-.493.074-.84.504-1.02.968a2.5 2.5 0 1 1-3.237-3.237c.464-.18.894-.527.967-1.02a1.026 1.026 0 0 0-.289-.877l-1.568-1.568A2.402 2.402 0 0 1 1.998 12c0-.617.236-1.234.706-1.704L4.23 8.77c.24-.24.581-.353.917-.303.515.077.877.528 1.073 1.01a2.5 2.5 0 1 0 3.259-3.259c-.482-.196-.933-.558-1.01-1.073-.05-.336.062-.676.303-.917l1.525-1.525A2.402 2.402 0 0 1 12 1.998c.617 0 1.234.236 1.704.706l1.568 1.568c.23.23.556.338.877.29.493-.074.84-.504 1.02-.968a2.5 2.5 0 1 1 3.237 3.237c-.464.18-.894.527-.967 1.02Z"/></svg>`,
    'store': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>`,
    'settings': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`,
    'terminal': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`,
  };

  function getIcon(name: string): string {
    return icons[name] || '';
  }
</script>

<aside
  class="h-full flex flex-col backdrop-blur-xl bg-black/20 transition-all duration-200 ease-out"
  style="width: {collapsed ? '60px' : '200px'}"
  aria-label="Main navigation"
>
  <!-- Logo -->
  <div class="flex items-center h-14 px-3">
    <div class="flex items-center gap-3 overflow-hidden">
      <div class="w-8 h-8 flex-shrink-0 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-glow-indigo">
        <svg class="w-4 h-4 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
        </svg>
      </div>
      {#if !collapsed}
        <span class="font-semibold text-zinc-100 tracking-tight whitespace-nowrap">Isolate</span>
      {/if}
    </div>
  </div>

  <!-- Navigation -->
  <nav class="flex-1 flex flex-col py-3 overflow-hidden" aria-label="Primary">
    <div class="px-2 space-y-0.5">
      {#each navItems as item}
        <a
          href={item.route}
          class="relative flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-150
            {isActive(item.route) 
              ? 'bg-white/5 text-white' 
              : 'text-zinc-400 hover:bg-white/5 hover:text-zinc-200'}"
          aria-current={isActive(item.route) ? 'page' : undefined}
          aria-label={collapsed ? item.name : undefined}
        >
          <!-- Active indicator pill -->
          {#if isActive(item.route)}
            <div class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-4 bg-indigo-500 rounded-r-full shadow-[0_0_10px_rgba(99,102,241,0.5)]"></div>
          {/if}
          <span class="w-5 h-5 flex-shrink-0">
            {@html getIcon(item.icon)}
          </span>
          {#if !collapsed}
            <span class="text-sm font-medium whitespace-nowrap overflow-hidden flex-1">{item.name}</span>
            <!-- Badge -->
            {#if item.badgeKey}
              {#if badgeLoading}
                <div class="w-6 h-5 bg-zinc-700/50 rounded-full animate-pulse"></div>
              {:else if badgeCounts[item.badgeKey] > 0}
                <span class="min-w-[20px] h-5 px-1.5 flex items-center justify-center text-[10px] font-semibold rounded-full bg-indigo-500/20 text-indigo-400 border border-indigo-500/30">
                  {badgeCounts[item.badgeKey]}
                </span>
              {/if}
            {/if}
          {:else if item.badgeKey}
            {#if badgeLoading}
              <!-- Collapsed skeleton dot -->
              <span class="absolute top-1 right-1 w-2 h-2 rounded-full bg-zinc-600 animate-pulse"></span>
            {:else if badgeCounts[item.badgeKey] > 0}
              <!-- Collapsed badge dot -->
              <span class="absolute top-1 right-1 w-2 h-2 rounded-full bg-indigo-500 shadow-[0_0_6px_rgba(99,102,241,0.6)]"></span>
            {/if}
          {/if}
        </a>
      {/each}
    </div>

    <!-- Plugins (App Dock style) -->
    {#if plugins.length > 0}
      <div class="mt-4 px-2" role="group" aria-label="Plugins">
        {#if !collapsed}
          <div class="px-3 py-1.5 text-[10px] font-semibold text-zinc-400 uppercase tracking-widest" id="plugins-heading">
            Plugins
          </div>
        {:else}
          <div class="h-px bg-white/5 mx-2 my-2" aria-hidden="true"></div>
        {/if}
        <div class="mt-1 space-y-0.5" aria-labelledby={!collapsed ? 'plugins-heading' : undefined}>
          {#each plugins as plugin}
            <a
              href={plugin.route || `/plugins/${plugin.id}`}
              class="relative flex items-center gap-3 px-3 py-1.5 rounded-lg transition-all duration-150
                {isActive(plugin.route || `/plugins/${plugin.id}`) 
                  ? 'bg-white/5 text-white' 
                  : 'text-zinc-400 hover:bg-white/5 hover:text-zinc-200'}"
              aria-current={isActive(plugin.route || `/plugins/${plugin.id}`) ? 'page' : undefined}
              aria-label={collapsed ? plugin.name : undefined}
            >
              {#if isActive(plugin.route || `/plugins/${plugin.id}`)}
                <div class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-3 bg-indigo-500 rounded-r-full shadow-[0_0_8px_rgba(99,102,241,0.5)]"></div>
              {/if}
              <span class="w-4 h-4 flex-shrink-0 flex items-center justify-center text-sm">
                {plugin.icon}
              </span>
              {#if !collapsed}
                <span class="text-xs font-medium whitespace-nowrap overflow-hidden">{plugin.name}</span>
              {/if}
            </a>
          {/each}
        </div>
      </div>
    {/if}

    <div class="flex-1"></div>

    <!-- System -->
    <div class="px-2 space-y-0.5 pt-2" aria-label="System navigation">
      {#each systemItems as item}
        <a
          href={item.route}
          class="relative flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-150
            {isActive(item.route) 
              ? 'bg-white/5 text-white' 
              : 'text-zinc-400 hover:bg-white/5 hover:text-zinc-300'}"
          aria-current={isActive(item.route) ? 'page' : undefined}
          aria-label={collapsed ? item.name : undefined}
        >
          {#if isActive(item.route)}
            <div class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-4 bg-indigo-500 rounded-r-full shadow-[0_0_10px_rgba(99,102,241,0.5)]"></div>
          {/if}
          <span class="w-5 h-5 flex-shrink-0">
            {@html getIcon(item.icon)}
          </span>
          {#if !collapsed}
            <span class="text-sm font-medium whitespace-nowrap overflow-hidden">{item.name}</span>
          {/if}
        </a>
      {/each}
    </div>
  </nav>

  <!-- Collapse -->
  <div class="p-2">
    <button
      onclick={toggleSidebar}
      class="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-zinc-400 hover:bg-white/5 hover:text-zinc-300 transition-all duration-150"
      aria-expanded={!collapsed}
      aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
    >
      <span class="w-5 h-5 flex-shrink-0 transition-transform duration-200" style="transform: rotate({collapsed ? '180deg' : '0deg'})" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M11 17l-5-5 5-5"/><path d="M18 17l-5-5 5-5"/></svg>
      </span>
      {#if !collapsed}
        <span class="text-sm">Collapse</span>
      {/if}
    </button>
  </div>
</aside>
