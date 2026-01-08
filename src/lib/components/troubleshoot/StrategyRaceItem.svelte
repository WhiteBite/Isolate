<script lang="ts">
  import type { StrategyTestState, StrategyTestStatus } from '$lib/stores/troubleshoot.svelte';

  interface Props {
    strategy: StrategyTestState;
    isBest?: boolean;
  }

  let { strategy, isBest = false }: Props = $props();

  const statusConfig: Record<StrategyTestStatus, { color: string; bgColor: string; label: string }> = {
    waiting: { color: 'text-white/40', bgColor: 'bg-white/10', label: 'Ожидание' },
    testing: { color: 'text-blue-400', bgColor: 'bg-blue-500/20', label: 'Тестирование' },
    success: { color: 'text-green-400', bgColor: 'bg-green-500/20', label: 'Успешно' },
    failed: { color: 'text-red-400', bgColor: 'bg-red-500/20', label: 'Не работает' },
    skipped: { color: 'text-white/30', bgColor: 'bg-white/5', label: 'Пропущено' },
  };

  let config = $derived(statusConfig[strategy.status]);
  let progressWidth = $derived(`${strategy.progress}%`);
</script>

<div 
  class="relative p-4 rounded-xl border transition-all duration-300
         {isBest ? 'bg-green-500/10 border-green-500/30' : 'bg-white/5 border-white/10'}"
  role="listitem"
  aria-label="{strategy.name}: {config.label}"
>
  <!-- Header -->
  <div class="flex items-center justify-between mb-3">
    <div class="flex items-center gap-2">
      <span class="font-medium text-white">{strategy.name}</span>
      {#if isBest}
        <span class="px-2 py-0.5 text-xs font-medium bg-green-500/20 text-green-400 rounded-full">
          Лучший
        </span>
      {/if}
    </div>
    
    <div class="flex items-center gap-3">
      {#if strategy.status === 'success' && strategy.latency !== null}
        <span 
          class="px-2 py-1 text-xs font-mono bg-white/10 text-white/80 rounded"
          aria-label="Задержка {strategy.latency} миллисекунд"
        >
          {strategy.latency}ms
        </span>
      {/if}
      
      <span class="text-sm {config.color}">
        {config.label}
      </span>
    </div>
  </div>

  <!-- Progress bar -->
  <div 
    class="relative h-2 rounded-full overflow-hidden {config.bgColor}"
    role="progressbar"
    aria-valuenow={strategy.progress}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label="Прогресс тестирования {strategy.name}"
  >
    <div 
      class="absolute inset-y-0 left-0 rounded-full transition-all duration-300 ease-out
             {strategy.status === 'testing' ? 'bg-blue-500 animate-pulse' : ''}
             {strategy.status === 'success' ? 'bg-green-500' : ''}
             {strategy.status === 'failed' ? 'bg-red-500' : ''}
             {strategy.status === 'waiting' ? 'bg-white/20' : ''}"
      style="width: {progressWidth}"
    >
      {#if strategy.status === 'testing'}
        <!-- Animated shimmer effect -->
        <div 
          class="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent
                 animate-shimmer"
          aria-hidden="true"
        ></div>
      {/if}
    </div>
  </div>

  <!-- Status indicator dot -->
  {#if strategy.status === 'testing'}
    <div 
      class="absolute top-4 right-4 w-2 h-2 rounded-full bg-blue-400 animate-ping"
      aria-hidden="true"
    ></div>
  {/if}
</div>

<style>
  @keyframes shimmer {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }
  
  .animate-shimmer {
    animation: shimmer 1.5s infinite;
  }
</style>
