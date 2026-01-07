<script lang="ts">
  import { BentoWidget } from '$lib/components';
  import type { OrchestraState, QueueItem } from './types';

  interface Props {
    state: OrchestraState;
    queueLength: number;
  }

  let { state, queueLength }: Props = $props();
</script>

<BentoWidget colspan={2} title="Progress" icon="📊">
  <div class="space-y-4">
    <!-- Progress bar -->
    <div>
      <div class="flex justify-between items-center mb-2">
        <span class="text-sm text-zinc-400">
          {state.testedItems} / {state.totalItems || queueLength} strategies
        </span>
        <span class="text-sm font-mono text-cyan-400">{state.progress.toFixed(0)}%</span>
      </div>
      <div class="h-3 bg-zinc-800/50 rounded-full overflow-hidden border border-white/5">
        <div 
          class="h-full bg-gradient-to-r from-cyan-500 to-indigo-500 rounded-full transition-all duration-500 ease-out"
          class:animate-pulse={state.status === 'learning'}
          style="width: {state.progress}%"
        ></div>
      </div>
    </div>

    <!-- Best score -->
    {#if state.bestScore > 0}
      <div class="flex items-center justify-between p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-lg">
        <div class="flex items-center gap-2">
          <span class="text-emerald-400">🏆</span>
          <span class="text-sm text-emerald-300">{state.bestStrategy}</span>
        </div>
        <span class="text-lg font-bold text-emerald-400">{state.bestScore.toFixed(1)}</span>
      </div>
    {/if}
  </div>
</BentoWidget>
