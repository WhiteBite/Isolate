<script lang="ts">
  import { page } from '$app/stores';
  import { installedPlugins, type PluginInfo } from '$lib/stores/plugins';

  interface Props {
    collapsed?: boolean;
    onToggle?: (collapsed: boolean) => void;
  }

  let { collapsed = $bindable(false), onToggle }: Props = $props();

  let plugins: PluginInfo[] = $state([]);
  installedPlugins.subscribe((value) => {
    plugins = value;
  });

  let currentPath = $derived($page.url.pathname);

  function toggleSidebar() {
    collapsed = !collapsed;
    onToggle?.(collapsed);
  }

  interface NavItem {
    id: string;
    name: string;
    icon: string;
    route: string;
  }

  const navItems: NavItem[] = [
    { id: 'dashboard', name: 'Dashboard', icon: 'layout-dashboard', route: '/' },
    { id: 'services', name: 'Services', icon: 'server', route: '/services' },
    { id: 'orchestra', name: 'Orchestra', icon: 'wand', route: '/orchestra' },
    { id: 'routing', name: 'Routing', icon: 'git-branch', route: '/routing' },
    { id: 'proxies', name: 'Proxies', icon: 'globe', route: '/proxies' },
  ];

  const systemItems: NavItem[] = [
    { id: 'marketplace', name: 'Marketplace', icon: 'store', route: '/marketplace' },
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
    'git-branch': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg>`,
    'globe': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>`,
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
  <nav class="flex-1 flex flex-col py-3 overflow-hidden">
    <div class="px-2 space-y-0.5">
      {#each navItems as item}
        <a
          href={item.route}
          class="relative flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-150
            {isActive(item.route) 
              ? 'bg-white/5 text-white' 
              : 'text-zinc-400 hover:bg-white/5 hover:text-zinc-200'}"
        >
          <!-- Active indicator pill -->
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

    <!-- Plugins (App Dock style) -->
    {#if plugins.length > 0}
      <div class="mt-4 px-2">
        {#if !collapsed}
          <div class="px-3 py-1.5 text-[10px] font-semibold text-zinc-500 uppercase tracking-widest">
            Plugins
          </div>
        {:else}
          <div class="h-px bg-white/5 mx-2 my-2"></div>
        {/if}
        <div class="mt-1 space-y-0.5">
          {#each plugins as plugin}
            <a
              href={plugin.route || `/plugins/${plugin.id}`}
              class="relative flex items-center gap-3 px-3 py-1.5 rounded-lg transition-all duration-150
                {isActive(plugin.route || `/plugins/${plugin.id}`) 
                  ? 'bg-white/5 text-white' 
                  : 'text-zinc-400 hover:bg-white/5 hover:text-zinc-200'}"
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
    <div class="px-2 space-y-0.5 pt-2">
      {#each systemItems as item}
        <a
          href={item.route}
          class="relative flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-150
            {isActive(item.route) 
              ? 'bg-white/5 text-white' 
              : 'text-zinc-500 hover:bg-white/5 hover:text-zinc-300'}"
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
      class="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-zinc-500 hover:bg-white/5 hover:text-zinc-300 transition-all duration-150"
    >
      <span class="w-5 h-5 flex-shrink-0 transition-transform duration-200" style="transform: rotate({collapsed ? '180deg' : '0deg'})">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M11 17l-5-5 5-5"/><path d="M18 17l-5-5 5-5"/></svg>
      </span>
      {#if !collapsed}
        <span class="text-sm">Collapse</span>
      {/if}
    </button>
  </div>
</aside>
