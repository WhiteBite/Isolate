<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import Toggle from '../Toggle.svelte';
  import type { ProxyConfig, ProxyProtocol } from '$lib/api';
  import { importSubscription } from '$lib/api';
  import { getProxyFlag, getProxyCountryName } from '$lib/utils/countries';

  interface Props {
    open: boolean;
    onclose: () => void;
    onadd: (gateway: Omit<ProxyConfig, 'id' | 'active'>) => void;
  }

  let { open, onclose, onadd }: Props = $props();

  // Tab state
  type TabType = 'single' | 'subscription';
  let activeTab = $state<TabType>('single');

  // Form state (Single URL tab)
  let urlInput = $state('');
  let name = $state('');
  let protocol = $state<ProxyProtocol>('vless');
  let server = $state('');
  let port = $state(443);
  let password = $state('');
  let uuid = $state('');
  let tls = $state(true);
  let sni = $state('');
  let transport = $state('');

  // UI state (Single URL tab)
  let parseError = $state('');
  let showManualFields = $state(false);

  // Subscription tab state
  let subscriptionUrl = $state('');
  let subscriptionLoading = $state(false);
  let subscriptionError = $state('');
  let fetchedProxies = $state<ProxyConfig[]>([]);
  let selectedProxyIds = $state<Set<string>>(new Set());

  // Derived state for subscription
  let selectedCount = $derived(selectedProxyIds.size);
  let allSelected = $derived(fetchedProxies.length > 0 && selectedProxyIds.size === fetchedProxies.length);

  // Derived state
  let needsAuth = $derived(
    ['vless', 'vmess', 'trojan', 'shadowsocks'].includes(protocol)
  );

  let needsUuid = $derived(['vless', 'vmess'].includes(protocol));

  // Validation
  let errors = $derived({
    name: !name.trim() ? 'Name is required' : '',
    server: !server.trim() ? 'Server is required' : '',
    port: port < 1 || port > 65535 ? 'Invalid port' : '',
    auth: needsAuth && !password && !uuid ? 'Password or UUID required' : ''
  });

  let isValid = $derived(
    name.trim() !== '' &&
    server.trim() !== '' &&
    port >= 1 && port <= 65535 &&
    (!needsAuth || password || uuid)
  );

  // Protocol options
  const protocols: { value: ProxyProtocol; label: string }[] = [
    { value: 'vless', label: 'VLESS' },
    { value: 'vmess', label: 'VMess' },
    { value: 'shadowsocks', label: 'Shadowsocks' },
    { value: 'trojan', label: 'Trojan' },
    { value: 'socks5', label: 'SOCKS5' },
    { value: 'http', label: 'HTTP' },
    { value: 'https', label: 'HTTPS' },
    { value: 'hysteria2', label: 'Hysteria2' },
    { value: 'tuic', label: 'TUIC' },
  ];

  // Reset form
  function resetForm() {
    urlInput = '';
    name = '';
    protocol = 'vless';
    server = '';
    port = 443;
    password = '';
    uuid = '';
    tls = true;
    sni = '';
    transport = '';
    parseError = '';
    showManualFields = false;
    // Reset subscription state
    subscriptionUrl = '';
    subscriptionError = '';
    fetchedProxies = [];
    selectedProxyIds = new Set();
    activeTab = 'single';
  }

  // Reset subscription tab
  function resetSubscription() {
    subscriptionUrl = '';
    subscriptionError = '';
    fetchedProxies = [];
    selectedProxyIds = new Set();
  }

  // Fetch subscription
  async function fetchSubscription() {
    if (!subscriptionUrl.trim()) {
      subscriptionError = 'Please enter a subscription URL';
      return;
    }

    subscriptionLoading = true;
    subscriptionError = '';
    fetchedProxies = [];
    selectedProxyIds = new Set();

    try {
      const proxies = await importSubscription(subscriptionUrl.trim());
      fetchedProxies = proxies;
      // Select all by default
      selectedProxyIds = new Set(proxies.map(p => p.id));
    } catch (e) {
      subscriptionError = e instanceof Error ? e.message : 'Failed to fetch subscription';
    } finally {
      subscriptionLoading = false;
    }
  }

  // Toggle proxy selection
  function toggleProxySelection(id: string) {
    const newSet = new Set(selectedProxyIds);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedProxyIds = newSet;
  }

  // Toggle all proxies
  function toggleAllProxies() {
    if (allSelected) {
      selectedProxyIds = new Set();
    } else {
      selectedProxyIds = new Set(fetchedProxies.map(p => p.id));
    }
  }

  // Import selected proxies
  function importSelectedProxies() {
    const selected = fetchedProxies.filter(p => selectedProxyIds.has(p.id));
    for (const proxy of selected) {
      const gateway: Omit<ProxyConfig, 'id' | 'active'> = {
        name: proxy.name,
        protocol: proxy.protocol,
        server: proxy.server,
        port: proxy.port,
        tls: proxy.tls,
        custom_fields: proxy.custom_fields || {}
      };
      if (proxy.password) gateway.password = proxy.password;
      if (proxy.uuid) gateway.uuid = proxy.uuid;
      if (proxy.sni) gateway.sni = proxy.sni;
      if (proxy.transport) gateway.transport = proxy.transport;
      onadd(gateway);
    }
    resetForm();
    onclose();
  }

  // Get protocol display name
  function getProtocolLabel(proto: ProxyProtocol): string {
    const labels: Record<ProxyProtocol, string> = {
      vless: 'VLESS',
      vmess: 'VMess',
      shadowsocks: 'SS',
      trojan: 'Trojan',
      socks5: 'SOCKS5',
      http: 'HTTP',
      https: 'HTTPS',
      hysteria: 'Hysteria',
      hysteria2: 'Hysteria2',
      tuic: 'TUIC',
      wireguard: 'WireGuard',
      ssh: 'SSH'
    };
    return labels[proto] || proto.toUpperCase();
  }

  // Parse URL and fill fields
  function parseUrl() {
    if (!urlInput.trim()) {
      parseError = '';
      return;
    }

    try {
      const url = urlInput.trim();
      parseError = '';

      // VLESS: vless://uuid@server:port?params#name
      if (url.startsWith('vless://')) {
        const match = url.match(/^vless:\/\/([^@]+)@([^:]+):(\d+)\??([^#]*)#?(.*)$/);
        if (match) {
          uuid = match[1];
          server = match[2];
          port = parseInt(match[3]);
          name = decodeURIComponent(match[5] || `VLESS ${server}`);
          protocol = 'vless';
          const params = new URLSearchParams(match[4]);
          tls = params.get('security') === 'tls' || params.get('security') === 'reality';
          sni = params.get('sni') || params.get('host') || '';
          transport = params.get('type') || '';
          showManualFields = true;
          return;
        }
      }

      // VMess: vmess://base64
      if (url.startsWith('vmess://')) {
        const base64 = url.slice(8);
        const decoded = atob(base64);
        const config = JSON.parse(decoded);
        uuid = config.id || '';
        server = config.add || config.host || '';
        port = parseInt(config.port) || 443;
        name = config.ps || config.remarks || `VMess ${server}`;
        protocol = 'vmess';
        tls = config.tls === 'tls';
        sni = config.sni || config.host || '';
        transport = config.net || '';
        showManualFields = true;
        return;
      }

      // Shadowsocks: ss://base64@server:port#name
      if (url.startsWith('ss://')) {
        protocol = 'shadowsocks';
        const withoutPrefix = url.slice(5);
        const hashIndex = withoutPrefix.indexOf('#');
        const mainPart = hashIndex > -1 ? withoutPrefix.slice(0, hashIndex) : withoutPrefix;
        const namePart = hashIndex > -1 ? decodeURIComponent(withoutPrefix.slice(hashIndex + 1)) : '';
        if (mainPart.includes('@')) {
          const [authPart, serverPart] = mainPart.split('@');
          const [srv, prt] = serverPart.split(':');
          server = srv;
          port = parseInt(prt) || 8388;
          try {
            const decoded = atob(authPart);
            password = decoded.split(':')[1] || '';
          } catch {
            password = authPart.split(':')[1] || '';
          }
        } else {
          const decoded = atob(mainPart);
          const match = decoded.match(/^([^:]+):([^@]+)@([^:]+):(\d+)$/);
          if (match) {
            password = match[2];
            server = match[3];
            port = parseInt(match[4]);
          }
        }
        name = namePart || `SS ${server}`;
        tls = false;
        showManualFields = true;
        return;
      }

      // Trojan: trojan://password@server:port?params#name
      if (url.startsWith('trojan://')) {
        const match = url.match(/^trojan:\/\/([^@]+)@([^:]+):(\d+)\??([^#]*)#?(.*)$/);
        if (match) {
          password = match[1];
          server = match[2];
          port = parseInt(match[3]);
          name = decodeURIComponent(match[5] || `Trojan ${server}`);
          protocol = 'trojan';
          const params = new URLSearchParams(match[4]);
          tls = true;
          sni = params.get('sni') || params.get('host') || '';
          showManualFields = true;
          return;
        }
      }

      // SOCKS5: socks5://user:pass@server:port
      if (url.startsWith('socks5://') || url.startsWith('socks://')) {
        const withoutPrefix = url.replace(/^socks5?:\/\//, '');
        const hashIndex = withoutPrefix.indexOf('#');
        const mainPart = hashIndex > -1 ? withoutPrefix.slice(0, hashIndex) : withoutPrefix;
        const namePart = hashIndex > -1 ? decodeURIComponent(withoutPrefix.slice(hashIndex + 1)) : '';
        if (mainPart.includes('@')) {
          const [authPart, serverPart] = mainPart.split('@');
          password = authPart.split(':')[1] || authPart;
          const [srv, prt] = serverPart.split(':');
          server = srv;
          port = parseInt(prt) || 1080;
        } else {
          const [srv, prt] = mainPart.split(':');
          server = srv;
          port = parseInt(prt) || 1080;
        }
        protocol = 'socks5';
        name = namePart || `SOCKS5 ${server}`;
        tls = false;
        showManualFields = true;
        return;
      }

      // HTTP/HTTPS proxy
      if (url.startsWith('http://') || url.startsWith('https://')) {
        const isHttps = url.startsWith('https://');
        const withoutPrefix = url.replace(/^https?:\/\//, '');
        const hashIndex = withoutPrefix.indexOf('#');
        const mainPart = hashIndex > -1 ? withoutPrefix.slice(0, hashIndex) : withoutPrefix;
        const namePart = hashIndex > -1 ? decodeURIComponent(withoutPrefix.slice(hashIndex + 1)) : '';
        if (mainPart.includes('@')) {
          const [authPart, serverPart] = mainPart.split('@');
          password = authPart.split(':')[1] || authPart;
          const [srv, prt] = serverPart.split(':');
          server = srv;
          port = parseInt(prt) || (isHttps ? 443 : 8080);
        } else {
          const [srv, prt] = mainPart.split(':');
          server = srv;
          port = parseInt(prt) || (isHttps ? 443 : 8080);
        }
        protocol = isHttps ? 'https' : 'http';
        name = namePart || `${isHttps ? 'HTTPS' : 'HTTP'} ${server}`;
        tls = isHttps;
        showManualFields = true;
        return;
      }

      parseError = 'Unsupported URL format';
    } catch (e) {
      parseError = `Failed to parse: ${e instanceof Error ? e.message : 'Unknown error'}`;
    }
  }

  // Handle submit
  function handleSubmit() {
    if (!isValid) return;
    const gateway: Omit<ProxyConfig, 'id' | 'active'> = {
      name: name.trim(),
      protocol,
      server: server.trim(),
      port,
      tls,
      custom_fields: {}
    };
    if (password) gateway.password = password;
    if (uuid) gateway.uuid = uuid;
    if (sni) gateway.sni = sni;
    if (transport) gateway.transport = transport;
    onadd(gateway);
    resetForm();
    onclose();
  }

  function handleClose() {
    resetForm();
    onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      handleClose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Add Gateway"
    tabindex="-1"
    class="fixed inset-0 z-50"
    onkeydown={handleKeydown}
  >
    <!-- Backdrop -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      transition:fade={{ duration: 200 }}
      onclick={handleBackdropClick}
    ></div>

    <!-- Drawer Panel -->
    <div
      class="absolute right-0 top-0 h-full w-[28rem] bg-zinc-950 border-l border-white/5 shadow-2xl flex flex-col"
      transition:fly={{ x: 450, duration: 300, opacity: 1 }}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-white/5 shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-indigo-500/10 flex items-center justify-center">
            <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 4v16m8-8H4" />
            </svg>
          </div>
          <h2 class="text-lg font-semibold text-white">Add Gateway</h2>
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="p-2 hover:bg-white/5 rounded-lg transition-colors text-zinc-400 hover:text-white"
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-white/5 shrink-0">
        <button
          type="button"
          onclick={() => activeTab = 'single'}
          class="flex-1 px-4 py-3 text-sm font-medium transition-colors relative
                 {activeTab === 'single' ? 'text-white' : 'text-zinc-500 hover:text-zinc-300'}"
        >
          Single URL
          {#if activeTab === 'single'}
            <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-indigo-500"></div>
          {/if}
        </button>
        <button
          type="button"
          onclick={() => activeTab = 'subscription'}
          class="flex-1 px-4 py-3 text-sm font-medium transition-colors relative
                 {activeTab === 'subscription' ? 'text-white' : 'text-zinc-500 hover:text-zinc-300'}"
        >
          Subscription
          {#if activeTab === 'subscription'}
            <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-indigo-500"></div>
          {/if}
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4 space-y-5">
        {#if activeTab === 'single'}
        <!-- URL Import Section -->
        <div class="space-y-3">
          <label for="url-import-input" class="block text-sm font-medium text-zinc-400">Import from URL</label>
          <div class="relative">
            <input
              id="url-import-input"
              type="text"
              bind:value={urlInput}
              oninput={parseUrl}
              placeholder="vless://, vmess://, ss://, trojan://, socks5://, http://"
              class="w-full bg-zinc-900 border border-white/10 rounded-lg pl-3 pr-10 py-2.5 text-white text-sm
                     placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                     focus:border-indigo-500/50 transition-colors"
            />
            {#if urlInput}
              <button
                type="button"
                onclick={() => { urlInput = ''; parseError = ''; }}
                class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-zinc-500 hover:text-zinc-300"
                aria-label="Clear input"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            {/if}
          </div>
          {#if parseError}
            <p class="text-xs text-red-400">{parseError}</p>
          {:else if urlInput && showManualFields}
            <p class="text-xs text-emerald-400 flex items-center gap-1">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              URL parsed successfully
            </p>
          {/if}
        </div>

        <!-- Divider -->
        <div class="flex items-center gap-3">
          <div class="flex-1 h-px bg-white/5"></div>
          <span class="text-xs text-zinc-600">or enter manually</span>
          <div class="flex-1 h-px bg-white/5"></div>
        </div>

        <!-- Manual Fields -->
        <div class="space-y-4">
          <!-- Name -->
          <div class="space-y-2">
            <label for="gateway-name" class="block text-sm font-medium text-zinc-400">Name</label>
            <input
              id="gateway-name"
              type="text"
              bind:value={name}
              placeholder="My Proxy Server"
              class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                     placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                     focus:border-indigo-500/50 transition-colors"
            />
          </div>

          <!-- Protocol -->
          <div class="space-y-2">
            <label for="gateway-protocol" class="block text-sm font-medium text-zinc-400">Protocol</label>
            <select
              id="gateway-protocol"
              bind:value={protocol}
              class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                     focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50
                     transition-colors cursor-pointer appearance-none"
              style="background-image: url('data:image/svg+xml;charset=UTF-8,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27 fill=%27none%27 stroke=%27%2371717a%27 stroke-width=%272%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27%3e%3cpolyline points=%276 9 12 15 18 9%27%3e%3c/polyline%3e%3c/svg%3e'); background-repeat: no-repeat; background-position: right 0.75rem center; background-size: 1rem;"
            >
              {#each protocols as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>

          <!-- Server & Port -->
          <div class="grid grid-cols-3 gap-3">
            <div class="col-span-2 space-y-2">
              <label for="gateway-server" class="block text-sm font-medium text-zinc-400">Server</label>
              <input
                id="gateway-server"
                type="text"
                bind:value={server}
                placeholder="example.com"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                       focus:border-indigo-500/50 transition-colors"
              />
            </div>
            <div class="space-y-2">
              <label for="gateway-port" class="block text-sm font-medium text-zinc-400">Port</label>
              <input
                id="gateway-port"
                type="number"
                bind:value={port}
                min="1"
                max="65535"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                       focus:border-indigo-500/50 transition-colors"
              />
            </div>
          </div>

          <!-- UUID (for VLESS/VMess) -->
          {#if needsUuid}
            <div class="space-y-2" transition:fly={{ y: -10, duration: 200 }}>
              <label for="gateway-uuid" class="block text-sm font-medium text-zinc-400">UUID</label>
              <input
                id="gateway-uuid"
                type="text"
                bind:value={uuid}
                placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm font-mono
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                       focus:border-indigo-500/50 transition-colors"
              />
            </div>
          {/if}

          <!-- Password (for Trojan/SS/SOCKS5) -->
          {#if needsAuth && !needsUuid}
            <div class="space-y-2" transition:fly={{ y: -10, duration: 200 }}>
              <label for="gateway-password" class="block text-sm font-medium text-zinc-400">Password</label>
              <input
                id="gateway-password"
                type="password"
                bind:value={password}
                placeholder="••••••••"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                       focus:border-indigo-500/50 transition-colors"
              />
            </div>
          {/if}

          <!-- TLS Toggle -->
          <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-xl border border-white/5">
            <div class="space-y-1">
              <p class="text-sm font-medium text-white">TLS Encryption</p>
              <p class="text-xs text-zinc-500">Enable TLS/SSL for secure connection</p>
            </div>
            <Toggle bind:checked={tls} />
          </div>

          <!-- SNI (when TLS enabled) -->
          {#if tls}
            <div class="space-y-2" transition:fly={{ y: -10, duration: 200 }}>
              <label for="gateway-sni" class="block text-sm font-medium text-zinc-400">
                SNI <span class="text-zinc-600">(optional)</span>
              </label>
              <input
                id="gateway-sni"
                type="text"
                bind:value={sni}
                placeholder="Server Name Indication"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                       focus:border-indigo-500/50 transition-colors"
              />
            </div>
          {/if}
        </div>

        <!-- Info Section -->
        <div class="p-4 bg-zinc-900/30 rounded-xl border border-white/5">
          <div class="flex gap-3">
            <div class="shrink-0 w-8 h-8 rounded-lg bg-zinc-800 flex items-center justify-center">
              <svg class="w-4 h-4 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div class="space-y-1">
              <p class="text-sm font-medium text-zinc-300">Supported formats</p>
              <p class="text-xs text-zinc-500">
                VLESS, VMess, Shadowsocks, Trojan, SOCKS5, HTTP/HTTPS, Hysteria2, TUIC
              </p>
            </div>
          </div>
        </div>
        {:else}
        <!-- Subscription Tab Content -->
        <div class="space-y-4">
          <!-- Subscription URL Input -->
          <div class="space-y-3">
            <label for="subscription-url-input" class="block text-sm font-medium text-zinc-400">Subscription URL</label>
            <div class="flex gap-2">
              <input
                id="subscription-url-input"
                type="text"
                bind:value={subscriptionUrl}
                placeholder="https://example.com/subscription"
                disabled={subscriptionLoading}
                class="flex-1 bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                       focus:border-indigo-500/50 transition-colors disabled:opacity-50"
              />
              <button
                type="button"
                onclick={fetchSubscription}
                disabled={subscriptionLoading || !subscriptionUrl.trim()}
                class="px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:bg-indigo-600/50 
                       disabled:cursor-not-allowed text-white text-sm font-medium rounded-lg 
                       transition-colors flex items-center gap-2 shrink-0"
              >
                {#if subscriptionLoading}
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Fetching...
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                  </svg>
                  Fetch
                {/if}
              </button>
            </div>
            {#if subscriptionError}
              <p class="text-xs text-red-400">{subscriptionError}</p>
            {/if}
          </div>

          <!-- Fetched Proxies List -->
          {#if fetchedProxies.length > 0}
            <div class="space-y-3">
              <!-- Header with count and select all -->
              <div class="flex items-center justify-between">
                <p class="text-sm text-zinc-400">
                  Found <span class="text-white font-medium">{fetchedProxies.length}</span> proxies
                </p>
                <button
                  type="button"
                  onclick={toggleAllProxies}
                  class="text-xs text-indigo-400 hover:text-indigo-300 transition-colors"
                >
                  {allSelected ? 'Deselect All' : 'Select All'}
                </button>
              </div>

              <!-- Proxy List -->
              <div class="space-y-2 max-h-[320px] overflow-y-auto pr-1">
                {#each fetchedProxies as proxy (proxy.id)}
                  <button
                    type="button"
                    onclick={() => toggleProxySelection(proxy.id)}
                    class="w-full flex items-center gap-3 p-3 rounded-lg border transition-colors text-left
                           {selectedProxyIds.has(proxy.id) 
                             ? 'bg-indigo-500/10 border-indigo-500/30' 
                             : 'bg-zinc-900/50 border-white/5 hover:border-white/10'}"
                  >
                    <!-- Checkbox -->
                    <div class="shrink-0 w-5 h-5 rounded border-2 flex items-center justify-center transition-colors
                                {selectedProxyIds.has(proxy.id) 
                                  ? 'bg-indigo-500 border-indigo-500' 
                                  : 'border-zinc-600'}">
                      {#if selectedProxyIds.has(proxy.id)}
                        <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                        </svg>
                      {/if}
                    </div>

                    <!-- Flag -->
                    <span class="text-lg flex-shrink-0" title={getProxyCountryName(proxy.country, proxy.server)}>{getProxyFlag(proxy.country, proxy.server)}</span>

                    <!-- Proxy Info -->
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="text-sm font-medium text-white truncate">{proxy.name}</span>
                        <span class="shrink-0 px-1.5 py-0.5 text-[10px] font-medium rounded bg-zinc-800 text-zinc-400">
                          {getProtocolLabel(proxy.protocol)}
                        </span>
                      </div>
                      <p class="text-xs text-zinc-500 truncate mt-0.5">
                        {proxy.server}:{proxy.port}
                      </p>
                    </div>
                  </button>
                {/each}
              </div>

              <!-- Selected count -->
              {#if selectedCount > 0}
                <p class="text-xs text-zinc-500 text-center">
                  {selectedCount} of {fetchedProxies.length} selected
                </p>
              {/if}
            </div>
          {:else if !subscriptionLoading && subscriptionUrl && !subscriptionError}
            <!-- Empty state after fetch -->
            <div class="p-8 text-center">
              <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center mx-auto mb-3">
                <svg class="w-6 h-6 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                </svg>
              </div>
              <p class="text-sm text-zinc-400">No proxies found</p>
              <p class="text-xs text-zinc-600 mt-1">Try a different subscription URL</p>
            </div>
          {:else if !subscriptionLoading}
            <!-- Initial state -->
            <div class="p-8 text-center">
              <div class="w-12 h-12 rounded-full bg-zinc-800 flex items-center justify-center mx-auto mb-3">
                <svg class="w-6 h-6 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
              </div>
              <p class="text-sm text-zinc-400">Enter a subscription URL</p>
              <p class="text-xs text-zinc-600 mt-1">Paste your subscription link and click Fetch</p>
            </div>
          {/if}

          <!-- Info Section -->
          <div class="p-4 bg-zinc-900/30 rounded-xl border border-white/5">
            <div class="flex gap-3">
              <div class="shrink-0 w-8 h-8 rounded-lg bg-zinc-800 flex items-center justify-center">
                <svg class="w-4 h-4 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <div class="space-y-1">
                <p class="text-sm font-medium text-zinc-300">Subscription import</p>
                <p class="text-xs text-zinc-500">
                  Supports base64-encoded subscriptions with multiple proxy URLs
                </p>
              </div>
            </div>
          </div>
        </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-white/5 shrink-0 space-y-3">
        <div class="flex gap-3">
          <button
            type="button"
            onclick={handleClose}
            class="flex-1 px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-white text-sm font-medium 
                   rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-white/10"
          >
            Cancel
          </button>
          {#if activeTab === 'single'}
            <button
              type="button"
              onclick={handleSubmit}
              disabled={!isValid}
              class="flex-1 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:bg-indigo-600/50 
                     disabled:cursor-not-allowed text-white text-sm font-medium rounded-lg 
                     transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500/50
                     flex items-center justify-center gap-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Add Gateway
            </button>
          {:else}
            <button
              type="button"
              onclick={importSelectedProxies}
              disabled={selectedCount === 0}
              class="flex-1 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:bg-indigo-600/50 
                     disabled:cursor-not-allowed text-white text-sm font-medium rounded-lg 
                     transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500/50
                     flex items-center justify-center gap-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
              </svg>
              Import {selectedCount > 0 ? `(${selectedCount})` : 'Selected'}
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
