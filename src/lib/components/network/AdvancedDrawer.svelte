<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { browser } from '$app/environment';
  import { toasts } from '$lib/stores/toast';
  import {
    isQuicBlocked,
    enableQuicBlock,
    disableQuicBlock,
    getDnsSettings,
    setDnsServer,
    getWindivertMode,
    setWindivertMode,
    applyDnsToSystem,
    restoreSystemDns,
    type DnsServer,
    type WindivertMode
  } from '$lib/api';

  interface Props {
    open: boolean;
    onclose: () => void;
  }

  let { open, onclose }: Props = $props();

  // Settings state
  let dnsServer = $state<DnsServer>('system');
  let customDns = $state('');
  let quicBlock = $state(false);
  let windivertMode = $state<WindivertMode>('normal');
  
  // Loading states
  let loadingQuic = $state(false);
  let loadingDns = $state(false);
  let loadingWindivert = $state(false);
  let loadingApplyDns = $state(false);
  let loadingRestoreDns = $state(false);
  let initialized = $state(false);
  
  // Confirmation dialog state
  let showApplyConfirm = $state(false);
  let showRestoreConfirm = $state(false);

  // Load settings when drawer opens
  $effect(() => {
    if (open && !initialized && browser) {
      initialized = true;
      loadSettings();
    }
  });

  async function loadSettings() {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) return;

    try {
      // Load QUIC block status
      quicBlock = await isQuicBlocked().catch(() => false);
      
      // Load DNS settings
      const dnsSettings = await getDnsSettings().catch(() => ({ server: 'system' as DnsServer, customAddress: undefined }));
      dnsServer = dnsSettings.server;
      customDns = dnsSettings.customAddress || '';
      
      // Load WinDivert mode
      windivertMode = await getWindivertMode().catch(() => 'normal' as WindivertMode);
    } catch (e) {
      console.error('Failed to load advanced settings:', e);
    }
  }

  // Handle QUIC toggle
  async function handleQuicToggle() {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      quicBlock = !quicBlock;
      return;
    }

    loadingQuic = true;
    try {
      if (quicBlock) {
        await disableQuicBlock();
        quicBlock = false;
        toasts.success('QUIC unblocked');
      } else {
        await enableQuicBlock();
        quicBlock = true;
        toasts.success('QUIC blocked');
      }
    } catch (e) {
      toasts.error(`Failed to toggle QUIC: ${e}`);
    } finally {
      loadingQuic = false;
    }
  }

  // Handle DNS change
  async function handleDnsChange() {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) return;

    loadingDns = true;
    try {
      await setDnsServer(dnsServer, dnsServer === 'custom' ? customDns : undefined);
      toasts.success('DNS settings saved');
    } catch (e) {
      toasts.error(`Failed to save DNS: ${e}`);
    } finally {
      loadingDns = false;
    }
  }

  // Handle WinDivert mode change
  async function handleWindivertChange() {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) return;

    loadingWindivert = true;
    try {
      await setWindivertMode(windivertMode);
      toasts.success('WinDivert mode saved');
    } catch (e) {
      toasts.error(`Failed to save WinDivert mode: ${e}`);
    } finally {
      loadingWindivert = false;
    }
  }

  // Handle Apply DNS to System
  async function handleApplyDns() {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      toasts.info('DNS applied (demo mode)');
      showApplyConfirm = false;
      return;
    }

    loadingApplyDns = true;
    try {
      await applyDnsToSystem();
      toasts.success('DNS applied to system');
    } catch (e) {
      toasts.error(`Failed to apply DNS: ${e}`);
    } finally {
      loadingApplyDns = false;
      showApplyConfirm = false;
    }
  }

  // Handle Restore System DNS
  async function handleRestoreDns() {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      toasts.info('DNS restored (demo mode)');
      showRestoreConfirm = false;
      return;
    }

    loadingRestoreDns = true;
    try {
      await restoreSystemDns();
      toasts.success('DNS restored to DHCP');
    } catch (e) {
      toasts.error(`Failed to restore DNS: ${e}`);
    } finally {
      loadingRestoreDns = false;
      showRestoreConfirm = false;
    }
  }

  // Handle Escape key
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onclose();
    }
  }

  // Handle backdrop click
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }

  // DNS options
  const dnsOptions = [
    { value: 'system', label: 'System Default' },
    { value: 'cloudflare', label: 'Cloudflare (1.1.1.1)' },
    { value: 'google', label: 'Google (8.8.8.8)' },
    { value: 'custom', label: 'Custom...' }
  ] as const;

  // WinDivert mode options
  const windivertOptions = [
    { value: 'normal', label: 'Normal', description: 'Standard packet filtering' },
    { value: 'autottl', label: 'Auto TTL', description: 'Automatic TTL manipulation' },
    { value: 'autohostlist', label: 'Auto Hostlist', description: 'Dynamic host detection' }
  ] as const;
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Advanced Settings"
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
      class="absolute right-0 top-0 h-full w-96 bg-zinc-950 border-l border-white/5 shadow-2xl flex flex-col"
      transition:fly={{ x: 400, duration: 300, opacity: 1 }}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-white/5 shrink-0">
        <h2 class="text-lg font-semibold text-white">Advanced Settings</h2>
        <button
          type="button"
          onclick={onclose}
          class="p-2 hover:bg-white/5 rounded-lg transition-colors text-zinc-400 hover:text-white"
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4 space-y-6">
        <!-- DNS Section -->
        <div class="space-y-3">
          <label for="dns-server-select" class="block text-sm font-medium text-zinc-400">DNS Server</label>
          <select
            id="dns-server-select"
            bind:value={dnsServer}
            onchange={handleDnsChange}
            class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                   focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50
                   transition-colors cursor-pointer appearance-none"
            style="background-image: url('data:image/svg+xml;charset=UTF-8,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27 fill=%27none%27 stroke=%27%2371717a%27 stroke-width=%272%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27%3e%3cpolyline points=%276 9 12 15 18 9%27%3e%3c/polyline%3e%3c/svg%3e'); background-repeat: no-repeat; background-position: right 0.75rem center; background-size: 1rem;"
          >
            {#each dnsOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>

          <!-- Custom DNS Input -->
          {#if dnsServer === 'custom'}
            <div transition:fly={{ y: -10, duration: 200 }}>
              <input
                type="text"
                bind:value={customDns}
                onblur={handleDnsChange}
                placeholder="Enter DNS address (e.g., 9.9.9.9)"
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                       focus:border-indigo-500/50 transition-colors"
              />
            </div>
          {/if}
        </div>

        <!-- System DNS Section -->
        <div class="p-4 bg-zinc-900/50 rounded-xl border border-white/5 space-y-3">
          <div class="flex items-center justify-between">
            <div class="space-y-1">
              <p class="text-sm font-medium text-white">System DNS</p>
              <p class="text-xs text-zinc-400">Apply DNS settings to Windows network adapters</p>
            </div>
            <span class="px-2 py-0.5 text-[10px] font-medium bg-amber-500/10 text-amber-400 rounded border border-amber-500/20">
              Admin
            </span>
          </div>
          
          <div class="flex gap-2">
            <!-- Apply to System Button -->
            <button
              type="button"
              onclick={() => showApplyConfirm = true}
              disabled={loadingApplyDns || dnsServer === 'system'}
              class="flex-1 px-3 py-2 text-sm font-medium rounded-lg transition-all
                     {dnsServer === 'system' 
                       ? 'bg-zinc-800 text-zinc-400 cursor-not-allowed' 
                       : 'bg-indigo-500/10 text-indigo-400 hover:bg-indigo-500/20 border border-indigo-500/20'}
                     {loadingApplyDns ? 'opacity-50' : ''}"
            >
              {#if loadingApplyDns}
                <span class="flex items-center justify-center gap-2">
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Applying...
                </span>
              {:else}
                Apply to System
              {/if}
            </button>
            
            <!-- Restore DHCP Button -->
            <button
              type="button"
              onclick={() => showRestoreConfirm = true}
              disabled={loadingRestoreDns}
              class="flex-1 px-3 py-2 text-sm font-medium rounded-lg transition-all
                     bg-zinc-800 text-zinc-300 hover:bg-zinc-700 border border-white/5
                     {loadingRestoreDns ? 'opacity-50' : ''}"
            >
              {#if loadingRestoreDns}
                <span class="flex items-center justify-center gap-2">
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Restoring...
                </span>
              {:else}
                Restore DHCP
              {/if}
            </button>
          </div>
          
          {#if dnsServer === 'system'}
            <p class="text-xs text-zinc-400 italic">
              Select a DNS server above to enable "Apply to System"
            </p>
          {/if}
        </div>

        <!-- QUIC Block -->
        <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-xl border border-white/5">
          <div class="space-y-1">
            <p class="text-sm font-medium text-white">Block QUIC</p>
            <p class="text-xs text-zinc-400">Block UDP/443 to force TCP connections</p>
          </div>
          <button
            type="button"
            onclick={handleQuicToggle}
            disabled={loadingQuic}
            aria-label={quicBlock ? 'Disable QUIC blocking' : 'Enable QUIC blocking'}
            class="relative w-11 h-6 rounded-full transition-colors duration-200 {quicBlock ? 'bg-emerald-500' : 'bg-zinc-700'} {loadingQuic ? 'opacity-50' : ''}"
          >
            <span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform duration-200 {quicBlock ? 'translate-x-5' : ''}"></span>
          </button>
        </div>

        <!-- Danger Zone -->
        <details class="group border border-red-500/20 rounded-xl overflow-hidden">
          <summary class="flex items-center gap-2 p-4 text-red-400 cursor-pointer hover:bg-red-500/5 transition-colors select-none">
            <svg class="w-4 h-4 transition-transform group-open:rotate-90" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
            <span class="text-sm font-medium">⚠️ Danger Zone</span>
          </summary>
          
          <div class="p-4 pt-0 space-y-4 border-t border-red-500/10">
            <p class="text-xs text-zinc-400 mt-3">
              These settings can affect system stability. Change only if you know what you're doing.
            </p>

            <!-- WinDivert Mode -->
            <div class="space-y-3">
              <label for="windivert-mode-select" class="block text-sm font-medium text-zinc-400">WinDivert Mode</label>
              <select
                id="windivert-mode-select"
                bind:value={windivertMode}
                onchange={handleWindivertChange}
                class="w-full bg-zinc-900 border border-red-500/20 rounded-lg px-3 py-2.5 text-white text-sm
                       focus:outline-none focus:ring-2 focus:ring-red-500/30 focus:border-red-500/30
                       transition-colors cursor-pointer appearance-none"
                style="background-image: url('data:image/svg+xml;charset=UTF-8,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27 fill=%27none%27 stroke=%27%2371717a%27 stroke-width=%272%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27%3e%3cpolyline points=%276 9 12 15 18 9%27%3e%3c/polyline%3e%3c/svg%3e'); background-repeat: no-repeat; background-position: right 0.75rem center; background-size: 1rem;"
              >
                {#each windivertOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>

              <!-- Mode Description -->
              <div class="p-3 bg-zinc-900/30 rounded-lg border border-white/5">
                <p class="text-xs text-zinc-400">
                  {#if windivertMode === 'normal'}
                    Standard packet filtering mode. Recommended for most users.
                  {:else if windivertMode === 'autottl'}
                    Automatically adjusts TTL values to bypass DPI. May cause issues with some networks.
                  {:else}
                    Dynamically detects and adds blocked hosts. Experimental feature.
                  {/if}
                </p>
              </div>
            </div>
          </div>
        </details>

        <!-- Info Section -->
        <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/10">
          <div class="flex gap-3">
            <div class="shrink-0 w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center">
              <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div class="space-y-1">
              <p class="text-sm font-medium text-indigo-300">Need help?</p>
              <p class="text-xs text-zinc-400">
                These settings are for advanced users. Default values work best for most scenarios.
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-white/5 shrink-0">
        <button
          type="button"
          onclick={onclose}
          class="w-full px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-white text-sm font-medium 
                 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
        >
          Done
        </button>
      </div>
    </div>
  </div>

  <!-- Apply DNS Confirmation Dialog -->
  {#if showApplyConfirm}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-[60] flex items-center justify-center"
      transition:fade={{ duration: 150 }}
    >
      <div class="absolute inset-0 bg-black/60" onclick={() => showApplyConfirm = false}></div>
      <div class="relative bg-zinc-900 rounded-xl border border-white/10 p-6 max-w-sm mx-4 shadow-2xl" transition:fly={{ y: 20, duration: 200 }}>
        <div class="flex items-start gap-4">
          <div class="shrink-0 w-10 h-10 rounded-full bg-amber-500/10 flex items-center justify-center">
            <svg class="w-5 h-5 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div class="flex-1">
            <h3 class="text-base font-semibold text-white">Apply DNS to System?</h3>
            <p class="mt-2 text-sm text-zinc-400">
              This will change DNS settings on all active network adapters. Requires administrator privileges.
            </p>
          </div>
        </div>
        <div class="flex gap-3 mt-6">
          <button
            type="button"
            onclick={() => showApplyConfirm = false}
            class="flex-1 px-4 py-2 text-sm font-medium text-zinc-300 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={handleApplyDns}
            disabled={loadingApplyDns}
            class="flex-1 px-4 py-2 text-sm font-medium text-white bg-indigo-500 hover:bg-indigo-600 rounded-lg transition-colors disabled:opacity-50"
          >
            {loadingApplyDns ? 'Applying...' : 'Apply'}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Restore DNS Confirmation Dialog -->
  {#if showRestoreConfirm}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-[60] flex items-center justify-center"
      transition:fade={{ duration: 150 }}
    >
      <div class="absolute inset-0 bg-black/60" onclick={() => showRestoreConfirm = false}></div>
      <div class="relative bg-zinc-900 rounded-xl border border-white/10 p-6 max-w-sm mx-4 shadow-2xl" transition:fly={{ y: 20, duration: 200 }}>
        <div class="flex items-start gap-4">
          <div class="shrink-0 w-10 h-10 rounded-full bg-blue-500/10 flex items-center justify-center">
            <svg class="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </div>
          <div class="flex-1">
            <h3 class="text-base font-semibold text-white">Restore DHCP?</h3>
            <p class="mt-2 text-sm text-zinc-400">
              This will reset DNS settings to automatic (DHCP) on all active network adapters. Requires administrator privileges.
            </p>
          </div>
        </div>
        <div class="flex gap-3 mt-6">
          <button
            type="button"
            onclick={() => showRestoreConfirm = false}
            class="flex-1 px-4 py-2 text-sm font-medium text-zinc-300 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={handleRestoreDns}
            disabled={loadingRestoreDns}
            class="flex-1 px-4 py-2 text-sm font-medium text-white bg-blue-500 hover:bg-blue-600 rounded-lg transition-colors disabled:opacity-50"
          >
            {loadingRestoreDns ? 'Restoring...' : 'Restore'}
          </button>
        </div>
      </div>
    </div>
  {/if}
{/if}
