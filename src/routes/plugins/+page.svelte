<script lang="ts">
  import { Badge, Button, Card, Modal, Spinner } from '$lib/components';
  import { browser } from '$app/environment';

  // Types for Plugin System (new format)
  interface PluginManifest {
    id: string;
    name: string;
    version: string;
    author: string;
    description?: string;
    plugin_type: 'service-checker' | 'strategy-provider' | 'hostlist-provider';
    service?: {
      id: string;
      name: string;
      icon: string;
      category: string;
      description?: string;
      endpoints: { id: string; name: string; url: string; method: string }[];
    };
    strategy?: {
      id: string;
      name: string;
      family: string;
      config_file: string;
    };
    hostlist?: {
      id: string;
      name: string;
      file: string;
    };
    permissions: {
      http?: string[];
      filesystem?: boolean;
      process?: boolean;
    };
  }

  interface LoadedPlugin {
    manifest: PluginManifest;
    enabled: boolean;
    path: string;
    error?: string;
  }

  let plugins = $state<LoadedPlugin[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let pluginsDir = $state('');
  let selectedPlugin = $state<LoadedPlugin | null>(null);
  let showDetails = $state(false);
  let initialized = $state(false);

  function getInvoke() {
    const tauri = (window as any).__TAURI__;
    if (!tauri?.core?.invoke) {
      throw new Error('Tauri API not available');
    }
    return tauri.core.invoke;
  }

  $effect(() => {
    if (browser && !initialized) {
      initialized = true;
      loadPlugins();
    }
  });

  async function loadPlugins() {
    loading = true;
    error = null;
    
    try {
      const invoke = getInvoke();
      plugins = await invoke('get_all_plugins_cmd');
      pluginsDir = await invoke('get_plugins_dir');
    } catch (e) {
      console.error('Failed to load plugins:', e);
      error = String(e);
    }
    
    loading = false;
  }

  async function openPluginsFolder() {
    try {
      const invoke = getInvoke();
      await invoke('open_plugins_folder');
    } catch (e) {
      // Fallback: copy path to clipboard
      try {
        await navigator.clipboard.writeText(pluginsDir);
        alert(`Путь скопирован:\n${pluginsDir}`);
      } catch (clipErr) {
        console.error('Failed to open folder:', e);
      }
    }
  }

  function showPluginDetails(plugin: LoadedPlugin) {
    selectedPlugin = plugin;
    showDetails = true;
  }

  async function togglePlugin(plugin: LoadedPlugin) {
    const idx = plugins.findIndex(p => p.manifest.id === plugin.manifest.id);
    if (idx >= 0) {
      plugins[idx] = { ...plugins[idx], enabled: !plugins[idx].enabled };
      // TODO: persist to backend
    }
  }

  async function deletePlugin(plugin: LoadedPlugin) {
    if (!confirm(`Удалить плагин "${plugin.manifest.name}"?`)) return;
    
    try {
      const invoke = getInvoke();
      await invoke('delete_plugin', { pluginId: plugin.manifest.id });
      await loadPlugins();
    } catch (e) {
      error = `Не удалось удалить плагин: ${e}`;
    }
  }

  async function reloadPlugin(plugin: LoadedPlugin) {
    try {
      const invoke = getInvoke();
      await invoke('reload_plugin', { pluginId: plugin.manifest.id });
      await loadPlugins();
    } catch (e) {
      error = `Не удалось перезагрузить плагин: ${e}`;
    }
  }

  // Get contributions summary for a plugin
  function getContributionsSummary(plugin: LoadedPlugin): { icon: string; label: string }[] {
    const contributions: { icon: string; label: string }[] = [];
    const m = plugin.manifest;
    
    if (m.service) {
      contributions.push({ 
        icon: m.service.icon || '🌐', 
        label: `${m.service.name} (сервис)` 
      });
    }
    if (m.strategy) {
      contributions.push({ 
        icon: '⚡', 
        label: `${m.strategy.name} (стратегия)` 
      });
    }
    if (m.hostlist) {
      contributions.push({ 
        icon: '📋', 
        label: `${m.hostlist.name} (hostlist)` 
      });
    }
    
    return contributions;
  }
</script>

<div class="p-8 space-y-6 min-h-screen bg-[#0a0e27]">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-white">Плагины</h1>
      <p class="text-[#a0a0a0] mt-1">Управление расширениями Isolate</p>
    </div>
    <div class="flex gap-3">
      <Button variant="secondary" onclick={openPluginsFolder}>
        📁 Папка плагинов
      </Button>
      <Button variant="primary" onclick={loadPlugins}>
        🔄 Обновить
      </Button>
    </div>
  </div>

  {#if error}
    <div class="bg-[#ff3333]/10 border border-[#ff3333]/30 rounded-xl p-4 text-[#ff3333]">
      {error}
    </div>
  {/if}

  {#if loading}
    <div class="flex items-center justify-center py-20">
      <Spinner size="lg" />
    </div>
  {:else if plugins.length === 0}
    <!-- Empty State -->
    <Card>
      <div class="text-center py-12">
        <div class="w-20 h-20 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-[#00d4ff]/20 to-[#00ff88]/20 flex items-center justify-center">
          <span class="text-4xl">🧩</span>
        </div>
        <h3 class="text-xl font-semibold text-white mb-2">Нет установленных плагинов</h3>
        <p class="text-[#a0a0a0] mb-6 max-w-md mx-auto">
          Плагины расширяют функциональность Isolate — добавляют новые сервисы, чекеры и виджеты.
        </p>
        
        <div class="bg-[#0d1229] rounded-xl p-6 max-w-lg mx-auto text-left space-y-4">
          <h4 class="text-white font-medium">Как установить плагин:</h4>
          <ol class="text-[#a0a0a0] text-sm space-y-2 list-decimal list-inside">
            <li>Скачайте плагин (папка с <code class="text-[#00d4ff]">plugin.json</code>)</li>
            <li>Поместите папку в директорию плагинов:</li>
          </ol>
          <code class="text-[#00d4ff] text-sm bg-[#1a1f3a] px-3 py-2 rounded block break-all">
            {pluginsDir}
          </code>
          <ol class="text-[#a0a0a0] text-sm space-y-2 list-decimal list-inside" start={3}>
            <li>Нажмите "Обновить" или перезапустите приложение</li>
          </ol>
        </div>
        
        <Button variant="secondary" class="mt-6" onclick={openPluginsFolder}>
          📁 Открыть папку плагинов
        </Button>
      </div>
    </Card>
  {:else}
    <!-- Plugin Cards Grid -->
    <div class="grid gap-4">
      {#each plugins as plugin}
        {@const contributions = getContributionsSummary(plugin)}
        
        <Card class="{plugin.error ? 'border-[#ff3333]/30' : plugin.enabled ? 'border-[#00d4ff]/20' : 'border-[#2a2f4a]/50 opacity-75'}">
          <div class="p-4">
            <!-- Plugin Header Row -->
            <div class="flex items-start justify-between gap-4">
              <div class="flex items-center gap-4 flex-1 min-w-0">
                <!-- Icon -->
                <div class="w-12 h-12 rounded-xl bg-gradient-to-br {plugin.enabled ? 'from-[#00d4ff]/20 to-[#00ff88]/20' : 'from-[#2a2f4a] to-[#1a1f3a]'} flex items-center justify-center text-2xl shrink-0">
                  {plugin.manifest.service?.icon || '🧩'}
                </div>
                
                <!-- Info -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <h3 class="text-lg font-semibold text-white">{plugin.manifest.name}</h3>
                    <span class="text-[#606080] text-sm">v{plugin.manifest.version}</span>
                    {#if plugin.error}
                      <Badge variant="error" label="Ошибка" />
                    {:else if !plugin.enabled}
                      <Badge variant="inactive" label="Отключён" />
                    {/if}
                  </div>
                  <p class="text-[#606080] text-sm">{plugin.manifest.author}</p>
                  <p class="text-[#a0a0a0] text-sm mt-1 line-clamp-1">
                    {plugin.manifest.description || 'Нет описания'}
                  </p>
                </div>
              </div>
              
              <!-- Toggle Button -->
              <button
                class="w-10 h-10 rounded-lg flex items-center justify-center text-xl transition-all {plugin.enabled ? 'bg-[#00d4ff]/20 hover:bg-[#00d4ff]/30 text-[#00d4ff]' : 'bg-[#2a2f4a] hover:bg-[#3a3f5a] text-[#606080]'}"
                onclick={() => togglePlugin(plugin)}
                title={plugin.enabled ? 'Отключить плагин' : 'Включить плагин'}
              >
                {plugin.enabled ? '⏸️' : '▶️'}
              </button>
            </div>

            <!-- Contributions -->
            {#if contributions.length > 0}
              <div class="mt-3 pt-3 border-t border-[#2a2f4a]">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="text-[#606080] text-sm">Добавляет:</span>
                  {#each contributions.slice(0, 3) as contrib}
                    <span class="px-2 py-0.5 bg-[#1a1f3a] rounded text-sm text-[#a0a0a0]">
                      {contrib.icon} {contrib.label}
                    </span>
                  {/each}
                  {#if contributions.length > 3}
                    <span class="text-[#606080] text-sm">+{contributions.length - 3}</span>
                  {/if}
                </div>
              </div>
            {/if}

            <!-- Error Message -->
            {#if plugin.error}
              <div class="mt-3 p-3 bg-[#ff3333]/10 rounded-lg">
                <p class="text-[#ff3333] text-sm">{plugin.error}</p>
              </div>
            {/if}

            <!-- Actions -->
            <div class="mt-3 pt-3 border-t border-[#2a2f4a] flex items-center justify-between">
              <Button variant="ghost" size="sm" onclick={() => showPluginDetails(plugin)}>
                ℹ️ Детали
              </Button>
              <div class="flex gap-2">
                <Button variant="ghost" size="sm" onclick={() => reloadPlugin(plugin)} title="Перезагрузить">
                  🔄
                </Button>
                <Button variant="ghost" size="sm" onclick={() => deletePlugin(plugin)} title="Удалить">
                  🗑️
                </Button>
              </div>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<!-- Plugin Details Modal -->
{#if showDetails && selectedPlugin}
  {@const contributions = getContributionsSummary(selectedPlugin)}
  
  <Modal title={selectedPlugin.manifest.name} onclose={() => showDetails = false}>
    <div class="space-y-5">
      <!-- Header -->
      <div class="flex items-center gap-4">
        <div class="w-16 h-16 rounded-xl bg-gradient-to-br from-[#00d4ff]/20 to-[#00ff88]/20 flex items-center justify-center text-3xl">
          {selectedPlugin.manifest.service?.icon || '🧩'}
        </div>
        <div>
          <h3 class="text-xl font-semibold text-white">{selectedPlugin.manifest.name}</h3>
          <p class="text-[#a0a0a0]">v{selectedPlugin.manifest.version} • {selectedPlugin.manifest.author}</p>
        </div>
      </div>

      <!-- Description -->
      {#if selectedPlugin.manifest.description}
        <div>
          <h4 class="text-sm font-medium text-[#606080] mb-1">Описание</h4>
          <p class="text-[#a0a0a0]">{selectedPlugin.manifest.description}</p>
        </div>
      {/if}

      <!-- Path -->
      <div>
        <h4 class="text-sm font-medium text-[#606080] mb-1">Путь к плагину</h4>
        <code class="text-[#00d4ff] text-sm bg-[#1a1f3a] px-3 py-2 rounded block break-all">
          {selectedPlugin.path}
        </code>
      </div>

      <!-- Permissions -->
      <div>
        <h4 class="text-sm font-medium text-[#606080] mb-2">Разрешения</h4>
        <div class="flex flex-wrap gap-2">
          {#if selectedPlugin.manifest.permissions.http?.length}
            <span class="px-2 py-1 bg-[#00d4ff]/10 text-[#00d4ff] rounded text-xs">
              🌐 HTTP: {selectedPlugin.manifest.permissions.http.join(', ')}
            </span>
          {/if}
          {#if selectedPlugin.manifest.permissions.filesystem}
            <span class="px-2 py-1 bg-[#ffaa00]/10 text-[#ffaa00] rounded text-xs">
              📁 Файловая система
            </span>
          {/if}
          {#if selectedPlugin.manifest.permissions.process}
            <span class="px-2 py-1 bg-[#ff3333]/10 text-[#ff3333] rounded text-xs">
              ⚙️ Процессы
            </span>
          {/if}
          {#if !selectedPlugin.manifest.permissions.http?.length && !selectedPlugin.manifest.permissions.filesystem && !selectedPlugin.manifest.permissions.process}
            <span class="text-[#606080] text-sm">Нет специальных разрешений</span>
          {/if}
        </div>
      </div>

      <!-- Contributions -->
      {#if contributions.length > 0}
        <div>
          <h4 class="text-sm font-medium text-[#606080] mb-2">Добавляет в приложение</h4>
          <div class="space-y-2">
            {#each contributions as contrib}
              <div class="flex items-center gap-2 px-3 py-2 bg-[#1a1f3a] rounded-lg">
                <span class="text-lg">{contrib.icon}</span>
                <span class="text-white text-sm">{contrib.label}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Actions -->
      <div class="flex gap-3 pt-2">
        <Button variant="secondary" class="flex-1" onclick={() => { showDetails = false; reloadPlugin(selectedPlugin!); }}>
          🔄 Перезагрузить
        </Button>
        <Button variant="danger" class="flex-1" onclick={() => { showDetails = false; deletePlugin(selectedPlugin!); }}>
          🗑️ Удалить
        </Button>
      </div>
    </div>
  </Modal>
{/if}
