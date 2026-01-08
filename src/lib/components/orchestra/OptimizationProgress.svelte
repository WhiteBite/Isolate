<script lang="ts">
  import { BentoWidget } from '$lib/components';
  import type { OrchestraState, QueueItem } from './types';
  import { t } from '$lib/i18n';

  interface Props {
    state: OrchestraState;
    queueLength: number;
  }

  let { state, queueLength }: Props = $props();

  // Calculate ETA based on elapsed time and progress
  let eta = $derived.by(() => {
    if (state.status !== 'learning' || state.progress <= 0 || state.elapsedTime <= 0) {
      return null;
    }
    
    const remainingPercent = 100 - state.progress;
    const msPerPercent = state.elapsedTime / state.progress;
    const remainingMs = remainingPercent * msPerPercent;
    
    return Math.round(remainingMs / 1000);
  });

  function formatTime(seconds: number): string {
    if (seconds < 60) return `${seconds} ${t('orchestra.progress.seconds')}`;
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return secs > 0 ? `${mins} ${t('orchestra.progress.minutes')} ${secs} ${t('orchestra.progress.seconds')}` : `${mins} ${t('orchestra.progress.minutes')}`;
  }

  function formatElapsed(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    return formatTime(seconds);
  }
</script>

<BentoWidget colspan={2} title={t('orchestra.widgets.progress')} icon="📊">
  <div class="space-y-4">
    <!-- Progress bar -->
    <div>
      <div class="flex justify-between items-center mb-2">
        <span class="text-sm text-zinc-400">
          {state.testedItems} / {state.totalItems || queueLength} {t('orchestra.progress.strategies')}
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

    <!-- Time info -->
    {#if state.status === 'learning' || state.elapsedTime > 0}
      <div class="flex items-center justify-between text-xs text-zinc-400">
        <div class="flex items-center gap-1.5">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{t('orchestra.progress.elapsed')}: {formatElapsed(state.elapsedTime)}</span>
        </div>
        {#if eta !== null && state.status === 'learning'}
          <div class="flex items-center gap-1.5 text-cyan-400">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
            <span>{t('orchestra.progress.remaining')}: ~{formatTime(eta)}</span>
          </div>
        {/if}
      </div>
    {/if}

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
