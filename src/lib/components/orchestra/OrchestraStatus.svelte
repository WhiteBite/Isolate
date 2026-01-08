<script lang="ts">
  import { BentoWidget, Spinner } from '$lib/components';
  import type { OrchestraState } from './types';
  import { 
    getOrchestraStatusColor, 
    getOrchestraStatusIcon, 
    getOrchestraStatusText,
    formatElapsedTime 
  } from './types';

  interface Props {
    state: OrchestraState;
  }

  let { state }: Props = $props();

  let statusColor = $derived(getOrchestraStatusColor(state.status));
  let statusIcon = $derived(getOrchestraStatusIcon(state.status));
  let statusText = $derived(getOrchestraStatusText(state.status));
  let formattedTime = $derived(formatElapsedTime(state.elapsedTime));
</script>

<BentoWidget colspan={2} title="Статус" icon="🎭">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-4">
      <!-- Status indicator -->
      <div class="relative">
        <div class="w-16 h-16 rounded-2xl bg-zinc-800/50 border border-white/5 flex items-center justify-center">
          <span class="text-3xl">{statusIcon}</span>
        </div>
        {#if state.status === 'learning' || state.status === 'running'}
          <div class="absolute inset-0 rounded-2xl border-2 border-amber-500/50 animate-pulse"></div>
        {/if}
      </div>
      
      <div>
        <div class="flex items-center gap-2">
          <span class="text-xl font-semibold {statusColor}">{statusText}</span>
          {#if state.status === 'learning'}
            <Spinner size="sm" />
          {/if}
        </div>
        <p class="text-sm text-zinc-500 mt-1">
          {#if state.currentItem}
            Тестируется: {state.currentItem}
          {:else if state.bestStrategy}
            Лучшая: {state.bestStrategy}
          {:else}
            Готов к оптимизации
          {/if}
        </p>
      </div>
    </div>

    <!-- Timer -->
    <div class="text-right">
      <div class="text-2xl font-mono text-zinc-300">{formattedTime}</div>
      <p class="text-xs text-zinc-500 mt-1">Время</p>
    </div>
  </div>
</BentoWidget>
