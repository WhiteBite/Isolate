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
      const next = steps[idx + 1];
      currentStep = next;
      
      // Auto-run actions on step enter
      if (next === 'diagnostics' && !diagnosticResult && !isDiagnosing) {
        runDiagnostics();
      } else if (next === 'optimization' && !optimizationResult && !isOptimizing) {
        runOptimization();
      }
    } else {
      completeOnboarding();
    }
  }

  function prevStep() {
    const idx = currentStepIndex;
    if (idx > 0) {
      currentStep = steps[idx - 1];
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
    <!-- Progress Indicator -->
    <div class="flex justify-center items-center gap-2 mb-8">
      {#each steps as step, i}
        <div 
          class="h-2 rounded-full transition-all duration-500 {i <= currentStepIndex ? 'bg-[#00d4ff]' : 'bg-[#2a2f4a]'} {i === currentStepIndex ? 'w-8' : 'w-4'}"
        ></div>
      {/each}
    </div>

    <!-- Step Content Container -->
    <div class="bg-[#1a1f3a] rounded-2xl p-8 border border-[#2a2f4a] min-h-[500px] flex flex-col">
      
      <!-- Step 1: Welcome -->
      {#if currentStep === 'welcome'}
        <div class="flex-1 flex flex-col items-center justify-center text-center space-y-6 animate-fade-in">
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
