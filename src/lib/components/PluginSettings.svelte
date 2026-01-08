<script lang="ts">
  import type { PluginInfo, PluginSetting, PluginSettingValue } from '$lib/stores/plugins';
  import { updatePluginSettings } from '$lib/stores/plugins';
  import { toasts } from '$lib/stores/toast';
  import { invoke } from '@tauri-apps/api/core';
  import BaseModal from './BaseModal.svelte';

  interface Props {
    open?: boolean;
    plugin: PluginInfo;
    onClose: () => void;
  }

  let { open = $bindable(true), plugin, onClose }: Props = $props();

  // Создаём локальную копию настроек для редактирования
  // svelte-ignore state_referenced_locally
  let localSettings = $state<PluginSetting[]>(
    plugin.settings ? JSON.parse(JSON.stringify(plugin.settings)) : []
  );
  
  // Original settings for comparison (loaded from backend or defaults)
  let originalSettings = $state<PluginSetting[]>([]);
  
  // Loading state
  let loading = $state(true);

  let hasChanges = $derived(() => {
    if (originalSettings.length === 0) return false;
    return JSON.stringify(localSettings) !== JSON.stringify(originalSettings);
  });

  // Load saved settings from backend on mount
  import { onMount } from 'svelte';
  
  onMount(() => {
    loadSettings();
  });
  
  async function loadSettings() {
    loading = true;
    try {
      const ready = await invoke<boolean>('is_backend_ready');
      if (!ready) {
        // Use defaults from plugin
        originalSettings = plugin.settings ? JSON.parse(JSON.stringify(plugin.settings)) : [];
        localSettings = JSON.parse(JSON.stringify(originalSettings));
        return;
      }
      
      // Get saved settings from backend
      const savedSettings = await invoke<Array<{id: string, value: PluginSettingValue}>>('get_plugin_settings', {
        pluginId: plugin.id
      });
      
      // Merge saved values with plugin defaults
      if (plugin.settings && savedSettings.length > 0) {
        const savedMap = new Map(savedSettings.map(s => [s.id, s.value]));
        originalSettings = plugin.settings.map(setting => ({
          ...setting,
          value: savedMap.has(setting.id) ? savedMap.get(setting.id)! : setting.value
        }));
      } else {
        originalSettings = plugin.settings ? JSON.parse(JSON.stringify(plugin.settings)) : [];
      }
      
      localSettings = JSON.parse(JSON.stringify(originalSettings));
    } catch (error) {
      console.warn('Failed to load plugin settings:', error);
      // Fallback to defaults
      originalSettings = plugin.settings ? JSON.parse(JSON.stringify(plugin.settings)) : [];
      localSettings = JSON.parse(JSON.stringify(originalSettings));
    } finally {
      loading = false;
    }
  }

  function updateSetting(id: string, value: PluginSettingValue) {
    localSettings = localSettings.map(s => 
      s.id === id ? { ...s, value } : s
    );
  }

  async function handleSave() {
    try {
      const ready = await invoke<boolean>('is_backend_ready');
      if (ready) {
        // Save to backend
        const settingsToSave = localSettings.map(s => ({
          id: s.id,
          value: s.value
        }));
        
        await invoke('set_plugin_settings', {
          pluginId: plugin.id,
          settings: settingsToSave
        });
      }
      
      // Update store
      updatePluginSettings(plugin.id, localSettings);
      originalSettings = JSON.parse(JSON.stringify(localSettings));
      
      toasts.success(`${plugin.name} settings saved`);
      handleClose();
    } catch (error) {
      console.error('Failed to save plugin settings:', error);
      toasts.error(`Failed to save settings: ${error}`);
    }
  }

  async function handleReset() {
    try {
      const ready = await invoke<boolean>('is_backend_ready');
      if (ready) {
        // Reset in backend
        await invoke('reset_plugin_settings', {
          pluginId: plugin.id
        });
      }
      
      // Reset to plugin defaults
      if (plugin.settings) {
        localSettings = JSON.parse(JSON.stringify(plugin.settings));
        originalSettings = JSON.parse(JSON.stringify(plugin.settings));
        
        // Update store with defaults
        updatePluginSettings(plugin.id, localSettings);
        
        toasts.success(`${plugin.name} settings reset to defaults`);
      }
    } catch (error) {
      console.error('Failed to reset plugin settings:', error);
      toasts.error(`Failed to reset settings: ${error}`);
    }
  }

  function handleClose() {
    open = false;
    onClose();
  }
</script>

<BaseModal bind:open onclose={handleClose} class="w-full max-w-lg overflow-hidden">
    <!-- Header -->
    <div class="flex items-center gap-4 p-5 border-b border-white/5">
      <div class="w-12 h-12 rounded-xl bg-zinc-800 flex items-center justify-center text-2xl">
        {plugin.icon}
      </div>
      <div class="flex-1">
        <h2 class="text-lg font-semibold text-zinc-100">{plugin.name}</h2>
        <p class="text-sm text-zinc-400">v{plugin.version} • Settings</p>
      </div>
      <button
        onclick={onClose}
        aria-label="Close settings"
        class="p-2 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-white/5 transition-colors"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
      </button>
    </div>

    <!-- Settings Content -->
    <div class="p-5 max-h-[60vh] overflow-y-auto space-y-4">
      {#if loading}
        <div class="text-center py-8">
          <div class="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto mb-3"></div>
          <p class="text-zinc-400">Loading settings...</p>
        </div>
      {:else if localSettings.length === 0}
        <div class="text-center py-8">
          <div class="text-4xl mb-3">⚙️</div>
          <p class="text-zinc-400">No settings available</p>
          <p class="text-sm text-zinc-400 mt-1">This plugin has no configurable settings</p>
        </div>
      {:else}
        {#each localSettings as setting (setting.id)}
          <div class="p-4 bg-zinc-800/50 rounded-xl border border-white/5">
            {#if setting.type === 'toggle'}
              <!-- Toggle Setting -->
              <div class="flex items-center justify-between">
                <div class="flex-1 pr-4">
                  <label for={setting.id} class="font-medium text-zinc-200 cursor-pointer">
                    {setting.label}
                  </label>
                  {#if setting.description}
                    <p class="text-sm text-zinc-500 mt-0.5">{setting.description}</p>
                  {/if}
                </div>
                <button
                  id={setting.id}
                  role="switch"
                  aria-checked={Boolean(setting.value)}
                  aria-label="Toggle {setting.label}"
                  onclick={() => updateSetting(setting.id, !setting.value)}
                  class="relative w-11 h-6 rounded-full transition-colors
                         {setting.value ? 'bg-indigo-500' : 'bg-zinc-700'}"
                >
                  <span 
                    class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform
                           {setting.value ? 'translate-x-5' : 'translate-x-0'}"
                  ></span>
                </button>
              </div>

            {:else if setting.type === 'select'}
              <!-- Select Setting -->
              <div>
                <label for={setting.id} class="block font-medium text-zinc-200 mb-1">
                  {setting.label}
                </label>
                {#if setting.description}
                  <p class="text-sm text-zinc-500 mb-2">{setting.description}</p>
                {/if}
                <select
                  id={setting.id}
                  value={setting.value}
                  onchange={(e) => updateSetting(setting.id, e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-zinc-900 border border-white/10 rounded-lg
                         text-zinc-100 focus:outline-none focus:border-indigo-500/50 cursor-pointer"
                >
                  {#each setting.options || [] as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                  {/each}
                </select>
              </div>

            {:else if setting.type === 'number'}
              <!-- Number Setting -->
              <div>
                <label for={setting.id} class="block font-medium text-zinc-200 mb-1">
                  {setting.label}
                </label>
                {#if setting.description}
                  <p class="text-sm text-zinc-500 mb-2">{setting.description}</p>
                {/if}
                <div class="flex items-center gap-3">
                  <input
                    id={setting.id}
                    type="number"
                    value={setting.value}
                    min={setting.min}
                    max={setting.max}
                    oninput={(e) => {
                      const val = parseInt(e.currentTarget.value);
                      if (!isNaN(val)) {
                        const clamped = Math.min(
                          Math.max(val, setting.min ?? -Infinity),
                          setting.max ?? Infinity
                        );
                        updateSetting(setting.id, clamped);
                      }
                    }}
                    class="flex-1 px-3 py-2 bg-zinc-900 border border-white/10 rounded-lg
                           text-zinc-100 focus:outline-none focus:border-indigo-500/50"
                  />
                  {#if setting.min !== undefined || setting.max !== undefined}
                    <span class="text-xs text-zinc-500 whitespace-nowrap">
                      {setting.min ?? '∞'} – {setting.max ?? '∞'}
                    </span>
                  {/if}
                </div>
              </div>

            {:else if setting.type === 'text'}
              <!-- Text Setting -->
              <div>
                <label for={setting.id} class="block font-medium text-zinc-200 mb-1">
                  {setting.label}
                </label>
                {#if setting.description}
                  <p class="text-sm text-zinc-500 mb-2">{setting.description}</p>
                {/if}
                <input
                  id={setting.id}
                  type="text"
                  value={setting.value}
                  placeholder={setting.placeholder}
                  oninput={(e) => updateSetting(setting.id, e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-zinc-900 border border-white/10 rounded-lg
                         text-zinc-100 placeholder-zinc-600
                         focus:outline-none focus:border-indigo-500/50"
                />
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-between p-5 border-t border-white/5 bg-zinc-900/50">
      <button
        onclick={handleReset}
        disabled={loading}
        class="px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
        title="Reset to default values"
      >
        Reset to Defaults
      </button>
      <div class="flex items-center gap-3">
        <button
          onclick={onClose}
          class="px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 
                 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleSave}
          disabled={loading || !hasChanges()}
          class="px-4 py-2 text-sm font-medium text-white
                 bg-indigo-500 hover:bg-indigo-600 rounded-lg transition-colors
                 disabled:opacity-50 disabled:cursor-not-allowed
                 flex items-center gap-2"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
            <polyline points="17 21 17 13 7 13 7 21"/>
            <polyline points="7 3 7 8 15 8"/>
          </svg>
          Save
        </button>
      </div>
    </div>
</BaseModal>
