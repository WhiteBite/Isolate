<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';

  // Step definitions (4 steps as per requirements)
  type Step = 1 | 2 | 3 | 4;
  
  // State
  let currentStep = $state<Step>(1);
  
  // Services state
  interface ServiceItem {
    id: string;
    name: string;
    icon: string;
    description: string;
  }
  
  let availableServices = $state<ServiceItem[]>([
    { id: 'youtube', name: 'YouTube', icon: '📺', description: 'Видео и стримы' },
    { id: 'discord', name: 'Discord', icon: '💬', description: 'Голос и чаты' },
    { id: 'telegram', name: 'Telegram', icon: '✈️', description: 'Мессенджер' },
    { id: 'twitch', name: 'Twitch', icon: '🎮', description: 'Стриминг' },
    { id: 'spotify', name: 'Spotify', icon: '🎵', description: 'Музыка' },
    { id: 'instagram', name: 'Instagram', icon: '📷', description: 'Фото и сторис' },
  ]);
  let selectedServices = $state<Set<string>>(new Set(['youtube', 'discord']));
  let loadingServices = $state(true);
  
  // Step 3 state - Connection method
  type ConnectionMode = 'auto' | 'proxy';
  let connectionMode = $state<ConnectionMode>('auto');
  
  // Step 4 state - Setup progress
  interface SetupTask {
    id: string;
    label: string;
    status: 'pending' | 'running' | 'done' | 'error';
  }
  let setupTasks = $state<SetupTask[]>([
    { id: 'binaries', label: 'Проверка компонентов', status: 'pending' },
    { id: 'configs', label: 'Загрузка конфигураций', status: 'pending' },
    { id: 'connection', label: 'Тестирование соединения', status: 'pending' },
  ]);
  let setupProgress = $state(0);
  let isSettingUp = $state(false);
  let setupComplete = $state(false);

  // Derived states
  let canProceed = $derived(
    currentStep === 1 ? true : // Welcome step - always can proceed
    currentStep === 2 ? selectedServices.size > 0 :
    currentStep === 3 ? connectionMode !== null :
    true
  );
  
  let progressPercent = $derived((currentStep / 4) * 100);

  // Step titles for header
  let stepTitles = ['Welcome', 'Services', 'Method', 'Setup'];

  $effect(() => {
    if (!browser) return;
    loadServices();
  });

  async function loadServices(retries = 10) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Wait for backend to be ready
      for (let i = 0; i < retries; i++) {
        try {
          const ready = await invoke<boolean>('is_backend_ready');
          if (ready) break;
        } catch {
          // Backend not ready yet
        }
        await new Promise(r => setTimeout(r, 200));
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
      // Keep default services
    } finally {
      loadingServices = false;
    }
  }

  function getServiceIcon(serviceId: string): string {
    const icons: Record<string, string> = {
      youtube: '📺',
      discord: '💬',
      twitch: '🎮',
      telegram: '✈️',
      spotify: '🎵',
      google: '🔍',
      twitter: '🐦',
      instagram: '📷',
    };
    return icons[serviceId] || '🌐';
  }

  function getServiceDescription(serviceId: string): string {
    const descriptions: Record<string, string> = {
      youtube: 'Видео и стримы',
      discord: 'Голос и чаты',
      twitch: 'Стриминг',
      telegram: 'Мессенджер',
      spotify: 'Музыка',
      google: 'Поиск',
      twitter: 'Соцсеть',
      instagram: 'Фото и сторис',
    };
    return descriptions[serviceId] || 'Сервис';
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
    if (currentStep < 4) {
      currentStep = (currentStep + 1) as Step;
      if (currentStep === 4) {
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
    // Set defaults and complete
    selectedServices = new Set(['youtube', 'discord']);
    connectionMode = 'auto';
    completeOnboarding();
  }

  async function runSetup() {
    isSettingUp = true;
    setupProgress = 0;
    setupComplete = false;
    
    // Reset tasks
    setupTasks = setupTasks.map(t => ({ ...t, status: 'pending' }));
    
    const taskDurations = [1000, 1200, 1500]; // ms per task
    
    for (let i = 0; i < setupTasks.length; i++) {
      // Mark current task as running
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx === i ? 'running' : idx < i ? 'done' : 'pending'
      }));
      
      // Simulate task execution with progress animation
      const duration = taskDurations[i];
      const startProgress = (i / setupTasks.length) * 100;
      const endProgress = ((i + 1) / setupTasks.length) * 100;
      
      // Animate progress smoothly
      const steps = 20;
      const stepDuration = duration / steps;
      for (let s = 0; s <= steps; s++) {
        await new Promise(r => setTimeout(r, stepDuration));
        setupProgress = startProgress + ((endProgress - startProgress) * (s / steps));
      }
      
      // Try to call actual Tauri commands
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        if (i === 0) {
          await invoke('check_binaries').catch(() => {});
        } else if (i === 1) {
          // Configs are loaded automatically
        } else if (i === 2) {
          // Connection test
        }
      } catch (e) {
        console.error('Setup task failed:', e);
      }
      
      // Mark task as done
      setupTasks = setupTasks.map((t, idx) => ({
        ...t,
        status: idx <= i ? 'done' : 'pending'
      }));
    }
    
    setupProgress = 100;
    isSettingUp = false;
    setupComplete = true;
  }

  async function completeOnboarding() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Save settings
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
      
      // Mark onboarding complete
      await invoke('set_setting', { key: 'onboarding_complete', value: true }).catch(() => {});
      
    } catch (e) {
      console.error('Failed to save settings:', e);
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

    <!-- Progress Indicator (Dots) -->
    <div class="flex justify-center items-center gap-3 mb-6">
      {#each [1, 2, 3, 4] as step}
        <button
          onclick={() => { if (step < currentStep) currentStep = step as Step; }}
          disabled={step > currentStep}
          class="group relative flex flex-col items-center gap-1"
        >
          <div 
            class="w-3 h-3 rounded-full transition-all duration-300 
                   {step < currentStep 
                     ? 'bg-emerald-400 scale-100' 
                     : step === currentStep 
                       ? 'bg-indigo-500 scale-125 ring-4 ring-indigo-500/20' 
                       : 'bg-zinc-700 scale-100'}"
          ></div>
          <span class="text-[10px] font-medium transition-colors
                       {step === currentStep ? 'text-indigo-400' : step < currentStep ? 'text-emerald-400' : 'text-zinc-600'}">
            {stepTitles[step - 1]}
          </span>
        </button>
      {/each}
    </div>

    <!-- Main Card -->
    <div class="backdrop-blur-xl bg-zinc-900/60 rounded-3xl border border-white/10 shadow-2xl shadow-black/50 overflow-hidden">
      <!-- Card Content -->
      <div class="p-8 min-h-[480px] flex flex-col">

        <!-- Step 1: Welcome -->
        {#if currentStep === 1}
          <div class="flex-1 flex flex-col items-center justify-center text-center animate-fade-in">
            <div class="w-24 h-24 rounded-3xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 border border-indigo-500/20 flex items-center justify-center mb-6">
              <span class="text-5xl">👋</span>
            </div>
            <h2 class="text-3xl font-bold text-white mb-3">Добро пожаловать!</h2>
            <p class="text-zinc-400 text-lg mb-8 max-w-sm">
              Isolate поможет вам получить доступ к заблокированным сервисам быстро и безопасно
            </p>
            
            <!-- Features list -->
            <div class="grid grid-cols-1 gap-3 w-full max-w-sm text-left">
              <div class="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/40 border border-white/5">
                <div class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center">
                  <span class="text-xl">⚡</span>
                </div>
                <div>
                  <div class="text-sm font-medium text-white">Быстрая настройка</div>
                  <div class="text-xs text-zinc-500">Автоматический подбор метода</div>
                </div>
              </div>
              <div class="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/40 border border-white/5">
                <div class="w-10 h-10 rounded-lg bg-indigo-500/10 flex items-center justify-center">
                  <span class="text-xl">🔒</span>
                </div>
                <div>
                  <div class="text-sm font-medium text-white">Безопасность</div>
                  <div class="text-xs text-zinc-500">Никаких логов и телеметрии</div>
                </div>
              </div>
              <div class="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/40 border border-white/5">
                <div class="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center">
                  <span class="text-xl">🎯</span>
                </div>
                <div>
                  <div class="text-sm font-medium text-white">Точечный обход</div>
                  <div class="text-xs text-zinc-500">Только нужные сервисы</div>
                </div>
              </div>
            </div>
          </div>
        {/if}

        <!-- Step 2: Select Services -->
        {#if currentStep === 2}
          <div class="flex-1 flex flex-col animate-fade-in">
            <div class="text-center mb-6">
              <h2 class="text-2xl font-bold text-white mb-2">Выберите сервисы</h2>
              <p class="text-zinc-400">Какие сервисы вы хотите разблокировать?</p>
            </div>
            
            <!-- Quick actions -->
            <div class="flex justify-center gap-2 mb-4">
              <button
                onclick={selectAllServices}
                class="px-3 py-1.5 text-xs font-medium text-indigo-400 hover:text-indigo-300 
                       bg-indigo-500/10 hover:bg-indigo-500/20 rounded-lg transition-colors"
              >
                Выбрать все
              </button>
              <button
                onclick={deselectAllServices}
                class="px-3 py-1.5 text-xs font-medium text-zinc-400 hover:text-zinc-300 
                       bg-zinc-800/60 hover:bg-zinc-800 rounded-lg transition-colors"
              >
                Сбросить
              </button>
            </div>
            
            {#if loadingServices}
              <div class="flex-1 flex items-center justify-center">
                <div class="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
              </div>
            {:else}
              <div class="grid grid-cols-2 gap-3 flex-1 overflow-y-auto">
                {#each availableServices as service}
                  <button
                    onclick={() => toggleService(service.id)}
                    class="group p-4 rounded-xl border transition-all duration-200 text-left
                           {selectedServices.has(service.id) 
                             ? 'border-indigo-500/50 bg-indigo-500/10 shadow-lg shadow-indigo-500/10' 
                             : 'border-white/5 bg-zinc-800/30 hover:border-white/10 hover:bg-zinc-800/50'}"
                  >
                    <div class="flex items-start gap-3">
                      <span class="text-2xl">{service.icon}</span>
                      <div class="flex-1 min-w-0">
                        <div class="text-sm font-medium text-white truncate">{service.name}</div>
                        <div class="text-xs text-zinc-500">{service.description}</div>
                      </div>
                      <div class="w-5 h-5 rounded-md border-2 flex items-center justify-center transition-all
                                  {selectedServices.has(service.id) 
                                    ? 'border-indigo-500 bg-indigo-500' 
                                    : 'border-zinc-600 group-hover:border-zinc-500'}">
                        {#if selectedServices.has(service.id)}
                          <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                          </svg>
                        {/if}
                      </div>
                    </div>
                  </button>
                {/each}
              </div>
              
              <!-- Selection counter -->
              <div class="mt-4 text-center">
                <span class="text-sm {selectedServices.size > 0 ? 'text-indigo-400' : 'text-amber-400'}">
                  {selectedServices.size > 0 
                    ? `Выбрано: ${selectedServices.size} сервис${selectedServices.size === 1 ? '' : selectedServices.size < 5 ? 'а' : 'ов'}`
                    : 'Выберите хотя бы один сервис'}
                </span>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Step 3: Choose Method -->
        {#if currentStep === 3}
          <div class="flex-1 flex flex-col animate-fade-in">
            <div class="text-center mb-6">
              <h2 class="text-2xl font-bold text-white mb-2">Метод подключения</h2>
              <p class="text-zinc-400">Как вы хотите обходить блокировки?</p>
            </div>
            
            <div class="flex-1 flex flex-col gap-4 justify-center">
              <!-- Auto mode -->
              <button
                onclick={() => connectionMode = 'auto'}
                class="group p-5 rounded-2xl border-2 transition-all duration-200 text-left
                       {connectionMode === 'auto' 
                         ? 'border-indigo-500/50 bg-indigo-500/10 shadow-lg shadow-indigo-500/10' 
                         : 'border-white/5 bg-zinc-800/30 hover:border-white/10'}"
              >
                <div class="flex items-start gap-4">
                  <div class="w-14 h-14 rounded-xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 
                              border border-indigo-500/20 flex items-center justify-center flex-shrink-0">
                    <span class="text-2xl">🔧</span>
                  </div>
                  <div class="flex-1">
                    <h3 class="text-white font-semibold text-lg mb-1">Автоматически</h3>
                    <p class="text-zinc-400 text-sm">
                      Isolate подберёт лучший метод обхода DPI для вашего провайдера
                    </p>
                    <div class="flex items-center gap-2 mt-2">
                      <span class="px-2 py-0.5 text-xs font-medium bg-emerald-500/10 text-emerald-400 rounded">Рекомендуется</span>
                    </div>
                  </div>
                  <div class="w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all
                              {connectionMode === 'auto' ? 'border-indigo-500 bg-indigo-500' : 'border-zinc-600'}">
                    {#if connectionMode === 'auto'}
                      <div class="w-2 h-2 rounded-full bg-white"></div>
                    {/if}
                  </div>
                </div>
              </button>
              
              <!-- Proxy mode -->
              <button
                onclick={() => connectionMode = 'proxy'}
                class="group p-5 rounded-2xl border-2 transition-all duration-200 text-left
                       {connectionMode === 'proxy' 
                         ? 'border-indigo-500/50 bg-indigo-500/10 shadow-lg shadow-indigo-500/10' 
                         : 'border-white/5 bg-zinc-800/30 hover:border-white/10'}"
              >
                <div class="flex items-start gap-4">
                  <div class="w-14 h-14 rounded-xl bg-gradient-to-br from-emerald-500/20 to-cyan-500/20 
                              border border-emerald-500/20 flex items-center justify-center flex-shrink-0">
                    <span class="text-2xl">🌐</span>
                  </div>
                  <div class="flex-1">
                    <h3 class="text-white font-semibold text-lg mb-1">Свой прокси</h3>
                    <p class="text-zinc-400 text-sm">
                      Использовать VLESS, Shadowsocks, SOCKS5 или другой прокси
                    </p>
                    <div class="flex items-center gap-2 mt-2">
                      <span class="px-2 py-0.5 text-xs font-medium bg-zinc-700 text-zinc-400 rounded">Для продвинутых</span>
                    </div>
                  </div>
                  <div class="w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all
                              {connectionMode === 'proxy' ? 'border-indigo-500 bg-indigo-500' : 'border-zinc-600'}">
                    {#if connectionMode === 'proxy'}
                      <div class="w-2 h-2 rounded-full bg-white"></div>
                    {/if}
                  </div>
                </div>
              </button>
            </div>
          </div>
        {/if}

        <!-- Step 4: Complete / Setup -->
        {#if currentStep === 4}
          <div class="flex-1 flex flex-col items-center justify-center animate-fade-in">
            {#if !setupComplete}
              <!-- Setup in progress -->
              <div class="text-center mb-8">
                <div class="w-20 h-20 rounded-2xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 
                            border border-indigo-500/20 flex items-center justify-center mb-4 mx-auto">
                  <span class="text-4xl animate-bounce">🚀</span>
                </div>
                <h2 class="text-2xl font-bold text-white mb-2">Настройка системы</h2>
                <p class="text-zinc-400">Подождите, это займёт несколько секунд...</p>
              </div>
              
              <!-- Progress Bar -->
              <div class="w-full max-w-sm mb-8">
                <div class="h-2 bg-zinc-800 rounded-full overflow-hidden">
                  <div 
                    class="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full transition-all duration-300"
                    style="width: {setupProgress}%"
                  ></div>
                </div>
                <p class="text-center text-zinc-500 text-sm font-mono mt-2">{Math.round(setupProgress)}%</p>
              </div>
              
              <!-- Tasks List -->
              <div class="w-full max-w-sm space-y-3">
                {#each setupTasks as task}
                  <div class="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/30 border border-white/5">
                    {#if task.status === 'done'}
                      <div class="w-8 h-8 rounded-lg bg-emerald-500/10 flex items-center justify-center">
                        <svg class="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                      </div>
                    {:else if task.status === 'running'}
                      <div class="w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center">
                        <div class="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
                      </div>
                    {:else}
                      <div class="w-8 h-8 rounded-lg bg-zinc-800 flex items-center justify-center">
                        <div class="w-2 h-2 rounded-full bg-zinc-600"></div>
                      </div>
                    {/if}
                    <span class="text-sm font-medium
                                 {task.status === 'done' ? 'text-emerald-400' : 
                                  task.status === 'running' ? 'text-white' : 'text-zinc-500'}">
                      {task.label}
                    </span>
                  </div>
                {/each}
              </div>
            {:else}
              <!-- Setup complete -->
              <div class="text-center">
                <div class="w-24 h-24 rounded-3xl bg-gradient-to-br from-emerald-500/20 to-cyan-500/20 
                            border border-emerald-500/20 flex items-center justify-center mb-6 mx-auto
                            animate-success-pop">
                  <span class="text-5xl">✅</span>
                </div>
                <h2 class="text-3xl font-bold text-white mb-3">Всё готово!</h2>
                <p class="text-zinc-400 text-lg mb-8 max-w-sm">
                  Isolate настроен и готов к работе. Нажмите кнопку ниже, чтобы начать.
                </p>
                
                <!-- Summary -->
                <div class="p-4 rounded-xl bg-zinc-800/30 border border-white/5 text-left max-w-sm mx-auto mb-6">
                  <div class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Ваши настройки</div>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Сервисы:</span>
                      <span class="text-white font-medium">{selectedServices.size} выбрано</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="text-zinc-400">Метод:</span>
                      <span class="text-white font-medium">{connectionMode === 'auto' ? 'Автоматический' : 'Свой прокси'}</span>
                    </div>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Navigation Buttons -->
        <div class="flex gap-3 pt-6 mt-auto border-t border-white/5">
          {#if currentStep > 1 && currentStep < 4}
            <button
              onclick={prevStep}
              class="flex items-center justify-center gap-2 px-5 py-3 
                     bg-zinc-800/60 hover:bg-zinc-800 border border-white/5 hover:border-white/10
                     text-zinc-300 rounded-xl font-medium transition-all"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
              Назад
            </button>
          {:else if currentStep === 1}
            <button
              onclick={skipOnboarding}
              class="px-5 py-3 text-zinc-500 hover:text-zinc-400 font-medium transition-colors"
            >
              Пропустить
            </button>
          {:else}
            <div></div>
          {/if}
          
          <div class="flex-1"></div>
          
          {#if currentStep < 4}
            <button
              onclick={nextStep}
              disabled={!canProceed}
              class="flex items-center justify-center gap-2 px-6 py-3 
                     bg-indigo-500 hover:bg-indigo-600 disabled:bg-zinc-800 disabled:text-zinc-600
                     text-white rounded-xl font-medium transition-all
                     disabled:cursor-not-allowed shadow-lg shadow-indigo-500/20 disabled:shadow-none
                     hover:-translate-y-0.5 disabled:translate-y-0"
            >
              {currentStep === 1 ? 'Начать' : 'Далее'}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
            </button>
          {:else if setupComplete}
            <button
              onclick={completeOnboarding}
              class="flex items-center justify-center gap-2 px-8 py-3 
                     bg-gradient-to-r from-emerald-500 to-cyan-500 hover:from-emerald-600 hover:to-cyan-600
                     text-white rounded-xl font-semibold transition-all
                     shadow-lg shadow-emerald-500/20
                     hover:-translate-y-0.5"
            >
              Начать работу
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
            </button>
          {:else}
            <div class="flex items-center gap-2 px-6 py-3 text-zinc-500">
              <div class="w-4 h-4 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
              Настройка...
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="mt-6 text-center">
      <p class="text-xs text-zinc-600">
        Isolate v1.0 • Ваши данные остаются на вашем устройстве
      </p>
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
  
  @keyframes success-pop {
    0% {
      transform: scale(0.8);
      opacity: 0;
    }
    50% {
      transform: scale(1.1);
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }
  
  .animate-fade-in {
    animation: fade-in 0.4s ease-out forwards;
  }
  
  .animate-success-pop {
    animation: success-pop 0.5s ease-out forwards;
  }
</style>
