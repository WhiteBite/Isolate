<script lang="ts">
  import { troubleshootStore } from '$lib/stores/troubleshoot.svelte';
  import { ProblemSelector, StrategySpeedtest, ResultsRecommendation } from './index';

  // Текущий шаг
  let currentStep = $derived(troubleshootStore.step);
  
  // Индекс шага для анимации
  let stepIndex = $derived(() => {
    switch (currentStep) {
      case 'select': return 0;
      case 'testing': return 1;
      case 'results': return 2;
      default: return 0;
    }
  });

  // Можно ли вернуться назад
  let canGoBack = $derived(
    currentStep === 'testing' && !troubleshootStore.isRunning
  );

  // Названия шагов для индикатора
  const steps = [
    { id: 'select', label: 'Проблема', icon: '🔍' },
    { id: 'testing', label: 'Тестирование', icon: '⚡' },
    { id: 'results', label: 'Результат', icon: '✓' }
  ];

  function goBack() {
    if (currentStep === 'testing' && !troubleshootStore.isRunning) {
      troubleshootStore.step = 'select';
      troubleshootStore.selectedProblem = null;
    }
  }

  function handleClose() {
    troubleshootStore.reset();
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header с индикатором шагов -->
  <div class="flex-shrink-0 px-6 pt-6 pb-4">
    <!-- Step indicator -->
    <div class="flex items-center justify-center gap-2 mb-6">
      {#each steps as step, i (step.id)}
        <div class="flex items-center">
          <!-- Step circle -->
          <div 
            class="flex items-center justify-center w-10 h-10 rounded-full
                   transition-all duration-300 ease-out
                   {i <= stepIndex() ? 'bg-blue-500 text-white scale-100' : 'bg-white/10 text-white/40 scale-90'}
                   {i === stepIndex() ? 'ring-4 ring-blue-500/30' : ''}"
            aria-current={i === stepIndex() ? 'step' : undefined}
          >
            {#if i < stepIndex()}
              <span class="text-lg">✓</span>
            {:else}
              <span class="text-lg">{step.icon}</span>
            {/if}
          </div>
          
          <!-- Connector line -->
          {#if i < steps.length - 1}
            <div 
              class="w-12 h-0.5 mx-2 transition-all duration-500
                     {i < stepIndex() ? 'bg-blue-500' : 'bg-white/10'}"
            ></div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Step labels -->
    <div class="flex justify-between px-2">
      {#each steps as step, i (step.id)}
        <span 
          class="text-xs font-medium transition-colors duration-300
                 {i <= stepIndex() ? 'text-white' : 'text-white/40'}"
        >
          {step.label}
        </span>
      {/each}
    </div>
  </div>

  <!-- Content area с анимацией -->
  <div class="flex-1 overflow-y-auto px-6 py-4">
    <div class="relative">
      <!-- Animated content wrapper -->
      {#key currentStep}
        <div 
          class="animate-in fade-in slide-in-from-right-4 duration-300"
        >
          {#if currentStep === 'select'}
            <ProblemSelector />
          {:else if currentStep === 'testing'}
            <StrategySpeedtest />
          {:else if currentStep === 'results'}
            <ResultsRecommendation />
          {/if}
        </div>
      {/key}
    </div>
  </div>

  <!-- Footer с навигацией -->
  <div class="flex-shrink-0 px-6 py-4 border-t border-white/10">
    <div class="flex items-center justify-between">
      <!-- Back button -->
      <div>
        {#if canGoBack}
          <button
            type="button"
            class="flex items-center gap-2 px-4 py-2 rounded-lg
                   text-white/60 hover:text-white hover:bg-white/10
                   transition-all duration-200"
            onclick={goBack}
          >
            <span aria-hidden="true">←</span>
            <span>Назад</span>
          </button>
        {:else}
          <div class="w-20"></div>
        {/if}
      </div>

      <!-- Current step info -->
      <div class="text-center">
        <span class="text-sm text-white/40">
          Шаг {stepIndex() + 1} из {steps.length}
        </span>
      </div>

      <!-- Close/Cancel button -->
      <div>
        <button
          type="button"
          class="flex items-center gap-2 px-4 py-2 rounded-lg
                 text-white/60 hover:text-white hover:bg-white/10
                 transition-all duration-200"
          onclick={handleClose}
        >
          <span>Закрыть</span>
          <span aria-hidden="true">✕</span>
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  /* Tailwind animate-in utilities */
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  
  @keyframes slideInFromRight {
    from { transform: translateX(1rem); }
    to { transform: translateX(0); }
  }
  
  .animate-in {
    animation: fadeIn 0.3s ease-out, slideInFromRight 0.3s ease-out;
  }
  
  .fade-in {
    animation-name: fadeIn;
  }
  
  .slide-in-from-right-4 {
    animation-name: slideInFromRight;
  }
  
  .duration-300 {
    animation-duration: 300ms;
  }
</style>
