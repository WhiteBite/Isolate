<script lang="ts">
  import { toasts } from '$lib/stores/toast';
  import { getProxyFlag, getProxyCountryName } from '$lib/utils/countries';
  
  interface Props {
    id: string;
    name: string;
    server: string;
    port: number;
    protocol: string;
    country?: string | null;
    ping?: number | null;
    active?: boolean;
    onEdit?: (() => void) | null;
    onDelete?: (() => void) | null;
    onToggle?: (() => void) | null;
    onCopy?: (() => void) | null;
    onShare?: (() => void) | null;
  }

  let {
    id,
    name,
    server,
    port,
    protocol,
    country = null,
    ping = null,
    active = false,
    onEdit = null,
    onDelete = null,
    onToggle = null,
    onCopy = null,
    onShare = null,
  }: Props = $props();
  
  let copying = $state(false);
  
  async function handleCopy(e: MouseEvent) {
    e.stopPropagation();
    if (onCopy) {
      onCopy();
    } else {
      // Default: copy server:port
      copying = true;
      try {
        await navigator.clipboard.writeText(`${server}:${port}`);
        toasts.success('Address copied');
      } catch {
        toasts.error('Failed to copy');
      }
      copying = false;
    }
  }

  // Use getProxyFlag for automatic country detection from server hostname
  let flag = $derived(getProxyFlag(country, server));
  let countryName = $derived(getProxyCountryName(country, server));
  
  // Ping color with gradient effect
  let pingColor = $derived(ping === null ? 'text-zinc-400' 
    : ping < 50 ? 'text-emerald-400' 
    : ping < 100 ? 'text-green-400' 
    : ping < 200 ? 'text-yellow-400' 
    : ping < 300 ? 'text-orange-400' 
    : 'text-red-400');

  let pingBg = $derived(ping === null ? 'bg-zinc-500/10' 
    : ping < 50 ? 'bg-emerald-500/10' 
    : ping < 100 ? 'bg-green-500/10' 
    : ping < 200 ? 'bg-yellow-500/10' 
    : ping < 300 ? 'bg-orange-500/10' 
    : 'bg-red-500/10');

  // Protocol colors with better contrast
  const protocolStyles: Record<string, string> = {
    'VLESS': 'bg-purple-500/15 text-purple-400 border-purple-500/20',
    'VMess': 'bg-violet-500/15 text-violet-400 border-violet-500/20',
    'Shadowsocks': 'bg-blue-500/15 text-blue-400 border-blue-500/20',
    'Trojan': 'bg-cyan-500/15 text-cyan-400 border-cyan-500/20',
    'SOCKS5': 'bg-orange-500/15 text-orange-400 border-orange-500/20',
    'HTTP': 'bg-zinc-500/15 text-zinc-400 border-zinc-500/20',
  };

  let protocolColor = $derived(protocolStyles[protocol] || 'bg-zinc-500/15 text-zinc-400 border-zinc-500/20');
</script>

<div
  class="group relative flex items-center gap-4 p-4 bg-zinc-900/40 border rounded-xl transition-all duration-200 cursor-pointer
    {active 
      ? 'ring-2 ring-indigo-500/50 border-indigo-500/30 bg-indigo-500/5' 
      : 'border-white/5 hover:border-white/10 hover:bg-zinc-900/60'}"
  onclick={onToggle}
  onkeydown={(e) => e.key === 'Enter' && onToggle?.()}
  role="button"
  tabindex="0"
>
  <!-- Active indicator -->
  {#if active}
    <div class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-indigo-500 rounded-r-full"></div>
  {/if}

  <!-- Флаг + Название -->
  <div class="flex items-center gap-3 min-w-0 flex-1">
    <div class="relative" title={countryName}>
      <span class="text-2xl flex-shrink-0 drop-shadow-sm">{flag}</span>
      {#if active}
        <div class="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 bg-emerald-500 rounded-full border-2 border-zinc-900"></div>
      {/if}
    </div>
    <div class="min-w-0">
      <div class="text-zinc-100 font-medium truncate group-hover:text-white transition-colors">{name}</div>
      <div class="text-zinc-400 text-sm truncate font-mono">{server}:{port}</div>
    </div>
  </div>

  <!-- Протокол badge -->
  <div class="flex-shrink-0">
    <span class="px-2.5 py-1 rounded-lg text-xs font-medium border {protocolColor}">
      {protocol}
    </span>
  </div>

  <!-- Пинг с индикатором -->
  <div class="flex-shrink-0 w-20 text-right">
    {#if ping !== null}
      <div class="inline-flex items-center gap-1.5 px-2 py-1 rounded-lg {pingBg}">
        <div class="w-1.5 h-1.5 rounded-full {ping < 100 ? 'bg-green-400' : ping < 200 ? 'bg-yellow-400' : 'bg-red-400'}"></div>
        <span class="{pingColor} text-sm font-medium tabular-nums">{ping}ms</span>
      </div>
    {:else}
      <span class="text-zinc-400 text-sm">—</span>
    {/if}
  </div>

  <!-- Кнопки действий -->
  <div class="flex items-center gap-1 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
    <!-- Share/QR button -->
    {#if onShare}
      <button
        class="p-2 rounded-lg text-zinc-400 hover:text-indigo-400 hover:bg-indigo-500/10 transition-all duration-200"
        onclick={(e) => { e.stopPropagation(); onShare?.(); }}
        title="Share QR code"
        aria-label="Share {name} QR code"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 0h.01M5 8h2a1 1 0 001-1V5a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1zm12 0h2a1 1 0 001-1V5a1 1 0 00-1-1h-2a1 1 0 00-1 1v2a1 1 0 001 1zM5 20h2a1 1 0 001-1v-2a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1z" />
        </svg>
      </button>
    {/if}
    <!-- Copy button -->
    <button
      class="p-2 rounded-lg text-zinc-400 hover:text-emerald-400 hover:bg-emerald-500/10 transition-all duration-200"
      onclick={handleCopy}
      title="Copy address"
      aria-label="Copy {name} address"
      disabled={copying}
    >
      {#if copying}
        <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
        </svg>
      {:else}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      {/if}
    </button>
    {#if onEdit}
      <button
        class="p-2 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-white/5 transition-all duration-200"
        onclick={(e) => { e.stopPropagation(); onEdit?.(); }}
        title="Edit"
        aria-label="Edit {name}"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
        </svg>
      </button>
    {/if}
    {#if onDelete}
      <button
        class="p-2 rounded-lg text-zinc-400 hover:text-red-400 hover:bg-red-500/10 transition-all duration-200"
        onclick={(e) => { e.stopPropagation(); onDelete?.(); }}
        title="Delete"
        aria-label="Delete {name}"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    {/if}
  </div>
</div>
