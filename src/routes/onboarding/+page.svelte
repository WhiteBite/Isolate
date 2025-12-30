<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';

  let currentStep = $state(1);
  const totalSteps = 4;
  
  let selectedServices = $state<string[]>(['discord', 'youtube']);
  let telemetryEnabled = $state(false);

  const availableServices = [
    { id: 'discord', name: 'Discord', description: 'Голосовые и видеозвонки' },
    { id: 'youtube', name: 'YouTube', description: 'Видеохостинг' },
    { id: 'telegram', name: 'Telegram', description: 'Мессенджер' }
  ];

  function toggleService(serviceId: string) {
    if (selectedServices.includes(serviceId)) {
      selectedServices = selectedServices.filter(id => id !== serviceId);
    } else {
      selectedServices = [...selectedServices, serviceId];
    }
  }

  function isServiceSelected(serviceId: string): boolean {
    return selectedServices.includes(serviceId);
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
          default_mode: 'turbo'
        }
      });
      
      // Save selected services
      for (const service of availableServices) {
        await invoke('toggle_service', {
          serviceId: service.id,
          enabled: selectedServices.includes(service.id)
        });
      }
      
      // Save telemetry preference
      await invoke('set_setting', { key: 'telemetry_enabled', value: telemetryEnabled });
      
      // Mark onboarding complete
      await invoke('set_setting', { key: 'onboarding_complete', value: true });
      
      goto('/');
    } catch (e) {
      console.error('Failed to complete onboarding:', e);
      // Navigate anyway to not block the user
      goto('/');
    }
  }
  
  function nextStep() {
    if (currentStep < totalSteps) {
      currentStep++;
    } else {
      completeOnboarding();
    }
  }
  
  function prevStep() {
    if (currentStep > 1) {
      currentStep--;
    }
  }

  function skipOnboarding() {
    completeOnboarding();
  }

  function getServiceButtonClass(serviceId: string): string {
    const isSelected = selectedServices.includes(serviceId);
    const base = 'w-full p-4 rounded-xl border-2 transition-all duration-200 text-left flex items-center gap-4';
    if (isSelected) {
      return `${base} border-primary-500 bg-primary-500/10`;
    }
    return `${base} border-gray-700 bg-gray-700/50 hover:border-gray-600`;
  }

  function getCheckboxClass(isChecked: boolean): string {
    const base = 'w-6 h-6 rounded-md border-2 flex items-center justify-center transition-colors';
    if (isChecked) {
      return `${base} border-primary-500 bg-primary-500`;
    }
    return `${base} border-gray-500`;
  }

  function getTelemetryButtonClass(): string {
    const base = 'w-full p-4 rounded-xl border-2 transition-all duration-200 text-left flex items-center gap-4';
    if (telemetryEnabled) {
      return `${base} border-primary-500 bg-primary-500/10`;
    }
    return `${base} border-gray-700 bg-gray-700/50`;
  }
</script>

<div class="flex flex-col items-center justify-center min-h-screen p-8">
  <div class="w-full max-w-md">
    <!-- Progress Indicator -->
    <div class="flex justify-center gap-2 mb-8">
      {#each Array(totalSteps) as _, i}
        <div 
          class="h-1.5 w-12 rounded-full transition-colors duration-300 {i + 1 <= currentStep ? 'bg-primary-500' : 'bg-gray-700'}"
        ></div>
      {/each}
    </div>

    <!-- Step Content -->
    <div class="bg-gray-800 rounded-2xl p-8 space-y-6 min-h-[400px] flex flex-col">
      
      <!-- Step 1: Welcome -->
      {#if currentStep === 1}
        <div class="flex-1 flex flex-col items-center justify-center text-center space-y-6">
          <div class="w-20 h-20 bg-primary-500/20 rounded-full flex items-center justify-center">
            <svg class="w-10 h-10 text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          
          <div>
            <h1 class="text-3xl font-bold text-white mb-3">Добро пожаловать в Isolate</h1>
            <p class="text-gray-400 leading-relaxed">
              Isolate автоматически находит и применяет лучший способ обхода DPI-блокировок для вашего интернет-провайдера.
            </p>
          </div>
          
          <div class="space-y-3 text-left w-full">
            <div class="flex items-center gap-3 text-gray-300">
              <svg class="w-5 h-5 text-primary-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <span>Автоматический подбор стратегии</span>
            </div>
            <div class="flex items-center gap-3 text-gray-300">
              <svg class="w-5 h-5 text-primary-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <span>Поддержка Discord, YouTube, Telegram</span>
            </div>
            <div class="flex items-center gap-3 text-gray-300">
              <svg class="w-5 h-5 text-primary-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <span>Работает в один клик</span>
            </div>
          </div>
        </div>

      <!-- Step 2: Select Services -->
      {:else if currentStep === 2}
        <div class="flex-1 flex flex-col">
          <div class="text-center mb-6">
            <h2 class="text-2xl font-bold text-white mb-2">Выберите сервисы</h2>
            <p class="text-gray-400">Какие сервисы вы хотите разблокировать?</p>
          </div>
          
          <div class="space-y-3 flex-1">
            {#each availableServices as service}
              <button
                onclick={() => toggleService(service.id)}
                class={getServiceButtonClass(service.id)}
              >
                <div class={getCheckboxClass(isServiceSelected(service.id))}>
                  {#if isServiceSelected(service.id)}
                    <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                    </svg>
                  {/if}
                </div>
                <div>
                  <div class="font-medium text-white">{service.name}</div>
                  <div class="text-sm text-gray-400">{service.description}</div>
                </div>
              </button>
            {/each}
          </div>
          
          {#if selectedServices.length === 0}
            <p class="text-amber-400 text-sm text-center mt-4">
              Выберите хотя бы один сервис
            </p>
          {/if}
        </div>

      <!-- Step 3: Telemetry -->
      {:else if currentStep === 3}
        <div class="flex-1 flex flex-col items-center justify-center text-center space-y-6">
          <div class="w-20 h-20 bg-gray-700 rounded-full flex items-center justify-center">
            <svg class="w-10 h-10 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
          </div>
          
          <div>
            <h2 class="text-2xl font-bold text-white mb-2">Помогите улучшить Isolate</h2>
            <p class="text-gray-400 leading-relaxed">
              Анонимная статистика помогает нам понять, какие стратегии работают лучше всего у разных провайдеров.
            </p>
          </div>
          
          <button
            onclick={() => telemetryEnabled = !telemetryEnabled}
            class={getTelemetryButtonClass()}
          >
            <div class={getCheckboxClass(telemetryEnabled)}>
              {#if telemetryEnabled}
                <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                </svg>
              {/if}
            </div>
            <div>
              <div class="font-medium text-white">Отправлять анонимную статистику</div>
              <div class="text-sm text-gray-400">Только данные о работе стратегий</div>
            </div>
          </button>
          
          <div class="text-xs text-gray-500 space-y-1">
            <p>Мы <strong>не</strong> собираем:</p>
            <p>• IP-адреса • Личные данные • Историю посещений</p>
          </div>
        </div>

      <!-- Step 4: Ready -->
      {:else if currentStep === 4}
        <div class="flex-1 flex flex-col items-center justify-center text-center space-y-6">
          <div class="w-20 h-20 bg-green-500/20 rounded-full flex items-center justify-center">
            <svg class="w-10 h-10 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          
          <div>
            <h2 class="text-2xl font-bold text-white mb-2">Всё готово!</h2>
            <p class="text-gray-400 leading-relaxed">
              Isolate настроен и готов к работе. Нажмите «Начать», чтобы запустить автоматическую оптимизацию.
            </p>
          </div>
          
          <div class="bg-gray-700/50 rounded-xl p-4 w-full text-left space-y-2">
            <div class="flex justify-between text-sm">
              <span class="text-gray-400">Сервисы:</span>
              <span class="text-white">
                {selectedServices.map(id => 
                  availableServices.find(s => s.id === id)?.name
                ).join(', ')}
              </span>
            </div>
            <div class="flex justify-between text-sm">
              <span class="text-gray-400">Телеметрия:</span>
              <span class="text-white">{telemetryEnabled ? 'Включена' : 'Отключена'}</span>
            </div>
          </div>
        </div>
      {/if}

      <!-- Navigation Buttons -->
      <div class="flex gap-3 pt-4">
        {#if currentStep > 1}
          <button
            onclick={prevStep}
            class="flex-1 py-3 px-6 bg-gray-700 hover:bg-gray-600 rounded-xl font-medium transition-colors"
          >
            Назад
          </button>
        {:else}
          <button
            onclick={skipOnboarding}
            class="flex-1 py-3 px-6 text-gray-400 hover:text-gray-300 rounded-xl font-medium transition-colors"
          >
            Пропустить
          </button>
        {/if}
        
        <button
          onclick={nextStep}
          disabled={currentStep === 2 && selectedServices.length === 0}
          class="flex-1 py-3 px-6 bg-primary-600 hover:bg-primary-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-xl font-medium transition-colors"
        >
          {currentStep === totalSteps ? 'Начать' : 'Далее'}
        </button>
      </div>
    </div>
  </div>
</div>
