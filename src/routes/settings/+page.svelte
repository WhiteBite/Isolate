<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';

  // VLESS config type
  interface VlessConfig {
    id: string;
    name: string;
    server: string;
    port: number;
    uuid: string;
    flow: string | null;
    security: string;
    sni: string | null;
    active: boolean;
  }

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

  // Privacy
  let telemetryEnabled = $state(false);

  // About
  let appVersion = $state('0.1.0');

  // VLESS
  let vlessUrl = $state('');
  let vlessConfigs = $state<VlessConfig[]>([]);
  let vlessImporting = $state(false);
  let vlessError = $state('');

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

      // Load telemetry setting
      const telemetrySetting = await invoke<boolean>('get_setting', { key: 'telemetry_enabled' }).catch(() => false);
      telemetryEnabled = telemetrySetting;

      // Load VLESS configs
      await loadVlessConfigs();
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  });

  async function loadVlessConfigs() {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const configs = await invoke<VlessConfig[]>('get_vless_configs').catch(() => []);
      vlessConfigs = configs;
    } catch (e) {
      console.error('Failed to load VLESS configs:', e);
    }
  }

  async function importVless() {
    if (!browser || !vlessUrl.trim()) return;
    
    vlessImporting = true;
    vlessError = '';
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke<VlessConfig>('import_vless', { url: vlessUrl.trim() });
      vlessUrl = '';
      await loadVlessConfigs();
    } catch (e) {
      vlessError = String(e);
      console.error('Failed to import VLESS:', e);
    } finally {
      vlessImporting = false;
    }
  }

  async function deleteVlessConfig(id: string) {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_vless_config', { id });
      await loadVlessConfigs();
    } catch (e) {
      console.error('Failed to delete VLESS config:', e);
    }
  }

  async function toggleVlessConfig(id: string, active: boolean) {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('toggle_vless_config', { id, active });
      await loadVlessConfigs();
    } catch (e) {
      console.error('Failed to toggle VLESS config:', e);
    }
  }

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

  async function saveTelemetrySetting(enabled: boolean) {
    if (!browser) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_setting', { key: 'telemetry_enabled', value: enabled });
    } catch (e) {
      console.error('Failed to save telemetry setting:', e);
    }
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

    <!-- VLESS Proxy Section -->
    <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
      <h2 class="text-lg font-medium text-white mb-4">VLESS Прокси</h2>
      
      <!-- Import form -->
      <div class="space-y-3">
        <input
          type="text"
          bind:value={vlessUrl}
          placeholder="vless://uuid@server:port?..."
          class="w-full bg-gray-700 border border-gray-600 text-gray-300 rounded-lg px-4 py-3 focus:ring-primary-500 focus:border-primary-500 placeholder-gray-500"
        />
        
        <button
          onclick={importVless}
          disabled={vlessImporting || !vlessUrl.trim()}
          class="w-full py-3 px-4 bg-primary-600 hover:bg-primary-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
        >
          {#if vlessImporting}
            <svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span>Импорт...</span>
          {:else}
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
            <span>Импортировать</span>
          {/if}
        </button>

        {#if vlessError}
          <p class="text-red-400 text-sm">{vlessError}</p>
        {/if}
      </div>

      <!-- Saved configs list -->
      {#if vlessConfigs.length > 0}
        <div class="space-y-3 pt-4 border-t border-gray-700">
          <p class="text-sm text-gray-400">Сохранённые конфигурации:</p>
          
          {#each vlessConfigs as config}
            <div class="flex items-center justify-between bg-gray-700/50 rounded-lg p-3">
              <div class="flex items-center gap-3 flex-1 min-w-0">
                <!-- Status indicator -->
                <button
                  onclick={() => toggleVlessConfig(config.id, !config.active)}
                  class="flex-shrink-0 w-3 h-3 rounded-full transition-colors {config.active ? 'bg-green-500' : 'bg-gray-500 hover:bg-gray-400'}"
                  title={config.active ? 'Активен' : 'Неактивен'}
                ></button>
                
                <div class="min-w-0 flex-1">
                  <p class="text-gray-200 font-medium truncate">{config.name}</p>
                  <p class="text-gray-500 text-xs truncate">{config.server}:{config.port}</p>
                </div>
              </div>
              
              <button
                onclick={() => deleteVlessConfig(config.id)}
                class="flex-shrink-0 p-2 text-gray-400 hover:text-red-400 hover:bg-gray-600 rounded-lg transition-colors"
                title="Удалить"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-gray-500 text-sm text-center py-4">Нет сохранённых конфигураций</p>
      {/if}
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

    <!-- Privacy Section -->
    <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
      <h2 class="text-lg font-medium text-white mb-4">Приватность</h2>
      
      <label class="flex items-start justify-between cursor-pointer">
        <div class="flex-1 mr-4">
          <span class="text-gray-300">Телеметрия</span>
          <p class="text-xs text-gray-500 mt-1">Отправлять анонимную статистику для улучшения приложения</p>
        </div>
        <input
          type="checkbox"
          bind:checked={telemetryEnabled}
          onchange={() => saveTelemetrySetting(telemetryEnabled)}
          class="w-5 h-5 rounded bg-gray-700 border-gray-600 text-primary-500 focus:ring-primary-500 focus:ring-offset-gray-800 cursor-pointer mt-1"
        />
      </label>

      <div class="text-xs text-gray-500 space-y-2 pt-2 border-t border-gray-700">
        <p class="flex items-start gap-2">
          <span class="text-green-500">✓</span>
          <span>Только: ASN провайдера, страна, ID стратегии, успешность</span>
        </p>
        <p class="flex items-start gap-2">
          <span class="text-red-500">✗</span>
          <span>Не собираем: IP-адреса, личные данные, историю</span>
        </p>
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
