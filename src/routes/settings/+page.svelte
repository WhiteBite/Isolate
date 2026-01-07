<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { settings } from '$lib/stores';
  import { themeStore, type Theme } from '$lib/stores/theme';
  import { 
    hotkeysStore, 
    formatHotkey, 
    parseKeyboardEvent,
    HOTKEY_ACTIONS,
    type HotkeyConfig,
    type HotkeysState
  } from '$lib/stores/hotkeys';
  import { get } from 'svelte/store';
  import { waitForBackend } from '$lib/utils/backend';
  import { t, setLocale, getLocale, type Locale } from '$lib/i18n';
  import ProviderSelector from '$lib/components/settings/ProviderSelector.svelte';
  import AutoRecoverySettings from '$lib/components/settings/AutoRecoverySettings.svelte';

  // Types
  type TabId = 'general' | 'routing' | 'hotkeys' | 'hostlists' | 'advanced';
  type Language = 'ru' | 'en';

  interface AppSettings {
    autoStart: boolean;
    minimizeToTray: boolean;
    notifications: boolean;
    language: Language;
    theme: Theme;
    // Routing
    domainExceptions: string[];
    perAppRouting: boolean;
    // Advanced
    windivertMode: 'normal' | 'autottl' | 'autohostlist';
    gameFilterMode: 'normal' | 'gaming';
    dnsOverride: string;
    blockQuic: boolean;
    tcpTimestamps: boolean;
    debugMode: boolean;
    // Auto Failover
    autoFailoverEnabled: boolean;
    failoverMaxFailures: number;
    failoverCooldownSecs: number;
  }

  interface ImportResult {
    settings_imported: boolean;
    proxies_imported: number;
    proxies_skipped: number;
    routing_rules_imported: number;
    routing_rules_skipped: number;
  }

  interface HostlistInfo {
    id: string;
    name: string;
    last_updated: string | null;
    size: number | null;
    domain_count: number | null;
    update_available: boolean;
    source_url: string | null;
  }

  interface UpdateCheckResult {
    id: string;
    has_update: boolean;
    current_count: number | null;
    error: string | null;
  }

  interface UpdateResult {
    updated_count: number;
    failed_count: number;
    updated: string[];
    failed: [string, string][];
  }

  // State
  let activeTab = $state<TabId>('general');
  let localSettings = $state<AppSettings>({
    autoStart: false,
    minimizeToTray: true,
    notifications: true,
    language: 'ru',
    theme: 'dark',
    domainExceptions: [],
    perAppRouting: false,
    windivertMode: 'normal',
    gameFilterMode: 'normal',
    dnsOverride: '',
    blockQuic: true,
    tcpTimestamps: false,
    debugMode: false,
    autoFailoverEnabled: false,
    failoverMaxFailures: 3,
    failoverCooldownSecs: 60
  });

  let saving = $state(false);
  let saveMessage = $state<{ text: string; type: 'success' | 'error' } | null>(null);
  let isTauri = $state(false);
  let newException = $state('');
  
  // Backup/Restore state
  let exporting = $state(false);
  let importing = $state(false);

  // Hostlists state
  let hostlists = $state<HostlistInfo[]>([]);
  let hostlistsLoading = $state(false);
  let checkingUpdates = $state(false);
  let updatingHostlists = $state(false);
  let hostlistUpdateResults = $state<UpdateCheckResult[]>([]);

  // Hotkeys state
  let hotkeys = $state<HotkeysState>(hotkeysStore.get());
  let recordingHotkey = $state<keyof HotkeysState | null>(null);
  let hotkeyConflict = $state<string | null>(null);

  // Tabs configuration
  const tabs: { id: TabId; label: string; icon: string }[] = [
    { id: 'general', label: 'General', icon: 'settings' },
    { id: 'routing', label: 'Routing', icon: 'route' },
    { id: 'hotkeys', label: 'Hotkeys', icon: 'keyboard' },
    { id: 'hostlists', label: 'Hostlists', icon: 'list' },
    { id: 'advanced', label: 'Advanced', icon: 'code' }
  ];

  // Initialize on component mount and navigation
  let initialized = $state(false);
  
  // Subscribe to hotkeys store
  $effect(() => {
    const unsubscribe = hotkeysStore.subscribe(state => {
      hotkeys = state;
    });
    return unsubscribe;
  });
  
  $effect(() => {
    if (!browser || initialized) return;
    initialized = true;
    
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;

    // Load initial settings from store (one-time)
    const currentSettings = get(settings);
    localSettings.autoStart = currentSettings.autoStart;
    localSettings.minimizeToTray = currentSettings.minimizeToTray;
    localSettings.blockQuic = currentSettings.blockQuic;
    
    // Load theme from themeStore
    localSettings.theme = themeStore.get();
    
    // Load language from i18n store
    localSettings.language = getLocale();

    if (isTauri) {
      loadSettings();
      loadHostlists();
    }
  });

  // Handle theme change
  function handleThemeChange(newTheme: Theme) {
    localSettings.theme = newTheme;
    themeStore.set(newTheme);
  }

  async function loadSettings() {
    if (!browser || !isTauri) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Wait for backend to be ready
      const ready = await waitForBackend(10, 200);
      if (!ready) {
        console.warn('[Settings] Backend not ready after retries');
        return;
      }
      
      const loaded = await invoke<Partial<AppSettings>>('get_settings').catch(() => ({}));
      if (loaded) {
        localSettings = { ...localSettings, ...loaded };
      }
      
      // Load game filter mode separately (in case it's not in get_settings response)
      const gameMode = await invoke<string>('get_game_filter_mode').catch(() => 'normal');
      localSettings.gameFilterMode = gameMode as 'normal' | 'gaming';
      
      // Load TCP timestamps status
      const tcpTimestampsStatus = await invoke<string>('get_tcp_timestamps_status').catch(() => 'Disabled');
      localSettings.tcpTimestamps = tcpTimestampsStatus === 'Enabled';
      
      // Load failover config
      const failoverConfig = await invoke<{ maxFailures: number; cooldownSecs: number }>('get_failover_config').catch(() => null);
      if (failoverConfig) {
        localSettings.failoverMaxFailures = failoverConfig.maxFailures;
        localSettings.failoverCooldownSecs = failoverConfig.cooldownSecs;
      }
      
      // Load failover status to get enabled state
      const failoverStatus = await invoke<{ enabled: boolean }>('get_failover_status').catch(() => null);
      if (failoverStatus) {
        localSettings.autoFailoverEnabled = failoverStatus.enabled;
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async function saveSettings() {
    if (!browser) return;
    
    saving = true;
    saveMessage = null;
    
    try {
      // Update global store
      settings.set({
        autoStart: localSettings.autoStart,
        autoApply: false,
        minimizeToTray: localSettings.minimizeToTray,
        blockQuic: localSettings.blockQuic,
        defaultMode: 'turbo'
      });

      if (isTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('save_settings', { settings: localSettings });
      }
      
      saveMessage = { text: 'Settings saved successfully', type: 'success' };
      setTimeout(() => { saveMessage = null; }, 3000);
    } catch (e) {
      console.error('Failed to save settings:', e);
      // Show specific error message to user
      const errorMessage = e instanceof Error ? e.message : String(e);
      saveMessage = { 
        text: `Failed to save: ${errorMessage}`, 
        type: 'error' 
      };
      // Keep error visible longer
      setTimeout(() => { saveMessage = null; }, 5000);
    } finally {
      saving = false;
    }
  }

  async function setSetting(key: string, value: unknown) {
    if (!browser || !isTauri) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_setting', { key, value }).catch(() => {});
    } catch (e) {
      console.error('Failed to set setting:', e);
    }
  }

  function addException() {
    if (newException.trim() && !localSettings.domainExceptions.includes(newException.trim())) {
      localSettings.domainExceptions = [...localSettings.domainExceptions, newException.trim()];
      newException = '';
    }
  }

  function removeException(domain: string) {
    localSettings.domainExceptions = localSettings.domainExceptions.filter(d => d !== domain);
  }

  // Hotkey recording functions
  function startRecordingHotkey(action: keyof HotkeysState) {
    recordingHotkey = action;
    hotkeyConflict = null;
  }

  function stopRecordingHotkey() {
    recordingHotkey = null;
    hotkeyConflict = null;
  }

  function handleHotkeyKeydown(e: KeyboardEvent) {
    if (!recordingHotkey) return;
    
    e.preventDefault();
    e.stopPropagation();
    
    const config = parseKeyboardEvent(e);
    if (!config) return; // Ignore modifier-only presses
    
    // Check for conflicts
    const conflict = hotkeysStore.hasConflict(recordingHotkey, config);
    if (conflict) {
      const conflictAction = HOTKEY_ACTIONS.find(a => a.id === conflict);
      hotkeyConflict = conflictAction?.label || conflict;
      return;
    }
    
    // Save the new hotkey
    hotkeysStore.setHotkey(recordingHotkey, config);
    stopRecordingHotkey();
  }

  function resetHotkey(action: keyof HotkeysState) {
    hotkeysStore.resetHotkey(action);
  }

  function resetAllHotkeys() {
    hotkeysStore.resetToDefaults();
  }

  async function resetOnboarding() {
    if (!confirm('This will reset the onboarding wizard. Continue?')) return;
    
    try {
      // Clear onboarding flag from localStorage
      if (browser) {
        localStorage.removeItem('onboarding_completed');
      }
      
      // Also try to reset via backend if available
      if (isTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('reset_onboarding').catch(() => {
          // Command may not exist, that's ok - localStorage is primary
        });
      }
      
      // Navigate to onboarding
      goto('/onboarding');
    } catch (e) {
      console.error('Failed to reset onboarding:', e);
    }
  }

  // Export configuration to JSON file
  async function exportConfig() {
    if (!browser || !isTauri || exporting) return;
    
    exporting = true;
    saveMessage = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { save } = await import('@tauri-apps/plugin-dialog');
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      
      // Get config JSON from backend
      const configJson = await invoke<string>('export_config');
      
      // Show save dialog
      const filePath = await save({
        title: 'Export Configuration',
        defaultPath: `isolate-config-${new Date().toISOString().split('T')[0]}.json`,
        filters: [
          { name: 'JSON', extensions: ['json'] },
          { name: 'All Files', extensions: ['*'] }
        ]
      });
      
      if (filePath) {
        // Write file
        await writeTextFile(filePath, configJson);
        saveMessage = { text: 'Configuration exported successfully', type: 'success' };
        setTimeout(() => { saveMessage = null; }, 3000);
      }
    } catch (e) {
      console.error('Failed to export config:', e);
      const errorMessage = e instanceof Error ? e.message : String(e);
      saveMessage = { text: `Export failed: ${errorMessage}`, type: 'error' };
      setTimeout(() => { saveMessage = null; }, 5000);
    } finally {
      exporting = false;
    }
  }

  // Import configuration from JSON file
  async function importConfig() {
    if (!browser || !isTauri || importing) return;
    
    importing = true;
    saveMessage = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { open } = await import('@tauri-apps/plugin-dialog');
      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      
      // Show open dialog
      const filePath = await open({
        title: 'Import Configuration',
        filters: [
          { name: 'JSON', extensions: ['json'] },
          { name: 'All Files', extensions: ['*'] }
        ],
        multiple: false
      });
      
      if (filePath && typeof filePath === 'string') {
        // Read file
        const configJson = await readTextFile(filePath);
        
        // Import via backend
        const result = await invoke<ImportResult>('import_config', { configJson });
        
        // Show result
        const parts: string[] = [];
        if (result.settings_imported) parts.push('settings');
        if (result.proxies_imported > 0) parts.push(`${result.proxies_imported} proxies`);
        if (result.routing_rules_imported > 0) parts.push(`${result.routing_rules_imported} rules`);
        
        if (parts.length > 0) {
          saveMessage = { text: `Imported: ${parts.join(', ')}`, type: 'success' };
          // Reload settings to reflect changes
          await loadSettings();
        } else {
          saveMessage = { text: 'No new data to import (all items already exist)', type: 'success' };
        }
        setTimeout(() => { saveMessage = null; }, 5000);
      }
    } catch (e) {
      console.error('Failed to import config:', e);
      const errorMessage = e instanceof Error ? e.message : String(e);
      saveMessage = { text: `Import failed: ${errorMessage}`, type: 'error' };
      setTimeout(() => { saveMessage = null; }, 5000);
    } finally {
      importing = false;
    }
  }

  // ============================================================================
  // Hostlists Functions
  // ============================================================================

  async function loadHostlists() {
    if (!browser || !isTauri) return;
    
    hostlistsLoading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      hostlists = await invoke<HostlistInfo[]>('get_hostlist_info');
    } catch (e) {
      console.error('Failed to load hostlists:', e);
    } finally {
      hostlistsLoading = false;
    }
  }

  async function checkHostlistUpdates() {
    if (!browser || !isTauri || checkingUpdates) return;
    
    checkingUpdates = true;
    hostlistUpdateResults = [];
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      hostlistUpdateResults = await invoke<UpdateCheckResult[]>('check_hostlist_updates');
      
      // Update hostlists with update availability
      hostlists = hostlists.map(h => ({
        ...h,
        update_available: hostlistUpdateResults.find(r => r.id === h.id)?.has_update ?? false
      }));
      
      const updatesAvailable = hostlistUpdateResults.filter(r => r.has_update).length;
      if (updatesAvailable > 0) {
        saveMessage = { text: `${updatesAvailable} update(s) available`, type: 'success' };
      } else {
        saveMessage = { text: 'All hostlists are up to date', type: 'success' };
      }
      setTimeout(() => { saveMessage = null; }, 3000);
    } catch (e) {
      console.error('Failed to check hostlist updates:', e);
      saveMessage = { text: `Check failed: ${e}`, type: 'error' };
      setTimeout(() => { saveMessage = null; }, 5000);
    } finally {
      checkingUpdates = false;
    }
  }

  async function updateAllHostlists() {
    if (!browser || !isTauri || updatingHostlists) return;
    
    updatingHostlists = true;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<UpdateResult>('update_hostlists');
      
      if (result.updated_count > 0) {
        saveMessage = { 
          text: `Updated ${result.updated_count} hostlist(s)${result.failed_count > 0 ? `, ${result.failed_count} failed` : ''}`, 
          type: result.failed_count > 0 ? 'error' : 'success' 
        };
      } else if (result.failed_count > 0) {
        saveMessage = { text: `Failed to update ${result.failed_count} hostlist(s)`, type: 'error' };
      } else {
        saveMessage = { text: 'No updates available', type: 'success' };
      }
      setTimeout(() => { saveMessage = null; }, 5000);
      
      // Reload hostlists to show updated info
      await loadHostlists();
      hostlistUpdateResults = [];
    } catch (e) {
      console.error('Failed to update hostlists:', e);
      saveMessage = { text: `Update failed: ${e}`, type: 'error' };
      setTimeout(() => { saveMessage = null; }, 5000);
    } finally {
      updatingHostlists = false;
    }
  }

  async function updateSingleHostlist(id: string) {
    if (!browser || !isTauri) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_single_hostlist', { id });
      saveMessage = { text: `Updated ${id}`, type: 'success' };
      setTimeout(() => { saveMessage = null; }, 3000);
      await loadHostlists();
    } catch (e) {
      console.error('Failed to update hostlist:', e);
      saveMessage = { text: `Failed to update ${id}: ${e}`, type: 'error' };
      setTimeout(() => { saveMessage = null; }, 5000);
    }
  }

  function formatDate(isoString: string | null): string {
    if (!isoString) return 'Never';
    try {
      const date = new Date(isoString);
      return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch {
      return 'Unknown';
    }
  }

  function formatSize(bytes: number | null): string {
    if (bytes === null) return '-';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<div class="p-8 min-h-screen bg-void">
  <!-- Header -->
  <div class="flex items-center justify-between mb-8">
    <div>
      <h1 class="text-3xl font-bold text-text-primary">{t('settings.title')}</h1>
      <p class="text-text-secondary mt-1">{t('settings.subtitle')}</p>
    </div>
    <div class="flex items-center gap-3">
      {#if saveMessage}
        <span class="text-sm animate-pulse {saveMessage.type === 'error' ? 'text-red-400' : 'text-indigo-400'}">
          {saveMessage.text}
        </span>
      {/if}
      <button
        onclick={saveSettings}
        disabled={saving}
        class="px-5 py-2.5 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-xl font-medium transition-all duration-200 flex items-center gap-2 shadow-lg shadow-indigo-500/20"
      >
        {#if saving}
          <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
          </svg>
        {/if}
        Save
      </button>
    </div>
  </div>

  <!-- Main Content with Vertical Tabs -->
  <div class="flex gap-6">
    <!-- Vertical Tabs -->
    <div class="w-48 flex-shrink-0">
      <nav class="bg-void-50 border border-glass-border rounded-xl p-2 space-y-1">
        {#each tabs as tab}
          <button
            onclick={() => activeTab = tab.id}
            class="w-full flex items-center gap-3 px-4 py-3 rounded-lg text-left transition-all duration-200 {activeTab === tab.id ? 'bg-indigo-500/10 border-l-2 border-indigo-500 text-indigo-400' : 'text-text-secondary hover:bg-void-100 hover:text-text-primary'}"
          >
            {#if tab.icon === 'settings'}
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
              </svg>
            {:else if tab.icon === 'route'}
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"/>
              </svg>
            {:else if tab.icon === 'code'}
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/>
              </svg>
            {:else if tab.icon === 'keyboard'}
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"/>
              </svg>
            {:else if tab.icon === 'list'}
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16"/>
              </svg>
            {/if}
            <span class="font-medium">{tab.label}</span>
          </button>
        {/each}
      </nav>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 bg-void-50 rounded-xl p-6 border border-glass-border">
      <!-- General Tab -->
      {#if activeTab === 'general'}
        <div>
          <h2 class="text-xl font-semibold text-text-primary mb-6">General Settings</h2>
          
          <div class="space-y-4">
            <!-- Autostart Toggle -->
            <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
              <div>
                <p class="text-text-primary font-medium">Autostart</p>
                <p class="text-text-secondary text-sm">Launch application on Windows startup</p>
              </div>
              <button
                aria-label="Toggle auto-start"
                role="switch"
                aria-checked={localSettings.autoStart}
                onclick={() => { localSettings.autoStart = !localSettings.autoStart; setSetting('autoStart', localSettings.autoStart); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.autoStart ? 'bg-indigo-500' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.autoStart ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- Language Dropdown -->
            <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
              <div>
                <p class="text-text-primary font-medium">{t('settings.general.language')}</p>
                <p class="text-text-secondary text-sm">{t('settings.general.languageDesc')}</p>
              </div>
              <select 
                bind:value={localSettings.language}
                onchange={() => { 
                  setSetting('language', localSettings.language);
                  setLocale(localSettings.language as Locale);
                }}
                class="bg-void-200 text-text-primary rounded-lg px-4 py-2 border border-glass-border focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/20 focus:outline-none cursor-pointer"
              >
                <option value="ru">{t('languages.ru')}</option>
                <option value="en">{t('languages.en')}</option>
              </select>
            </div>

            <!-- Theme Dropdown -->
            <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border dark:bg-void-100 dark:border-glass-border light:bg-gray-100 light:border-gray-200">
              <div>
                <p class="text-text-primary font-medium">{t('settings.general.theme')}</p>
                <p class="text-text-secondary text-sm">{t('settings.general.themeDesc')}</p>
              </div>
              <select 
                bind:value={localSettings.theme}
                onchange={(e) => handleThemeChange(e.currentTarget.value as Theme)}
                class="bg-void-200 text-text-primary rounded-lg px-4 py-2 border border-glass-border focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/20 focus:outline-none cursor-pointer"
              >
                <option value="dark">{t('themes.dark')}</option>
                <option value="light">{t('themes.light')}</option>
                <option value="system">System</option>
              </select>
            </div>

            <!-- Minimize to Tray Toggle -->
            <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
              <div>
                <p class="text-text-primary font-medium">Minimize to Tray</p>
                <p class="text-text-secondary text-sm">Minimize to system tray when closing</p>
              </div>
              <button
                aria-label="Toggle minimize to tray"
                role="switch"
                aria-checked={localSettings.minimizeToTray}
                onclick={() => { localSettings.minimizeToTray = !localSettings.minimizeToTray; setSetting('minimizeToTray', localSettings.minimizeToTray); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.minimizeToTray ? 'bg-indigo-500' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.minimizeToTray ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- Notifications Toggle -->
            <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
              <div>
                <p class="text-text-primary font-medium">Notifications</p>
                <p class="text-text-secondary text-sm">Show system notifications</p>
              </div>
              <button
                aria-label="Toggle notifications"
                role="switch"
                aria-checked={localSettings.notifications}
                onclick={() => { localSettings.notifications = !localSettings.notifications; setSetting('notifications', localSettings.notifications); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.notifications ? 'bg-indigo-500' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.notifications ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- ISP Provider Profile -->
            <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
              <ProviderSelector />
            </div>
          </div>
        </div>
      {/if}

      <!-- Routing Tab -->
      {#if activeTab === 'routing'}
        <div>
          <h2 class="text-xl font-semibold text-text-primary mb-6">Routing Settings</h2>
          
          <div class="space-y-6">
            <!-- Domain Exceptions -->
            <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
              <div class="mb-4">
                <p class="text-text-primary font-medium">Domain Exceptions</p>
                <p class="text-text-secondary text-sm">Domains that bypass DPI filtering</p>
              </div>
              
              <!-- Add new exception -->
              <div class="flex gap-2 mb-4">
                <input
                  type="text"
                  bind:value={newException}
                  placeholder="example.com"
                  onkeydown={(e) => e.key === 'Enter' && addException()}
                  class="flex-1 bg-void-200 text-text-primary rounded-lg px-4 py-2 border border-glass-border focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/20 focus:outline-none placeholder-text-muted"
                />
                <button
                  onclick={addException}
                  class="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 text-white rounded-lg font-medium transition-colors shadow-lg shadow-indigo-500/20"
                >
                  Add
                </button>
              </div>

              <!-- Exception list -->
              <div class="space-y-2 max-h-48 overflow-y-auto">
                {#if localSettings.domainExceptions.length === 0}
                  <p class="text-text-muted text-sm italic">No exceptions configured</p>
                {:else}
                  {#each localSettings.domainExceptions as domain}
                    <div class="flex items-center justify-between p-2 bg-void-200 rounded-lg border border-glass-border">
                      <span class="text-text-primary text-sm font-mono">{domain}</span>
                      <button
                        aria-label="Remove domain"
                        onclick={() => removeException(domain)}
                        class="text-red-400 hover:text-red-300 transition-colors"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                        </svg>
                      </button>
                    </div>
                  {/each}
                {/if}
              </div>
            </div>

            <!-- Per-App Routing (Future) -->
            <div class="p-4 bg-void-100/50 rounded-xl border border-dashed border-glass-border">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-text-muted font-medium">Per-App Routing</p>
                  <p class="text-text-muted/70 text-sm">Route traffic from specific applications through DPI bypass</p>
                </div>
                <span class="px-3 py-1 bg-void-200 text-text-muted text-xs rounded-full">Coming Soon</span>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <!-- Hotkeys Tab -->
      {#if activeTab === 'hotkeys'}
        <div>
          <h2 class="text-xl font-semibold text-text-primary mb-6">Keyboard Shortcuts</h2>
          
          <div class="space-y-4">
            <!-- Hotkey list -->
            {#each HOTKEY_ACTIONS as action}
              <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
                <div class="flex-1">
                  <p class="text-text-primary font-medium">{action.label}</p>
                  <p class="text-text-secondary text-sm">{action.description}</p>
                </div>
                
                <div class="flex items-center gap-2">
                  {#if recordingHotkey === action.id}
                    <!-- Recording mode -->
                    <div class="flex items-center gap-2">
                      <input
                        type="text"
                        readonly
                        placeholder="Press keys..."
                        onkeydown={handleHotkeyKeydown}
                        onblur={stopRecordingHotkey}
                        class="w-40 bg-indigo-500/10 text-indigo-400 rounded-lg px-4 py-2 border-2 border-indigo-500 focus:outline-none text-center font-mono animate-pulse"
                      />
                      <button
                        onclick={stopRecordingHotkey}
                        class="p-2 text-text-muted hover:text-text-primary transition-colors"
                        aria-label="Cancel"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                        </svg>
                      </button>
                    </div>
                  {:else}
                    <!-- Display mode -->
                    <button
                      onclick={() => startRecordingHotkey(action.id)}
                      class="px-4 py-2 bg-void-200 hover:bg-void-300 text-text-primary rounded-lg font-mono text-sm border border-glass-border transition-colors min-w-[140px]"
                    >
                      {formatHotkey(hotkeys[action.id])}
                    </button>
                    <button
                      onclick={() => resetHotkey(action.id)}
                      class="p-2 text-text-muted hover:text-indigo-400 transition-colors"
                      aria-label="Reset to default"
                      title="Reset to default"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                      </svg>
                    </button>
                  {/if}
                </div>
              </div>
              
              {#if recordingHotkey === action.id && hotkeyConflict}
                <div class="ml-4 -mt-2 mb-2 text-red-400 text-sm flex items-center gap-2">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                  </svg>
                  Conflict with "{hotkeyConflict}"
                </div>
              {/if}
            {/each}

            <!-- Reset all button -->
            <div class="pt-4 border-t border-glass-border">
              <button
                onclick={resetAllHotkeys}
                class="px-4 py-2 bg-void-200 hover:bg-void-300 text-text-secondary hover:text-text-primary rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                </svg>
                Reset All to Defaults
              </button>
            </div>

            <!-- Help text -->
            <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
              <p class="text-indigo-400 text-sm flex items-start gap-2">
                <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                <span>Click on a shortcut to change it. Press the new key combination, then release. Use Ctrl, Alt, or Shift with any key.</span>
              </p>
            </div>
          </div>
        </div>
      {/if}

      <!-- Hostlists Tab -->
      {#if activeTab === 'hostlists'}
        <div>
          <div class="flex items-center justify-between mb-6">
            <div>
              <h2 class="text-xl font-semibold text-text-primary">Hostlists</h2>
              <p class="text-text-secondary text-sm mt-1">Manage domain lists for DPI bypass</p>
            </div>
            <div class="flex items-center gap-2">
              <button
                onclick={checkHostlistUpdates}
                disabled={checkingUpdates || updatingHostlists}
                class="px-4 py-2 bg-void-200 hover:bg-void-300 disabled:opacity-50 text-text-primary rounded-lg font-medium transition-colors flex items-center gap-2"
              >
                {#if checkingUpdates}
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                  </svg>
                {/if}
                Check Updates
              </button>
              <button
                onclick={updateAllHostlists}
                disabled={updatingHostlists || checkingUpdates}
                class="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white rounded-lg font-medium transition-colors flex items-center gap-2 shadow-lg shadow-indigo-500/20"
              >
                {#if updatingHostlists}
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
                  </svg>
                {/if}
                Update All
              </button>
            </div>
          </div>

          <div class="space-y-4">
            {#if hostlistsLoading}
              <div class="flex items-center justify-center py-12">
                <svg class="w-8 h-8 animate-spin text-indigo-500" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              </div>
            {:else if hostlists.length === 0}
              <div class="text-center py-12 text-text-muted">
                <svg class="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                </svg>
                <p>No hostlists found</p>
              </div>
            {:else}
              {#each hostlists as hostlist}
                <div class="p-4 bg-void-100 rounded-xl border border-glass-border hover:border-indigo-500/30 transition-colors">
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <div class="flex items-center gap-3">
                        <h3 class="text-text-primary font-medium">{hostlist.name}</h3>
                        {#if hostlist.update_available}
                          <span class="px-2 py-0.5 bg-indigo-500/20 text-indigo-400 text-xs rounded-full">
                            Update available
                          </span>
                        {/if}
                        {#if hostlist.source_url}
                          <span class="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 text-xs rounded-full">
                            Auto-update
                          </span>
                        {/if}
                      </div>
                      <div class="flex items-center gap-4 mt-2 text-sm text-text-secondary">
                        <span class="flex items-center gap-1">
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                          </svg>
                          {hostlist.domain_count ?? 0} domains
                        </span>
                        <span class="flex items-center gap-1">
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"/>
                          </svg>
                          {formatSize(hostlist.size)}
                        </span>
                        <span class="flex items-center gap-1">
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                          </svg>
                          {formatDate(hostlist.last_updated)}
                        </span>
                      </div>
                    </div>
                    <div class="flex items-center gap-2">
                      {#if hostlist.source_url}
                        <button
                          onclick={() => updateSingleHostlist(hostlist.id)}
                          disabled={updatingHostlists}
                          class="p-2 text-text-muted hover:text-indigo-400 transition-colors disabled:opacity-50"
                          title="Update this hostlist"
                        >
                          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                          </svg>
                        </button>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            {/if}

            <!-- Info box -->
            <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20 mt-6">
              <p class="text-indigo-400 text-sm flex items-start gap-2">
                <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                <span>Hostlists contain domains that are processed by DPI bypass strategies. Updates are fetched from GitHub repositories with the latest domain lists.</span>
              </p>
            </div>
          </div>
        </div>
      {/if}

      <!-- Advanced Tab -->
      {#if activeTab === 'advanced'}
        <div>
          <h2 class="text-xl font-semibold text-text-primary mb-6">Advanced Settings</h2>
          
          <div class="space-y-6">
            <!-- Auto Recovery Settings Component -->
            <AutoRecoverySettings />

            <!-- Block QUIC -->
            <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
              <div>
                <p class="text-text-primary font-medium">Block QUIC</p>
                <p class="text-text-secondary text-sm">Block UDP/443 to force TCP connections</p>
              </div>
              <button
                aria-label="Toggle block QUIC"
                role="switch"
                aria-checked={localSettings.blockQuic}
                onclick={() => { localSettings.blockQuic = !localSettings.blockQuic; setSetting('blockQuic', localSettings.blockQuic); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.blockQuic ? 'bg-indigo-500' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.blockQuic ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- TCP Timestamps -->
            <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
              <div>
                <p class="text-text-primary font-medium">TCP Timestamps</p>
                <p class="text-text-secondary text-sm">Enable RFC 1323 timestamps (may help bypass some DPI)</p>
              </div>
              <button
                aria-label="Toggle TCP timestamps"
                role="switch"
                aria-checked={localSettings.tcpTimestamps}
                onclick={async () => {
                  const newValue = !localSettings.tcpTimestamps;
                  if (isTauri) {
                    try {
                      const { invoke } = await import('@tauri-apps/api/core');
                      await invoke('set_tcp_timestamps_enabled', { enabled: newValue });
                      localSettings.tcpTimestamps = newValue;
                    } catch (e) {
                      console.error('Failed to set TCP timestamps:', e);
                      saveMessage = { text: `Failed: ${e}`, type: 'error' };
                      setTimeout(() => { saveMessage = null; }, 5000);
                    }
                  } else {
                    localSettings.tcpTimestamps = newValue;
                  }
                }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.tcpTimestamps ? 'bg-indigo-500' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.tcpTimestamps ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- Debug Mode -->
            <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
              <div>
                <p class="text-text-primary font-medium">Debug Mode</p>
                <p class="text-text-secondary text-sm">Enable verbose logging for troubleshooting</p>
              </div>
              <button
                aria-label="Toggle debug mode"
                role="switch"
                aria-checked={localSettings.debugMode}
                onclick={() => { localSettings.debugMode = !localSettings.debugMode; setSetting('debugMode', localSettings.debugMode); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.debugMode ? 'bg-indigo-500' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.debugMode ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- Backup & Restore -->
            <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
              <div class="mb-4">
                <p class="text-text-primary font-medium flex items-center gap-2">
                  <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"/>
                  </svg>
                  Backup & Restore
                </p>
                <p class="text-text-secondary text-sm mt-1">Export or import your configuration (settings, proxies, routing rules)</p>
              </div>
              
              <div class="flex gap-3">
                <button
                  onclick={exportConfig}
                  disabled={exporting || !isTauri}
                  class="flex-1 px-4 py-2.5 bg-indigo-500/10 hover:bg-indigo-500/20 disabled:opacity-50 disabled:cursor-not-allowed text-indigo-400 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
                >
                  {#if exporting}
                    <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/>
                    </svg>
                  {/if}
                  Export Config
                </button>
                
                <button
                  onclick={importConfig}
                  disabled={importing || !isTauri}
                  class="flex-1 px-4 py-2.5 bg-emerald-500/10 hover:bg-emerald-500/20 disabled:opacity-50 disabled:cursor-not-allowed text-emerald-400 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
                >
                  {#if importing}
                    <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
                    </svg>
                  {/if}
                  Import Config
                </button>
              </div>
              
              <p class="text-text-muted text-xs mt-3">
                Note: Passwords are not exported for security reasons. You'll need to re-enter them after import.
              </p>
            </div>

            <!-- Danger Zone -->
            <details class="border border-red-500/20 rounded-xl overflow-hidden">
              <summary class="p-4 bg-red-500/5 text-red-400 cursor-pointer hover:bg-red-500/10 transition-colors flex items-center gap-2">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                </svg>
                <span class="font-medium">Danger Zone — Advanced Settings</span>
              </summary>
              
              <div class="p-4 space-y-4 bg-void-100">
                <p class="text-text-secondary text-sm mb-4">
                  ⚠️ These settings can affect system stability. Change only if you know what you're doing.
                </p>

                <!-- WinDivert Mode -->
                <div class="flex items-center justify-between p-4 bg-void-200 rounded-xl border border-glass-border">
                  <div>
                    <p class="text-text-primary font-medium">WinDivert Mode</p>
                    <p class="text-text-secondary text-sm">Driver operation mode</p>
                  </div>
                  <select 
                    bind:value={localSettings.windivertMode}
                    onchange={() => setSetting('windivertMode', localSettings.windivertMode)}
                    class="bg-void-200 text-text-primary rounded-lg px-4 py-2 border border-glass-border focus:border-red-500 focus:ring-1 focus:ring-red-500/20 focus:outline-none cursor-pointer"
                  >
                    <option value="normal">Normal</option>
                    <option value="autottl">Auto TTL</option>
                    <option value="autohostlist">Auto Hostlist</option>
                  </select>
                </div>

                <!-- Game Filter Mode -->
                <div class="flex items-center justify-between p-4 bg-void-200 rounded-xl border border-glass-border">
                  <div>
                    <p class="text-text-primary font-medium flex items-center gap-2">
                      🎮 Game Filter Mode
                    </p>
                    <p class="text-text-secondary text-sm">Port filtering for games (requires restart)</p>
                  </div>
                  <div class="flex items-center gap-3">
                    <button
                      onclick={async () => {
                        const newMode = localSettings.gameFilterMode === 'normal' ? 'gaming' : 'normal';
                        localSettings.gameFilterMode = newMode;
                        if (isTauri) {
                          const { invoke } = await import('@tauri-apps/api/core');
                          await invoke('set_game_filter_mode', { mode: newMode }).catch(console.error);
                        }
                      }}
                      class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.gameFilterMode === 'gaming' ? 'bg-emerald-500' : 'bg-void-300'}"
                      aria-label="Toggle game filter mode"
                      role="switch"
                      aria-checked={localSettings.gameFilterMode === 'gaming'}
                    >
                      <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.gameFilterMode === 'gaming' ? 'translate-x-6' : 'translate-x-0'}"></span>
                    </button>
                    <span class="text-xs font-mono px-2 py-1 rounded {localSettings.gameFilterMode === 'gaming' ? 'bg-emerald-500/20 text-emerald-400' : 'bg-void-300 text-text-muted'}">
                      {localSettings.gameFilterMode === 'gaming' ? '1024-65535' : '80,443'}
                    </span>
                  </div>
                </div>

                <!-- DNS Override -->
                <div class="flex items-center justify-between p-4 bg-void-200 rounded-xl border border-glass-border">
                  <div>
                    <p class="text-text-primary font-medium">DNS Override</p>
                    <p class="text-text-secondary text-sm">Custom DNS server (leave empty for system default)</p>
                  </div>
                  <input
                    type="text"
                    bind:value={localSettings.dnsOverride}
                    placeholder="8.8.8.8"
                    onchange={() => setSetting('dnsOverride', localSettings.dnsOverride)}
                    class="w-40 bg-void-200 text-text-primary rounded-lg px-4 py-2 border border-glass-border focus:border-red-500 focus:ring-1 focus:ring-red-500/20 focus:outline-none placeholder-text-muted"
                  />
                </div>

                <!-- Reset to Defaults -->
                <div class="pt-4 border-t border-red-500/20 flex flex-wrap gap-3">
                  <button
                    onclick={() => {
                      localSettings.windivertMode = 'normal';
                      localSettings.gameFilterMode = 'normal';
                      localSettings.dnsOverride = '';
                      localSettings.blockQuic = true;
                      localSettings.tcpTimestamps = false;
                      localSettings.debugMode = false;
                    }}
                    class="px-4 py-2 bg-red-500/10 hover:bg-red-500/20 text-red-400 rounded-lg text-sm font-medium transition-colors"
                  >
                    Reset Advanced Settings
                  </button>
                  <button
                    onclick={resetOnboarding}
                    class="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-text-secondary hover:text-text-primary rounded-lg text-sm font-medium transition-colors"
                  >
                    Reset Onboarding
                  </button>
                </div>
              </div>
            </details>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
