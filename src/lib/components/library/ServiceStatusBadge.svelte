<script lang="ts">
  import type { ServiceStatus } from '$lib/stores/library.svelte';

  interface Props {
    status: ServiceStatus;
    ping?: number;
    showPing?: boolean;
  }

  let { status, ping, showPing = true }: Props = $props();

  const statusConfig = $derived({
    accessible: {
      label: 'Доступен',
      bgClass: 'bg-emerald-500/20',
      textClass: 'text-emerald-400',
      dotClass: 'bg-emerald-400'
    },
    blocked: {
      label: 'Заблокирован',
      bgClass: 'bg-red-500/20',
      textClass: 'text-red-400',
      dotClass: 'bg-red-400'
    },
    unknown: {
      label: 'Неизвестно',
      bgClass: 'bg-zinc-500/20',
      textClass: 'text-zinc-400',
      dotClass: 'bg-zinc-400'
    },
    checking: {
      label: 'Проверка...',
      bgClass: 'bg-blue-500/20',
      textClass: 'text-blue-400',
      dotClass: 'bg-blue-400'
    }
  }[status]);
</script>

<div 
  class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium {statusConfig.bgClass} {statusConfig.textClass}"
  role="status"
  aria-label="Статус: {statusConfig.label}"
>
  {#if status === 'checking'}
    <span class="w-2 h-2 rounded-full {statusConfig.dotClass} animate-pulse"></span>
  {:else}
    <span class="w-2 h-2 rounded-full {statusConfig.dotClass}"></span>
  {/if}
  
  <span>{statusConfig.label}</span>
  
  {#if showPing && ping !== undefined && status === 'accessible'}
    <span class="text-zinc-500">•</span>
    <span class="text-zinc-400">{ping}ms</span>
  {/if}
</div>
