<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { Button, Badge, Spinner } from '$lib/components';

  // Types
  interface ProxyConfig {
    id: string;
    name: string;
    type: string;
  }

  interface DomainRoute {
    domain: string;
    proxyId: string;
    proxyName?: string;
  }

  interface AppRoute {
    appName: string;
    appPath: string;
    proxyId: string;
    proxyName?: string;
  }

  interface InstalledApp {
    name: string;
    path: string;
    icon?: string;
  }

  // State
  let activeTab = $state<'domain' | 'app'>('domain');
  let loading = $state(true);
  let proxies = $state<ProxyConfig[]>([]);
  let domainRoutes = $state<DomainRoute[]>([]);
  let appRoutes = $state<AppRoute[]>([]);
  let installedApps = $state<InstalledApp[]>([]);
  let appsLoading = $state(false);
  
  // Form state
  let newDomain = $state('');
  let newDomainProxy = $state('');
  let newAppPath = $state('');
  let newAppProxy = $state('');
  
  // Toast state
  let toasts = $state<{id: number; type: 'success' | 'error'; message: string}[]>([]);
  let toastId = 0;

  function showToast(type: 'success' | 'error', message: string) {
    const id = ++toastId;
    toasts = [...toasts, { id, type, message }];
    setTimeout(() => { toasts = toasts.filter(t => t.id !== id); }, 3000);
  }

  onMount(async () => {
    if (!browser) return;
    await loadData();
  });

  async function loadData() {
    loading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const [loadedProxies, loadedDomainRoutes, loadedAppRoutes] = await Promise.all([
        invoke<ProxyConfig[]>('get_proxies'),
        invoke<DomainRoute[]>('get_domain_routes'),
        invoke<AppRoute[]>('get_app_routes')
      ]);
      proxies = loadedProxies;
      domainRoutes = loadedDomainRoutes;
      appRoutes = loadedAppRoutes;
      if (proxies.length > 0) {
        newDomainProxy = proxies[0].id;
        newAppProxy = proxies[0].id;
      }
    } catch (e) {
      showToast('error', `Ошибка загрузки: ${e}`);
    } finally {
      loading = false;
    }
  }

  async function loadInstalledApps() {
    appsLoading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      installedApps = await invoke<InstalledApp[]>('get_installed_apps');
      showToast('success', `Найдено ${installedApps.length} приложений`);
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    } finally {
      appsLoading = false;
    }
  }

  async function handleAddDomainRoute() {
    if (!newDomain.trim() || !newDomainProxy) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('add_domain_route', { domain: newDomain.trim(), proxyId: newDomainProxy });
      showToast('success', 'Маршрут добавлен');
      newDomain = '';
      await loadData();
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    }
  }

  async function handleRemoveDomainRoute(domain: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('remove_domain_route', { domain });
      showToast('success', 'Маршрут удалён');
      await loadData();
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    }
  }

  async function handleAddAppRoute() {
    if (!newAppPath || !newAppProxy) return;
    const selectedApp = installedApps.find(a => a.path === newAppPath);
    if (!selectedApp) {
      showToast('error', 'Выберите приложение');
      return;
    }
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('add_app_route', { 
        appName: selectedApp.name, 
        appPath: selectedApp.path, 
        proxyId: newAppProxy 
      });
      showToast('success', 'Маршрут добавлен');
      newAppPath = '';
      await loadData();
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    }
  }

  async function handleRemoveAppRoute(appPath: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('remove_app_route', { appPath });
      showToast('success', 'Маршрут удалён');
      await loadData();
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    }
  }

  function getProxyName(proxyId: string): string {
    const proxy = proxies.find(p => p.id === proxyId);
    return proxy?.name || proxyId;
  }
</script>

<div class="min-h-screen bg-[#0a0e27] p-6">
  <!-- Toast notifications -->
  <div class="fixed top-4 right-4 z-50 space-y-2">
    {#each toasts as toast (toast.id)}
      <div class="flex items-center gap-3 px-4 py-3 rounded-lg border shadow-lg {toast.type === 'success' ? 'bg-green-500/10 border-green-500/50 text-green-400' : 'bg-red-500/10 border-red-500/50 text-red-400'}">
        <span class="text-sm">{toast.message}</span>
      </div>
    {/each}
  </div>

  <!-- Header -->
  <div class="flex items-center gap-4 mb-6">
    <button onclick={() => goto('/')} class="p-2 hover:bg-[#1a1f3a] rounded-lg transition-colors">
      <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
    </button>
    <h1 class="text-2xl font-bold text-white">Маршрутизация</h1>
  </div>

  <!-- Tabs -->
  <div class="flex gap-1 mb-6 bg-[#1a1f3a] p-1 rounded-lg w-fit">
    <button 
      onclick={() => activeTab = 'domain'}
      class="px-4 py-2 rounded-md text-sm font-medium transition-colors {activeTab === 'domain' ? 'bg-[#00d4ff] text-black' : 'text-gray-400 hover:text-white'}"
    >
      По доменам
    </button>
    <button 
      onclick={() => { activeTab = 'app'; if (installedApps.length === 0) loadInstalledApps(); }}
      class="px-4 py-2 rounded-md text-sm font-medium transition-colors {activeTab === 'app' ? 'bg-[#00d4ff] text-black' : 'text-gray-400 hover:text-white'}"
    >
      По приложениям
    </button>
  </div>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <Spinner />
    </div>
  {:else if proxies.length === 0}
    <div class="bg-[#1a1f3a] rounded-xl border border-gray-700 p-8 text-center">
      <p class="text-gray-400 mb-4">Сначала добавьте прокси</p>
      <Button variant="primary" onclick={() => goto('/proxies')}>Перейти к прокси</Button>
    </div>
  {:else}
    <!-- Per-domain Tab -->
    {#if activeTab === 'domain'}
      <div class="space-y-4">
        <!-- Add domain form -->
        <div class="bg-[#1a1f3a] rounded-xl border border-gray-700 p-4">
          <div class="flex gap-3 items-end">
            <div class="flex-1">
              <label class="block text-sm font-medium text-gray-300 mb-1">Домен</label>
              <input 
                type="text" 
                bind:value={newDomain}
                placeholder="example.com или *.example.com"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent"
              />
            </div>
            <div class="w-48">
              <label class="block text-sm font-medium text-gray-300 mb-1">Прокси</label>
              <select 
                bind:value={newDomainProxy}
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent"
              >
                {#each proxies as proxy (proxy.id)}
                  <option value={proxy.id}>{proxy.name}</option>
                {/each}
              </select>
            </div>
            <Button variant="primary" onclick={handleAddDomainRoute}>Добавить</Button>
          </div>
        </div>

        <!-- Domain routes table -->
        <div class="bg-[#1a1f3a] rounded-xl border border-gray-700 overflow-hidden">
          {#if domainRoutes.length === 0}
            <div class="text-center py-8 text-gray-400">
              <p>Нет маршрутов по доменам</p>
              <p class="text-sm mt-1">Добавьте домен для маршрутизации через прокси</p>
            </div>
          {:else}
            <table class="w-full">
              <thead class="bg-[#0a0e27]">
                <tr>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Домен</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Прокси</th>
                  <th class="px-4 py-3 text-right text-sm font-medium text-gray-400">Действия</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-700">
                {#each domainRoutes as route (route.domain)}
                  <tr class="hover:bg-[#252b4a] transition-colors">
                    <td class="px-4 py-3 text-white font-mono text-sm">{route.domain}</td>
                    <td class="px-4 py-3">
                      <Badge label={getProxyName(route.proxyId)} variant="info" size="sm" />
                    </td>
                    <td class="px-4 py-3 text-right">
                      <button 
                        onclick={() => handleRemoveDomainRoute(route.domain)}
                        class="p-1.5 hover:bg-red-500/20 rounded text-red-400 transition-colors"
                        title="Удалить"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Per-app Tab -->
    {#if activeTab === 'app'}
      <div class="space-y-4">
        <!-- Add app form -->
        <div class="bg-[#1a1f3a] rounded-xl border border-gray-700 p-4">
          <div class="flex gap-3 items-end">
            <div class="flex-1">
              <label class="block text-sm font-medium text-gray-300 mb-1">Приложение</label>
              <select 
                bind:value={newAppPath}
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent"
              >
                <option value="">Выберите приложение...</option>
                {#each installedApps as app (app.path)}
                  <option value={app.path}>{app.name}</option>
                {/each}
              </select>
            </div>
            <div class="w-48">
              <label class="block text-sm font-medium text-gray-300 mb-1">Прокси</label>
              <select 
                bind:value={newAppProxy}
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent"
              >
                {#each proxies as proxy (proxy.id)}
                  <option value={proxy.id}>{proxy.name}</option>
                {/each}
              </select>
            </div>
            <Button variant="primary" onclick={handleAddAppRoute}>Добавить</Button>
            <Button variant="secondary" loading={appsLoading} onclick={loadInstalledApps}>
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </Button>
          </div>
        </div>

        <!-- App routes table -->
        <div class="bg-[#1a1f3a] rounded-xl border border-gray-700 overflow-hidden">
          {#if appRoutes.length === 0}
            <div class="text-center py-8 text-gray-400">
              <p>Нет маршрутов по приложениям</p>
              <p class="text-sm mt-1">Выберите приложение для маршрутизации через прокси</p>
            </div>
          {:else}
            <table class="w-full">
              <thead class="bg-[#0a0e27]">
                <tr>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Приложение</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Путь</th>
                  <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Прокси</th>
                  <th class="px-4 py-3 text-right text-sm font-medium text-gray-400">Действия</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-700">
                {#each appRoutes as route (route.appPath)}
                  <tr class="hover:bg-[#252b4a] transition-colors">
                    <td class="px-4 py-3 text-white">{route.appName}</td>
                    <td class="px-4 py-3 text-gray-400 font-mono text-xs truncate max-w-xs" title={route.appPath}>
                      {route.appPath}
                    </td>
                    <td class="px-4 py-3">
                      <Badge label={getProxyName(route.proxyId)} variant="info" size="sm" />
                    </td>
                    <td class="px-4 py-3 text-right">
                      <button 
                        onclick={() => handleRemoveAppRoute(route.appPath)}
                        class="p-1.5 hover:bg-red-500/20 rounded text-red-400 transition-colors"
                        title="Удалить"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</div>
