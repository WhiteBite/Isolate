<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { waitForBackend } from '$lib/utils/backend';
  import {
    StepIndicator,
    ServiceSelector,
    MethodSelector,
    SetupStep,
    WelcomeStep,
    OnboardingNavigation,
    ProviderStep,
    type Step,
    type ConnectionMode,
    type ServiceItem,
    type SetupTask
  } from '$lib/components/onboarding';
  import { mockOnboardingServices, mockConflicts } from '$lib/mocks';
  import type { ConflictInfo } from '$lib/api/types';

  // State
  let currentStep = $state<Step>(1);
  
  // Services state
  let availableServices = $state<ServiceItem[]>([...mockOnboardingServices]);
  let selectedServices = $state<Set<string>>(new Set(['youtube', 'discord']));
  let loadingServices = $state(true);
  
  // Step 3 state - Provider selection
  let selectedProvider = $state<string | null>(null);
  
  // Step 4 state - Connection method
  let connectionMode = $state<ConnectionMode>('auto');
  
  // Conflicts state
  let detectedConflicts = $state<ConflictInfo[]>([]);
  let hasBlockingConflicts = $derived(
    detectedConflicts.some(c => c.severity === 'critical' || c.severity === 'high')
  );
  
  // Step 5 state - Setup progress
  let setupTasks = $state<SetupTask[]>([
    { id: 'conflicts', label: 'Checking for conflicts', status: 'pending' },
    { id: 'binaries', label: 'Checking components', status: 'pending' },
    { id: 'download', label: 'Downloading components', status: 'pending' },
    { id: 'configs', label: 'Loading configurations', status: 'pending' },
    { id: 'connection', label: 'Testing connection', status: 'pending' },
  ]);
  let setupProgress = $state(0);
  let isSettingUp = $state(false);
  let setupComplete = $state(false);
  let downloadProgress = $state<{ name: string; percent: number } | null>(null);

  // Derived states
  let canProceed = $derived(
    currentStep === 1 ? true :
    currentStep === 2 ? selectedServices.size > 0 :
    currentStep === 3 ? true : // Provider is optional
    currentStep === 4 ? connectionMode !== null :
    true
  );

  // Step titles for header
  const stepTitles = ['Welcome', 'Services', 'Provider', 'Method', 'Setup'];

  import { onMount } from 'svelte';
  onMount(() => {
    loadServices();
  });

  async function loadServices() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      const ready = await waitForBackend(30, 300);
      if (!ready) {
        console.warn('[Onboarding] Backend not ready after retries');
        loadingServices = false;
        return;
      }
      
      const services = await invoke<{ id: string; name: string; critical?: boolean }[]>('get_services');
      if (services && services.length > 0) {
        availableServices = services.map(s => ({
          id: s.id,
          name: s.name,
          icon: getServiceIcon(s.id),
          description: getServiceDescription(s.id)
        }));
      }
    } catch (e) {
      console.error('Failed to load services:', e);
    } finally {
      loadingServices = false;
    }
  }

  function getServiceIcon(serviceId: string): string {
    const icons: Record<string, string> = {
      youtube: '📺', discord: '💬', twitch: '🎮', telegram: '✈️',
      spotify: '🎵', google: '🔍', twitter: '🐦', instagram: '📷',
    };
    return icons[serviceId] || '🌐';
  }

  function getServiceDescription(serviceId: string): string {
    const descriptions: Record<string, string> = {
      youtube: 'Video & streams', discord: 'Voice & chat', twitch: 'Streaming',
      telegram: 'Messenger', spotify: 'Music', google: 'Search',
      twitter: 'Social', instagram: 'Photos & stories',
    };
    return descriptions[serviceId] || 'Service';
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

  function selectAllServices() {
    selectedServices = new Set(availableServices.map(s => s.id));
  }

  function deselectAllServices() {
    selectedServices = new Set();
  }

  function nextStep() {
    if (currentStep < 5) {
      currentStep = (currentStep + 1) as Step;
      if (currentStep === 5) {
        runSetup();
      }
    }
  }

  function prevStep() {
    if (currentStep > 1) {
      currentStep = (currentStep - 1) as Step;
    }
  }

  function skipOnboarding() {
    selectedServices = new Set(['youtube', 'discord']);
    connectionMode = 'auto';
    completeOnboarding();
  }

  function handleStepClick(step: Step) {
    if (step < currentStep) {
      currentStep = step;
    }
  }

  async function runSetup() {
    isSettingUp = true;
    setupProgress = 0;
    setupComplete = false;
    downloadProgress = null;
    detectedConflicts = [];
    
    setupTasks = setupTasks.map(t => ({ ...t, status: 'pending', error: undefined }));
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');
      
      // Task 0: Check for conflicts
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx === 0 ? 'running' : 'pending'
      }));
      setupProgress = 2;
      
      try {
        const conflicts = await invoke<ConflictInfo[]>('check_conflicts');
        detectedConflicts = conflicts;
        
        const hasCritical = conflicts.some(c => c.severity === 'critical' || c.severity === 'high');
        setupTasks = setupTasks.map((t, idx) => ({
          ...t,
          status: idx === 0 ? (hasCritical ? 'error' : 'done') : 'pending',
          error: idx === 0 && hasCritical ? `${conflicts.length} conflict(s) detected` : undefined
        }));
      } catch (e) {
        // If check fails, continue anyway
        setupTasks = setupTasks.map((t, idx) => ({
          ...t,
          status: idx === 0 ? 'done' : 'pending'
        }));
      }
      setupProgress = 10;
      
      // Task 1: Check binaries
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx === 1 ? 'running' : idx < 1 ? (t.status === 'error' ? 'error' : 'done') : 'pending'
      }));
      setupProgress = 12;
      
      let checkResult: { missing: string[]; present: string[]; all_present: boolean };
      try {
        checkResult = await invoke('check_binaries');
      } catch (e) {
        checkResult = { missing: ['winws.exe', 'sing-box.exe'], present: [], all_present: false };
      }
      
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx === 1 ? 'done' : idx < 1 ? t.status : 'pending'
      }));
      setupProgress = 18;
      
      // Task 2: Download missing binaries if needed
      if (!checkResult.all_present && checkResult.missing.length > 0) {
        setupTasks = setupTasks.map((t, idx) => ({
          ...t,
          status: idx === 2 ? 'running' : idx < 2 ? t.status : 'pending'
        }));
        
        const unlisten = await listen<{ binary_name: string; percentage: number; phase: string }>('binaries:progress', (event) => {
          downloadProgress = {
            name: event.payload.binary_name,
            percent: event.payload.percentage
          };
          setupProgress = 18 + (event.payload.percentage * 0.52);
        });
        
        try {
          await invoke('download_binaries');
          setupTasks = setupTasks.map((t, idx) => ({
            ...t,
            status: idx <= 2 ? (idx < 2 ? t.status : 'done') : 'pending'
          }));
        } catch (e) {
          console.error('Download failed:', e);
          setupTasks = setupTasks.map((t, idx) => ({
            ...t,
            status: idx === 2 ? 'error' : idx < 2 ? t.status : 'pending',
            error: idx === 2 ? String(e) : t.error
          }));
        } finally {
          unlisten();
          downloadProgress = null;
        }
      } else {
        setupTasks = setupTasks.map((t, idx) => ({
          ...t,
          status: idx <= 2 ? (idx < 2 ? t.status : 'done') : 'pending'
        }));
      }
      setupProgress = 70;
      
      // Task 3: Load configs
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx === 3 ? 'running' : idx < 3 ? t.status : 'pending'
      }));
      
      await new Promise(r => setTimeout(r, 500));
      
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx <= 3 ? (idx < 3 ? t.status : 'done') : 'pending'
      }));
      setupProgress = 85;
      
      // Task 4: Test connection
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx === 4 ? 'running' : idx < 4 ? t.status : 'pending'
      }));
      
      await new Promise(r => setTimeout(r, 800));
      
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx === 4 ? 'done' : t.status
      }));
      setupProgress = 100;
      
    } catch (e) {
      console.error('Setup failed:', e);
    }
    
    isSettingUp = false;
    setupComplete = true;
  }

  async function completeOnboarding() {
    if (!browser) return;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    // Сохраняем в localStorage для браузера
    localStorage.setItem('onboarding_completed', 'true');
    
    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        
        await invoke('save_settings', {
          settings: {
            auto_start: false,
            auto_apply: connectionMode === 'auto',
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
        }).catch(() => {});
        
        await invoke('set_setting', { key: 'onboarding_complete', value: true }).catch(() => {});
        
      } catch (e) {
        console.error('Failed to save settings:', e);
      }
    }
    
    goto('/');
  }
</script>

<div class="min-h-screen bg-zinc-950 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-indigo-900/20 via-zinc-950 to-zinc-950 flex flex-col items-center justify-center p-6">
  <!-- Background decorations -->
  <div class="fixed inset-0 overflow-hidden pointer-events-none">
    <div class="absolute -top-40 -right-40 w-80 h-80 bg-indigo-500/10 rounded-full blur-3xl"></div>
    <div class="absolute -bottom-40 -left-40 w-80 h-80 bg-purple-500/10 rounded-full blur-3xl"></div>
  </div>

  <div class="relative w-full max-w-xl z-10">
    <!-- Logo & Title -->
    <div class="text-center mb-8">
      <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 shadow-lg shadow-indigo-500/25 mb-4">
        <span class="text-3xl">🛡️</span>
      </div>
      <h1 class="text-2xl font-bold text-white">Isolate</h1>
    </div>

    <!-- Progress Indicator -->
    <StepIndicator 
      {currentStep} 
      {stepTitles} 
      onStepClick={handleStepClick} 
    />

    <!-- Main Card -->
    <div class="backdrop-blur-xl bg-zinc-900/60 rounded-3xl border border-white/10 shadow-2xl shadow-black/50 overflow-hidden">
      <div class="p-8 min-h-[480px] flex flex-col">

        <!-- Step 1: Welcome -->
        {#if currentStep === 1}
          <WelcomeStep />
        {/if}

        <!-- Step 2: Select Services -->
        {#if currentStep === 2}
          <ServiceSelector
            services={availableServices}
            {selectedServices}
            loading={loadingServices}
            onToggle={toggleService}
            onSelectAll={selectAllServices}
            onDeselectAll={deselectAllServices}
          />
        {/if}

        <!-- Step 3: Select Provider -->
        {#if currentStep === 3}
          <ProviderStep
            onSelect={(id) => selectedProvider = id}
          />
        {/if}

        <!-- Step 4: Choose Method -->
        {#if currentStep === 4}
          <MethodSelector
            {connectionMode}
            onSelect={(mode) => connectionMode = mode}
          />
        {/if}

        <!-- Step 5: Setup -->
        {#if currentStep === 5}
          <SetupStep
            {setupTasks}
            {setupProgress}
            {setupComplete}
            {downloadProgress}
            selectedServicesCount={selectedServices.size}
            {connectionMode}
            conflicts={detectedConflicts}
            onComplete={completeOnboarding}
          />
        {/if}

        <!-- Navigation Buttons -->
        <OnboardingNavigation
          {currentStep}
          {canProceed}
          {setupComplete}
          {isSettingUp}
          onNext={nextStep}
          onPrev={prevStep}
          onSkip={skipOnboarding}
          onComplete={completeOnboarding}
        />
      </div>
    </div>

    <!-- Footer -->
    <div class="mt-6 text-center">
      <p class="text-xs text-zinc-400">
        Isolate v1.0 • Your data stays on your device
      </p>
    </div>
  </div>
</div>
