<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { Modal, Button, Spinner, ProxyCard } from '$lib/components';
  import { toasts } from '$lib/stores/toast';

  // Types - matches Rust ProxyConfig
  type ProxyProtocol = 'vless' | 'vmess' | 'shadowsocks' | 'socks5' | 'http' | 'trojan';
  
  interface ProxyConfig {
    id: string;
    name: string;
    protocol: ProxyProtocol;
    server: string;
    port: number;
    username?: string;
    password?: string;
    uuid?: string;
    tls: boolean;
    sni?: string;
    transport?: string;
    custom_fields?: Record<string, string>;
    active: boolean;
    country?: string;
    ping?: number;
  }

  // Country detection by server hostname/domain
  function getCountryFromServer(server: string): string | null {
    const s = server.toLowerCase();
    
    // Country code patterns in hostname
    const ccPatterns: [RegExp, string][] = [
      [/\.us\d*\.|us-|usa-|america|united.?states/i, 'US'],
      [/\.de\d*\.|de-|germany|deutschland/i, 'DE'],
      [/\.nl\d*\.|nl-|netherlands|holland/i, 'NL'],
      [/\.gb\d*\.|\.uk\d*\.|uk-|gb-|britain|england/i, 'GB'],
      [/\.fr\d*\.|fr-|france/i, 'FR'],
      [/\.jp\d*\.|jp-|japan|tokyo/i, 'JP'],
      [/\.sg\d*\.|sg-|singapore/i, 'SG'],
      [/\.hk\d*\.|hk-|hongkong|hong.?kong/i, 'HK'],
      [/\.kr\d*\.|kr-|korea|seoul/i, 'KR'],
      [/\.ca\d*\.|ca-|canada|toronto|vancouver/i, 'CA'],
      [/\.au\d*\.|au-|australia|sydney|melbourne/i, 'AU'],
      [/\.ru\d*\.|ru-|russia|moscow/i, 'RU'],
      [/\.ua\d*\.|ua-|ukraine|kiev|kyiv/i, 'UA'],
      [/\.pl\d*\.|pl-|poland|warsaw/i, 'PL'],
      [/\.fi\d*\.|fi-|finland|helsinki/i, 'FI'],
      [/\.se\d*\.|se-|sweden|stockholm/i, 'SE'],
      [/\.ch\d*\.|ch-|switzerland|zurich/i, 'CH'],
      [/\.at\d*\.|at-|austria|vienna/i, 'AT'],
      [/\.it\d*\.|it-|italy|milan|rome/i, 'IT'],
      [/\.es\d*\.|es-|spain|madrid/i, 'ES'],
      [/\.tr\d*\.|tr-|turkey|istanbul/i, 'TR'],
      [/\.in\d*\.|in-|india|mumbai/i, 'IN'],
      [/\.br\d*\.|br-|brazil|saopaulo/i, 'BR'],
      [/\.mx\d*\.|mx-|mexico/i, 'MX'],
      [/\.tw\d*\.|tw-|taiwan|taipei/i, 'TW'],
      [/\.id\d*\.|id-|indonesia|jakarta/i, 'ID'],
      [/\.th\d*\.|th-|thailand|bangkok/i, 'TH'],
      [/\.vn\d*\.|vn-|vietnam|hanoi/i, 'VN'],
      [/\.my\d*\.|my-|malaysia|kuala/i, 'MY'],
      [/\.ph\d*\.|ph-|philippines|manila/i, 'PH'],
      [/\.ae\d*\.|ae-|uae|dubai|emirates/i, 'AE'],
      [/\.il\d*\.|il-|israel|telaviv/i, 'IL'],
      [/\.za\d*\.|za-|southafrica|johannesburg/i, 'ZA'],
      [/\.ar\d*\.|ar-|argentina|buenos/i, 'AR'],
      [/\.cl\d*\.|cl-|chile|santiago/i, 'CL'],
      [/\.no\d*\.|no-|norway|oslo/i, 'NO'],
      [/\.dk\d*\.|dk-|denmark|copenhagen/i, 'DK'],
      [/\.ie\d*\.|ie-|ireland|dublin/i, 'IE'],
      [/\.pt\d*\.|pt-|portugal|lisbon/i, 'PT'],
      [/\.cz\d*\.|cz-|czech|prague/i, 'CZ'],
      [/\.ro\d*\.|ro-|romania|bucharest/i, 'RO'],
      [/\.hu\d*\.|hu-|hungary|budapest/i, 'HU'],
      [/\.bg\d*\.|bg-|bulgaria|sofia/i, 'BG'],
      [/\.gr\d*\.|gr-|greece|athens/i, 'GR'],
      [/\.be\d*\.|be-|belgium|brussels/i, 'BE'],
      [/\.lu\d*\.|lu-|luxembourg/i, 'LU'],
      [/\.ee\d*\.|ee-|estonia|tallinn/i, 'EE'],
      [/\.lv\d*\.|lv-|latvia|riga/i, 'LV'],
      [/\.lt\d*\.|lt-|lithuania|vilnius/i, 'LT'],
      [/\.sk\d*\.|sk-|slovakia|bratislava/i, 'SK'],
      [/\.si\d*\.|si-|slovenia|ljubljana/i, 'SI'],
      [/\.hr\d*\.|hr-|croatia|zagreb/i, 'HR'],
      [/\.rs\d*\.|rs-|serbia|belgrade/i, 'RS'],
      [/\.md\d*\.|md-|moldova|chisinau/i, 'MD'],
      [/\.by\d*\.|by-|belarus|minsk/i, 'BY'],
      [/\.kz\d*\.|kz-|kazakhstan|almaty/i, 'KZ'],
      [/\.ge\d*\.|ge-|georgia|tbilisi/i, 'GE'],
      [/\.am\d*\.|am-|armenia|yerevan/i, 'AM'],
      [/\.az\d*\.|az-|azerbaijan|baku/i, 'AZ'],
    ];
    
    for (const [pattern, code] of ccPatterns) {
      if (pattern.test(s)) return code;
    }
    
    // Check TLD
    const tldMatch = s.match(/\.([a-z]{2})$/);
    if (tldMatch) {
      const tld = tldMatch[1].toUpperCase();
      // Common country TLDs
      const tldCountries: Record<string, string> = {
        'DE': 'DE', 'NL': 'NL', 'FR': 'FR', 'UK': 'GB', 'JP': 'JP',
        'KR': 'KR', 'RU': 'RU', 'UA': 'UA', 'PL': 'PL', 'SE': 'SE',
        'FI': 'FI', 'NO': 'NO', 'DK': 'DK', 'CH': 'CH', 'AT': 'AT',
        'IT': 'IT', 'ES': 'ES', 'PT': 'PT', 'BR': 'BR', 'MX': 'MX',
        'CA': 'CA', 'AU': 'AU', 'NZ': 'NZ', 'SG': 'SG', 'HK': 'HK',
        'TW': 'TW', 'IN': 'IN', 'ID': 'ID', 'TH': 'TH', 'VN': 'VN',
        'MY': 'MY', 'PH': 'PH', 'TR': 'TR', 'IL': 'IL', 'AE': 'AE',
        'ZA': 'ZA', 'AR': 'AR', 'CL': 'CL', 'CO': 'CO', 'PE': 'PE',
        'CZ': 'CZ', 'RO': 'RO', 'HU': 'HU', 'BG': 'BG', 'GR': 'GR',
        'BE': 'BE', 'IE': 'IE', 'LU': 'LU', 'EE': 'EE', 'LV': 'LV',
        'LT': 'LT', 'SK': 'SK', 'SI': 'SI', 'HR': 'HR', 'RS': 'RS',
      };
      if (tldCountries[tld]) return tldCountries[tld];
    }
    
    return null;
  }

  // State
  let proxies = $state<ProxyConfig[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let initialized = $state(false);
  
  // Modal states
  let showAddModal = $state(false);
  let showEditModal = $state(false);
  let showSubscriptionModal = $state(false);
  let editingProxy = $state<ProxyConfig | null>(null);
  
  // Subscription import state
  let subscriptionUrl = $state('');
  let subscriptionLoading = $state(false);
  let subscriptionError = $state<string | null>(null);
  
  // Add modal tab state
  type AddTab = 'paste' | 'file' | 'manual';
  let addTab = $state<AddTab>('paste');
  
  // Paste tab state
  let pasteUrl = $state('');
  let pasteLoading = $state(false);

  // Manual form state
  let formProtocol = $state<ProxyProtocol>('vless');
  let formName = $state('');
  let formServer = $state('');
  let formPort = $state(443);
  let formUuid = $state('');
  let formTls = $state(true);
  let formSni = $state('');
  let formTransport = $state('tcp');
  let formAlterId = $state(0);
  let formSecurity = $state('auto');
  let formMethod = $state('aes-256-gcm');
  let formPassword = $state('');
  
  let isTauri = $state(false);

  // Supported proxy URL protocols
  const PROXY_URL_REGEX = /^(vless|vmess|ss|trojan):\/\/.+/i;

  function parseProxyUrl(text: string): string | null {
    const trimmed = text.trim();
    if (PROXY_URL_REGEX.test(trimmed)) {
      return trimmed;
    }
    return null;
  }

  async function checkClipboardForProxy() {
    if (!browser) return;
    try {
      const text = await navigator.clipboard.readText();
      const proxyUrl = parseProxyUrl(text);
      if (proxyUrl) {
        pasteUrl = proxyUrl;
        console.log('[Proxies] Auto-pasted proxy URL from clipboard');
      }
    } catch (e) {
      console.log('[Proxies] Could not read clipboard:', e);
    }
  }

  function handlePaste(event: ClipboardEvent) {
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      return;
    }

    const clipboardText = event.clipboardData?.getData('text');
    if (!clipboardText) return;

    const proxyUrl = parseProxyUrl(clipboardText);
    if (!proxyUrl) return;

    event.preventDefault();
    console.log('[Proxies] Detected proxy URL in paste');

    (async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('import_proxy_url', { url: proxyUrl });
        toasts.success('Прокси импортирован из буфера обмена');
        await loadProxies();
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        toasts.error(`Ошибка импорта: ${errorMessage}`);
      }
    })();
  }

  let pasteListenerAdded = false;

  $effect(() => {
    if (!browser || initialized) return;
    initialized = true;
    
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    if (!isTauri) {
      loading = false;
    } else {
      loadProxies();
    }
  });

  async function loadProxies() {
    if (!pasteListenerAdded && browser) {
      document.addEventListener('paste', handlePaste as EventListener);
      pasteListenerAdded = true;
    }
    
    loading = true;
    error = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const rawProxies = await invoke<ProxyConfig[]>('get_proxies');
      // Enrich proxies with detected country if not set
      proxies = rawProxies.map(p => ({
        ...p,
        country: p.country || getCountryFromServer(p.server)
      }));
    } catch (e) {
      console.error('[Proxies] Failed to load:', e);
      proxies = [];
    } finally {
      loading = false;
    }
  }

  // Parse base64 subscription content
  function parseSubscriptionContent(content: string): string[] {
    const lines: string[] = [];
    
    // Try to decode base64
    let decoded = content;
    try {
      // Check if it's base64 encoded
      if (/^[A-Za-z0-9+/=\s]+$/.test(content.trim())) {
        decoded = atob(content.trim().replace(/\s/g, ''));
      }
    } catch {
      // Not base64, use as-is
    }
    
    // Split by newlines and filter valid proxy URLs
    const rawLines = decoded.split(/[\r\n]+/);
    for (const line of rawLines) {
      const trimmed = line.trim();
      if (PROXY_URL_REGEX.test(trimmed)) {
        lines.push(trimmed);
      }
    }
    
    return lines;
  }

  async function handleSubscriptionImport() {
    if (!subscriptionUrl.trim()) return;
    
    subscriptionLoading = true;
    subscriptionError = null;
    
    try {
      // Fetch subscription content
      const response = await fetch(subscriptionUrl.trim());
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const content = await response.text();
      const proxyUrls = parseSubscriptionContent(content);
      
      if (proxyUrls.length === 0) {
        subscriptionError = 'Подписка не содержит прокси-ссылок';
        return;
      }
      
      const { invoke } = await import('@tauri-apps/api/core');
      let imported = 0;
      let failed = 0;
      
      for (const url of proxyUrls) {
        try {
          await invoke('import_proxy_url', { url });
          imported++;
        } catch (e) {
          console.error('[Subscription] Failed to import:', url, e);
          failed++;
        }
      }
      
      if (imported > 0) {
        toasts.success(`Импортировано ${imported} прокси${failed > 0 ? `, ${failed} ошибок` : ''}`);
        showSubscriptionModal = false;
        subscriptionUrl = '';
        await loadProxies();
      } else {
        subscriptionError = 'Не удалось импортировать ни одного прокси';
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      subscriptionError = `Ошибка загрузки: ${msg}`;
    } finally {
      subscriptionLoading = false;
    }
  }

  function openSubscriptionModal() {
    subscriptionUrl = '';
    subscriptionError = null;
    showSubscriptionModal = true;
  }

  function openAddModal() {
    addTab = 'paste';
    pasteUrl = '';
    resetForm();
    showAddModal = true;
    checkClipboardForProxy();
  }

  async function handlePasteImport() {
    if (!pasteUrl.trim()) return;
    pasteLoading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('import_proxy_url', { url: pasteUrl });
      toasts.success('Прокси импортирован');
      showAddModal = false;
      pasteUrl = '';
      await loadProxies();
    } catch (e) {
      toasts.error(`Ошибка импорта: ${e}`);
    } finally {
      pasteLoading = false;
    }
  }

  async function handleFileImport(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    try {
      const text = await file.text();
      const lines = text.split('\n').filter(line => PROXY_URL_REGEX.test(line.trim()));
      
      if (lines.length === 0) {
        toasts.error('Файл не содержит прокси-ссылок');
        return;
      }

      const { invoke } = await import('@tauri-apps/api/core');
      let imported = 0;
      for (const line of lines) {
        try {
          await invoke('import_proxy_url', { url: line.trim() });
          imported++;
        } catch (e) {
          console.error('[Proxies] Failed to import line:', e);
        }
      }
      
      toasts.success(`Импортировано ${imported} из ${lines.length} прокси`);
      showAddModal = false;
      await loadProxies();
    } catch (e) {
      toasts.error(`Ошибка чтения файла: ${e}`);
    }
  }

  async function handleAddProxy() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const proxy: Partial<ProxyConfig> = {
        name: formName,
        protocol: formProtocol,
        server: formServer,
        port: formPort,
      };
      
      if (formProtocol === 'vless') {
        proxy.uuid = formUuid;
        proxy.tls = formTls;
        proxy.sni = formSni;
        proxy.transport = formTransport;
      } else if (formProtocol === 'vmess') {
        proxy.uuid = formUuid;
      } else if (formProtocol === 'shadowsocks') {
        proxy.password = formPassword;
      }
      
      await invoke('add_proxy', { proxy });
      toasts.success('Прокси добавлен');
      showAddModal = false;
      resetForm();
      await loadProxies();
    } catch (e) {
      toasts.error(`Ошибка: ${e}`);
    }
  }

  async function handleUpdateProxy() {
    if (!editingProxy) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const proxy: Partial<ProxyConfig> = {
        id: editingProxy.id,
        name: formName,
        protocol: formProtocol,
        server: formServer,
        port: formPort,
      };
      
      if (formProtocol === 'vless') {
        proxy.uuid = formUuid;
        proxy.tls = formTls;
        proxy.sni = formSni;
        proxy.transport = formTransport;
      } else if (formProtocol === 'vmess') {
        proxy.uuid = formUuid;
      } else if (formProtocol === 'shadowsocks') {
        proxy.password = formPassword;
      }
      
      await invoke('update_proxy', { proxy });
      toasts.success('Прокси обновлён');
      showEditModal = false;
      editingProxy = null;
      resetForm();
      await loadProxies();
    } catch (e) {
      toasts.error(`Ошибка: ${e}`);
    }
  }

  async function handleDeleteProxy(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_proxy', { id });
      toasts.success('Прокси удалён');
      await loadProxies();
    } catch (e) {
      toasts.error(`Ошибка: ${e}`);
    }
  }

  async function handleToggleProxy(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('apply_proxy', { id });
      toasts.success('Прокси применён');
      await loadProxies();
    } catch (e) {
      toasts.error(`Ошибка: ${e}`);
    }
  }

  function openEditModal(proxy: ProxyConfig) {
    editingProxy = proxy;
    formProtocol = proxy.protocol;
    formName = proxy.name;
    formServer = proxy.server;
    formPort = proxy.port;
    formUuid = proxy.uuid || '';
    formTls = proxy.tls ?? true;
    formSni = proxy.sni || '';
    formTransport = proxy.transport || 'tcp';
    formPassword = proxy.password || '';
    showEditModal = true;
  }

  function resetForm() {
    formProtocol = 'vless';
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

  function getProtocolDisplay(protocol: ProxyProtocol): string {
    const map: Record<ProxyProtocol, string> = {
      vless: 'VLESS',
      vmess: 'VMess',
      shadowsocks: 'Shadowsocks',
      trojan: 'Trojan',
      socks5: 'SOCKS5',
      http: 'HTTP'
    };
    return map[protocol] || protocol.toUpperCase();
  }

  const ssMethodOptions = [
    'aes-128-gcm', 'aes-256-gcm', 'chacha20-ietf-poly1305',
    '2022-blake3-aes-128-gcm', '2022-blake3-aes-256-gcm', '2022-blake3-chacha20-poly1305'
  ];

  const transportOptions = ['tcp', 'ws', 'grpc', 'http'];
</script>

<div class="min-h-screen bg-zinc-950 p-8 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-4">
      <button 
        onclick={() => goto('/')} 
        class="p-2 bg-zinc-900/40 border border-white/5 hover:border-white/10 rounded-xl transition-all duration-200 hover:-translate-y-0.5"
      >
        <svg class="w-5 h-5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h1 class="text-3xl font-bold text-zinc-100">Proxies</h1>
    </div>
    <div class="flex items-center gap-3">
      <button 
        onclick={openSubscriptionModal}
        class="flex items-center gap-2 px-4 py-2.5 bg-zinc-900/60 border border-white/5 hover:border-emerald-500/30 hover:bg-emerald-500/10 text-zinc-300 hover:text-emerald-400 font-medium text-sm rounded-xl transition-all duration-200 hover:-translate-y-0.5"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
        </svg>
        Import Subscription
      </button>
      <button 
        onclick={openAddModal}
        class="flex items-center gap-2 px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 text-white font-medium text-sm rounded-xl transition-all duration-200 hover:-translate-y-0.5"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Add
      </button>
    </div>
  </div>

  <!-- Proxy Cards List -->
  <div class="space-y-3">
    {#if loading}
      <div class="flex items-center justify-center py-12">
        <Spinner />
      </div>
    {:else if error}
      <div class="text-center py-12 text-red-400">{error}</div>
    {:else if proxies.length === 0}
      <div class="text-center py-12 bg-zinc-900/40 border border-white/5 border-t-white/10 rounded-xl">
        <div class="text-4xl mb-4">🌐</div>
        <p class="text-lg text-zinc-300">Нет добавленных прокси</p>
        <p class="text-sm mt-2 text-zinc-500">Нажмите Add или вставьте ссылку (Ctrl+V)</p>
      </div>
    {:else}
      {#each proxies as proxy, i (proxy.id)}
        <div class="transform transition-all duration-200 hover:-translate-y-0.5" style="animation-delay: {i * 50}ms">
          <ProxyCard
            id={proxy.id}
            name={proxy.name}
            server={proxy.server}
            port={proxy.port}
            protocol={getProtocolDisplay(proxy.protocol)}
            country={proxy.country}
            ping={proxy.ping}
            active={proxy.active}
            onEdit={() => openEditModal(proxy)}
            onDelete={() => handleDeleteProxy(proxy.id)}
            onToggle={() => handleToggleProxy(proxy.id)}
          />
        </div>
      {/each}
    {/if}
  </div>
</div>

<!-- Add Proxy Modal with Tabs -->
<Modal bind:open={showAddModal} title="Add Proxy">
  <!-- Tabs -->
  <div class="flex gap-1 mb-6 p-1 bg-zinc-900/60 rounded-xl">
    <button
      class="flex-1 px-4 py-2.5 rounded-lg text-sm font-medium transition-all duration-200
        {addTab === 'paste' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60'}"
      onclick={() => { addTab = 'paste'; checkClipboardForProxy(); }}
    >
      Paste Link
    </button>
    <button
      class="flex-1 px-4 py-2.5 rounded-lg text-sm font-medium transition-all duration-200
        {addTab === 'file' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60'}"
      onclick={() => addTab = 'file'}
    >
      Import File
    </button>
    <button
      class="flex-1 px-4 py-2.5 rounded-lg text-sm font-medium transition-all duration-200
        {addTab === 'manual' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60'}"
      onclick={() => addTab = 'manual'}
    >
      Manual
    </button>
  </div>

  <!-- Tab Content: Paste Link -->
  {#if addTab === 'paste'}
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-2">Proxy URL</label>
        <textarea
          bind:value={pasteUrl}
          placeholder="vless://... или vmess://... или ss://..."
          rows="4"
          class="w-full px-4 py-3 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 focus:ring-1 focus:ring-white/5 font-mono text-sm resize-none transition-all duration-200"
        ></textarea>
        <p class="mt-2 text-xs text-zinc-500">Поддерживаются: VLESS, VMess, Shadowsocks, Trojan</p>
      </div>
      <div class="flex justify-end gap-3">
        <Button variant="secondary" onclick={() => showAddModal = false}>Cancel</Button>
        <button 
          onclick={handlePasteImport}
          disabled={pasteLoading}
          class="px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white font-medium text-sm rounded-xl transition-all duration-200"
        >
          {pasteLoading ? 'Importing...' : 'Import'}
        </button>
      </div>
    </div>
  {/if}

  <!-- Tab Content: Import File -->
  {#if addTab === 'file'}
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-2">Select File</label>
        <div class="border-2 border-dashed border-white/10 rounded-xl p-8 text-center hover:border-indigo-500/30 transition-all duration-200">
          <input
            type="file"
            accept=".txt,.conf,.json"
            onchange={handleFileImport}
            class="hidden"
            id="file-input"
          />
          <label for="file-input" class="cursor-pointer">
            <svg class="w-12 h-12 mx-auto text-zinc-500 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            <p class="text-zinc-400">Нажмите для выбора файла</p>
            <p class="text-xs text-zinc-600 mt-1">.txt, .conf, .json</p>
          </label>
        </div>
      </div>
      <div class="flex justify-end">
        <Button variant="secondary" onclick={() => showAddModal = false}>Cancel</Button>
      </div>
    </div>
  {/if}

  <!-- Tab Content: Manual -->
  {#if addTab === 'manual'}
    <form onsubmit={(e) => { e.preventDefault(); handleAddProxy(); }} class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-1">Protocol</label>
        <select bind:value={formProtocol} class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 transition-all duration-200">
          <option value="vless">VLESS</option>
          <option value="vmess">VMess</option>
          <option value="shadowsocks">Shadowsocks</option>
          <option value="socks5">SOCKS5</option>
          <option value="http">HTTP</option>
        </select>
      </div>
      
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-1">Name</label>
        <input type="text" bind:value={formName} placeholder="My Proxy" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 transition-all duration-200" required />
      </div>
      
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-zinc-400 mb-1">Server</label>
          <input type="text" bind:value={formServer} placeholder="example.com" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 font-mono transition-all duration-200" required />
        </div>
        <div>
          <label class="block text-sm font-medium text-zinc-400 mb-1">Port</label>
          <input type="number" bind:value={formPort} min="1" max="65535" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 font-mono transition-all duration-200" required />
        </div>
      </div>
      
      {#if formProtocol === 'vless' || formProtocol === 'vmess'}
        <div>
          <label class="block text-sm font-medium text-zinc-400 mb-1">UUID</label>
          <input type="text" bind:value={formUuid} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 font-mono text-sm transition-all duration-200" required />
        </div>
      {/if}
      
      {#if formProtocol === 'vless'}
        <div class="flex items-center gap-3">
          <input type="checkbox" id="tls" bind:checked={formTls} class="w-4 h-4 rounded bg-zinc-900/60 border-white/10 text-indigo-500 focus:ring-indigo-500/30 transition-all duration-200" />
          <label for="tls" class="text-sm text-zinc-400">TLS</label>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-zinc-400 mb-1">SNI</label>
            <input type="text" bind:value={formSni} placeholder="example.com" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 font-mono transition-all duration-200" />
          </div>
          <div>
            <label class="block text-sm font-medium text-zinc-400 mb-1">Transport</label>
            <select bind:value={formTransport} class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 transition-all duration-200">
              {#each transportOptions as opt}
                <option value={opt}>{opt.toUpperCase()}</option>
              {/each}
            </select>
          </div>
        </div>
      {/if}
      
      {#if formProtocol === 'shadowsocks'}
        <div>
          <label class="block text-sm font-medium text-zinc-400 mb-1">Encryption Method</label>
          <select bind:value={formMethod} class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 transition-all duration-200">
            {#each ssMethodOptions as opt}
              <option value={opt}>{opt}</option>
            {/each}
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-zinc-400 mb-1">Password</label>
          <input type="password" bind:value={formPassword} class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 transition-all duration-200" required />
        </div>
      {/if}
      
      <div class="flex justify-end gap-3 pt-4">
        <Button variant="secondary" onclick={() => showAddModal = false}>Cancel</Button>
        <button type="submit" class="px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 text-white font-medium text-sm rounded-xl transition-all duration-200">Save</button>
      </div>
    </form>
  {/if}
</Modal>

<!-- Edit Proxy Modal -->
<Modal bind:open={showEditModal} title="Edit Proxy">
  <form onsubmit={(e) => { e.preventDefault(); handleUpdateProxy(); }} class="space-y-4">
    <div>
      <label class="block text-sm font-medium text-zinc-400 mb-1">Protocol</label>
      <select bind:value={formProtocol} class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 transition-all duration-200">
        <option value="vless">VLESS</option>
        <option value="vmess">VMess</option>
        <option value="shadowsocks">Shadowsocks</option>
        <option value="socks5">SOCKS5</option>
        <option value="http">HTTP</option>
      </select>
    </div>
    
    <div>
      <label class="block text-sm font-medium text-zinc-400 mb-1">Name</label>
      <input type="text" bind:value={formName} placeholder="My Proxy" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 transition-all duration-200" required />
    </div>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-1">Server</label>
        <input type="text" bind:value={formServer} placeholder="example.com" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 font-mono transition-all duration-200" required />
      </div>
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-1">Port</label>
        <input type="number" bind:value={formPort} min="1" max="65535" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 font-mono transition-all duration-200" required />
      </div>
    </div>
    
    {#if formProtocol === 'vless' || formProtocol === 'vmess'}
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-1">UUID</label>
        <input type="text" bind:value={formUuid} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 font-mono text-sm transition-all duration-200" required />
      </div>
    {/if}
    
    {#if formProtocol === 'vless'}
      <div class="flex items-center gap-3">
        <input type="checkbox" id="edit-tls" bind:checked={formTls} class="w-4 h-4 rounded bg-zinc-900/60 border-white/10 text-indigo-500 focus:ring-indigo-500/30 transition-all duration-200" />
        <label for="edit-tls" class="text-sm text-zinc-400">TLS</label>
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-zinc-400 mb-1">SNI</label>
          <input type="text" bind:value={formSni} placeholder="example.com" class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-white/10 font-mono transition-all duration-200" />
        </div>
        <div>
          <label class="block text-sm font-medium text-zinc-400 mb-1">Transport</label>
          <select bind:value={formTransport} class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 transition-all duration-200">
            {#each transportOptions as opt}
              <option value={opt}>{opt.toUpperCase()}</option>
            {/each}
          </select>
        </div>
      </div>
    {/if}
    
    {#if formProtocol === 'shadowsocks'}
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-1">Encryption Method</label>
        <select bind:value={formMethod} class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 transition-all duration-200">
          {#each ssMethodOptions as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>
      <div>
        <label class="block text-sm font-medium text-zinc-400 mb-1">Password</label>
        <input type="password" bind:value={formPassword} class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 focus:outline-none focus:border-white/10 transition-all duration-200" required />
      </div>
    {/if}
    
    <div class="flex justify-end gap-3 pt-4">
      <Button variant="secondary" onclick={() => { showEditModal = false; editingProxy = null; }}>Cancel</Button>
      <button type="submit" class="px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 text-white font-medium text-sm rounded-xl transition-all duration-200">Save</button>
    </div>
  </form>
</Modal>

<!-- Subscription Import Modal -->
<Modal bind:open={showSubscriptionModal} title="Import Subscription">
  <div class="space-y-4">
    <div class="p-4 bg-emerald-500/10 border border-emerald-500/20 rounded-xl">
      <div class="flex items-start gap-3">
        <svg class="w-5 h-5 text-emerald-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <div class="text-sm text-emerald-300/90">
          <p class="font-medium mb-1">Что такое подписка?</p>
          <p class="text-emerald-300/70">Подписка — это URL, который возвращает список прокси-серверов. Обычно предоставляется VPN-провайдером и автоматически обновляется.</p>
        </div>
      </div>
    </div>
    
    <div>
      <label class="block text-sm font-medium text-zinc-400 mb-2">Subscription URL</label>
      <input
        type="url"
        bind:value={subscriptionUrl}
        placeholder="https://example.com/subscription/..."
        class="w-full px-4 py-3 bg-zinc-900/60 border border-white/5 rounded-xl text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-emerald-500/30 focus:ring-1 focus:ring-emerald-500/20 font-mono text-sm transition-all duration-200"
      />
      <p class="mt-2 text-xs text-zinc-500">Поддерживаются: base64-encoded списки, plain text со ссылками</p>
    </div>
    
    {#if subscriptionError}
      <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-xl">
        <p class="text-sm text-red-400">{subscriptionError}</p>
      </div>
    {/if}
    
    <div class="flex justify-end gap-3 pt-2">
      <Button variant="secondary" onclick={() => showSubscriptionModal = false}>Cancel</Button>
      <button 
        onclick={handleSubscriptionImport}
        disabled={subscriptionLoading || !subscriptionUrl.trim()}
        class="flex items-center gap-2 px-4 py-2.5 bg-emerald-500 hover:bg-emerald-600 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium text-sm rounded-xl transition-all duration-200"
      >
        {#if subscriptionLoading}
          <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Importing...
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
          </svg>
          Import
        {/if}
      </button>
    </div>
  </div>
</Modal>
