<script lang="ts">
  import type { ProxyConfig } from '$lib/api';

  type ConnectionState = 'connected' | 'disconnected' | 'error';

  interface Props {
    tunRunning: boolean;
    systemProxySet: boolean;
    activeGateway?: ProxyConfig | null;
    loading?: boolean;
  }

  let { 
    tunRunning, 
    systemProxySet, 
    activeGateway = null,
    loading = false 
  }: Props = $props();

  // Derived state
  let connectionState = $derived.by((): ConnectionState => {
    if (loading) return 'disconnected';
    if (tunRunning || systemProxySet) {
      return activeGateway ? 'connected' : 'error';
    }
    return 'disconnected';
  });

  let statusText = $derived.by(() => {
    if (loading) return 'Loading...';
    switch (connectionState) {
      case 'connected':
        return 'Connected';
      case 'error':
        return 'No Gateway';
      case 'disconnected':
      default:
        return 'Disconnected';
    }
  });

  let statusColor = $derived.by(() => {
    switch (connectionState) {
      case 'connected':
        return 'bg-emerald-500';
      case 'error':
        return 'bg-amber-500';
      case 'disconnected':
      default:
        return 'bg-zinc-500';
    }
  });

  let modeText = $derived.by(() => {
    if (tunRunning) return 'TUN';
    if (systemProxySet) return 'Proxy';
    return null;
  });
</script>

<div class="flex items-center gap-3 px-3 py-1.5 bg-zinc-900/50 rounded-lg border border-white/5">
  <!-- Status indicator -->
  <div class="flex items-center gap-2">
    <span class="relative flex h-2.5 w-2.5">
      {#if connectionState === 'connected'}
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full {statusColor} opacity-75"></span>
      {/if}
      <span class="relative inline-flex rounded-full h-2.5 w-2.5 {statusColor}"></span>
    </span>
    <span class="text-sm font-medium {connectionState === 'connected' ? 'text-emerald-400' : connectionState === 'error' ? 'text-amber-400' : 'text-zinc-400'}">
      {statusText}
    </span>
  </div>

  <!-- Divider -->
  {#if activeGateway || modeText}
    <div class="w-px h-4 bg-white/10"></div>
  {/if}

  <!-- Active gateway info -->
  {#if activeGateway}
    <div class="flex items-center gap-2">
      <!-- Country flag or protocol icon -->
      {#if activeGateway.country}
        <span class="text-xs uppercase font-mono text-zinc-500">{activeGateway.country}</span>
      {:else}
        <span class="text-xs uppercase font-mono text-zinc-500">{activeGateway.protocol}</span>
      {/if}
      
      <!-- Gateway name -->
      <span class="text-sm text-zinc-300 max-w-[120px] truncate" title={activeGateway.name}>
        {activeGateway.name}
      </span>

      <!-- Latency -->
      {#if activeGateway.ping}
        <span class="text-xs font-mono {activeGateway.ping < 100 ? 'text-emerald-400' : activeGateway.ping < 200 ? 'text-amber-400' : 'text-red-400'}">
          {activeGateway.ping}ms
        </span>
      {/if}
    </div>
  {:else if modeText && connectionState !== 'disconnected'}
    <span class="text-xs text-zinc-500">via {modeText}</span>
  {/if}
</div>
