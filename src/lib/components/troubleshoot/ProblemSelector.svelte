<script lang="ts">
  import { troubleshootStore, type ServiceProblem } from '$lib/stores/troubleshoot.svelte';
  import { libraryStore } from '$lib/stores/library.svelte';

  const categoryLabels: Record<string, string> = {
    video: 'Видео',
    social: 'Социальные сети',
    gaming: 'Игры',
    other: 'Другое'
  };

  const categoryOrder = ['video', 'social', 'gaming', 'other'];

  let groupedProblems = $derived(() => {
    const groups: Record<string, ServiceProblem[]> = {};
    for (const problem of troubleshootStore.problems) {
      if (!groups[problem.category]) {
        groups[problem.category] = [];
      }
      groups[problem.category].push(problem);
    }
    return groups;
  });
  
  let hasProblems = $derived(troubleshootStore.problems.length > 0);
  let isDisabled = $derived(troubleshootStore.isRunning || troubleshootStore.problemsLoading);

  function handleSelect(problem: ServiceProblem) {
    if (isDisabled) return;
    troubleshootStore.selectProblem(problem);
  }

  function handleKeyDown(event: KeyboardEvent, problem: ServiceProblem) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleSelect(problem);
    }
  }
  
  /**
   * Получает текущий метод доступа для сервиса из Library
   */
  function getCurrentMethod(serviceId: string): string | null {
    const rule = libraryStore.rules.find(r => r.id === serviceId);
    if (!rule) return null;
    
    // Возвращаем только если это конкретная стратегия
    if (rule.currentMethod.type === 'strategy' && rule.currentMethod.strategyName) {
      return rule.currentMethod.strategyName;
    }
    if (rule.currentMethod.type === 'vless') {
      return 'VLESS';
    }
    if (rule.currentMethod.type === 'proxy' && rule.currentMethod.proxyName) {
      return rule.currentMethod.proxyName;
    }
    
    return null;
  }
</script>

<div class="space-y-6">
  <div class="text-center mb-8">
    <h2 class="text-xl font-semibold text-white mb-2">Что не работает?</h2>
    <p class="text-white/60 text-sm">Выберите сервис, с которым возникли проблемы</p>
  </div>
  
  <!-- Loading state -->
  {#if troubleshootStore.problemsLoading}
    <div class="flex flex-col items-center justify-center py-12 gap-4">
      <div class="w-8 h-8 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"></div>
      <p class="text-white/50 text-sm">Загрузка сервисов...</p>
    </div>
  {:else if !hasProblems}
    <!-- Empty state -->
    <div class="flex flex-col items-center justify-center py-12 gap-4 text-center">
      <div class="w-16 h-16 rounded-full bg-white/5 flex items-center justify-center text-3xl">
        🔍
      </div>
      <div>
        <h3 class="text-white font-medium mb-1">Нет доступных сервисов</h3>
        <p class="text-white/50 text-sm max-w-xs">
          Добавьте сервисы в Library, чтобы диагностировать проблемы с ними
        </p>
      </div>
      <a 
        href="/library" 
        class="mt-2 px-4 py-2 rounded-lg bg-blue-500 hover:bg-blue-400 text-white text-sm font-medium transition-colors"
      >
        Перейти в Library
      </a>
    </div>
  {:else}
    <!-- Hint -->
    <div class="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20 mb-4">
      <p class="text-blue-400/80 text-sm flex items-center gap-2">
        <span>💡</span>
        Мы протестируем несколько стратегий и найдём лучшую для вашего подключения
      </p>
    </div>

    {#each categoryOrder as category}
      {#if groupedProblems()[category]?.length}
        <div class="space-y-3">
          <h3 class="text-sm font-medium text-white/50 uppercase tracking-wider px-1">
            {categoryLabels[category]}
          </h3>
          
          <div 
            class="grid grid-cols-1 sm:grid-cols-2 gap-3"
            role="listbox"
            aria-label={categoryLabels[category]}
          >
            {#each groupedProblems()[category] as problem (problem.id)}
              {@const currentMethod = getCurrentMethod(problem.serviceId)}
              <button
                type="button"
                role="option"
                aria-selected="false"
                disabled={isDisabled}
                class="group relative flex items-start gap-4 p-4 rounded-xl
                       bg-white/5 border border-white/10
                       hover:bg-white/10 hover:border-white/20
                       focus:outline-none focus:ring-2 focus:ring-blue-500/50
                       transition-all duration-200 text-left
                       {isDisabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}"
                onclick={() => handleSelect(problem)}
                onkeydown={(e) => handleKeyDown(e, problem)}
              >
                <!-- Icon -->
                <span 
                  class="flex-shrink-0 w-12 h-12 flex items-center justify-center
                         text-2xl bg-white/5 rounded-lg
                         {!isDisabled ? 'group-hover:bg-white/10 group-hover:scale-110' : ''}
                         transition-all duration-200"
                  aria-hidden="true"
                >
                  {problem.icon}
                </span>
                
                <!-- Content -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-0.5">
                    <h4 class="font-medium text-white {!isDisabled ? 'group-hover:text-blue-400' : ''} transition-colors">
                      {problem.serviceName}
                    </h4>
                    {#if currentMethod}
                      <span class="px-2 py-0.5 text-xs bg-indigo-500/20 text-indigo-400 rounded border border-indigo-500/30">
                        {currentMethod}
                      </span>
                    {/if}
                  </div>
                  <p class="text-sm text-white/50 mt-0.5 line-clamp-2">
                    {problem.description}
                  </p>
                </div>

                <!-- Arrow indicator -->
                <span 
                  class="absolute right-4 top-1/2 -translate-y-1/2 
                         text-white/20 {!isDisabled ? 'group-hover:text-white/50 group-hover:translate-x-1' : ''}
                         transition-all duration-200"
                  aria-hidden="true"
                >
                  →
                </span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    {/each}
  {/if}
</div>
