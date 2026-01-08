<script lang="ts">
  /**
   * IpsetSettings Component
   * 
   * UI for managing ipset configuration, modes, and updates.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { browser } from '$app/environment';
  import Toggle from '$lib/components/Toggle.svelte';
  import Button from '$lib/components/Button.svelte';

  // Types matching backend IpsetStats
  type IpsetMode = 'any' | 'none' | 'loaded';

  interface IpsetStats {
    mode: IpsetMode;
    ip_count: number;
    ipv4_count: number;
    ipv6_count: number;
    cidr_count: number;
    file_size: number | null;
    last_updated: string | null;
    source_url: string | null;
    file_exists: boolean;
    auto_update: boolean;
  }

  interface IpsetUpdateResult {
    success: boolean;
    ip_count: number;
    ipv4_count: number;
    ipv6_count: number;
    cidr_count: number;
    source_url: string;
    error: string | null;
    timestamp: string;
  }

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let stats = $state<IpsetStats | null>(null);
  let loading = $state(true);
  let updating = $state(false);
  let changingMode = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' | 'warning' } | null>(null);
  let isTauri = $state(false);

  // Mode descriptions
  const modeDescriptions: Record<IpsetMode, { title: string; description: string }> = {
    any: {
      title: 'Any (Disabled)',
      description: 'IP filtering disabled. All traffic passes through without ipset checks.'
    },
    none: {
      title: 'None (Block All)',
      description: 'Block all traffic that would match ipset rules. Use for testing.'
    },
    loaded: {
      title: 'Loaded (Active)',
      description: 'Use loaded ipset for filtering. Requires ipset file to exist.'
    }
  };

  // Derived
  let formattedDate = $derived(
    stats?.last_updated 
      ? new Date(stats.last_updated).toLocaleString() 
      : 'Never'
  );

  let formattedSize = $derived(() => {
    if (!stats?.file_size) return '-';
    const bytes = stats.file_size;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  });

  let canUseLoadedMode = $derived(stats?.file_exists ?? false);

  // Demo data for non-Tauri environment
  const demoStats: IpsetStats = {
    mode: 'loaded',
    ip_count: 15234,
    ipv4_count: 12456,
    ipv6_count: 2778,
    cidr_count: 342,
    file_size: 245760,
    last_updated: new Date().toISOString(),
    source_url: 'https://github.com/example/ipset',
    file_exists: true,
    auto_update: true
  };

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadStats();
    } else {
      // Demo mode
      stats = demoStats;
      loading = false;
    }
  });

  async function loadStats() {
    if (!browser || !isTauri) return;
    
    loading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      stats = await invoke<IpsetStats>('get_ipset_stats');
    } catch (e) {
      console.error('Failed to load ipset stats:', e);
      showMessage('Failed to load ipset stats', 'error');
    } finally {
      loading = false;
    }
  }

  async function handleModeChange(newMode: IpsetMode) {
    if (!browser || !isTauri || changingMode) return;
    if (stats?.mode === newMode) return;
    
    // Prevent switching to loaded mode if file doesn't exist
    if (newMode === 'loaded' && !canUseLoadedMode) {
      showMessage('Cannot use loaded mode: ipset file does not exist', 'warning');
      return;
    }
    
    changingMode = true;
    message = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_ipset_mode', { mode: newMode });
      showMessage(`Mode changed to ${modeDescriptions[newMode].title}`, 'success');
      await loadStats();
    } catch (e) {
      console.error('Failed to change ipset mode:', e);
      showMessage(`Failed to change mode: ${e}`, 'error');
    } finally {
      changingMode = false;
    }
  }

  async function handleUpdateNow() {
    if (!browser || !isTauri || updating) return;
    
    updating = true;
    message = null;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<IpsetUpdateResult>('update_ipset_from_sources');
      
      if (result.success) {
        showMessage(
          `Updated: ${result.ip_count.toLocaleString()} IPs (${result.ipv4_count} IPv4, ${result.ipv6_count} IPv6)`,
          'success'
        );
        await loadStats();
      } else {
        showMessage(result.error || 'Update failed', 'error');
      }
    } catch (e) {
      console.error('Failed to update ipset:', e);
      showMessage(`Update failed: ${e}`, 'error');
    } finally {
      updating = false;
    }
  }

  async function handleAutoUpdateToggle(enabled: boolean) {
    if (!browser || !isTauri || !stats) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_ipset_auto_update', { enabled });
      stats = { ...stats, auto_update: enabled };
      showMessage(enabled ? 'Auto-update enabled' : 'Auto-update disabled', 'success');
    } catch (e) {
      console.error('Failed to toggle auto-update:', e);
      showMessage(`Failed to toggle auto-update: ${e}`, 'error');
    }
  }

  async function handleRestoreBackup() {
    if (!browser || !isTauri) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('restore_ipset_backup');
      showMessage('Restored from backup', 'success');
      await loadStats();
    } catch (e) {
      console.error('Failed to restore backup:', e);
      showMessage(`Restore failed: ${e}`, 'error');
    }
  }

  function showMessage(text: string, type: 'success' | 'error' | 'warning') {
    message = { text, type };
    setTimeout(() => { message = null; }, 4000);
  }
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-text-primary">IP Set Management</h2>
    {#if message}
      <span class="text-sm animate-pulse {message.type === 'error' ? 'text-red-400' : message.type === 'warning' ? 'text-amber-400' : 'text-indigo-400'}">
        {message.text}
      </span>
    {/if}
  </div>
  
  {#if loading}
    <div class="flex items-center justify-center py-12">
      <svg class="w-8 h-8 animate-spin text-indigo-500" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
    </div>
  {:else}
    <div class="space-y-4">
      <!-- Mode Selection -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <p class="text-text-primary font-medium mb-4">Filtering Mode</p>
        <div class="space-y-3" role="radiogroup" aria-label="IP Set filtering mode">
          {#each (['any', 'none', 'loaded'] as const) as mode}
            {@const isSelected = stats?.mode === mode}
            {@const isDisabled = mode === 'loaded' && !canUseLoadedMode}
            <button
              type="button"
              role="radio"
              aria-checked={isSelected}
              disabled={changingMode || isDisabled}
              onclick={() => handleModeChange(mode)}
              class="w-full p-4 rounded-lg border text-left transition-all
                {isSelected 
                  ? 'bg-indigo-500/10 border-indigo-500/50' 
                  : 'bg-void-200/50 border-glass-border hover:border-glass-border-hover'}
                {isDisabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
                {changingMode ? 'opacity-70' : ''}"
            >
              <div class="flex items-start gap-3">
                <!-- Radio indicator -->
                <div class="mt-0.5 w-5 h-5 rounded-full border-2 flex items-center justify-center flex-shrink-0
                  {isSelected ? 'border-indigo-500' : 'border-text-muted'}">
                  {#if isSelected}
                    <div class="w-2.5 h-2.5 rounded-full bg-indigo-500"></div>
                  {/if}
                </div>
                
                <div class="flex-1">
                  <p class="text-text-primary font-medium flex items-center gap-2">
                    {modeDescriptions[mode].title}
                    {#if mode === 'loaded' && !canUseLoadedMode}
                      <span class="text-xs text-amber-400">(No file)</span>
                    {/if}
                  </p>
                  <p class="text-text-secondary text-sm mt-1">
                    {modeDescriptions[mode].description}
                  </p>
                </div>
              </div>
            </button>
          {/each}
        </div>
      </div>

      <!-- Statistics Cards -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <p class="text-text-muted text-xs uppercase tracking-wide">Total IPs</p>
          <p class="text-text-primary text-2xl font-semibold mt-1">
            {stats?.ip_count?.toLocaleString() ?? 0}
          </p>
        </div>
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <p class="text-text-muted text-xs uppercase tracking-wide">IPv4</p>
          <p class="text-text-primary text-2xl font-semibold mt-1">
            {stats?.ipv4_count?.toLocaleString() ?? 0}
          </p>
        </div>
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <p class="text-text-muted text-xs uppercase tracking-wide">IPv6</p>
          <p class="text-text-primary text-2xl font-semibold mt-1">
            {stats?.ipv6_count?.toLocaleString() ?? 0}
          </p>
        </div>
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <p class="text-text-muted text-xs uppercase tracking-wide">CIDR Blocks</p>
          <p class="text-text-primary text-2xl font-semibold mt-1">
            {stats?.cidr_count?.toLocaleString() ?? 0}
          </p>
        </div>
      </div>

      <!-- File Status -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <div class="flex items-start justify-between">
          <div>
            <p class="text-text-primary font-medium">Ipset File</p>
            <p class="text-text-secondary text-sm mt-1">
              {stats?.file_exists 
                ? `${formattedSize()} • Last updated: ${formattedDate}` 
                : 'File does not exist'}
            </p>
          </div>
          <span class="px-2 py-1 text-xs font-medium rounded-lg
            {stats?.file_exists 
              ? 'bg-emerald-500/10 text-emerald-400' 
              : 'bg-amber-500/10 text-amber-400'}">
            {stats?.file_exists ? 'Exists' : 'Missing'}
          </span>
        </div>
        
        {#if stats?.source_url}
          <div class="mt-3 pt-3 border-t border-glass-border">
            <span class="text-text-muted text-xs">Source:</span>
            <span class="text-text-secondary text-xs ml-2 break-all">{stats.source_url}</span>
          </div>
        {/if}
      </div>

      <!-- Update Actions -->
      <div class="flex items-center gap-3" role="group" aria-label="Ipset update actions">
        <Button 
          variant="primary" 
          onclick={handleUpdateNow}
          loading={updating}
          disabled={updating}
          aria-label="Update ipset now"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          Update Now
        </Button>
        <Button 
          variant="secondary" 
          onclick={handleRestoreBackup}
          disabled={updating || !stats?.file_exists}
          aria-label="Restore ipset from backup"
        >
          Restore Backup
        </Button>
      </div>

      <!-- Auto-Update Toggle -->
      <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
        <div id="ipset-auto-update-label">
          <p class="text-text-primary font-medium">Auto-Update</p>
          <p class="text-text-secondary text-sm">
            Automatically update ipset every 24 hours
          </p>
        </div>
        <Toggle 
          checked={stats?.auto_update ?? false}
          onchange={handleAutoUpdateToggle}
          aria-labelledby="ipset-auto-update-label"
        />
      </div>

      <!-- Info Box -->
      <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
        <p class="text-indigo-400 text-sm flex items-start gap-2">
          <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span>Ipset contains IP addresses that are known to be blocked. Primary source is zapret-discord-youtube GitHub repository with IP addresses for Discord and YouTube.</span>
        </p>
      </div>
    </div>
  {/if}
</div>
