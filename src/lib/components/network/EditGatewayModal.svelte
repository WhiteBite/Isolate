<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import Toggle from '../Toggle.svelte';
  import type { ProxyConfig, ProxyProtocol } from '$lib/api';

  interface Props {
    open: boolean;
    onclose: () => void;
    onsave: (gateway: ProxyConfig) => void;
    gateway: ProxyConfig | null;
  }

  let { open, onclose, onsave, gateway }: Props = $props();

  // Form state
  let name = $state('');
  let protocol = $state<ProxyProtocol>('vless');
  let server = $state('');
  let port = $state(443);
  let password = $state('');
  let uuid = $state('');
  let tls = $state(true);
  let sni = $state('');
  let transport = $state('');

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

  // Populate form when gateway changes
  $effect(() => {
    if (gateway && open) {
      name = gateway.name || '';
      protocol = gateway.protocol || 'vless';
      server = gateway.server || '';
      port = gateway.port || 443;
      password = gateway.password || '';
      uuid = gateway.uuid || '';
      tls = gateway.tls ?? true;
      sni = gateway.sni || '';
      transport = gateway.transport || '';
    }
  });

  // Reset form
  function resetForm() {
    name = '';
    protocol = 'vless';
    server = '';
    port = 443;
    password = '';
    uuid = '';
    tls = true;
    sni = '';
    transport = '';
  }

  // Handle submit
  function handleSubmit() {
    if (!isValid || !gateway) return;
    
    const updatedGateway: ProxyConfig = {
      ...gateway,
      name: name.trim(),
      protocol,
      server: server.trim(),
      port,
      tls,
      custom_fields: gateway.custom_fields || {}
    };
    
    if (password) updatedGateway.password = password;
    else delete updatedGateway.password;
    
    if (uuid) updatedGateway.uuid = uuid;
    else delete updatedGateway.uuid;
    
    if (sni) updatedGateway.sni = sni;
    else delete updatedGateway.sni;
    
    if (transport) updatedGateway.transport = transport;
    else delete updatedGateway.transport;
    
    onsave(updatedGateway);
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

{#if open && gateway}
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Edit Gateway"
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
          <div class="w-9 h-9 rounded-lg bg-amber-500/10 flex items-center justify-center">
            <svg class="w-5 h-5 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </div>
          <h2 class="text-lg font-semibold text-white">Edit Gateway</h2>
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

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4 space-y-5">
        <!-- Manual Fields -->
        <div class="space-y-4">
          <!-- Name -->
          <div class="space-y-2">
            <label for="edit-gateway-name" class="block text-sm font-medium text-zinc-400">Name</label>
            <input
              id="edit-gateway-name"
              type="text"
              bind:value={name}
              placeholder="My Proxy Server"
              class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                     placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-amber-500/50 
                     focus:border-amber-500/50 transition-colors"
            />
          </div>

          <!-- Protocol -->
          <div class="space-y-2">
            <label for="edit-gateway-protocol" class="block text-sm font-medium text-zinc-400">Protocol</label>
            <select
              id="edit-gateway-protocol"
              bind:value={protocol}
              class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                     focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:border-amber-500/50
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
              <label for="edit-gateway-server" class="block text-sm font-medium text-zinc-400">Server</label>
              <input
                id="edit-gateway-server"
                type="text"
                bind:value={server}
                placeholder="example.com"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-amber-500/50 
                       focus:border-amber-500/50 transition-colors"
              />
            </div>
            <div class="space-y-2">
              <label for="edit-gateway-port" class="block text-sm font-medium text-zinc-400">Port</label>
              <input
                id="edit-gateway-port"
                type="number"
                bind:value={port}
                min="1"
                max="65535"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-amber-500/50 
                       focus:border-amber-500/50 transition-colors"
              />
            </div>
          </div>

          <!-- UUID (for VLESS/VMess) -->
          {#if needsUuid}
            <div class="space-y-2" transition:fly={{ y: -10, duration: 200 }}>
              <label for="edit-gateway-uuid" class="block text-sm font-medium text-zinc-400">UUID</label>
              <input
                id="edit-gateway-uuid"
                type="text"
                bind:value={uuid}
                placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm font-mono
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-amber-500/50 
                       focus:border-amber-500/50 transition-colors"
              />
            </div>
          {/if}

          <!-- Password (for Trojan/SS/SOCKS5) -->
          {#if needsAuth && !needsUuid}
            <div class="space-y-2" transition:fly={{ y: -10, duration: 200 }}>
              <label for="edit-gateway-password" class="block text-sm font-medium text-zinc-400">Password</label>
              <input
                id="edit-gateway-password"
                type="password"
                bind:value={password}
                placeholder="••••••••"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-amber-500/50 
                       focus:border-amber-500/50 transition-colors"
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
              <label for="edit-gateway-sni" class="block text-sm font-medium text-zinc-400">
                SNI <span class="text-zinc-600">(optional)</span>
              </label>
              <input
                id="edit-gateway-sni"
                type="text"
                bind:value={sni}
                placeholder="Server Name Indication"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-amber-500/50 
                       focus:border-amber-500/50 transition-colors"
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
              <p class="text-sm font-medium text-zinc-300">Gateway ID</p>
              <p class="text-xs text-zinc-500 font-mono">{gateway.id}</p>
            </div>
          </div>
        </div>
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
          <button
            type="button"
            onclick={handleSubmit}
            disabled={!isValid}
            class="flex-1 px-4 py-2.5 bg-amber-600 hover:bg-amber-500 disabled:bg-amber-600/50 
                   disabled:cursor-not-allowed text-white text-sm font-medium rounded-lg 
                   transition-colors focus:outline-none focus:ring-2 focus:ring-amber-500/50
                   flex items-center justify-center gap-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            Save Changes
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
