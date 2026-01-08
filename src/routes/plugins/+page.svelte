<script lang="ts">
  import { browser } from '$app/environment';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { waitForBackend } from '$lib/utils/backend';
  import { toasts } from '$lib/stores/toast';
  import { Spinner } from '$lib/components';
  import { installedPlugins, loadPluginsFromBackend, installPlugin, reloadAllPlugins } from '$lib/stores/plugins';
  import { mockMarketPlugins } from '$lib/mocks';

  interface PluginManifest {
    id: string; name: string; version: string; author: string; description?: string; plugin_type: string;
    service?: { id: string; name: string; icon: string; category: string };
    strategy?: { id: string; name: string; family: string };
    hostlist?: { id: string; name: string };
    permissions: { http?: string[]; filesystem?: boolean; process?: boolean };
  }
  interface LoadedPlugin { manifest: PluginManifest; enabled: boolean; path: string; error?: string; }
  interface MarketPlugin { id: string; name: string; desc: string; author: string; ver: string; icon: string; dl: number; inst: boolean; cat: string; }

  let activeTab = $derived($page.url.searchParams.get('tab') || 'installed');
  let localPlugins = $state<LoadedPlugin[]>([]);
  let loading = $state(true);
  let reloading = $state(false);
  let pluginsDir = $state('');
  let initialized = $state(false);
  let selectedId = $state<string | null>(null);
  let selectedMarketId = $state<string | null>(null);
  let searchQuery = $state('');

  let market = $state<MarketPlugin[]>([...mockMarketPlugins]);

  let filtered = $derived(market.filter(p => p.name.toLowerCase().includes(searchQuery.toLowerCase())));
  let selected = $derived(localPlugins.find(p => p.manifest.id === selectedId));
  let selMarket = $derived(market.find(p => p.id === selectedMarketId));

  function getInvoke() { const t = (window as any).__TAURI__; return t?.core?.invoke; }
  function setTab(t: string) { goto(`/plugins?tab=${t}`, { replaceState: true, noScroll: true }); }

  import { onMount } from 'svelte';
  onMount(() => { if (!initialized) { initialized = true; load(); } });

  async function load() {
    loading = true;
    try {
      const ready = await waitForBackend(30, 300);
      if (!ready) { loading = false; return; }
      const inv = getInvoke();
      if (!inv) { loading = false; return; }
      const loaded: LoadedPlugin[] = await inv('get_all_plugins_cmd');
      pluginsDir = await inv('get_plugins_dir');
      try { const st: Record<string, boolean> = await inv('get_all_plugin_states'); for (const p of loaded) if (p.manifest.id in st) p.enabled = st[p.manifest.id]; } catch {}
      localPlugins = loaded;
      if (loaded.length > 0 && !selectedId) selectedId = loaded[0].manifest.id;
      await loadPluginsFromBackend();
    } catch (e) { console.error(e); }
    loading = false;
  }

  async function refresh() {
    if (reloading) return; reloading = true;
    try { const r = await reloadAllPlugins(); toasts.success(`Перезагружено ${r.plugins_loaded}`); await load(); } catch { toasts.error('Ошибка'); }
    reloading = false;
  }

  async function openFolder() {
    try { const inv = getInvoke(); if (inv) await inv('open_plugins_folder'); } catch { await navigator.clipboard.writeText(pluginsDir); toasts.info('Путь скопирован'); }
  }

  async function toggle(p: LoadedPlugin) {
    const i = localPlugins.findIndex(x => x.manifest.id === p.manifest.id);
    if (i >= 0) { localPlugins[i] = { ...localPlugins[i], enabled: !localPlugins[i].enabled }; localPlugins = [...localPlugins];
      try { const inv = getInvoke(); if (inv) await inv('set_plugin_enabled', { pluginId: p.manifest.id, enabled: localPlugins[i].enabled }); } catch {} }
  }

  async function del(p: LoadedPlugin) {
    if (!confirm(`Удалить "${p.manifest.name}"?`)) return;
    try { const inv = getInvoke(); if (inv) await inv('delete_plugin', { pluginId: p.manifest.id }); await load(); toasts.success('Удалён'); } catch (e) { toasts.error(`${e}`); }
  }

  function inst(id: string) { const p = market.find(x => x.id === id); if (p && !p.inst) { p.inst = true; market = [...market]; installPlugin(p as any); toasts.success(`${p.name} установлен`); } }
</script>

<div class="h-full flex">
  <div class="w-60 flex-shrink-0 border-r border-white/5 flex flex-col bg-black/20">
    <div class="p-2.5 border-b border-white/5">
      <div class="flex items-center justify-between mb-2">
        <span class="text-xs font-semibold text-zinc-200">Plugins</span>
        <div class="flex gap-0.5">
          <button onclick={openFolder} title="Открыть папку" class="p-1 rounded text-zinc-400 hover:text-zinc-300 hover:bg-white/5"><svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg></button>
          <button onclick={refresh} disabled={reloading} title="Обновить" class="p-1 rounded text-zinc-400 hover:text-zinc-300 hover:bg-white/5 disabled:opacity-50"><svg class="w-3.5 h-3.5 {reloading ? 'animate-spin' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg></button>
        </div>
      </div>
      <div class="flex p-0.5 bg-white/5 rounded" role="tablist">
        <button role="tab" aria-selected={activeTab === 'installed'} class="flex-1 px-2 py-0.5 text-[10px] font-medium rounded {activeTab === 'installed' ? 'bg-indigo-500/20 text-indigo-400' : 'text-zinc-400'}" onclick={() => setTab('installed')}>Installed ({localPlugins.length})</button>
        <button role="tab" aria-selected={activeTab === 'marketplace'} class="flex-1 px-2 py-0.5 text-[10px] font-medium rounded {activeTab === 'marketplace' ? 'bg-indigo-500/20 text-indigo-400' : 'text-zinc-400'}" onclick={() => setTab('marketplace')}>Market</button>
      </div>
    </div>
    <div class="flex-1 overflow-y-auto">
      {#if loading}<div class="flex justify-center py-6"><Spinner size="sm" /></div>
      {:else if activeTab === 'installed'}
        {#if localPlugins.length === 0}<div class="p-3 text-center text-[10px] text-zinc-600">Нет плагинов<br/><button onclick={() => setTab('marketplace')} class="text-indigo-400 mt-1">Marketplace →</button></div>
        {:else}{#each localPlugins as p}<button class="w-full px-2.5 py-1.5 flex items-center gap-2 text-left border-b border-white/[0.02] {selectedId === p.manifest.id ? 'bg-white/5' : 'hover:bg-white/[0.02]'} {!p.enabled ? 'opacity-40' : ''}" onclick={() => selectedId = p.manifest.id}>
          <span class="text-sm">{p.manifest.service?.icon || '🧩'}</span>
          <div class="flex-1 min-w-0"><div class="text-[11px] text-zinc-200 truncate">{p.manifest.name}</div><div class="text-[9px] text-zinc-400">{p.manifest.author}</div></div>
          {#if p.error}<span class="w-1.5 h-1.5 rounded-full bg-red-500"></span>{:else if p.enabled}<span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>{/if}
        </button>{/each}{/if}
      {:else}
        <div class="p-1.5 border-b border-white/5"><input type="text" placeholder="Поиск..." bind:value={searchQuery} class="w-full px-2 py-1 text-[10px] bg-white/5 border border-white/10 rounded text-zinc-300 placeholder-zinc-600 focus:outline-none focus:border-indigo-500/50" /></div>
        {#each filtered as p}<button class="w-full px-2.5 py-1.5 flex items-center gap-2 text-left border-b border-white/[0.02] {selectedMarketId === p.id ? 'bg-white/5' : 'hover:bg-white/[0.02]'}" onclick={() => selectedMarketId = p.id}>
          <span class="text-sm">{p.icon}</span>
          <div class="flex-1 min-w-0"><div class="text-[11px] text-zinc-200 truncate">{p.name}</div><div class="text-[9px] text-zinc-400 truncate">{p.desc}</div></div>
          {#if p.inst}<span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>{/if}
        </button>{/each}
      {/if}
    </div>
  </div>

  <div class="flex-1 flex flex-col bg-black/10">
    {#if activeTab === 'installed' && selected}
      <div class="p-3 border-b border-white/5 flex items-center gap-2.5">
        <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-indigo-500/20 to-purple-500/20 flex items-center justify-center text-lg">{selected.manifest.service?.icon || '🧩'}</div>
        <div class="flex-1 min-w-0"><div class="text-xs font-medium text-zinc-100">{selected.manifest.name}</div><div class="text-[9px] text-zinc-400">{selected.manifest.author} • v{selected.manifest.version}</div></div>
        <button onclick={() => toggle(selected!)} class="px-2 py-1 text-[10px] font-medium rounded {selected.enabled ? 'bg-amber-500/10 text-amber-400' : 'bg-emerald-500/10 text-emerald-400'}">{selected.enabled ? 'Откл' : 'Вкл'}</button>
        <button onclick={() => del(selected!)} class="px-2 py-1 text-[10px] font-medium rounded bg-red-500/10 text-red-400">Удалить</button>
      </div>
      <div class="flex-1 p-3 overflow-y-auto space-y-2.5 text-[11px]">
        {#if selected.error}<div class="p-2 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-[10px]">{selected.error}</div>{/if}
        {#if selected.manifest.description}<div><div class="text-[9px] text-zinc-400 mb-0.5">Описание</div><div class="text-zinc-300">{selected.manifest.description}</div></div>{/if}
        <div><div class="text-[9px] text-zinc-400 mb-0.5">Тип</div><span class="px-1.5 py-0.5 bg-white/5 rounded text-zinc-400 text-[10px]">{selected.manifest.plugin_type}</span></div>
        {#if selected.manifest.service || selected.manifest.strategy || selected.manifest.hostlist}
          <div><div class="text-[9px] text-zinc-400 mb-1">Добавляет</div>
            {#if selected.manifest.service}<div class="flex items-center gap-1.5 px-2 py-1 bg-white/5 rounded mb-1"><span>{selected.manifest.service.icon}</span><span class="text-zinc-300">{selected.manifest.service.name}</span></div>{/if}
            {#if selected.manifest.strategy}<div class="flex items-center gap-1.5 px-2 py-1 bg-white/5 rounded mb-1"><span>⚡</span><span class="text-zinc-300">{selected.manifest.strategy.name}</span></div>{/if}
            {#if selected.manifest.hostlist}<div class="flex items-center gap-1.5 px-2 py-1 bg-white/5 rounded"><span>📋</span><span class="text-zinc-300">{selected.manifest.hostlist.name}</span></div>{/if}
          </div>
        {/if}
        <div><div class="text-[9px] text-zinc-400 mb-1">Разрешения</div>
          <div class="flex flex-wrap gap-1">
            {#if selected.manifest.permissions.http?.length}<span class="px-1.5 py-0.5 bg-indigo-500/10 text-indigo-400 rounded text-[9px]">🌐 HTTP</span>{/if}
            {#if selected.manifest.permissions.filesystem}<span class="px-1.5 py-0.5 bg-amber-500/10 text-amber-400 rounded text-[9px]">📁 FS</span>{/if}
            {#if selected.manifest.permissions.process}<span class="px-1.5 py-0.5 bg-red-500/10 text-red-400 rounded text-[9px]">⚙️ Proc</span>{/if}
            {#if !selected.manifest.permissions.http?.length && !selected.manifest.permissions.filesystem && !selected.manifest.permissions.process}<span class="text-zinc-600 text-[9px]">Нет</span>{/if}
          </div>
        </div>
        <div><div class="text-[9px] text-zinc-400 mb-0.5">Путь</div><code class="block text-[8px] text-zinc-600 bg-white/5 px-1.5 py-1 rounded break-all">{selected.path}</code></div>
      </div>
    {:else if activeTab === 'marketplace' && selMarket}
      <div class="p-3 border-b border-white/5 flex items-center gap-2.5">
        <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-indigo-500/20 to-purple-500/20 flex items-center justify-center text-lg">{selMarket.icon}</div>
        <div class="flex-1 min-w-0"><div class="text-xs font-medium text-zinc-100">{selMarket.name}</div><div class="text-[9px] text-zinc-400">{selMarket.author} • v{selMarket.ver}</div></div>
        {#if selMarket.inst}<span class="px-2 py-1 text-[10px] font-medium rounded bg-emerald-500/10 text-emerald-400">Установлен</span>
        {:else}<button onclick={() => inst(selMarket!.id)} class="px-2.5 py-1 text-[10px] font-medium rounded bg-indigo-500 text-white hover:bg-indigo-600">Установить</button>{/if}
      </div>
      <div class="flex-1 p-3 overflow-y-auto space-y-2.5 text-[11px]">
        <div><div class="text-[9px] text-zinc-400 mb-0.5">Описание</div><div class="text-zinc-300">{selMarket.desc}</div></div>
        <div><div class="text-[9px] text-zinc-400 mb-0.5">Категория</div><span class="px-1.5 py-0.5 bg-white/5 rounded text-zinc-400 text-[10px] capitalize">{selMarket.cat}</span></div>
        <div><div class="text-[9px] text-zinc-400 mb-0.5">Загрузки</div><span class="text-zinc-300">{selMarket.dl.toLocaleString()}</span></div>
      </div>
    {:else}
      <div class="flex-1 flex items-center justify-center"><div class="text-center opacity-40"><div class="text-2xl mb-1">🧩</div><div class="text-[10px] text-zinc-600">Выберите плагин</div></div></div>
    {/if}
  </div>
</div>
