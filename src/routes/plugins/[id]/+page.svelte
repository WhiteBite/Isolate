<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { installedPlugins, loadPluginsFromBackend, togglePlugin, uninstallPlugin, updatePluginSettings, type PluginInfo, type PluginSetting } from '$lib/stores/plugins';
  import { toasts } from '$lib/stores/toast';

  // Get plugin from store
  let allPlugins = $state<PluginInfo[]>([]);
  
  // Subscribe to store
  installedPlugins.subscribe(value => {
    allPlugins = value;
  });

  // Get plugin ID from route params
  let pluginId = $derived($page.params.id);
  let plugin = $derived(allPlugins.find(p => p.id === pluginId) || null);
  
  // Widget type detection based on plugin type/id
  let hasWidget = $derived(plugin?.type === 'ui-plugin');
  let widgetType = $derived.by(() => {
    if (!plugin) return null;
    if (plugin.id.includes('speed')) return 'speed-test';
    if (plugin.id.includes('latency')) return 'latency-monitor';
    return null;
  });
  
  // Load plugins from backend on mount
  let loading = $state(true);
  
  $effect(() => {
    loadPluginsFromBackend().then(plugins => {
      // Always set plugins from backend (empty array if none available)
      installedPlugins.set(plugins);
    }).catch(console.warn).finally(() => {
      loading = false;
    });
  });

  // Dynamic plugin settings state
  let pluginSettings = $state<PluginSetting[]>([]);
  let settingsLoading = $state(true);
  let settingsSaving = $state(false);
  
  // Load plugin settings when plugin changes
  $effect(() => {
    if (plugin) {
      loadPluginSettings();
    }
  });
  
  async function loadPluginSettings() {
    if (!plugin) return;
    
    settingsLoading = true;
    try {
      const ready = await invoke<boolean>('is_backend_ready');
      if (!ready) {
        // Use defaults from plugin
        pluginSettings = plugin.settings ? JSON.parse(JSON.stringify(plugin.settings)) : [];
        return;
      }
      
      // Get saved settings from backend
      const savedSettings = await invoke<Array<{id: string, value: any}>>('get_plugin_settings', {
        pluginId: plugin.id
      });
      
      // Merge saved values with plugin defaults
      if (plugin.settings && savedSettings.length > 0) {
        const savedMap = new Map(savedSettings.map(s => [s.id, s.value]));
        pluginSettings = plugin.settings.map(setting => ({
          ...setting,
          value: savedMap.has(setting.id) ? savedMap.get(setting.id) : setting.value
        }));
      } else {
        pluginSettings = plugin.settings ? JSON.parse(JSON.stringify(plugin.settings)) : [];
      }
    } catch (error) {
      console.warn('Failed to load plugin settings:', error);
      // Fallback to defaults
      pluginSettings = plugin.settings ? JSON.parse(JSON.stringify(plugin.settings)) : [];
    } finally {
      settingsLoading = false;
    }
  }
  
  async function updateSetting(id: string, value: any) {
    // Update local state
    pluginSettings = pluginSettings.map(s => 
      s.id === id ? { ...s, value } : s
    );
    
    // Save to backend
    await saveSettings();
  }
  
  async function saveSettings() {
    if (!plugin || settingsSaving) return;
    
    settingsSaving = true;
    try {
      const ready = await invoke<boolean>('is_backend_ready');
      if (ready) {
        const settingsToSave = pluginSettings.map(s => ({
          id: s.id,
          value: s.value
        }));
        
        await invoke('set_plugin_settings', {
          pluginId: plugin.id,
          settings: settingsToSave
        });
      }
      
      // Update store
      updatePluginSettings(plugin.id, pluginSettings);
    } catch (error) {
      console.error('Failed to save plugin settings:', error);
      toasts.error(`Failed to save settings: ${error}`);
    } finally {
      settingsSaving = false;
    }
  }
  
  async function resetSettings() {
    if (!plugin) return;
    
    try {
      const ready = await invoke<boolean>('is_backend_ready');
      if (ready) {
        await invoke('reset_plugin_settings', {
          pluginId: plugin.id
        });
      }
      
      // Reset to plugin defaults
      if (plugin.settings) {
        pluginSettings = JSON.parse(JSON.stringify(plugin.settings));
        updatePluginSettings(plugin.id, pluginSettings);
        toasts.success('Settings reset to defaults');
      }
    } catch (error) {
      console.error('Failed to reset plugin settings:', error);
      toasts.error(`Failed to reset settings: ${error}`);
    }
  }

  // Speed Test Widget State
  let isTesting = $state(false);
  let downloadSpeed = $state<number | null>(null);
  let uploadSpeed = $state<number | null>(null);
  let testProgress = $state(0);
  let testPhase = $state<'idle' | 'download' | 'upload' | 'done'>('idle');

  // Latency Monitor State
  let isMonitoring = $state(false);
  let latencyValues = $state<number[]>([]);
  let currentLatency = $state<number | null>(null);
  let latencyError = $state(false);
  
  // Target selection
  type LatencyTarget = { name: string; url: string; host: string };
  const latencyTargets: LatencyTarget[] = [
    { name: 'Cloudflare', url: 'https://1.1.1.1/cdn-cgi/trace', host: '1.1.1.1' },
    { name: 'Google', url: 'https://8.8.8.8', host: '8.8.8.8' },
    { name: 'Yandex', url: 'https://77.88.8.8', host: '77.88.8.8' },
  ];
  let selectedTargetIndex = $state(0);
  let selectedTarget = $derived(latencyTargets[selectedTargetIndex]);
  
  // Filter out error values (-1) for stats
  let validLatencyValues = $derived(latencyValues.filter(v => v >= 0));
  let minLatency = $derived(validLatencyValues.length > 0 ? Math.min(...validLatencyValues) : null);
  let maxLatency = $derived(validLatencyValues.length > 0 ? Math.max(...validLatencyValues) : null);
  let avgLatency = $derived(validLatencyValues.length > 0 ? Math.round(validLatencyValues.reduce((a, b) => a + b, 0) / validLatencyValues.length) : null);
  
  // Latency quality indicator
  let latencyQuality = $derived.by(() => {
    if (currentLatency === null) return null;
    if (currentLatency < 0 || latencyError) return 'error';
    if (currentLatency < 50) return 'good';
    if (currentLatency <= 100) return 'medium';
    return 'bad';
  });
  
  let latencyColor = $derived.by(() => {
    switch (latencyQuality) {
      case 'good': return 'text-emerald-400';
      case 'medium': return 'text-amber-400';
      case 'bad': return 'text-red-400';
      case 'error': return 'text-red-500';
      default: return 'text-indigo-400';
    }
  });

  function handleBack() {
    goto('/plugins');
  }

  function handleToggleEnabled() {
    if (plugin && pluginId) {
      togglePlugin(pluginId);
    }
  }

  function handleDelete() {
    if (plugin && pluginId) {
      uninstallPlugin(pluginId);
    }
    goto('/plugins');
  }

  // Speed Test Functions
  let speedTestError = $state<string | null>(null);
  
  const DOWNLOAD_SIZE = 10_000_000; // 10MB
  const CLOUDFLARE_DOWN_URL = `https://speed.cloudflare.com/__down?bytes=${DOWNLOAD_SIZE}`;

  async function testDownloadSpeed(): Promise<number> {
    const startTime = performance.now();
    let loadedBytes = 0;
    
    const response = await fetch(CLOUDFLARE_DOWN_URL, {
      method: 'GET',
      cache: 'no-store',
    });
    
    if (!response.ok) {
      throw new Error(`Download failed: ${response.status} ${response.statusText}`);
    }
    
    const reader = response.body?.getReader();
    if (!reader) {
      throw new Error('ReadableStream not supported');
    }
    
    const contentLength = parseInt(response.headers.get('content-length') || String(DOWNLOAD_SIZE));
    
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      
      loadedBytes += value.length;
      // Update progress (0-50% for download)
      testProgress = Math.round((loadedBytes / contentLength) * 50);
    }
    
    const endTime = performance.now();
    const durationSeconds = (endTime - startTime) / 1000;
    
    // Calculate speed: (bytes * 8 bits) / seconds / 1_000_000 = Mbps
    const speedMbps = (loadedBytes * 8) / durationSeconds / 1_000_000;
    return speedMbps;
  }

  async function testUploadSpeed(): Promise<number> {
    // Use Tauri backend to bypass CORS
    const result = await invoke<{ speed_mbps: number; bytes_sent: number; duration_ms: number }>('test_upload_speed');
    
    // Update progress (50-100% for upload) - simulate progress since backend doesn't stream it
    for (let i = 50; i <= 100; i += 10) {
      testProgress = i;
      await new Promise(r => setTimeout(r, 50));
    }
    
    return result.speed_mbps;
  }

  async function runSpeedTest() {
    if (isTesting) return;
    
    isTesting = true;
    testProgress = 0;
    downloadSpeed = null;
    uploadSpeed = null;
    speedTestError = null;
    testPhase = 'download';
    
    try {
      // Download test
      const downSpeed = await testDownloadSpeed();
      downloadSpeed = Math.round(downSpeed * 10) / 10; // Round to 1 decimal
      
      // Upload test
      testPhase = 'upload';
      const upSpeed = await testUploadSpeed();
      uploadSpeed = Math.round(upSpeed * 10) / 10; // Round to 1 decimal
      
      testPhase = 'done';
      testProgress = 100;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      speedTestError = errorMessage;
      testPhase = 'idle';
      testProgress = 0;
      console.error('Speed test failed:', error);
    } finally {
      isTesting = false;
    }
  }

  // Latency Monitor Functions
  let latencyInterval: ReturnType<typeof setInterval> | null = null;

  async function measureLatency(): Promise<number> {
    const start = performance.now();
    try {
      await fetch(selectedTarget.url, { 
        method: 'HEAD',
        mode: 'no-cors',
        cache: 'no-store'
      });
      return Math.round(performance.now() - start);
    } catch {
      return -1; // Error
    }
  }

  function toggleMonitoring() {
    if (isMonitoring) {
      stopMonitoring();
    } else {
      startMonitoring();
    }
  }

  async function startMonitoring() {
    isMonitoring = true;
    latencyValues = [];
    latencyError = false;
    currentLatency = null;
    
    // Immediate first measurement
    const firstLatency = await measureLatency();
    if (!isMonitoring) return; // Check if stopped during measurement
    
    currentLatency = firstLatency;
    latencyError = firstLatency < 0;
    latencyValues = [firstLatency];
    
    // Continue with interval
    latencyInterval = setInterval(async () => {
      const latency = await measureLatency();
      if (!isMonitoring) return; // Check if stopped during measurement
      
      currentLatency = latency;
      latencyError = latency < 0;
      latencyValues = [...latencyValues.slice(-29), latency];
    }, 1000);
  }

  function stopMonitoring() {
    isMonitoring = false;
    if (latencyInterval) {
      clearInterval(latencyInterval);
      latencyInterval = null;
    }
  }
  
  function selectTarget(index: number) {
    if (isMonitoring) {
      stopMonitoring();
    }
    selectedTargetIndex = index;
    latencyValues = [];
    currentLatency = null;
    latencyError = false;
  }

  function sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  function formatSpeed(speed: number | null): string {
    if (speed === null) return '--';
    return speed.toFixed(0);
  }
</script>

<div class="h-full bg-void overflow-y-auto">
  {#if plugin}
    <div class="p-6 max-w-3xl mx-auto">
      <!-- Header with Back Button -->
      <div class="flex items-center gap-4 mb-6">
        <button
          aria-label="Go back"
          onclick={handleBack}
          class="w-10 h-10 flex items-center justify-center rounded-lg bg-void-50 border border-glass-border
                 text-text-secondary hover:text-text-primary hover:bg-void-100 transition-colors"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5"/><path d="M12 19l-7-7 7-7"/>
          </svg>
        </button>
        <h1 class="text-2xl font-semibold text-text-primary">{plugin.name}</h1>
      </div>

      <!-- Plugin Info Card -->
      <div class="bg-void-50 border border-glass-border rounded-xl p-5 mb-6">
        <div class="flex items-start gap-4">
          <div class="w-14 h-14 rounded-xl bg-void-100 border border-glass-border flex items-center justify-center text-2xl flex-shrink-0">
            {plugin.icon || '🔧'}
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-3 mb-2">
              <span class="px-2.5 py-1 rounded-full text-xs font-medium
                {plugin.enabled 
                  ? 'bg-neon-green/10 text-neon-green border border-neon-green/30' 
                  : 'bg-void-200 text-text-muted border border-glass-border'}">
                {plugin.enabled ? '✓ Active' : 'Disabled'}
              </span>
            </div>
            <p class="text-text-secondary text-sm mb-3">{plugin.description}</p>
            <div class="flex items-center gap-4 text-xs text-text-muted">
              <span>Version: <span class="text-text-secondary">{plugin.version}</span></span>
              <span>Author: <span class="text-text-secondary">{plugin.author}</span></span>
            </div>
          </div>
        </div>
      </div>

      <!-- Speed Test Widget -->
      {#if hasWidget && plugin.enabled && widgetType === 'speed-test'}
        <div class="bg-void-50 border border-glass-border rounded-xl p-5 mb-6">
          <h2 class="text-sm font-medium text-text-primary mb-4">Speed Test</h2>
          
          <div class="flex items-center justify-between gap-6">
            <!-- Download -->
            <div class="flex-1 text-center">
              <div class="text-[10px] uppercase tracking-wider text-zinc-400 mb-1">Download</div>
              <div class="flex items-baseline justify-center gap-1">
                <span class="text-3xl font-bold text-cyan-400 tabular-nums" class:animate-pulse={testPhase === 'download'}>
                  {formatSpeed(downloadSpeed)}
                </span>
                <span class="text-xs text-zinc-400">Mbps</span>
              </div>
            </div>
            
            <!-- Test Button -->
            <button
              onclick={runSpeedTest}
              disabled={isTesting}
              class="relative w-20 h-20 rounded-full bg-gradient-to-br from-cyan-500/20 to-purple-500/20
                     border border-white/10 flex items-center justify-center transition-all
                     hover:border-cyan-400/50 hover:shadow-lg hover:shadow-cyan-500/20
                     disabled:opacity-50 disabled:cursor-not-allowed group"
            >
              {#if isTesting}
                <svg class="absolute inset-0 w-full h-full -rotate-90" viewBox="0 0 80 80">
                  <circle cx="40" cy="40" r="36" fill="none" stroke="rgba(34, 211, 238, 0.2)" stroke-width="3"/>
                  <circle cx="40" cy="40" r="36" fill="none" stroke="rgb(34, 211, 238)" stroke-width="3"
                    stroke-linecap="round"
                    stroke-dasharray={2 * Math.PI * 36}
                    stroke-dashoffset={2 * Math.PI * 36 * (1 - testProgress / 100)}
                    class="transition-all duration-200"/>
                </svg>
                <span class="text-2xl">⚡</span>
              {:else}
                <span class="text-3xl group-hover:scale-110 transition-transform">⚡</span>
              {/if}
            </button>
            
            <!-- Upload -->
            <div class="flex-1 text-center">
              <div class="text-[10px] uppercase tracking-wider text-zinc-400 mb-1">Upload</div>
              <div class="flex items-baseline justify-center gap-1">
                <span class="text-3xl font-bold text-purple-400 tabular-nums" class:animate-pulse={testPhase === 'upload'}>
                  {formatSpeed(uploadSpeed)}
                </span>
                <span class="text-xs text-zinc-400">Mbps</span>
              </div>
            </div>
          </div>
          
          <div class="text-center text-xs text-zinc-400 mt-4">
            {#if speedTestError}
              <span class="text-red-400">⚠️ {speedTestError}</span>
            {:else}
              {testPhase === 'idle' ? 'Click to start test' :
               testPhase === 'download' ? 'Testing download...' :
               testPhase === 'upload' ? 'Testing upload...' : 'Test complete'}
            {/if}
          </div>
        </div>
      {/if}

      <!-- Latency Monitor Widget -->
      {#if hasWidget && plugin.enabled && widgetType === 'latency-monitor'}
        <div class="bg-void-50 border border-glass-border rounded-xl p-5 mb-6">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-sm font-medium text-text-primary">Latency Monitor</h2>
            <button
              onclick={toggleMonitoring}
              class="px-3 py-1.5 rounded-lg text-xs font-medium transition-all
                     {isMonitoring 
                       ? 'bg-red-500/20 text-red-400 border border-red-500/30' 
                       : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'}"
            >
              {isMonitoring ? 'Stop' : 'Start'}
            </button>
          </div>
          
          <!-- Target Selection -->
          <div class="flex items-center gap-2 mb-4">
            <span class="text-[10px] uppercase tracking-wider text-zinc-400">Target:</span>
            <div class="flex gap-1">
              {#each latencyTargets as target, index}
                <button
                  onclick={() => selectTarget(index)}
                  class="px-2 py-1 rounded text-xs transition-all
                         {selectedTargetIndex === index 
                           ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' 
                           : 'bg-void-100 text-text-muted border border-glass-border hover:text-text-secondary'}"
                >
                  {target.name}
                </button>
              {/each}
            </div>
          </div>
          
          <!-- Target Host Display -->
          <div class="text-center mb-2">
            <span class="text-[10px] uppercase tracking-wider text-zinc-400">
              Pinging: <span class="text-text-secondary font-mono">{selectedTarget.host}</span>
            </span>
          </div>
          
          <!-- Current Latency with Quality Indicator -->
          <div class="text-center mb-4">
            <div class="text-4xl font-bold tabular-nums {latencyColor}">
              {#if currentLatency === null}
                --
              {:else if currentLatency < 0}
                Error
              {:else}
                {currentLatency}
              {/if}
              {#if currentLatency === null || currentLatency >= 0}
                <span class="text-lg text-zinc-400">ms</span>
              {/if}
            </div>
            <!-- Quality Badge -->
            {#if latencyQuality && latencyQuality !== 'error'}
              <div class="mt-1">
                <span class="px-2 py-0.5 rounded-full text-[10px] font-medium
                  {latencyQuality === 'good' ? 'bg-emerald-500/20 text-emerald-400' : ''}
                  {latencyQuality === 'medium' ? 'bg-amber-500/20 text-amber-400' : ''}
                  {latencyQuality === 'bad' ? 'bg-red-500/20 text-red-400' : ''}">
                  {latencyQuality === 'good' ? '● Excellent' : ''}
                  {latencyQuality === 'medium' ? '● Good' : ''}
                  {latencyQuality === 'bad' ? '● Poor' : ''}
                </span>
              </div>
            {/if}
            {#if latencyQuality === 'error'}
              <div class="mt-1">
                <span class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-red-500/20 text-red-400">
                  ● Connection Failed
                </span>
              </div>
            {/if}
          </div>
          
          <!-- Stats -->
          <div class="grid grid-cols-3 gap-4 text-center">
            <div>
              <div class="text-[10px] uppercase tracking-wider text-zinc-400 mb-1">Min</div>
              <div class="text-lg font-semibold text-emerald-400 tabular-nums">{minLatency ?? '--'}</div>
            </div>
            <div>
              <div class="text-[10px] uppercase tracking-wider text-zinc-400 mb-1">Avg</div>
              <div class="text-lg font-semibold text-amber-400 tabular-nums">{avgLatency ?? '--'}</div>
            </div>
            <div>
              <div class="text-[10px] uppercase tracking-wider text-zinc-400 mb-1">Max</div>
              <div class="text-lg font-semibold text-red-400 tabular-nums">{maxLatency ?? '--'}</div>
            </div>
          </div>
          
          <!-- Simple bar chart with color coding -->
          {#if latencyValues.length > 0}
            <div class="flex items-end gap-0.5 h-16 mt-4">
              {#each latencyValues as val}
                {@const barColor = val < 0 ? 'bg-red-500/70' : val < 50 ? 'bg-emerald-500/50' : val <= 100 ? 'bg-amber-500/50' : 'bg-red-500/50'}
                {@const barHeight = val < 0 ? 100 : Math.min(100, (val / 150) * 100)}
                <div 
                  class="flex-1 rounded-t transition-all duration-200 {barColor}"
                  style="height: {barHeight}%"
                ></div>
              {/each}
            </div>
            <div class="flex justify-between text-[9px] text-zinc-600 mt-1">
              <span>30s ago</span>
              <span>now</span>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Settings Card -->
      <div class="bg-void-50 border border-glass-border rounded-xl p-5 mb-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-sm font-medium text-text-primary">Plugin Settings</h2>
          {#if pluginSettings.length > 0}
            <button
              onclick={resetSettings}
              class="text-xs text-text-muted hover:text-text-secondary transition-colors"
              title="Reset to default values"
            >
              Reset
            </button>
          {/if}
        </div>
        
        {#if settingsLoading}
          <div class="flex items-center justify-center py-6">
            <div class="w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
          </div>
        {:else if pluginSettings.length === 0}
          <div class="text-center py-6">
            <div class="text-2xl mb-2">⚙️</div>
            <p class="text-sm text-text-muted">No settings available</p>
          </div>
        {:else}
          <div class="space-y-4">
            {#each pluginSettings as setting (setting.id)}
              {#if setting.type === 'toggle'}
                <!-- Toggle Setting -->
                <label class="flex items-center justify-between cursor-pointer group">
                  <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-void-100 border border-glass-border flex items-center justify-center text-text-muted">
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
                      </svg>
                    </div>
                    <div>
                      <span class="text-sm text-text-secondary">{setting.label}</span>
                      {#if setting.description}
                        <p class="text-xs text-text-muted mt-0.5">{setting.description}</p>
                      {/if}
                    </div>
                  </div>
                  <button 
                    aria-label="Toggle {setting.label}"
                    role="switch"
                    aria-checked={Boolean(setting.value)}
                    onclick={() => updateSetting(setting.id, !setting.value)}
                    disabled={settingsSaving}
                    class="relative w-11 h-6 rounded-full transition-colors {setting.value ? 'bg-indigo-500' : 'bg-void-200'}
                           disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform {setting.value ? 'translate-x-5' : ''}"></span>
                  </button>
                </label>
                
              {:else if setting.type === 'select'}
                <!-- Select Setting -->
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-void-100 border border-glass-border flex items-center justify-center text-text-muted">
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M6 9l6 6 6-6"/>
                      </svg>
                    </div>
                    <div>
                      <span class="text-sm text-text-secondary">{setting.label}</span>
                      {#if setting.description}
                        <p class="text-xs text-text-muted mt-0.5">{setting.description}</p>
                      {/if}
                    </div>
                  </div>
                  <select
                    value={setting.value}
                    onchange={(e) => updateSetting(setting.id, e.currentTarget.value)}
                    disabled={settingsSaving}
                    class="px-3 py-1.5 bg-void-100 border border-glass-border rounded-lg text-sm text-text-secondary
                           focus:outline-none focus:border-indigo-500/50 cursor-pointer
                           disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {#each setting.options || [] as option (option.value)}
                      <option value={option.value}>{option.label}</option>
                    {/each}
                  </select>
                </div>
                
              {:else if setting.type === 'number'}
                <!-- Number Setting -->
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-lg bg-void-100 border border-glass-border flex items-center justify-center text-text-muted">
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M4 7h16M4 12h16M4 17h10"/>
                      </svg>
                    </div>
                    <div>
                      <span class="text-sm text-text-secondary">{setting.label}</span>
                      {#if setting.description}
                        <p class="text-xs text-text-muted mt-0.5">{setting.description}</p>
                      {/if}
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <input
                      type="number"
                      value={setting.value}
                      min={setting.min}
                      max={setting.max}
                      onchange={(e) => {
                        const val = parseInt(e.currentTarget.value);
                        if (!isNaN(val)) {
                          const clamped = Math.min(
                            Math.max(val, setting.min ?? -Infinity),
                            setting.max ?? Infinity
                          );
                          updateSetting(setting.id, clamped);
                        }
                      }}
                      disabled={settingsSaving}
                      class="w-20 px-2 py-1.5 bg-void-100 border border-glass-border rounded-lg text-sm text-text-secondary text-center
                             focus:outline-none focus:border-indigo-500/50
                             disabled:opacity-50 disabled:cursor-not-allowed"
                    />
                    {#if setting.min !== undefined || setting.max !== undefined}
                      <span class="text-xs text-text-muted">
                        ({setting.min ?? '∞'}-{setting.max ?? '∞'})
                      </span>
                    {/if}
                  </div>
                </div>
                
              {:else if setting.type === 'text'}
                <!-- Text Setting -->
                <div>
                  <div class="flex items-center gap-3 mb-2">
                    <div class="w-8 h-8 rounded-lg bg-void-100 border border-glass-border flex items-center justify-center text-text-muted">
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/>
                      </svg>
                    </div>
                    <div>
                      <span class="text-sm text-text-secondary">{setting.label}</span>
                      {#if setting.description}
                        <p class="text-xs text-text-muted mt-0.5">{setting.description}</p>
                      {/if}
                    </div>
                  </div>
                  <input
                    type="text"
                    value={setting.value}
                    placeholder={setting.placeholder}
                    onchange={(e) => updateSetting(setting.id, e.currentTarget.value)}
                    disabled={settingsSaving}
                    class="w-full px-3 py-2 bg-void-100 border border-glass-border rounded-lg text-sm text-text-secondary
                           placeholder-text-muted focus:outline-none focus:border-indigo-500/50
                           disabled:opacity-50 disabled:cursor-not-allowed"
                  />
                </div>
              {/if}
            {/each}
          </div>
        {/if}
        
        {#if settingsSaving}
          <div class="flex items-center justify-center gap-2 mt-4 text-xs text-text-muted">
            <div class="w-3 h-3 border border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
            Saving...
          </div>
        {/if}
      </div>

      <!-- Action Buttons -->
      <div class="flex gap-3">
        <button onclick={handleToggleEnabled}
          class="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-void-100 border border-glass-border rounded-xl
                 text-text-secondary font-medium text-sm hover:bg-void-200 transition-colors">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/>
          </svg>
          {plugin.enabled ? 'Disable' : 'Enable'}
        </button>
        <button onclick={handleDelete}
          class="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-red-500/10 border border-red-500/30 rounded-xl
                 text-red-400 font-medium text-sm hover:bg-red-500/20 transition-colors">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          Delete
        </button>
      </div>
    </div>
  {:else if loading}
    <div class="h-full flex flex-col items-center justify-center text-center p-6">
      <div class="w-20 h-20 rounded-2xl bg-void-50 border border-glass-border flex items-center justify-center mb-4">
        <svg class="w-10 h-10 text-text-muted animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10" stroke-dasharray="32" stroke-dashoffset="8"/>
        </svg>
      </div>
      <h2 class="text-xl font-semibold text-text-primary mb-2">Loading...</h2>
      <p class="text-sm text-text-muted max-w-xs">Loading plugin information</p>
    </div>
  {:else}
    <div class="h-full flex flex-col items-center justify-center text-center p-6">
      <div class="w-20 h-20 rounded-2xl bg-void-50 border border-glass-border flex items-center justify-center mb-4">
        <svg class="w-10 h-10 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/><path d="M12 8v4"/><path d="M12 16h.01"/>
        </svg>
      </div>
      <h2 class="text-xl font-semibold text-text-primary mb-2">Plugin Not Found</h2>
      <p class="text-sm text-text-muted max-w-xs mb-6">Plugin "{pluginId}" is not installed.</p>
      <button onclick={handleBack}
        class="flex items-center gap-2 px-4 py-2 bg-indigo-500/10 border border-indigo-500/30 rounded-lg text-indigo-400 font-medium text-sm">
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 12H5"/><path d="M12 19l-7-7 7-7"/>
        </svg>
        Back to Plugins
      </button>
    </div>
  {/if}
</div>

<style>
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  .animate-pulse { animation: pulse 1s ease-in-out infinite; }
</style>
