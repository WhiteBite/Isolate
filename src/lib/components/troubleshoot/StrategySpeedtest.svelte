<script lang="ts">
  import { troubleshootStore } from '$lib/stores/troubleshoot.svelte';
  import StrategyRaceItem from './StrategyRaceItem.svelte';

  let totalProgress = $derived(() => {
    if (troubleshootStore.strategies.length === 0) return 0;
    const sum = troubleshootStore.strategies.reduce((acc, s) => acc + s.progress, 0);
    return Math.round(sum / troubleshootStore.strategies.length);
  });

  let completedCount = $derived(
    troubleshootStore.strategies.filter(s => s.status === 'success' || s.status === 'failed').length
  );

  let totalCount = $derived(troubleshootStore.strategies.length);
  
  // Текущая тестируемая стратегия
  let currentStrategy = $derived(
    troubleshootStore.strategies.find(s => s.status === 'testing')
  );
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="text-center">
    <h2 class="text-xl font-semibold text-white mb-2">
      Тестирование стратегий
    </h2>
    <p class="text-white/60 text-sm">
      {#if troubleshootStore.selectedProblem}
        Ищем лучший способ разблокировать {troubleshootStore.selectedProblem.serviceName}
      {:else}
        Проверяем доступные стратегии
      {/if}
    </p>
  </div>

  <!-- Current testing indicator -->
  {#if currentStrategy && troubleshootStore.isRunning}
    <div class="p-4 rounded-xl bg-blue-500/10 border border-blue-500/20">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-full bg-blue-500/20 flex items-center justify-center">
          <div class="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"></div>
        </div>
        <div>
          <p class="text-blue-400 font-medium">
            Тестируем {currentStrategy.name}
          </p>
          <p class="text-blue-400/60 text-sm">
            {#if troubleshootStore.selectedProblem}
              Проверяем доступ к {troubleshootStore.selectedProblem.serviceName}...
            {:else}
              Проверяем подключение...
            {/if}
          </p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Overall progress -->
  <div class="bg-white/5 rounded-xl p-4 border border-white/10">
    <div class="flex items-center justify-between mb-2">
      <span class="text-sm text-white/60">Общий прогресс</span>
      <span class="text-sm font-medium text-white">
        {completedCount} / {totalCount}
      </span>
    </div>
    
    <div 
      class="h-3 bg-white/10 rounded-full overflow-hidden"
      role="progressbar"
      aria-valuenow={totalProgress()}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label="Общий прогресс тестирования"
    >
      <div 
        class="h-full bg-gradient-to-r from-blue-500 to-purple-500 rounded-full
               transition-all duration-500 ease-out"
        style="width: {totalProgress()}%"
      ></div>
    </div>
    
    <!-- Time estimate -->
    {#if troubleshootStore.isRunning && totalCount > 0}
      <p class="text-xs text-white/40 mt-2">
        Примерное время: ~{(totalCount - completedCount) * 5} сек
      </p>
    {/if}
  </div>

  <!-- Strategy race list -->
  {#if troubleshootStore.strategies.length > 0}
    <div 
      class="space-y-3"
      role="list"
      aria-label="Список тестируемых стратегий"
    >
      {#each troubleshootStore.strategies as strategy (strategy.id)}
        <StrategyRaceItem 
          {strategy} 
          isBest={troubleshootStore.bestStrategy?.id === strategy.id}
        />
      {/each}
    </div>
  {:else if !troubleshootStore.isRunning}
    <!-- Empty state - no strategies available -->
    <div class="flex flex-col items-center justify-center py-8 gap-4 text-center">
      <div class="w-14 h-14 rounded-full bg-amber-500/10 flex items-center justify-center text-2xl">
        ⚠️
      </div>
      <div>
        <h3 class="text-white font-medium mb-1">Нет доступных стратегий</h3>
        <p class="text-white/50 text-sm max-w-xs">
          Для тестирования нужны настроенные стратегии обхода
        </p>
      </div>
      <button
        type="button"
        class="mt-2 px-4 py-2 rounded-lg bg-white/10 hover:bg-white/15 text-white text-sm font-medium transition-colors"
        onclick={() => troubleshootStore.reset()}
      >
        Вернуться назад
      </button>
    </div>
  {/if}

  <!-- Loading indicator -->
  {#if troubleshootStore.isRunning && !currentStrategy}
    <div class="flex items-center justify-center gap-3 py-4">
      <div class="flex gap-1" aria-hidden="true">
        <span class="w-2 h-2 bg-blue-400 rounded-full animate-bounce" style="animation-delay: 0ms"></span>
        <span class="w-2 h-2 bg-blue-400 rounded-full animate-bounce" style="animation-delay: 150ms"></span>
        <span class="w-2 h-2 bg-blue-400 rounded-full animate-bounce" style="animation-delay: 300ms"></span>
      </div>
      <span class="text-sm text-white/50">Загрузка стратегий...</span>
    </div>
  {/if}

  <!-- Cancel button -->
  <div class="flex justify-center">
    <button
      type="button"
      class="px-4 py-2 text-sm text-white/50 hover:text-white/80
             transition-colors duration-200"
      onclick={() => troubleshootStore.reset()}
    >
      Отменить
    </button>
  </div>
  
  <!-- Info hint -->
  <div class="text-center">
    <p class="text-xs text-white/30">
      Стратегии тестируются последовательно для стабильности
    </p>
  </div>
</div>
