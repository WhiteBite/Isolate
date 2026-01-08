<script lang="ts">
  import { type ProxyConfig, exportProxyUrl } from '$lib/api';
  import { getProxyFlag } from '$lib/utils/countries';
  import { toasts } from '$lib/stores/toast';
  
  interface Props {
    gateway: ProxyConfig;
    selected: boolean;
    onclick: () => void;
    ontest: () => void;
    ondelete: () => void;
    onedit: () => void;
    onactivate?: () => void;
    ondeactivate?: () => void;
    onshare?: () => void;
  }

  let {
    gateway,
    selected,
    onclick,
    ontest,
    ondelete,
    onedit,
    onactivate,
    ondeactivate,
    onshare,
  }: Props = $props();
  
  let showContextMenu = $state(false);
  let contextMenuPos = $state({ x: 0, y: 0 });
  let copyingUrl = $state(false);
  
  let flag = $derived(getProxyFlag(gateway.country, gateway.server));
  
  // Ping color
  let pingColor = $derived(gateway.ping === null || gateway.ping === undefined ? 'text-zinc-400' 
    : gateway.ping < 100 ? 'text-emerald-400' 
    : gateway.ping < 200 ? 'text-yellow-400' 
    : 'text-red-400');

  // Protocol display
  function getProtocolDisplay(protocol: string): string {
    const map: Record<string, string> = {
      vless: 'VLESS',
      vmess: 'VMess',
      shadowsocks: 'SS',
      trojan: 'Trojan',
      socks5: 'SOCKS5',
      http: 'HTTP',
      https: 'HTTPS',
      tuic: 'TUIC',
      hysteria: 'HY',
      hysteria2: 'HY2',
      wireguard: 'WG',
      ssh: 'SSH',
    };
    return map[protocol] || protocol.toUpperCase();
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    contextMenuPos = { x: e.clientX, y: e.clientY };
    showContextMenu = true;
  }

  function closeContextMenu() {
    showContextMenu = false;
  }

  function handleTest() {
    closeContextMenu();
    ontest();
  }

  async function handleCopyUrl() {
    if (copyingUrl) return;
    
    closeContextMenu();
    copyingUrl = true;
    
    try {
      const url = await exportProxyUrl(gateway.id);
      await navigator.clipboard.writeText(url);
      toasts.success('URL copied');
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to export URL';
      toasts.error(message);
    } finally {
      copyingUrl = false;
    }
  }

  function handleEdit() {
    closeContextMenu();
    onedit();
  }

  function handleDelete() {
    closeContextMenu();
    ondelete();
  }

  function handleActivate() {
    closeContextMenu();
    onactivate?.();
  }

  function handleDeactivate() {
    closeContextMenu();
    ondeactivate?.();
  }

  function handleShare() {
    closeContextMenu();
    onshare?.();
  }

  // Close context menu on click outside
  function handleWindowClick() {
    if (showContextMenu) {
      closeContextMenu();
    }
  }
</script>

<svelte:window onclick={handleWindowClick} />

<button 
  class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-150
         {gateway.active 
           ? 'bg-emerald-500/10 border border-emerald-500/30 shadow-[0_0_12px_rgba(16,185,129,0.15)]' 
           : selected 
             ? 'bg-indigo-500/10 border-l-2 border-indigo-500 pl-2.5 border-t-0 border-r-0 border-b-0' 
             : 'hover:bg-white/5 border-l-2 border-transparent'}"
  onclick={onclick}
  oncontextmenu={handleContextMenu}
>
  <!-- Active indicator dot -->
  {#if gateway.active}
    <span class="relative flex-shrink-0">
      <span class="absolute -left-1 top-1/2 -translate-y-1/2 w-2 h-2 bg-emerald-400 rounded-full"></span>
      <span class="absolute -left-1 top-1/2 -translate-y-1/2 w-2 h-2 bg-emerald-400 rounded-full animate-ping opacity-75"></span>
    </span>
  {/if}
  
  <!-- Flag -->
  <span class="text-lg flex-shrink-0 {gateway.active ? 'ml-2' : ''}">{flag}</span>
  
  <!-- Name & Server -->
  <div class="flex-1 min-w-0 text-left">
    <p class="text-sm text-white truncate font-medium">{gateway.name}</p>
    <p class="text-xs text-zinc-400 truncate font-mono">{gateway.server}:{gateway.port}</p>
  </div>
  
  <!-- Protocol badge -->
  <span class="px-1.5 py-0.5 text-[10px] font-medium {gateway.active ? 'bg-emerald-500/20 text-emerald-400' : 'bg-zinc-800 text-zinc-400'} rounded flex-shrink-0">
    {getProtocolDisplay(gateway.protocol)}
  </span>
  
  <!-- Ping -->
  {#if gateway.ping !== null && gateway.ping !== undefined}
    <span class="{pingColor} text-xs font-medium tabular-nums flex-shrink-0 w-12 text-right">
      {gateway.ping}ms
    </span>
  {/if}
</button>

<!-- Context Menu -->
{#if showContextMenu}
  <div 
    class="fixed z-50 bg-zinc-900 border border-white/10 rounded-lg shadow-xl py-1 min-w-[140px]"
    style="left: {contextMenuPos.x}px; top: {contextMenuPos.y}px;"
  >
    <!-- Activate/Deactivate -->
    {#if gateway.active}
      <button 
        class="w-full px-3 py-2 text-left text-sm text-amber-400 hover:bg-amber-500/10 flex items-center gap-2"
        onclick={handleDeactivate}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
        </svg>
        Deactivate
      </button>
    {:else}
      <button 
        class="w-full px-3 py-2 text-left text-sm text-emerald-400 hover:bg-emerald-500/10 flex items-center gap-2"
        onclick={handleActivate}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M5 13l4 4L19 7" />
        </svg>
        Activate
      </button>
    {/if}
    
    <div class="border-t border-white/5 my-1"></div>
    
    <button 
      class="w-full px-3 py-2 text-left text-sm text-zinc-300 hover:bg-white/5 flex items-center gap-2"
      onclick={handleTest}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
      Test
    </button>
    <button 
      class="w-full px-3 py-2 text-left text-sm text-zinc-300 hover:bg-white/5 flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
      onclick={handleCopyUrl}
      disabled={copyingUrl}
    >
      {#if copyingUrl}
        <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      {:else}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      {/if}
      Copy URL
    </button>
    {#if onshare}
      <button 
        class="w-full px-3 py-2 text-left text-sm text-indigo-400 hover:bg-indigo-500/10 flex items-center gap-2"
        onclick={handleShare}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 0h.01M5 8h2a1 1 0 001-1V5a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1zm12 0h2a1 1 0 001-1V5a1 1 0 00-1-1h-2a1 1 0 00-1 1v2a1 1 0 001 1zM5 20h2a1 1 0 001-1v-2a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1z" />
        </svg>
        Share QR
      </button>
    {/if}
    <button 
      class="w-full px-3 py-2 text-left text-sm text-zinc-300 hover:bg-white/5 flex items-center gap-2"
      onclick={handleEdit}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
      </svg>
      Edit
    </button>
    <div class="border-t border-white/5 my-1"></div>
    <button 
      class="w-full px-3 py-2 text-left text-sm text-red-400 hover:bg-red-500/10 flex items-center gap-2"
      onclick={handleDelete}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
      Delete
    </button>
  </div>
{/if}
