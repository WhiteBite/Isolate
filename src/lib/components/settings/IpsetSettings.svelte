<script lang="ts">
  /**
   * IpsetSettings Component
   * 
   * UI for managing ipset configuration and updates.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { browser } from '$app/environment';
  import Toggle from '$lib/components/Toggle.svelte';
  import Button from '$lib/components/Button.svelte';
  import type { IpsetInfo, IpsetSource, IpsetUpdateResult } from '$lib/api/ipset';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let ipsetInfo = $state<IpsetInfo | null>(null);
  let sources = $state<IpsetSource[]>([]);
  let loading = $state(true);
  let updating = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' } | null>(null);
  let isTauri = $state(false);

  // Derived
  let formattedDate = $derived(
    ipsetInfo?.last_updated 
      ? new Date(ipsetInfo.last_updated).toLocaleString() 
      : 'Never'
  );

  let formattedSize = $derived(() => {
    if (!ipsetInfo?.size) return '-';
    const bytes = ipsetInfo.size;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  });

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadIpsetInfo();
      loadSources();
    } else {
      loading = false;
    }
  });

  async function loadIpsetInfo() {
    if (!browser || !isTauri) return;
    
    loading = true;
    try {
      const { getIpsetInfo } = await import('$lib/api/ipset');
      ipsetInfo = await getIpsetInfo();
    } catch (e) {
      console.error('Failed to load ipset info:', e);
      showMessage('Failed to load ipset info', 'error');
    } finally {
      loading = false;
    }
  }

  async function loadSources() {
    if (!browser || !isTauri) return;
    
    try {
      const { getIpsetSources } = await import('$lib/api/ipset');
      sources = await getIpsetSources();
    } catch (e) {
      console.error('Failed to load ipset sources:', e);
    }
  }

  async function handleUpdateNow() {
    if (!browser || !isTauri || updating) return;
    
    updating = true;
    message = null;
    
    try {
      const { updateIpsetFromSources } = await import('$lib/api/ipset');
      const result: IpsetUpdateResult = await updateIpsetFromSources();
      
      if (result.success) {
        showMessage(`Updated: ${result.ip_count.toLocaleString()} IPs (${result.ipv4_count} IPv4, ${result.ipv6_count} IPv6)`, 'success');
        await loadIpsetInfo();
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

  async function handleRestoreBackup() {
    if (!browser || !isTauri) return;
    
    try {
      const { restoreIpsetBackup } = await import('$lib/api/ipset');
      await restoreIpsetBackup();
      showMessage('Restored from backup', 'success');
      await loadIpsetInfo();
    } catch (e) {
      console.error('Failed to restore backup:', e);
      showMessage(`Restore failed: ${e}`, 'error');
    }
  }

  async function handleAutoUpdateToggle(enabled: boolean) {
    if (!browser || !isTauri || !ipsetInfo) return;
    
    try {
      const { setIpsetAutoUpdate } = await import('$lib/api/ipset');
      await setIpsetAutoUpdate(enabled);
      ipsetInfo = { ...ipsetInfo, auto_update_enabled: enabled };
      showMessage(enabled ? 'Auto-update enabled' : 'Auto-update disabled', 'success');
    } catch (e) {
      console.error('Failed to toggle auto-update:', e);
      showMessage(`Failed to toggle auto-update: ${e}`, 'error');
    }
  }

  function showMessage(text: string, type: 'success' | 'error') {
    message = { text, type };
    setTimeout(() => { message = null; }, 3000);
  }
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-text-primary">Ipset Configuration</h2>
    {#if message}
      <span class="text-sm animate-pulse {message.type === 'error' ? 'text-red-400' : 'text-indigo-400'}">
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
      <!-- Current Ipset Info -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <div class="flex items-start justify-between">
          <div>
            <p class="text-text-primary font-medium">Current Ipset</p>
            <p class="text-text-secondary text-sm mt-1">
              {ipsetInfo?.ip_count?.toLocaleString() ?? 0} IP addresses • {formattedSize()}
            </p>
          </div>
          {#if ipsetInfo?.update_available}
            <span class="px-2 py-1 bg-indigo-500/10 text-indigo-400 text-xs font-medium rounded-lg">
              Update Available
            </span>
          {/if}
        </div>
        
        <div class="mt-4 grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-text-muted">Last Updated:</span>
            <span class="text-text-secondary ml-2">{formattedDate}</span>
          </div>
          <div>
            <span class="text-text-muted">IPv4 / IPv6:</span>
            <span class="text-text-secondary ml-2">
              {ipsetInfo?.ipv4_count ?? 0} / {ipsetInfo?.ipv6_count ?? 0}
            </span>
          </div>
          <div>
            <span class="text-text-muted">CIDR Blocks:</span>
            <span class="text-text-secondary ml-2">{ipsetInfo?.cidr_count ?? 0}</span>
          </div>
          {#if ipsetInfo?.source_url}
            <div class="col-span-2">
              <span class="text-text-muted">Source:</span>
              <span class="text-text-secondary ml-2 text-xs truncate">{ipsetInfo.source_url}</span>
            </div>
          {/if}
        </div>
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
          disabled={updating}
          aria-label="Restore ipset from backup"
        >
          Restore Backup
        </Button>
      </div>

      <!-- Available Sources -->
      {#if sources.length > 0}
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <p class="text-text-primary font-medium mb-4">Available Sources</p>
          <div class="space-y-2">
            {#each sources.filter(s => s.enabled) as source}
              <div class="flex items-center gap-3 p-3 rounded-lg bg-void-200/50">
                <div class="flex-1">
                  <p class="text-text-primary font-medium text-sm">{source.name}</p>
                  {#if source.description}
                    <p class="text-text-secondary text-xs mt-0.5">{source.description}</p>
                  {/if}
                </div>
                <span class="text-text-muted text-xs">Priority: {source.priority}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Auto-Update Toggle -->
      <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
        <div id="ipset-auto-update-label">
          <p class="text-text-primary font-medium">Auto-Update</p>
          <p class="text-text-secondary text-sm">
            Automatically update ipset every 24 hours
          </p>
        </div>
        <Toggle 
          checked={ipsetInfo?.auto_update_enabled ?? false}
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
