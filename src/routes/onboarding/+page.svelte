<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import type { Service, DiagnosticResult, OptimizationProgress } from '$lib/api';

  // Step definitions
  type Step = 'welcome' | 'services' | 'diagnostics' | 'optimization' | 'complete';
  const steps: Step[] = ['welcome', 'services', 'diagnostics', 'optimization', 'complete'];
  
  // State
  let currentStep = $state<Step>('welcome');
  let currentStepIndex = $derived(steps.indexOf(currentStep));
  let stepDirection = $state<'forward' | 'backward'>('forward');
  let isTransitioning = $state(false);
  
  // Services state
  let availableServices = $state<Service[]>([]);
  let selectedServices = $state<Set<string>>(new Set(['youtube', 'discord']));
  let loadingServices = $state(true);
  
  // Diagnostics state
  let diagnosticResult = $state<DiagnosticResult | null>(null);
  let diagnosticError = $state<string | null>(null);
  let isDiagnosing = $state(false);
  
  // Optimization state
  let optimizationProgress = $state<OptimizationProgress | null>(null);
  let optimizationResult = $state<{ strategy_id: string; strategy_name: string; score: number } | null>(null);
  let optimizationError = $state<string | null>(null);
  let isOptimizing = $state(false);
  
  // Derived states
  let canProceed = $derived(
    currentStep === 'welcome' ||
    (currentStep === 'services' && selectedServices.size > 0) ||
    (currentStep === 'diagnostics' && !isDiagnosing) ||
    (currentStep === 'optimization' && !isOptimizing) ||
    currentStep === 'complete'
  );
  
  let progressPercent = $derived(((currentStepIndex + 1) / steps.length) * 100);
  
  // Event listeners
  let unlistenProgress: UnlistenFn | null = null;
  let unlistenComplete: UnlistenFn | null = null;
  let unlistenFailed: UnlistenFn | null = null;

  onMount(async () => {
    if (!browser) return;
    await loadServices();
    await setupEventListeners();
  });

  onDestroy(() => {
    unlistenProgress?.();
    unlistenComplete?.();
    unlistenFailed?.();
  });
  
  // Auto-run diagnostics/optimization when entering those steps
  $effect(() => {
    if (currentStep === 'diagnostics' && !diagnosticResult && !isDiagnosing && !diagnosticError) {
      runDiagnostics();
    }
  });
  
  $effect(() => {
    if (currentStep === 'optimization' && !optimizationResult && !isOptimizing && !optimizationError) {
      runOptimization();
    }
  });

  async function loadServices() {
    try {
      const { getServices } = await import('$lib/api');
      availableServices = await getServices();
      if (availableServices.length === 0) {
        // Fallback services
        availableServices = [
          { id: 'youtube', name: 'YouTube', critical: true },
          { id: 'discord', name: 'Discord', critical: true },
          { id: 'twitch', name: 'Twitch', critical: false },
          { id: 'telegram', name: 'Telegram', critical: false },
          { id: 'spotify', name: 'Spotify', critical: false }
        ];
      }
    } catch (e) {
      console.error('Failed to load services:', e);
      availableServices = [
        { id: 'youtube', name: 'YouTube', critical: true },
        { id: 'discord', name: 'Discord', critical: true },
        { id: 'twitch', name: 'Twitch', critical: false },
        { id: 'telegram', name: 'Telegram', critical: false }
      ];
    } finally {
      loadingServices = false;
    }
  }

  async function setupEventListeners() {
    const { listen } = await import('@tauri-apps/api/event');
    
    unlistenProgress = await listen('optimization:progress', (event) => {
      optimizationProgress = event.payload as OptimizationProgress;
    });
    
    unlistenComplete = await listen('optimization:complete', (event) => {
      const result = event.payload as { strategy_id: string; strategy_name: string; score: number };
      optimizationResult = result;
      isOptimizing = false;
      // Auto-advance to complete step
      setTimeout(() => {
        currentStep = 'complete';
      }, 1500);
    });
    
    unlistenFailed = await listen('optimization:failed', (event) => {
      optimizationError = event.payload as string;
      isOptimizing = false;
    });
  }

  function toggleService(serviceId: string) {
    const newSet = new Set(selectedServices);
    if (newSet.has(serviceId)) {
      newSet.delete(serviceId);
    } else {
      newSet.add(serviceId);
    }
    selectedServices = newSet;
  }

  async function runDiagnostics() {
    isDiagnosing = true;
    diagnosticError = null;
    diagnosticResult = null;
    
    try {
      const { diagnose } = await import('$lib/api');
      diagnosticResult = await diagnose();
    } catch (e) {
      console.error('Diagnostics failed:', e);
      diagnosticError = String(e);
    } finally {
      isDiagnosing = false;
    }
  }

  async function runOptimization() {
    isOptimizing = true;
    optimizationError = null;
    optimizationResult = null;
    optimizationProgress = null;
    
    try {
      const { runOptimization: optimize } = await import('$lib/api');
      await optimize('turbo');
    } catch (e) {
      console.error('Optimization failed:', e);
      optimizationError = String(e);
      isOptimizing = false;
    }
  }

  async function completeOnboarding() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Save settings
      await invoke('save_settings', {
        settings: {
          auto_start: false,
          auto_apply: true,
          minimize_to_tray: true,
          block_quic: true,
          default_mode: 'turbo',
          system_proxy: false,
          tun_mode: false,
          per_domain_routing: false,
          per_app_routing: false,
          test_timeout: 5,
          test_services: Array.from(selectedServices),
          language: 'ru',
          telemetry_enabled: false
        }
      });
      
      // Mark onboarding complete
      await invoke('set_setting', { key: 'onboarding_complete', value: true }).catch(() => {});
      
      goto('/');
    } catch (e) {
      console.error('Failed to complete onboarding:', e);
      goto('/');
    }
  }

  function nextStep() {
    const idx = currentStepIndex;
    if (idx < steps.length - 1) {
      stepDirection = 'forward';
      isTransitioning = true;
      setTimeout(() => {
        currentStep = steps[idx + 1];
        isTransitioning = false;
      }, 150);
    } else {
      completeOnboarding();
    }
  }

  function prevStep() {
    const idx = currentStepIndex;
    if (idx > 0) {
      stepDirection = 'backward';
      isTransitioning = true;
      setTimeout(() => {
        currentStep = steps[idx - 1];
        isTransitioning = false;
      }, 150);
    }
  }

  function skipOnboarding() {
    completeOnboarding();
  }

  function getServiceIcon(serviceId: string): string {
    const icons: Record<string, string> = {
      youtube: '📺',
      discord: '💬',
      twitch: '🎮',
      telegram: '✈️',
      spotify: '🎵',
      google: '🔍'
    };
    return icons[serviceId] || '🌐';
  }

  function getStageText(stage: string): string {
    const stages: Record<string, string> = {
      'initializing': 'Инициализация...',
      'checking_cache': 'Проверка кэша...',
      'diagnosing': 'Диагностика DPI...',
      'selecting_candidates': 'Выбор стратегий...',
      'testing_vless': 'Тестирование VLESS...',
      'testing_zapret': 'Тестирование Zapret...',
      'selecting_best': 'Выбор лучшей...',
      'applying': 'Применение стратегии...',
      'completed': 'Завершено!',
      'failed': 'Ошибка'
    };
    return stages[stage] || stage;
  }
</script>

<div class="min-h-screen bg-[#0a0e27] flex flex-col items-center justify-center p-8">
  <div class="w-full max-w-lg">
    <!-- Progress Bar (линия) -->
    <div class="mb-2">
      <div class="h-1 bg-[#2a2f4a] rounded-full overflow-hidden">
        <div 
          class="h-full bg-gradient-to-r from-[#00d4ff] to-[#00ff88] rounded-full transition-all duration-500 ease-out"
          style="width: {progressPercent}%"
        ></div>
      </div>
    </div>
    
    <!-- Progress Indicator (точки) -->
    <div class="flex justify-center items-center gap-2 mb-8">
      {#each steps as step, i}
        <button 
          onclick={() => {
            if (i < currentStepIndex) {
              stepDirection = 'backward';
              isTransitioning = true;
              setTimeout(() => {
                currentStep = steps[i];
                isTransitioning = false;
              }, 150);
            }
          }}
          disabled={i >= currentStepIndex}
          class="h-2 rounded-full transition-all duration-500 {i <= currentStepIndex ? 'bg-[#00d4ff]' : 'bg-[#2a2f4a]'} {i === currentStepIndex ? 'w-8' : 'w-4'} {i < currentStepIndex ? 'cursor-pointer hover:bg-[#00b8e6]' : 'cursor-default'}"
        ></button>
      {/each}
    </div>

    <!-- Step Content Container -->
    <div class="bg-[#1a1f3a] rounded-2xl p-8 border border-[#2a2f4a] shadow-2xl shadow-[#00d4ff]/5 min-h-[500px] flex flex-col overflow-hidden">
      
      <!-- Step 1: Welcome -->
      {#if currentStep === 'welcome'}
        <div class="flex-1 flex flex-col items-center justify-center text-center space-y-6 {isTransitioning ? 'animate-fade-out' : 'animate-fade-in'} {stepDirection === 'backward' ? 'slide-from-left' : 'slide-from-right'}">
          <div class="w-24 h-24 bg-[#00d4ff]/20 rounded-full flex items-center justify-center">
            <svg class="w-12 h-12 text-[#00d4ff]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          
          <div>
            <h1 class="text-3xl font-bold text-white mb-3">Добро пожаловать в Isolate</h1>
            <p class="text-[#a0a0a0] leading-relaxed max-w-md">
              Isolate автоматически находит и применяет лучший способ обхода DPI-блокировок для вашего интернет-провайдера.
            </p>
          </div>
          
          <div class="space-y-3 text-left w-full max-w-sm">
            <div class="flex items-center gap-3 text-[#a0a0a0]">
              <div class="w-8 h-8 rounded-lg bg-[#00ff88]/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-4 h-4 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <span>Автоматический подбор стратегии</span>
            </div>
            <div class="flex items-center gap-3 text-[#a0a0a0]">
              <div class="w-8 h-8 rounded-lg bg-[#00ff88]/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-4 h-4 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <span>Поддержка Discord, YouTube, Telegram</span>
            </div>
            <div class="flex items-center gap-3 text-[#a0a0a0]">
              <div class="w-8 h-8 rounded-lg bg-[#00ff88]/20 flex items-center justify-center flex-shrink-0">
                <svg class="w-4 h-4 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <span>Работает в один клик</span>
            </div>
          </div>
        </div>
      {/if}

      <!-- Step 2: Services -->
      {#if currentStep === 'services'}
        <div class="flex-1 flex flex-col {isTransitioning ? 'animate-fade-out' : 'animate-fade-in'} {stepDirection === 'backward' ? 'slide-from-left' : 'slide-from-right'}">
          <div class="text-center mb-6">
            <h2 class="text-2xl font-bold text-white mb-2">Выберите сервисы</h2>
            <p class="text-[#a0a0a0]">Какие сервисы вы хотите разблокировать?</p>
          </div>
          
          {#if loadingServices}
            <div class="flex-1 flex items-center justify-center">
              <div class="w-8 h-8 border-2 border-[#00d4ff] border-t-transparent rounded-full animate-spin"></div>
            </div>
          {:else}
            <div class="space-y-3 flex-1 overflow-y-auto">
              {#each availableServices as service}
                <button
                  onclick={() => toggleService(service.id)}
                  class="w-full p-4 rounded-xl border-2 transition-all duration-200 text-left flex items-center gap-4 {selectedServices.has(service.id) ? 'border-[#00d4ff] bg-[#00d4ff]/10' : 'border-[#2a2f4a] bg-[#2a2f4a]/50 hover:border-[#3a3f5a]'}"
                >
                  <div class="w-10 h-10 rounded-lg bg-[#2a2f4a] flex items-center justify-center text-xl">
                    {getServiceIcon(service.id)}
                  </div>
                  <div class="flex-1">
                    <div class="font-medium text-white flex items-center gap-2">
                      {service.name}
                      {#if service.critical}
                        <span class="text-xs px-2 py-0.5 bg-[#ffaa00]/20 text-[#ffaa00] rounded">Популярный</span>
                      {/if}
                    </div>
                  </div>
                  <div class="w-6 h-6 rounded-md border-2 flex items-center justify-center transition-colors {selectedServices.has(service.id) ? 'border-[#00d4ff] bg-[#00d4ff]' : 'border-[#a0a0a0]'}">
                    {#if selectedServices.has(service.id)}
                      <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                      </svg>
                    {/if}
                  </div>
                </button>
              {/each}
            </div>
            
            {#if selectedServices.size === 0}
              <p class="text-[#ffaa00] text-sm text-center mt-4">
                Выберите хотя бы один сервис для продолжения
              </p>
            {/if}
          {/if}
        </div>
      {/if}

      <!-- Step 3: Diagnostics -->
      {#if currentStep === 'diagnostics'}
        <div class="flex-1 flex flex-col items-center justify-center text-center space-y-6 {isTransitioning ? 'animate-fade-out' : 'animate-fade-in'} {stepDirection === 'backward' ? 'slide-from-left' : 'slide-from-right'}">
          {#if isDiagnosing}
            <div class="w-24 h-24 bg-[#00d4ff]/20 rounded-full flex items-center justify-center">
              <div class="w-16 h-16 border-4 border-[#00d4ff] border-t-transparent rounded-full animate-spin"></div>
            </div>
            <div>
              <h2 class="text-2xl font-bold text-white mb-2">Диагностика сети</h2>
              <p class="text-[#a0a0a0]">Анализируем тип DPI вашего провайдера...</p>
            </div>
            <div class="w-full max-w-xs">
              <div class="h-1 bg-[#2a2f4a] rounded-full overflow-hidden">
                <div class="h-full bg-[#00d4ff] rounded-full animate-progress"></div>
              </div>
            </div>
          {:else if diagnosticError}
            <div class="w-24 h-24 bg-[#ff3333]/20 rounded-full flex items-center justify-center">
              <svg class="w-12 h-12 text-[#ff3333]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <div>
              <h2 class="text-2xl font-bold text-white mb-2">Ошибка диагностики</h2>
              <p class="text-[#a0a0a0] mb-4">{diagnosticError}</p>
            </div>
            <button
              onclick={runDiagnostics}
              class="px-6 py-3 bg-[#00d4ff] hover:bg-[#00b8e6] text-white rounded-xl font-medium transition-colors"
            >
              Повторить
            </button>
          {:else if diagnosticResult}
            <div class="w-24 h-24 bg-[#00ff88]/20 rounded-full flex items-center justify-center">
              <svg class="w-12 h-12 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div>
              <h2 class="text-2xl font-bold text-white mb-2">Диагностика завершена</h2>
              <p class="text-[#a0a0a0]">Определён профиль вашего провайдера</p>
            </div>
            
            <div class="bg-[#2a2f4a]/50 rounded-xl p-4 w-full text-left space-y-3">
              <div class="flex justify-between">
                <span class="text-[#a0a0a0]">Тип DPI:</span>
                <span class="text-white font-medium">{diagnosticResult.profile.kind}</span>
              </div>
              {#if diagnosticResult.profile.details}
                <div class="flex justify-between">
                  <span class="text-[#a0a0a0]">Детали:</span>
                  <span class="text-white">{diagnosticResult.profile.details}</span>
                </div>
              {/if}
              <div class="flex justify-between">
                <span class="text-[#a0a0a0]">Заблокировано:</span>
                <span class="text-[#ff3333]">{diagnosticResult.blocked_services.length} сервисов</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[#a0a0a0]">Рекомендуемые стратегии:</span>
                <span class="text-[#00d4ff]">{diagnosticResult.profile.candidate_families.join(', ') || 'Все'}</span>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Step 4: Optimization -->
      {#if currentStep === 'optimization'}
        <div class="flex-1 flex flex-col items-center justify-center text-center space-y-6 {isTransitioning ? 'animate-fade-out' : 'animate-fade-in'} {stepDirection === 'backward' ? 'slide-from-left' : 'slide-from-right'}">
          {#if isOptimizing}
            <div class="w-24 h-24 bg-[#ffaa00]/20 rounded-full flex items-center justify-center relative">
              <div class="w-16 h-16 border-4 border-[#ffaa00] border-t-transparent rounded-full animate-spin"></div>
              <span class="absolute text-[#ffaa00] font-bold text-lg">
                {optimizationProgress?.percent ?? 0}%
              </span>
            </div>
            <div>
              <h2 class="text-2xl font-bold text-white mb-2">Оптимизация</h2>
              <p class="text-[#a0a0a0]">
                {optimizationProgress ? getStageText(optimizationProgress.stage) : 'Запуск...'}
              </p>
            </div>
            
            <div class="w-full max-w-sm space-y-2">
              <div class="h-2 bg-[#2a2f4a] rounded-full overflow-hidden">
                <div 
                  class="h-full bg-[#ffaa00] rounded-full transition-all duration-300"
                  style="width: {optimizationProgress?.percent ?? 0}%"
                ></div>
              </div>
              {#if optimizationProgress}
                <p class="text-sm text-[#a0a0a0]">{optimizationProgress.message}</p>
                {#if optimizationProgress.tested_count > 0}
                  <p class="text-xs text-[#a0a0a0]">
                    Протестировано: {optimizationProgress.tested_count} / {optimizationProgress.total_count}
                  </p>
                {/if}
              {/if}
            </div>
          {:else if optimizationError}
            <div class="w-24 h-24 bg-[#ff3333]/20 rounded-full flex items-center justify-center">
              <svg class="w-12 h-12 text-[#ff3333]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <div>
              <h2 class="text-2xl font-bold text-white mb-2">Ошибка оптимизации</h2>
              <p class="text-[#a0a0a0] mb-4">{optimizationError}</p>
            </div>
            <div class="flex gap-3">
              <button
                onclick={runOptimization}
                class="px-6 py-3 bg-[#00d4ff] hover:bg-[#00b8e6] text-white rounded-xl font-medium transition-colors"
              >
                Повторить
              </button>
              <button
                onclick={() => currentStep = 'complete'}
                class="px-6 py-3 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-xl font-medium transition-colors"
              >
                Пропустить
              </button>
            </div>
          {:else if optimizationResult}
            <div class="w-24 h-24 bg-[#00ff88]/20 rounded-full flex items-center justify-center">
              <svg class="w-12 h-12 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div>
              <h2 class="text-2xl font-bold text-white mb-2">Оптимизация завершена!</h2>
              <p class="text-[#a0a0a0]">Найдена лучшая стратегия для вашего провайдера</p>
            </div>
            
            <div class="bg-[#2a2f4a]/50 rounded-xl p-4 w-full text-left space-y-3">
              <div class="flex justify-between">
                <span class="text-[#a0a0a0]">Стратегия:</span>
                <span class="text-white font-medium">{optimizationResult.strategy_name}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[#a0a0a0]">Оценка:</span>
                <span class="text-[#00ff88] font-medium">{optimizationResult.score}%</span>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Step 5: Complete -->
      {#if currentStep === 'complete'}
        <div class="flex-1 flex flex-col items-center justify-center text-center space-y-6 {isTransitioning ? 'animate-fade-out' : 'animate-fade-in'} {stepDirection === 'backward' ? 'slide-from-left' : 'slide-from-right'}">
          <div class="w-24 h-24 bg-[#00ff88]/20 rounded-full flex items-center justify-center">
            <svg class="w-12 h-12 text-[#00ff88]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          </div>
          
          <div>
            <h2 class="text-2xl font-bold text-white mb-2">Всё готово!</h2>
            <p class="text-[#a0a0a0] leading-relaxed">
              Isolate настроен и готов к работе.
              {#if optimizationResult}
                Стратегия <span class="text-[#00d4ff]">{optimizationResult.strategy_name}</span> уже применена.
              {:else}
                Нажмите «Начать», чтобы перейти к главному экрану.
              {/if}
            </p>
          </div>
          
          <div class="bg-[#2a2f4a]/50 rounded-xl p-4 w-full text-left space-y-3">
            <div class="flex justify-between text-sm">
              <span class="text-[#a0a0a0]">Выбранные сервисы:</span>
              <span class="text-white">
                {Array.from(selectedServices).map(id => 
                  availableServices.find(s => s.id === id)?.name || id
                ).join(', ')}
              </span>
            </div>
            {#if optimizationResult}
              <div class="flex justify-between text-sm">
                <span class="text-[#a0a0a0]">Активная стратегия:</span>
                <span class="text-[#00ff88]">{optimizationResult.strategy_name}</span>
              </div>
            {/if}
            {#if diagnosticResult}
              <div class="flex justify-between text-sm">
                <span class="text-[#a0a0a0]">Профиль DPI:</span>
                <span class="text-white">{diagnosticResult.profile.kind}</span>
              </div>
            {/if}
          </div>
          
          <div class="flex items-center gap-2 text-sm text-[#a0a0a0]">
            <svg class="w-4 h-4 text-[#00d4ff]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>Вы можете изменить настройки в любое время</span>
          </div>
        </div>
      {/if}

      <!-- Navigation Buttons -->
      <div class="flex gap-3 pt-6 mt-auto border-t border-[#2a2f4a]">
        {#if currentStep === 'welcome'}
          <button
            onclick={skipOnboarding}
            class="flex-1 py-3 px-6 text-[#a0a0a0] hover:text-white rounded-xl font-medium transition-colors"
          >
            Пропустить
          </button>
        {:else if currentStep !== 'diagnostics' && currentStep !== 'optimization'}
          <button
            onclick={prevStep}
            class="flex-1 py-3 px-6 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-xl font-medium transition-colors"
          >
            Назад
          </button>
        {:else}
          <div class="flex-1"></div>
        {/if}
        
        {#if currentStep === 'diagnostics'}
          {#if isDiagnosing}
            <button
              disabled
              class="flex-1 py-3 px-6 bg-[#2a2f4a] text-[#a0a0a0] rounded-xl font-medium cursor-not-allowed"
            >
              Диагностика...
            </button>
          {:else}
            <button
              onclick={nextStep}
              class="flex-1 py-3 px-6 bg-[#00d4ff] hover:bg-[#00b8e6] text-white rounded-xl font-medium transition-colors"
            >
              {diagnosticResult ? 'Далее' : 'Пропустить'}
            </button>
          {/if}
        {:else if currentStep === 'optimization'}
          {#if isOptimizing}
            <button
              disabled
              class="flex-1 py-3 px-6 bg-[#2a2f4a] text-[#a0a0a0] rounded-xl font-medium cursor-not-allowed"
            >
              Оптимизация...
            </button>
          {:else}
            <button
              onclick={nextStep}
              class="flex-1 py-3 px-6 bg-[#00d4ff] hover:bg-[#00b8e6] text-white rounded-xl font-medium transition-colors"
            >
              {optimizationResult ? 'Далее' : 'Пропустить'}
            </button>
          {/if}
        {:else if currentStep === 'services'}
          <button
            onclick={nextStep}
            disabled={selectedServices.size === 0}
            class="flex-1 py-3 px-6 bg-[#00d4ff] hover:bg-[#00b8e6] disabled:bg-[#2a2f4a] disabled:text-[#a0a0a0] disabled:cursor-not-allowed text-white rounded-xl font-medium transition-colors"
          >
            Далее
          </button>
        {:else if currentStep === 'complete'}
          <button
            onclick={completeOnboarding}
            class="flex-1 py-3 px-6 bg-[#00ff88] hover:bg-[#00e67a] text-[#0a0e27] rounded-xl font-bold transition-colors"
          >
            Начать использование
          </button>
        {:else}
          <button
            onclick={nextStep}
            class="flex-1 py-3 px-6 bg-[#00d4ff] hover:bg-[#00b8e6] text-white rounded-xl font-medium transition-colors"
          >
            Далее
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  @keyframes fade-out {
    from {
      opacity: 1;
      transform: translateY(0);
    }
    to {
      opacity: 0;
      transform: translateY(-10px);
    }
  }
  
  @keyframes slide-in-right {
    from {
      opacity: 0;
      transform: translateX(30px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }
  
  @keyframes slide-in-left {
    from {
      opacity: 0;
      transform: translateX(-30px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }
  
  @keyframes progress {
    0% {
      width: 0%;
    }
    50% {
      width: 70%;
    }
    100% {
      width: 100%;
    }
  }
  
  @keyframes pulse-glow {
    0%, 100% {
      box-shadow: 0 0 20px rgba(0, 212, 255, 0.3);
    }
    50% {
      box-shadow: 0 0 40px rgba(0, 212, 255, 0.6);
    }
  }
  
  @keyframes spin-slow {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
  
  .animate-fade-in {
    animation: fade-in 0.4s ease-out forwards;
  }
  
  .animate-fade-out {
    animation: fade-out 0.15s ease-in forwards;
  }
  
  .slide-from-right {
    animation: slide-in-right 0.4s ease-out forwards;
  }
  
  .slide-from-left {
    animation: slide-in-left 0.4s ease-out forwards;
  }
  
  .animate-progress {
    animation: progress 2s ease-in-out infinite;
  }
  
  .animate-pulse-glow {
    animation: pulse-glow 2s ease-in-out infinite;
  }
  
  .animate-spin-slow {
    animation: spin-slow 3s linear infinite;
  }
</style>
