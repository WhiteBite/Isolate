<script lang="ts">
  import { 
    ProblemSelector, 
    StrategySpeedtest, 
    ResultsRecommendation,
    troubleshootStore 
  } from '$lib/components/troubleshoot';
  import { t } from '$lib/i18n';

  // Derived state from store
  let currentStep = $derived(troubleshootStore.step);
  let selectedProblem = $derived(troubleshootStore.selectedProblem);

  function handleStartTest() {
    troubleshootStore.startTesting();
  }

  function handleReset() {
    troubleshootStore.reset();
  }
</script>

<div class="p-6 space-y-6">
  <!-- Header -->
  <header class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold text-white">{t('troubleshoot.title')}</h1>
      <p class="text-zinc-400">{t('troubleshoot.subtitle')}</p>
    </div>
    
    {#if currentStep !== 'select'}
      <button
        type="button"
        onclick={handleReset}
        class="flex items-center gap-2 px-4 py-2 rounded-lg bg-white/5 hover:bg-white/10 text-zinc-400 hover:text-white transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        {t('troubleshoot.buttons.reset')}
      </button>
    {/if}
  </header>

  <!-- Progress Steps -->
  <div class="flex items-center gap-2">
    {#each ['select', 'testing', 'results'] as step, index}
      {@const isActive = currentStep === step}
      {@const isPast = ['select', 'testing', 'results'].indexOf(currentStep) > index}
      
      <div class="flex items-center gap-2">
        <div 
          class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-colors
            {isActive ? 'bg-indigo-500 text-white' : isPast ? 'bg-indigo-500/20 text-indigo-400' : 'bg-white/5 text-zinc-400'}"
        >
          {#if isPast}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          {:else}
            {index + 1}
          {/if}
        </div>
        
        <span class="text-sm {isActive ? 'text-white' : 'text-zinc-400'}">
          {t(`troubleshoot.steps.${step}`)}
        </span>
        
        {#if index < 2}
          <div class="w-12 h-px {isPast ? 'bg-indigo-500/50' : 'bg-white/10'}"></div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Main Content -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Wizard Panel (2/3 width) -->
    <div class="lg:col-span-2">
      <div class="bg-zinc-900/50 rounded-2xl border border-white/5 overflow-hidden">
        {#if currentStep === 'select'}
          <ProblemSelector />
        {:else if currentStep === 'testing'}
          <StrategySpeedtest />
        {:else if currentStep === 'results'}
          <ResultsRecommendation />
        {/if}
      </div>
    </div>

    <!-- AI Pilot Panel (1/3 width) -->
    <div class="space-y-4">
      <!-- AI Assistant Card -->
      <div class="bg-gradient-to-br from-indigo-500/10 to-purple-500/10 rounded-2xl border border-indigo-500/20 p-5">
        <div class="flex items-start gap-3 mb-4">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center flex-shrink-0">
            <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
          </div>
          <div>
            <h3 class="text-white font-semibold">{t('troubleshoot.aiAssistant.title')}</h3>
            <p class="text-zinc-400 text-sm">{t('troubleshoot.aiAssistant.subtitle')}</p>
          </div>
        </div>

        <div class="space-y-3">
          {#if currentStep === 'select' && !selectedProblem}
            <p class="text-zinc-300 text-sm">
              {t('troubleshoot.aiAssistant.selectPrompt')}
            </p>
          {:else if currentStep === 'select' && selectedProblem}
            <p class="text-zinc-300 text-sm">
              {t('troubleshoot.aiAssistant.selectedServicePrefix')} <span class="text-indigo-400 font-medium">{selectedProblem.serviceName}</span>. {t('troubleshoot.aiAssistant.selectedServiceSuffix')}
            </p>
            <button
              type="button"
              onclick={handleStartTest}
              class="w-full py-2.5 px-4 rounded-xl bg-indigo-500 hover:bg-indigo-600 text-white font-medium transition-colors flex items-center justify-center gap-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              {t('troubleshoot.buttons.startTest')}
            </button>
          {:else if currentStep === 'testing'}
            <p class="text-zinc-300 text-sm">
              {t('troubleshoot.aiAssistant.testingPrefix')} <span class="text-indigo-400 font-medium">{selectedProblem?.serviceName}</span>...
            </p>
            <div class="flex items-center gap-2 text-sm text-zinc-400">
              <div class="w-4 h-4 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
              {t('troubleshoot.aiAssistant.testingNote')}
            </div>
          {:else if currentStep === 'results'}
            <p class="text-zinc-300 text-sm">
              {t('troubleshoot.aiAssistant.resultsReady')}
            </p>
          {/if}
        </div>
      </div>

      <!-- Quick Tips -->
      <div class="bg-zinc-900/50 rounded-2xl border border-white/5 p-5">
        <h4 class="text-white font-medium mb-3 flex items-center gap-2">
          <svg class="w-4 h-4 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          {t('troubleshoot.tips.title')}
        </h4>
        <ul class="space-y-2 text-sm text-zinc-400">
          <li class="flex items-start gap-2">
            <span class="text-zinc-400">•</span>
            {t('troubleshoot.tips.closeVpn')}
          </li>
          <li class="flex items-start gap-2">
            <span class="text-zinc-400">•</span>
            {t('troubleshoot.tips.runAsAdmin')}
          </li>
          <li class="flex items-start gap-2">
            <span class="text-zinc-400">•</span>
            {t('troubleshoot.tips.tryVless')}
          </li>
        </ul>
      </div>

      <!-- System Status -->
      <div class="bg-zinc-900/50 rounded-2xl border border-white/5 p-5">
        <h4 class="text-white font-medium mb-3 flex items-center gap-2">
          <svg class="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
          </svg>
          {t('troubleshoot.systemStatus.title')}
        </h4>
        <div class="space-y-2">
          <div class="flex items-center justify-between text-sm">
            <span class="text-zinc-400">{t('troubleshoot.systemStatus.windivert')}</span>
            <span class="text-emerald-400 flex items-center gap-1">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
              {t('troubleshoot.systemStatus.ready')}
            </span>
          </div>
          <div class="flex items-center justify-between text-sm">
            <span class="text-zinc-400">{t('troubleshoot.systemStatus.internet')}</span>
            <span class="text-emerald-400 flex items-center gap-1">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
              {t('troubleshoot.systemStatus.connected')}
            </span>
          </div>
          <div class="flex items-center justify-between text-sm">
            <span class="text-zinc-400">{t('troubleshoot.systemStatus.adminRights')}</span>
            <span class="text-emerald-400 flex items-center gap-1">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
              {t('troubleshoot.systemStatus.available')}
            </span>
          </div>
        </div>
      </div>

      <!-- Advanced Options -->
      <div class="bg-zinc-900/50 rounded-2xl border border-white/5 p-5">
        <h4 class="text-white font-medium mb-3 flex items-center gap-2">
          <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          {t('troubleshoot.advanced.title')}
        </h4>
        <p class="text-sm text-zinc-400 mb-3">
          {t('troubleshoot.advanced.description')}
        </p>
        <a 
          href="/orchestra"
          class="flex items-center justify-center gap-2 w-full py-2.5 px-4 rounded-xl bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-400 text-sm font-medium transition-colors border border-indigo-500/20"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          {t('troubleshoot.advanced.openOrchestra')}
        </a>
      </div>
    </div>
  </div>
</div>
