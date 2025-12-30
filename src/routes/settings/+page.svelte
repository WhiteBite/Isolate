<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';

  // General settings
  let autoStart = $state(false);
  let autoApply = $state(false);
  let minimizeToTray = $state(true);

  // Services
  let services = $state<{id: string; name: string; enabled: boolean}[]>([
    { id: 'discord', name: 'Discord', enabled: true },
    { id: 'youtube', name: 'YouTube', enabled: true },
    { id: 'telegram', name: 'Telegram', enabled: false }
  ]);

  // Advanced
  let blockQuic = $state(true);
  let defaultMode = $state<'turbo' | 'deep'>('turbo');

  // About
  let appVersion = $state('0.1.0');

  onMount(async () => {
    if (!browser) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Load settings from backend
      const settings = await invoke<{
        auto_start: boolean;
        auto_apply: boolean;
        minimize_to_tray: boolean;
        block_quic: boolean;
        default_mode: 'turbo' | 'deep';
      }>('get_settings').catch(() => null);

      if (settings) {
        autoStart = settings.auto_start;
        autoApply = settings.auto_apply;
        minimizeToTray = settings.minimize_to_tray;
        blockQuic = settings.block_quic;
        defaultMode = settings.default_mode;
      }

      // Load services
      const loadedServices = await invoke<{id: string; name: string; enabled: boolean}[]>('get_services_settings').catch(() => null);
      if (loadedServices && loadedServices.length > 0) {
        services = loadedServices;
      }

      // Get app version
      const version = await invoke<string>('get_app_version').catch(() => '0.1.0');
      appVersion = version;
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  });

  async function saveSettings() {
    if (!browser) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_settings', {
        settings: {
          auto_start: autoStart,
          auto_apply: autoApply,
          minimize_to_tray: minimizeToTray,
          block_quic: blockQuic,
          default_mode: defaultMode
        }
      });
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }

  async function toggleService(serviceId: string) {
    const service = services.find(s => s.id === serviceId);
    if (!service) return;
    
    const newEnabled = !service.enabled;
    services = services.map(s => 
      s.id === serviceId ? { ...s, enabled: newEnabled } : s
    );

    if (!browser) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('toggle_service', { serviceId, enabled: newEnabled });
    } catch (e) {
      console.error('Failed to toggle service:', e);
    }
  }

  function handleBack() {
    goto('/');
  }

  // Auto-save on changes
  $effect(() => {
    if (browser) {
      saveSettings();
    }
  });
</script>

<div class="flex flex-col items-center min-h-screen p-8">
  <div class="w-full max-w-md space-y-6">
    <!-- Header -->
    <div class="flex items-center gap-4">
      <button
        onclick={handleBack}
        class="p-2 hover:bg-gray-800 rounded-lg transition-colors"
        aria-label="Назад"
      >
        <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h1 class="text-2xl font-bold text-white">Настройки</h1>
    </div>

    <!-- General Section -->
    <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
      <h2 class="text-lg font-medium text-white mb-4">Основные</h2>
      
      <label class="flex items-center justify-between cursor-pointer">
        <span class="text-gray-300">Автозапуск</span>
        <input
          type="checkbox"
          bind:checked={autoStart}
          class="w-5 h-5 rounded bg-gray-700 border-gray-600 text-primary-500 focus:ring-primary-500 focus:ring-offset-gray-800 cursor-pointer"
        />
      </label>

      <label class="flex items-center justify-between cursor-pointer">
        <span class="text-gray-300">Автоприменение</span>
        <input
          type="checkbox"
          bind:checked={autoApply}
          class="w-5 h-5 rounded bg-gray-700 border-gray-600 text-primary-500 focus:ring-primary-500 focus:ring-offset-gray-800 cursor-pointer"
        />
      </label>

      <label class="flex items-center justify-between cursor-pointer">
        <span class="text-gray-300">Сворачивать в трей</span>
        <input
          type="checkbox"
          bind:checked={minimizeToTray}
          class="w-5 h-5 rounded bg-gray-700 border-gray-600 text-primary-500 focus:ring-primary-500 focus:ring-offset-gray-800 cursor-pointer"
        />
      </label>
    </div>

    <!-- Services Section -->
    <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
      <h2 class="text-lg font-medium text-white mb-4">Сервисы</h2>
      
      {#each services as service}
        <label class="flex items-center justify-between cursor-pointer">
          <span class="text-gray-300">{service.name}</span>
          <input
            type="checkbox"
            checked={service.enabled}
            onchange={() => toggleService(service.id)}
            class="w-5 h-5 rounded bg-gray-700 border-gray-600 text-primary-500 focus:ring-primary-500 focus:ring-offset-gray-800 cursor-pointer"
          />
        </label>
      {/each}
    </div>

    <!-- Advanced Section -->
    <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
      <h2 class="text-lg font-medium text-white mb-4">Расширенные</h2>
      
      <label class="flex items-center justify-between cursor-pointer">
        <div>
          <span class="text-gray-300">Блокировать QUIC</span>
          <p class="text-xs text-gray-500 mt-1">Принудительно использовать TCP</p>
        </div>
        <input
          type="checkbox"
          bind:checked={blockQuic}
          class="w-5 h-5 rounded bg-gray-700 border-gray-600 text-primary-500 focus:ring-primary-500 focus:ring-offset-gray-800 cursor-pointer"
        />
      </label>

      <div class="flex items-center justify-between">
        <div>
          <span class="text-gray-300">Режим по умолчанию</span>
          <p class="text-xs text-gray-500 mt-1">Turbo — быстрый, Deep — тщательный</p>
        </div>
        <select
          bind:value={defaultMode}
          class="bg-gray-700 border border-gray-600 text-gray-300 rounded-lg px-3 py-2 focus:ring-primary-500 focus:border-primary-500 cursor-pointer"
        >
          <option value="turbo">Turbo</option>
          <option value="deep">Deep</option>
        </select>
      </div>
    </div>

    <!-- About Section -->
    <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
      <h2 class="text-lg font-medium text-white mb-4">О приложении</h2>
      
      <div class="flex items-center justify-between">
        <span class="text-gray-300">Версия</span>
        <span class="text-gray-400">{appVersion}</span>
      </div>

      <div class="flex items-center justify-between">
        <span class="text-gray-300">Isolate</span>
        <span class="text-gray-500 text-sm">Обход DPI-блокировок</span>
      </div>
    </div>

    <!-- Back Button -->
    <button
      onclick={handleBack}
      class="w-full py-3 px-6 bg-gray-700 hover:bg-gray-600 rounded-xl font-medium transition-colors"
    >
      Назад
    </button>
  </div>
</div>
