<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { Modal, Badge, Button, Spinner } from '$lib/components';

  // Types
  interface ProxyConfig {
    id: string;
    name: string;
    type: 'vless' | 'vmess' | 'shadowsocks' | 'socks5' | 'http';
    server: string;
    port: number;
    status: 'active' | 'inactive' | 'testing';
    // VLESS specific
    uuid?: string;
    tls?: boolean;
    sni?: string;
    transport?: string;
    // VMess specific
    alterId?: number;
    security?: string;
    // Shadowsocks specific
    method?: string;
    password?: string;
  }

  // State
  let proxies = $state<ProxyConfig[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  
  // Modal states
  let showAddModal = $state(false);
  let showImportUrlModal = $state(false);
  let showImportSubModal = $state(false);
  let showEditModal = $state(false);
  let editingProxy = $state<ProxyConfig | null>(null);
  
  // Form state
  let formType = $state<ProxyConfig['type']>('vless');
  let formName = $state('');
  let formServer = $state('');
  let formPort = $state(443);
  // VLESS fields
  let formUuid = $state('');
  let formTls = $state(true);
  let formSni = $state('');
  let formTransport = $state('tcp');
  // VMess fields
  let formAlterId = $state(0);
  let formSecurity = $state('auto');
  // Shadowsocks fields
  let formMethod = $state('aes-256-gcm');
  let formPassword = $state('');
  
  // Import state
  let importUrl = $state('');
  let importLoading = $state(false);
  
  // Toast state
  let toasts = $state<{id: number; type: 'success' | 'error'; message: string}[]>([]);
  let toastId = 0;

  function showToast(type: 'success' | 'error', message: string) {
    const id = ++toastId;
    toasts = [...toasts, { id, type, message }];
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id);
    }, 3000);
  }

  onMount(async () => {
    if (!browser) return;
    await loadProxies();
  });

  async function loadProxies() {
    loading = true;
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      proxies = await invoke<ProxyConfig[]>('get_proxies');
    } catch (e) {
      error = String(e);
      showToast('error', `Ошибка загрузки: ${e}`);
    } finally {
      loading = false;
    }
  }

  async function handleAddProxy() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const proxy: Partial<ProxyConfig> = {
        name: formName,
        type: formType,
        server: formServer,
        port: formPort,
      };
      
      if (formType === 'vless') {
        proxy.uuid = formUuid;
        proxy.tls = formTls;
        proxy.sni = formSni;
        proxy.transport = formTransport;
      } else if (formType === 'vmess') {
        proxy.uuid = formUuid;
        proxy.alterId = formAlterId;
        proxy.security = formSecurity;
      } else if (formType === 'shadowsocks') {
        proxy.method = formMethod;
        proxy.password = formPassword;
      }
      
      await invoke('add_proxy', { proxy });
      showToast('success', 'Прокси добавлен');
      showAddModal = false;
      resetForm();
      await loadProxies();
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    }
  }

  async function handleUpdateProxy() {
    if (!editingProxy) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const proxy: Partial<ProxyConfig> = {
        id: editingProxy.id,
        name: formName,
        type: formType,
        server: formServer,
        port: formPort,
      };
      
      if (formType === 'vless') {
        proxy.uuid = formUuid;
        proxy.tls = formTls;
        proxy.sni = formSni;
        proxy.transport = formTransport;
      } else if (formType === 'vmess') {
        proxy.uuid = formUuid;
        proxy.alterId = formAlterId;
        proxy.security = formSecurity;
      } else if (formType === 'shadowsocks') {
        proxy.method = formMethod;
        proxy.password = formPassword;
      }
      
      await invoke('update_proxy', { proxy });
      showToast('success', 'Прокси обновлён');
      showEditModal = false;
      editingProxy = null;
      resetForm();
      await loadProxies();
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    }
  }

  async function handleDeleteProxy(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_proxy', { id });
      showToast('success', 'Прокси удалён');
      await loadProxies();
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    }
  }

  async function handleApplyProxy(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('apply_proxy', { id });
      showToast('success', 'Прокси применён');
      await loadProxies();
    } catch (e) {
      showToast('error', `Ошибка: ${e}`);
    }
  }

  async function handleTestProxy(id: string) {
    const proxy = proxies.find(p => p.id === id);
    if (proxy) {
      proxy.status = 'testing';
      proxies = [...proxies];
    }
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('test_proxy', { id });
      showToast('success', 'Прокси работает');
      await loadProxies();
    } catch (e) {
      showToast('error', `Тест не пройден: ${e}`);
      await loadProxies();
    }
  }

  async function handleImportUrl() {
    if (!importUrl.trim()) return;
    importLoading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('import_proxy_url', { url: importUrl });
      showToast('success', 'Прокси импортирован');
      showImportUrlModal = false;
      importUrl = '';
      await loadProxies();
    } catch (e) {
      showToast('error', `Ошибка импорта: ${e}`);
    } finally {
      importLoading = false;
    }
  }

  async function handleImportSubscription() {
    if (!importUrl.trim()) return;
    importLoading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('import_subscription', { url: importUrl });
      showToast('success', 'Подписка импортирована');
      showImportSubModal = false;
      importUrl = '';
      await loadProxies();
    } catch (e) {
      showToast('error', `Ошибка импорта: ${e}`);
    } finally {
      importLoading = false;
    }
  }

  function openEditModal(proxy: ProxyConfig) {
    editingProxy = proxy;
    formType = proxy.type;
    formName = proxy.name;
    formServer = proxy.server;
    formPort = proxy.port;
    formUuid = proxy.uuid || '';
    formTls = proxy.tls ?? true;
    formSni = proxy.sni || '';
    formTransport = proxy.transport || 'tcp';
    formAlterId = proxy.alterId ?? 0;
    formSecurity = proxy.security || 'auto';
    formMethod = proxy.method || 'aes-256-gcm';
    formPassword = proxy.password || '';
    showEditModal = true;
  }

  function resetForm() {
    formType = 'vless';
    formName = '';
    formServer = '';
    formPort = 443;
    formUuid = '';
    formTls = true;
    formSni = '';
    formTransport = 'tcp';
    formAlterId = 0;
    formSecurity = 'auto';
    formMethod = 'aes-256-gcm';
    formPassword = '';
  }

  function getProxyTypeBadge(type: ProxyConfig['type']): 'info' | 'active' | 'warning' {
    const map: Record<ProxyConfig['type'], 'info' | 'active' | 'warning'> = {
      vless: 'info',
      vmess: 'active',
      shadowsocks: 'warning',
      socks5: 'inactive' as 'info',
      http: 'inactive' as 'info'
    };
    return map[type] || 'info';
  }

  const ssMethodOptions = [
    'aes-128-gcm', 'aes-256-gcm', 'chacha20-ietf-poly1305',
    '2022-blake3-aes-128-gcm', '2022-blake3-aes-256-gcm', '2022-blake3-chacha20-poly1305'
  ];

  const transportOptions = ['tcp', 'ws', 'grpc', 'http'];
  const vmessSecurityOptions = ['auto', 'aes-128-gcm', 'chacha20-poly1305', 'none'];
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
  <div class="flex items-center justify-between mb-6">
    <div class="flex items-center gap-4">
      <button onclick={() => goto('/')} class="p-2 hover:bg-[#1a1f3a] rounded-lg transition-colors">
        <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h1 class="text-2xl font-bold text-white">Прокси</h1>
    </div>
    <div class="flex gap-2">
      <Button variant="secondary" size="sm" onclick={() => { showImportUrlModal = true; importUrl = ''; }}>
        Импорт из URL
      </Button>
      <Button variant="secondary" size="sm" onclick={() => { showImportSubModal = true; importUrl = ''; }}>
        Импорт подписки
      </Button>
      <Button variant="primary" size="sm" onclick={() => { resetForm(); showAddModal = true; }}>
        Добавить вручную
      </Button>
    </div>
  </div>

  <!-- Proxies Table -->
  <div class="bg-[#1a1f3a] rounded-xl border border-gray-700 overflow-hidden">
    {#if loading}
      <div class="flex items-center justify-center py-12">
        <Spinner />
      </div>
    {:else if error}
      <div class="text-center py-12 text-red-400">{error}</div>
    {:else if proxies.length === 0}
      <div class="text-center py-12 text-gray-400">
        <p>Нет добавленных прокси</p>
        <p class="text-sm mt-2">Добавьте прокси вручную или импортируйте из URL</p>
      </div>
    {:else}
      <table class="w-full">
        <thead class="bg-[#0a0e27]">
          <tr>
            <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Имя</th>
            <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Тип</th>
            <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Сервер</th>
            <th class="px-4 py-3 text-left text-sm font-medium text-gray-400">Статус</th>
            <th class="px-4 py-3 text-right text-sm font-medium text-gray-400">Действия</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-700">
          {#each proxies as proxy (proxy.id)}
            <tr class="hover:bg-[#252b4a] transition-colors">
              <td class="px-4 py-3 text-white">{proxy.name}</td>
              <td class="px-4 py-3">
                <Badge label={proxy.type.toUpperCase()} variant={getProxyTypeBadge(proxy.type)} size="sm" />
              </td>
              <td class="px-4 py-3 text-gray-300 font-mono text-sm">{proxy.server}:{proxy.port}</td>
              <td class="px-4 py-3">
                {#if proxy.status === 'active'}
                  <Badge label="Активен" variant="active" size="sm" dot />
                {:else if proxy.status === 'testing'}
                  <Badge label="Тестирование..." variant="warning" size="sm" dot />
                {:else}
                  <Badge label="Неактивен" variant="inactive" size="sm" />
                {/if}
              </td>
              <td class="px-4 py-3">
                <div class="flex items-center justify-end gap-2">
                  <button onclick={() => handleApplyProxy(proxy.id)} class="p-1.5 hover:bg-[#00d4ff]/20 rounded text-[#00d4ff] transition-colors" title="Применить">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                  </button>
                  <button onclick={() => openEditModal(proxy)} class="p-1.5 hover:bg-gray-600 rounded text-gray-400 transition-colors" title="Редактировать">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </button>
                  <button onclick={() => handleTestProxy(proxy.id)} class="p-1.5 hover:bg-yellow-500/20 rounded text-yellow-400 transition-colors" title="Тестировать">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                  </button>
                  <button onclick={() => handleDeleteProxy(proxy.id)} class="p-1.5 hover:bg-red-500/20 rounded text-red-400 transition-colors" title="Удалить">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>

<!-- Add Proxy Modal -->
<Modal bind:open={showAddModal} title="Добавить прокси">
  <form onsubmit={(e) => { e.preventDefault(); handleAddProxy(); }} class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-gray-300 mb-1">Тип</label>
      <select bind:value={formType} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent">
        <option value="vless">VLESS</option>
        <option value="vmess">VMess</option>
        <option value="shadowsocks">Shadowsocks</option>
        <option value="socks5">SOCKS5</option>
        <option value="http">HTTP</option>
      </select>
    </div>
    
    <div>
      <label class="block text-sm font-medium text-gray-300 mb-1">Название</label>
      <input type="text" bind:value={formName} placeholder="My Proxy" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" required />
    </div>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Сервер</label>
        <input type="text" bind:value={formServer} placeholder="example.com" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" required />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Порт</label>
        <input type="number" bind:value={formPort} min="1" max="65535" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" required />
      </div>
    </div>
    
    {#if formType === 'vless'}
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">UUID</label>
        <input type="text" bind:value={formUuid} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent font-mono text-sm" required />
      </div>
      <div class="flex items-center gap-3">
        <input type="checkbox" id="tls" bind:checked={formTls} class="w-4 h-4 rounded bg-gray-700 border-gray-600 text-[#00d4ff] focus:ring-[#00d4ff]" />
        <label for="tls" class="text-sm text-gray-300">TLS</label>
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">SNI</label>
        <input type="text" bind:value={formSni} placeholder="example.com" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Transport</label>
        <select bind:value={formTransport} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent">
          {#each transportOptions as opt}
            <option value={opt}>{opt.toUpperCase()}</option>
          {/each}
        </select>
      </div>
    {/if}
    
    {#if formType === 'vmess'}
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">UUID</label>
        <input type="text" bind:value={formUuid} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent font-mono text-sm" required />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Alter ID</label>
        <input type="number" bind:value={formAlterId} min="0" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Security</label>
        <select bind:value={formSecurity} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent">
          {#each vmessSecurityOptions as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>
    {/if}
    
    {#if formType === 'shadowsocks'}
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Метод шифрования</label>
        <select bind:value={formMethod} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent">
          {#each ssMethodOptions as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Пароль</label>
        <input type="password" bind:value={formPassword} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" required />
      </div>
    {/if}
    
    <div class="flex justify-end gap-3 pt-4">
      <Button variant="secondary" onclick={() => showAddModal = false}>Отмена</Button>
      <Button variant="primary" onclick={handleAddProxy}>Сохранить</Button>
    </div>
  </form>
</Modal>

<!-- Edit Proxy Modal -->
<Modal bind:open={showEditModal} title="Редактировать прокси">
  <form onsubmit={(e) => { e.preventDefault(); handleUpdateProxy(); }} class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-gray-300 mb-1">Тип</label>
      <select bind:value={formType} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent">
        <option value="vless">VLESS</option>
        <option value="vmess">VMess</option>
        <option value="shadowsocks">Shadowsocks</option>
        <option value="socks5">SOCKS5</option>
        <option value="http">HTTP</option>
      </select>
    </div>
    
    <div>
      <label class="block text-sm font-medium text-gray-300 mb-1">Название</label>
      <input type="text" bind:value={formName} placeholder="My Proxy" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" required />
    </div>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Сервер</label>
        <input type="text" bind:value={formServer} placeholder="example.com" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" required />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Порт</label>
        <input type="number" bind:value={formPort} min="1" max="65535" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" required />
      </div>
    </div>
    
    {#if formType === 'vless'}
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">UUID</label>
        <input type="text" bind:value={formUuid} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent font-mono text-sm" required />
      </div>
      <div class="flex items-center gap-3">
        <input type="checkbox" id="edit-tls" bind:checked={formTls} class="w-4 h-4 rounded bg-gray-700 border-gray-600 text-[#00d4ff] focus:ring-[#00d4ff]" />
        <label for="edit-tls" class="text-sm text-gray-300">TLS</label>
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">SNI</label>
        <input type="text" bind:value={formSni} placeholder="example.com" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Transport</label>
        <select bind:value={formTransport} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent">
          {#each transportOptions as opt}
            <option value={opt}>{opt.toUpperCase()}</option>
          {/each}
        </select>
      </div>
    {/if}
    
    {#if formType === 'vmess'}
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">UUID</label>
        <input type="text" bind:value={formUuid} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent font-mono text-sm" required />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Alter ID</label>
        <input type="number" bind:value={formAlterId} min="0" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" />
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Security</label>
        <select bind:value={formSecurity} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent">
          {#each vmessSecurityOptions as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>
    {/if}
    
    {#if formType === 'shadowsocks'}
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Метод шифрования</label>
        <select bind:value={formMethod} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent">
          {#each ssMethodOptions as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">Пароль</label>
        <input type="password" bind:value={formPassword} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent" required />
      </div>
    {/if}
    
    <div class="flex justify-end gap-3 pt-4">
      <Button variant="secondary" onclick={() => { showEditModal = false; editingProxy = null; }}>Отмена</Button>
      <Button variant="primary" onclick={handleUpdateProxy}>Сохранить</Button>
    </div>
  </form>
</Modal>

<!-- Import URL Modal -->
<Modal bind:open={showImportUrlModal} title="Импорт из URL">
  <form onsubmit={(e) => { e.preventDefault(); handleImportUrl(); }} class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-gray-300 mb-1">URL прокси</label>
      <input 
        type="text" 
        bind:value={importUrl} 
        placeholder="vless://... или vmess://... или ss://..."
        class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent font-mono text-sm"
        required 
      />
      <p class="mt-1 text-xs text-gray-500">Поддерживаются форматы: VLESS, VMess, Shadowsocks</p>
    </div>
    <div class="flex justify-end gap-3 pt-4">
      <Button variant="secondary" onclick={() => showImportUrlModal = false}>Отмена</Button>
      <Button variant="primary" loading={importLoading} onclick={handleImportUrl}>Импортировать</Button>
    </div>
  </form>
</Modal>

<!-- Import Subscription Modal -->
<Modal bind:open={showImportSubModal} title="Импорт подписки">
  <form onsubmit={(e) => { e.preventDefault(); handleImportSubscription(); }} class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-gray-300 mb-1">URL подписки</label>
      <input 
        type="url" 
        bind:value={importUrl} 
        placeholder="https://example.com/subscription"
        class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:ring-2 focus:ring-[#00d4ff] focus:border-transparent"
        required 
      />
      <p class="mt-1 text-xs text-gray-500">Подписка будет автоматически обновляться</p>
    </div>
    <div class="flex justify-end gap-3 pt-4">
      <Button variant="secondary" onclick={() => showImportSubModal = false}>Отмена</Button>
      <Button variant="primary" loading={importLoading} onclick={handleImportSubscription}>Импортировать</Button>
    </div>
  </form>
</Modal>
