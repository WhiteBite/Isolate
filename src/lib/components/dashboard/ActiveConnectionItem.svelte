<script lang="ts">
  import type { ActiveConnection } from '$lib/stores/dashboard.svelte';

  interface Props {
    connection: ActiveConnection;
  }

  let { connection }: Props = $props();

  const methodConfig = $derived({
    direct: { label: 'Direct', color: 'bg-slate-600 text-slate-200' },
    strategy: { label: connection.strategyName || 'Strategy', color: 'bg-blue-600 text-blue-100' },
    proxy: { label: connection.proxyName || 'Proxy', color: 'bg-purple-600 text-purple-100' },
    vless: { label: 'VLESS', color: 'bg-emerald-600 text-emerald-100' }
  }[connection.method]);

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
  }

  function formatDuration(seconds: number): string {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
  }
</script>

<div class="flex items-center gap-3 py-2 px-3 rounded-lg bg-slate-800/30 hover:bg-slate-800/50 transition-colors">
  <!-- Domain -->
  <div class="flex-1 min-w-0">
    <p class="text-sm font-medium text-slate-200 truncate">{connection.domain}</p>
  </div>

  <!-- Method badge -->
  <span class="shrink-0 px-2 py-0.5 text-xs font-medium rounded-full {methodConfig.color}">
    {methodConfig.label}
  </span>

  <!-- Traffic -->
  <div class="shrink-0 text-right">
    <p class="text-xs text-slate-400">{formatBytes(connection.bytesTransferred)}</p>
  </div>

  <!-- Duration -->
  <div class="shrink-0 w-16 text-right">
    <p class="text-xs text-slate-500">{formatDuration(connection.duration)}</p>
  </div>
</div>
