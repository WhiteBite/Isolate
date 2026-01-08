<script lang="ts">
  import { troubleshootStore } from '$lib/stores/troubleshoot.svelte';

  let successfulStrategies = $derived(
    troubleshootStore.strategies
      .filter(s => s.status === 'success')
      .sort((a, b) => (a.latency || 999) - (b.latency || 999))
  );

  let failedStrategies = $derived(
    troubleshootStore.strategies.filter(s => s.status === 'failed')
  );

  let hasResults = $derived(successfulStrategies.length > 0);
  
  // Топ-2 стратегии для сравнения
  let topTwo = $derived(successfulStrategies.slice(0, 2));
  
  // Разница в латенси между топ-2
  let latencyDiff = $derived(() => {
    if (topTwo.length < 2) return null;
    const diff = (topTwo[1].latency || 0) - (topTwo[0].latency || 0);
    return diff > 0 ? diff : null;
  });
  
  let isApplying = $state(false);
  
  async function handleApply() {
    isApplying = true;
    await troubleshootStore.applyResult();
    isApplying = false;
  }
  
  function retryApply() {
    troubleshootStore.applyError = null;
    handleApply();
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="text-center">
    <h2 class="text-xl font-semibold text-white mb-2">
      {#if hasResults}
        Найдено решение! 🎉
      {:else}
        Решение не найдено 😔
      {/if}
    </h2>
    <p class="text-white/60 text-sm">
      {#if troubleshootStore.selectedProblem}
        {#if hasResults}
          Для {troubleshootStore.selectedProblem.serviceName} подходит {successfulStrategies.length} 
          {successfulStrategies.length === 1 ? 'стратегия' : 'стратегий'}
        {:else}
          Не удалось найти рабочую стратегию для {troubleshootStore.selectedProblem.serviceName}
        {/if}
      {/if}
    </p>
  </div>

  {#if hasResults && troubleshootStore.bestStrategy}
    <!-- Best strategy card -->
    <div 
      class="relative p-6 rounded-2xl bg-gradient-to-br from-green-500/20 to-emerald-500/10
             border border-green-500/30"
      role="region"
      aria-label="Рекомендуемая стратегия"
    >
      <!-- Badge -->
      <div class="absolute -top-3 left-4">
        <span class="px-3 py-1 text-xs font-semibold bg-green-500 text-white rounded-full shadow-lg">
          Рекомендуется
        </span>
      </div>

      <div class="flex items-center justify-between mt-2">
        <div>
          <h3 class="text-2xl font-bold text-white">
            {troubleshootStore.bestStrategy.name}
          </h3>
          <p class="text-green-400/80 text-sm mt-1">
            Лучшая производительность для вашего подключения
          </p>
        </div>
        
        {#if troubleshootStore.bestStrategy.latency !== null}
          <div class="text-right">
            <div class="text-3xl font-bold text-green-400">
              {troubleshootStore.bestStrategy.latency}
              <span class="text-lg font-normal">ms</span>
            </div>
            <div class="text-xs text-white/50">задержка</div>
          </div>
        {/if}
      </div>

      <!-- Apply button -->
      <button
        type="button"
        disabled={isApplying}
        class="w-full mt-6 py-3 px-4 rounded-xl font-medium
               bg-green-500 hover:bg-green-400 text-white
               focus:outline-none focus:ring-2 focus:ring-green-500/50 focus:ring-offset-2 focus:ring-offset-transparent
               transition-all duration-200 transform hover:scale-[1.02] active:scale-[0.98]
               disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
        onclick={handleApply}
      >
        {#if isApplying}
          <span class="flex items-center justify-center gap-2">
            <span class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
            Применяем...
          </span>
        {:else}
          Применить стратегию
        {/if}
      </button>
      
      <!-- Apply error -->
      {#if troubleshootStore.applyError}
        <div class="mt-3 p-3 rounded-lg bg-red-500/10 border border-red-500/20">
          <p class="text-red-400 text-sm mb-2">
            Не удалось применить: {troubleshootStore.applyError}
          </p>
          <button
            type="button"
            class="text-sm text-red-400 hover:text-red-300 underline"
            onclick={retryApply}
          >
            Попробовать снова
          </button>
        </div>
      {/if}
    </div>
    
    <!-- Comparison with second best -->
    {#if topTwo.length >= 2 && latencyDiff()}
      <div class="p-4 rounded-xl bg-white/5 border border-white/10">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-full bg-blue-500/20 flex items-center justify-center text-blue-400">
            📊
          </div>
          <div class="flex-1">
            <p class="text-white/80 text-sm">
              <span class="font-medium text-white">{topTwo[0].name}</span> быстрее чем 
              <span class="font-medium text-white">{topTwo[1].name}</span> на 
              <span class="text-green-400 font-medium">{latencyDiff()}ms</span>
            </p>
          </div>
        </div>
      </div>
    {/if}
    
    <!-- Summary of what will change -->
    <div class="p-4 rounded-xl bg-blue-500/10 border border-blue-500/20">
      <h4 class="text-blue-400 font-medium mb-2 flex items-center gap-2">
        <span>ℹ️</span> Что будет изменено
      </h4>
      <p class="text-blue-400/80 text-sm">
        Для сервиса <span class="font-medium">{troubleshootStore.selectedProblem?.serviceName}</span> будет 
        установлена стратегия <span class="font-medium">{troubleshootStore.bestStrategy.name}</span>.
        Вы сможете изменить это в Library.
      </p>
    </div>
  {/if}

  <!-- Results table -->
  {#if troubleshootStore.strategies.length > 0}
    <div class="bg-white/5 rounded-xl border border-white/10 overflow-hidden">
      <div class="px-4 py-3 border-b border-white/10">
        <h3 class="text-sm font-medium text-white/80">Все результаты</h3>
      </div>
      
      <div class="divide-y divide-white/5">
        {#each troubleshootStore.strategies as strategy (strategy.id)}
          <div 
            class="flex items-center justify-between px-4 py-3
                   {strategy.status === 'success' ? 'bg-green-500/5' : ''}
                   {strategy.status === 'failed' ? 'bg-red-500/5' : ''}"
          >
            <div class="flex items-center gap-3">
              <!-- Status icon -->
              <span 
                class="w-6 h-6 flex items-center justify-center rounded-full text-sm
                       {strategy.status === 'success' ? 'bg-green-500/20 text-green-400' : ''}
                       {strategy.status === 'failed' ? 'bg-red-500/20 text-red-400' : ''}"
                aria-hidden="true"
              >
                {#if strategy.status === 'success'}✓{:else}✕{/if}
              </span>
              
              <span class="font-medium text-white">
                {strategy.name}
              </span>
              
              {#if troubleshootStore.bestStrategy?.id === strategy.id}
                <span class="px-2 py-0.5 text-xs bg-green-500/20 text-green-400 rounded">
                  Лучший
                </span>
              {/if}
            </div>
            
            <div class="flex items-center gap-4">
              {#if strategy.latency !== null}
                <span class="text-sm font-mono text-white/60">
                  {strategy.latency}ms
                </span>
              {/if}
              
              <span 
                class="text-sm
                       {strategy.status === 'success' ? 'text-green-400' : ''}
                       {strategy.status === 'failed' ? 'text-red-400' : ''}"
              >
                {strategy.status === 'success' ? 'Работает' : 'Не работает'}
              </span>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Action buttons -->
  <div class="flex gap-3">
    <button
      type="button"
      class="flex-1 py-3 px-4 rounded-xl font-medium
             bg-white/10 hover:bg-white/15 text-white
             border border-white/10 hover:border-white/20
             focus:outline-none focus:ring-2 focus:ring-white/20
             transition-all duration-200"
      onclick={() => troubleshootStore.reset()}
    >
      Выбрать другую проблему
    </button>
    
    {#if !hasResults}
      <button
        type="button"
        class="flex-1 py-3 px-4 rounded-xl font-medium
               bg-blue-500 hover:bg-blue-400 text-white
               focus:outline-none focus:ring-2 focus:ring-blue-500/50
               transition-all duration-200"
        onclick={() => {
          troubleshootStore.step = 'testing';
          troubleshootStore.startTesting();
        }}
      >
        Попробовать снова
      </button>
    {/if}
  </div>

  <!-- Help text for no results -->
  {#if !hasResults}
    <div class="p-4 rounded-xl bg-amber-500/10 border border-amber-500/20">
      <p class="text-sm text-amber-400/90">
        <strong>Совет:</strong> Попробуйте использовать VLESS прокси или проверьте 
        подключение к интернету. Если проблема сохраняется — обратитесь в поддержку.
      </p>
    </div>
  {/if}
  
  <!-- Success link to Library -->
  {#if hasResults}
    <div class="text-center">
      <a 
        href="/library" 
        class="text-sm text-white/40 hover:text-white/60 transition-colors"
      >
        Посмотреть в Library →
      </a>
    </div>
  {/if}
</div>
