<script lang="ts">
  import { BentoWidget, Spinner } from '$lib/components';
  import type { QueueItem } from './types';
  import { getStatusBadgeClass, getStatusIcon } from './types';

  interface Props {
    queue: QueueItem[];
  }

  let { queue }: Props = $props();
</script>

<BentoWidget colspan={2} rowspan={2} title="Очередь стратегий" icon="📋">
  <div class="h-full flex flex-col">
    <div class="flex-1 overflow-auto space-y-2 pr-1 -mr-1">
      {#each queue as item, i (item.id)}
        <div 
          class="flex items-center gap-3 p-3 rounded-lg transition-all duration-200
            {item.status === 'testing' 
              ? 'bg-amber-500/10 border border-amber-500/20' 
              : 'bg-zinc-800/30 border border-white/5 hover:bg-zinc-800/50'}"
        >
          <!-- Index -->
          <div class="w-6 h-6 rounded-full bg-zinc-800 flex items-center justify-center text-xs text-zinc-500 font-mono">
            {i + 1}
          </div>

          <!-- Info -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-zinc-200 truncate">{item.name}</span>
              {#if item.status === 'testing'}
                <Spinner size="xs" />
              {/if}
            </div>
            <div class="flex items-center gap-2 mt-0.5">
              {#if item.score !== undefined}
                <span class="text-xs text-zinc-500">Score: <span class="text-cyan-400">{item.score.toFixed(1)}</span></span>
              {/if}
              {#if item.latency !== undefined}
                <span class="text-xs text-zinc-500">Задержка: <span class="text-zinc-400">{item.latency.toFixed(0)}ms</span></span>
              {/if}
            </div>
          </div>

          <!-- Status badge -->
          <div class="px-2 py-1 rounded-md text-xs font-medium border {getStatusBadgeClass(item.status)}">
            <span class="mr-1">{getStatusIcon(item.status)}</span>
            {item.status}
          </div>
        </div>
      {/each}

      {#if queue.length === 0}
        <div class="flex flex-col items-center justify-center h-32 text-zinc-500">
          <span class="text-3xl mb-2">📭</span>
          <span class="text-sm">Нет стратегий в очереди</span>
        </div>
      {/if}
    </div>
  </div>
</BentoWidget>
