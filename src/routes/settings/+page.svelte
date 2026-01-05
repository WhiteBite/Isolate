<script lang="ts">
  import { browser } from '$app/environment';
  import { settings } from '$lib/stores';

  // Types
  type TabId = 'general' | 'routing' | 'advanced';
  type Theme = 'dark' | 'light' | 'system';
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
    dnsOverride: string;
    blockQuic: boolean;
    debugMode: boolean;
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
    dnsOverride: '',
    blockQuic: true,
    debugMode: false
  });

  let saving = $state(false);
  let saveMessage = $state<string | null>(null);
  let isTauri = $state(false);
  let newException = $state('');

  // Tabs configuration
  const tabs: { id: TabId; label: string; icon: string }[] = [
    { id: 'general', label: 'General', icon: 'settings' },
    { id: 'routing', label: 'Routing', icon: 'route' },
    { id: 'advanced', label: 'Advanced', icon: 'code' }
  ];

  // Initialize on component mount and navigation
  $effect(() => {
    if (!browser) return;
    
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;

    // Subscribe to settings store
    const unsubSettings = settings.subscribe(v => {
      localSettings.autoStart = v.autoStart;
      localSettings.minimizeToTray = v.minimizeToTray;
      localSettings.blockQuic = v.blockQuic;
    });

    if (isTauri) {
      loadSettings();
    }

    return () => {
      unsubSettings();
    };
  });

  async function loadSettings(retries = 10) {
    if (!browser || !isTauri) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Wait for backend to be ready with retry logic
      for (let i = 0; i < retries; i++) {
        try {
          const ready = await invoke<boolean>('is_backend_ready');
          if (!ready) {
            await new Promise(r => setTimeout(r, 200));
            continue;
          }
          
          const loaded = await invoke<Partial<AppSettings>>('get_settings').catch(() => ({}));
          if (loaded) {
            localSettings = { ...localSettings, ...loaded };
          }
          return;
        } catch {
          await new Promise(r => setTimeout(r, 200));
        }
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
        await invoke('save_settings', { settings: localSettings }).catch(() => {});
      }
      
      saveMessage = 'Settings saved';
      setTimeout(() => { saveMessage = null; }, 3000);
    } catch (e) {
      console.error('Failed to save settings:', e);
      saveMessage = 'Error saving settings';
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
</script>

<div class="p-8 min-h-screen bg-void">
  <!-- Header -->
  <div class="flex items-center justify-between mb-8">
    <div>
      <h1 class="text-3xl font-bold text-white">Settings</h1>
      <p class="text-text-secondary mt-1">Configure application preferences</p>
    </div>
    <div class="flex items-center gap-3">
      {#if saveMessage}
        <span class="text-neon-green text-sm animate-pulse">{saveMessage}</span>
      {/if}
      <button
        onclick={saveSettings}
        disabled={saving}
        class="px-5 py-2.5 bg-electric hover:bg-electric-dark disabled:opacity-50 text-white rounded-xl font-medium transition-all duration-200 flex items-center gap-2"
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
      <nav class="space-y-1">
        {#each tabs as tab}
          <button
            onclick={() => activeTab = tab.id}
            class="w-full flex items-center gap-3 px-4 py-3 text-left transition-all duration-200 {activeTab === tab.id ? 'bg-electric/10 border-l-2 border-electric text-electric' : 'text-text-secondary hover:bg-void-100 hover:text-white'}"
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
            {/if}
            <span class="font-medium">{tab.label}</span>
          </button>
        {/each}
      </nav>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 bg-void-50 rounded-2xl p-6 border border-glass-border">
      <!-- General Tab -->
      {#if activeTab === 'general'}
        <div>
          <h2 class="text-xl font-semibold text-white mb-6">General Settings</h2>
          
          <div class="space-y-4">
            <!-- Autostart Toggle -->
            <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
              <div>
                <p class="text-white font-medium">Autostart</p>
                <p class="text-text-secondary text-sm">Launch application on Windows startup</p>
              </div>
              <button
                onclick={() => { localSettings.autoStart = !localSettings.autoStart; setSetting('autoStart', localSettings.autoStart); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.autoStart ? 'bg-electric' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.autoStart ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- Language Dropdown -->
            <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
              <div>
                <p class="text-white font-medium">Language</p>
                <p class="text-text-secondary text-sm">Interface language</p>
              </div>
              <select 
                bind:value={localSettings.language}
                onchange={() => setSetting('language', localSettings.language)}
                class="bg-void-100 text-white rounded-lg px-4 py-2 border border-glass-border focus:border-electric focus:outline-none cursor-pointer"
              >
                <option value="ru">Русский</option>
                <option value="en">English</option>
              </select>
            </div>

            <!-- Theme Dropdown -->
            <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
              <div>
                <p class="text-white font-medium">Theme</p>
                <p class="text-text-secondary text-sm">Application appearance</p>
              </div>
              <select 
                bind:value={localSettings.theme}
                onchange={() => setSetting('theme', localSettings.theme)}
                class="bg-void-100 text-white rounded-lg px-4 py-2 border border-glass-border focus:border-electric focus:outline-none cursor-pointer"
              >
                <option value="dark">Dark</option>
                <option value="light">Light</option>
                <option value="system">System</option>
              </select>
            </div>

            <!-- Minimize to Tray Toggle -->
            <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
              <div>
                <p class="text-white font-medium">Minimize to tray</p>
                <p class="text-text-secondary text-sm">Minimize to system tray on close</p>
              </div>
              <button
                onclick={() => { localSettings.minimizeToTray = !localSettings.minimizeToTray; setSetting('minimizeToTray', localSettings.minimizeToTray); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.minimizeToTray ? 'bg-electric' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.minimizeToTray ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- Notifications Toggle -->
            <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
              <div>
                <p class="text-white font-medium">Notifications</p>
                <p class="text-text-secondary text-sm">Show system notifications</p>
              </div>
              <button
                onclick={() => { localSettings.notifications = !localSettings.notifications; setSetting('notifications', localSettings.notifications); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.notifications ? 'bg-electric' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.notifications ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Routing Tab -->
      {#if activeTab === 'routing'}
        <div>
          <h2 class="text-xl font-semibold text-white mb-6">Routing Settings</h2>
          
          <div class="space-y-6">
            <!-- Domain Exceptions -->
            <div class="p-4 bg-void-100/50 rounded-xl">
              <div class="mb-4">
                <p class="text-white font-medium">Domain Exceptions</p>
                <p class="text-text-secondary text-sm">Domains that bypass DPI circumvention</p>
              </div>
              
              <!-- Add new exception -->
              <div class="flex gap-2 mb-4">
                <input
                  type="text"
                  bind:value={newException}
                  placeholder="example.com"
                  onkeydown={(e) => e.key === 'Enter' && addException()}
                  class="flex-1 bg-void-100 text-white rounded-lg px-4 py-2 border border-glass-border focus:border-electric focus:outline-none placeholder-text-muted"
                />
                <button
                  onclick={addException}
                  class="px-4 py-2 bg-electric hover:bg-electric-dark text-white rounded-lg font-medium transition-colors"
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
                    <div class="flex items-center justify-between p-2 bg-void-50 rounded-lg">
                      <span class="text-white text-sm font-mono">{domain}</span>
                      <button
                        onclick={() => removeException(domain)}
                        class="text-neon-red hover:text-neon-red/80 transition-colors"
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
            <div class="p-4 bg-void-100/30 rounded-xl border border-dashed border-glass-border">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-text-muted font-medium">Per-App Routing</p>
                  <p class="text-text-muted/70 text-sm">Route specific applications through DPI bypass</p>
                </div>
                <span class="px-3 py-1 bg-void-200 text-text-muted text-xs rounded-full">Coming Soon</span>
              </div>
            </div>
          </div>
        </div>
      {/if}

      <!-- Advanced Tab -->
      {#if activeTab === 'advanced'}
        <div>
          <h2 class="text-xl font-semibold text-white mb-6">Advanced Settings</h2>
          
          <div class="space-y-6">
            <!-- Block QUIC -->
            <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
              <div>
                <p class="text-white font-medium">Block QUIC</p>
                <p class="text-text-secondary text-sm">Block UDP/443 to force TCP connections</p>
              </div>
              <button
                onclick={() => { localSettings.blockQuic = !localSettings.blockQuic; setSetting('blockQuic', localSettings.blockQuic); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.blockQuic ? 'bg-neon-green' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.blockQuic ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- Debug Mode -->
            <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
              <div>
                <p class="text-white font-medium">Debug Mode</p>
                <p class="text-text-secondary text-sm">Enable verbose logging for troubleshooting</p>
              </div>
              <button
                onclick={() => { localSettings.debugMode = !localSettings.debugMode; setSetting('debugMode', localSettings.debugMode); }}
                class="relative w-12 h-6 rounded-full transition-colors duration-200 {localSettings.debugMode ? 'bg-neon-yellow' : 'bg-void-200'}"
              >
                <span class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {localSettings.debugMode ? 'translate-x-6' : 'translate-x-0'}"></span>
              </button>
            </div>

            <!-- Danger Zone -->
            <details class="border border-neon-red/20 rounded-xl overflow-hidden">
              <summary class="p-4 bg-neon-red/5 text-neon-red cursor-pointer hover:bg-neon-red/10 transition-colors flex items-center gap-2">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                </svg>
                <span class="font-medium">Danger Zone — Advanced Settings</span>
              </summary>
              
              <div class="p-4 space-y-4 bg-void-50">
                <p class="text-text-secondary text-sm mb-4">
                  ⚠️ These settings can affect system stability. Change only if you know what you're doing.
                </p>

                <!-- WinDivert Mode -->
                <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
                  <div>
                    <p class="text-white font-medium">WinDivert Mode</p>
                    <p class="text-text-secondary text-sm">Driver operation mode</p>
                  </div>
                  <select 
                    bind:value={localSettings.windivertMode}
                    onchange={() => setSetting('windivertMode', localSettings.windivertMode)}
                    class="bg-void-100 text-white rounded-lg px-4 py-2 border border-glass-border focus:border-neon-red focus:outline-none cursor-pointer"
                  >
                    <option value="normal">Normal</option>
                    <option value="autottl">Auto TTL</option>
                    <option value="autohostlist">Auto Hostlist</option>
                  </select>
                </div>

                <!-- DNS Override -->
                <div class="flex items-center justify-between p-4 bg-void-100/50 rounded-xl">
                  <div>
                    <p class="text-white font-medium">DNS Override</p>
                    <p class="text-text-secondary text-sm">Custom DNS server (leave empty for system default)</p>
                  </div>
                  <input
                    type="text"
                    bind:value={localSettings.dnsOverride}
                    placeholder="8.8.8.8"
                    onchange={() => setSetting('dnsOverride', localSettings.dnsOverride)}
                    class="w-40 bg-void-100 text-white rounded-lg px-4 py-2 border border-glass-border focus:border-neon-red focus:outline-none placeholder-text-muted"
                  />
                </div>

                <!-- Reset to Defaults -->
                <div class="pt-4 border-t border-neon-red/20">
                  <button
                    onclick={() => {
                      localSettings.windivertMode = 'normal';
                      localSettings.dnsOverride = '';
                      localSettings.blockQuic = true;
                      localSettings.debugMode = false;
                    }}
                    class="px-4 py-2 bg-neon-red/10 hover:bg-neon-red/20 text-neon-red rounded-lg text-sm font-medium transition-colors"
                  >
                    Reset Advanced Settings
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
