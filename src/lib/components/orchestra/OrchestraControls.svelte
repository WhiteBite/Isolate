<script lang="ts">
  import { BentoWidget } from '$lib/components';
  import type { OrchestraState, OptimizationMode } from './types';
  import { t } from '$lib/i18n';

  interface Props {
    state: OrchestraState;
    mode: OptimizationMode;
    autoApply: boolean;
    selectedServicesCount: number;
    onModeChange: (mode: OptimizationMode) => void;
    onAutoApplyChange: (value: boolean) => void;
    onStart: () => void;
    onPause: () => void;
    onStop: () => void;
  }

  let { 
    state, 
    mode, 
    autoApply, 
    selectedServicesCount,
    onModeChange, 
    onAutoApplyChange, 
    onStart, 
    onPause, 
    onStop 
  }: Props = $props();

  let canStart = $derived(
    (state.status === 'idle' || state.status === 'completed' || state.status === 'error') && 
    selectedServicesCount > 0
  );
  let canPause = $derived(state.status === 'running' || state.status === 'learning');
  let canStop = $derived(state.status !== 'idle');
</script>

<BentoWidget title={t('orchestra.widgets.controls')} icon="🎮">
  <div class="space-y-3">
    <!-- Mode selector -->
    <div class="flex gap-2">
      <button
        onclick={() => onModeChange('turbo')}
        class="flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-all
          {mode === 'turbo' 
            ? 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30' 
            : 'bg-zinc-800/50 text-zinc-400 border border-white/5 hover:bg-zinc-800'}"
        title={t('orchestra.controls.mode.turboTitle')}
      >
        ⚡ {t('orchestra.controls.mode.turbo')}
      </button>
      <button
        onclick={() => onModeChange('deep')}
        class="flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-all
          {mode === 'deep' 
            ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' 
            : 'bg-zinc-800/50 text-zinc-400 border border-white/5 hover:bg-zinc-800'}"
        title={t('orchestra.controls.mode.deepTitle')}
      >
        🔬 {t('orchestra.controls.mode.deep')}
      </button>
    </div>
    
    <!-- Mode description -->
    <p class="text-xs text-zinc-400 px-1">
      {mode === 'turbo' ? t('orchestra.controls.mode.turboDesc') : t('orchestra.controls.mode.deepDesc')}
    </p>

    <!-- Auto-apply toggle -->
    <label class="flex items-center justify-between p-3 bg-zinc-800/30 rounded-lg cursor-pointer hover:bg-zinc-800/50 transition-colors">
      <span class="text-sm text-zinc-300">{t('orchestra.controls.autoApply')}</span>
      <input 
        type="checkbox" 
        checked={autoApply}
        onchange={(e) => onAutoApplyChange(e.currentTarget.checked)}
        class="w-5 h-5 rounded bg-zinc-700 border-zinc-600 text-cyan-500 focus:ring-cyan-500 focus:ring-offset-zinc-900"
      />
    </label>

    <!-- Action buttons -->
    <div class="flex gap-2">
      {#if canStart}
        <button
          onclick={onStart}
          disabled={selectedServicesCount === 0}
          class="flex-1 px-4 py-3 rounded-xl font-semibold text-sm
            {selectedServicesCount > 0 
              ? 'bg-gradient-to-r from-cyan-500 to-indigo-500 text-white hover:from-cyan-400 hover:to-indigo-400 shadow-lg shadow-cyan-500/20'
              : 'bg-zinc-800/50 text-zinc-400 cursor-not-allowed'}
            transition-all"
        >
          ▶ {t('orchestra.controls.start')}
        </button>
      {:else}
        <button
          onclick={onPause}
          disabled={!canPause}
          class="flex-1 px-4 py-3 rounded-xl font-semibold text-sm
            {canPause 
              ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30 hover:bg-amber-500/30' 
              : 'bg-zinc-800/50 text-zinc-400 cursor-not-allowed'}"
        >
          ⏸ {t('orchestra.controls.pause')}
        </button>
        <button
          onclick={onStop}
          disabled={!canStop}
          class="flex-1 px-4 py-3 rounded-xl font-semibold text-sm
            {canStop 
              ? 'bg-red-500/20 text-red-400 border border-red-500/30 hover:bg-red-500/30' 
              : 'bg-zinc-800/50 text-zinc-400 cursor-not-allowed'}"
        >
          ⏹ {t('orchestra.controls.stop')}
        </button>
      {/if}
    </div>
  </div>
</BentoWidget>
