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
  let loading = $state(true);
  let updating = $state(false);
  let checkingUpdate = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' } | null>(null);
  let isTauri = $state(false);

  // Derived
  let formattedDate = $derived(
    ipsetInfo?.last_updated 
      ? new Date(ipsetInfo.last_updated).toLocaleString() 
      : 'Never'
  );

  let formattedSize = $derived(() => {
    if (!ipsetInfo?.size_bytes) return '-';
    const bytes = ipsetInfo.size_bytes;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  });

  // Available sources
  const sources: { id: IpsetSource; name: string; description: string }[] = [
    { id: 'antifilter', name: 'Antifilter', description: 'Community-maintained list of blocked IPs' },
    { id: 'community', name: 'Community', description: 'Crowdsourced IP list' },
    { id: 'custom', name: 'Custom', description: 'Your own ipset file' }
  ];

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadIpsetInfo();
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

  async function handleUpdateNow() {
    if (!browser || !isTauri || updating) return;
    
    updating = true;
    message = null;
    
    try {
      const { updateIpset } = await import('$lib/api/ipset');
      const result: IpsetUpdateResult = await updateIpset();
      
      if (result.success) {
        showMessage(`Updated successfully: ${result.ip_count.toLocaleString()} IPs`, 'success');
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

  async function handleCheckUpdate() {
    if (!browser || !isTauri || checkingUpdate) return;
    
    checkingUpdate = true;
    
    try {
      const { checkIpsetUpdate } = await import('$lib/api/ipset');
      const hasUpdate = await checkIpsetUpdate();
      
      if (hasUpdate) {
        showMessage('Update available!', 'success');
        if (ipsetInfo) {
          ipsetInfo = { ...ipsetInfo, update_available: true };
        }
      } else {
        showMessage('Already up to date', 'success');
      }
    } catch (e) {
      console.error('Failed to check update:', e);
      showMessage(`Check failed: ${e}`, 'error');
    } finally {
      checkingUpdate = false;
    }
  }

  async function handleSourceChange(source: IpsetSource) {
    if (!browser || !isTauri || !ipsetInfo) return;
    
    try {
      const { setIpsetSource } = await import('$lib/api/ipset');
      await setIpsetSource(source);
      ipsetInfo = { ...ipsetInfo, source };
      showMessage('Source changed', 'success');
    } catch (e) {
      console.error('Failed to change source:', e);
      showMessage(`Failed to change source: ${e}`, 'error');
    }
  }

  async function handleAutoUpdateToggle(enabled: boolean) {
    if (!browser || !isTauri || !ipsetInfo) return;
    
    try {
      const { toggleIpsetAutoUpdate } = await import('$lib/api/ipset');
      await toggleIpsetAutoUpdate(enabled);
      ipsetInfo = { ...ipsetInfo, auto_update: enabled };
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
            <span class="text-text-muted">Source:</span>
            <span class="text-text-secondary ml-2 capitalize">{ipsetInfo?.source ?? 'Unknown'}</span>
          </div>
        </div>
      </div>

      <!-- Update Actions -->
      <div class="flex items-center gap-3" role="group" aria-label="Ipset update actions">
        <Button 
          variant="primary" 
          onclick={handleUpdateNow}
          loading={updating}
          disabled={updating || checkingUpdate}
          aria-label="Update ipset now"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          Update Now
        </Button>
        <Button 
          variant="secondary" 
          onclick={handleCheckUpdate}
          loading={checkingUpdate}
          disabled={updating || checkingUpdate}
          aria-label="Check for ipset updates"
        >
          Check for Updates
        </Button>
      </div>

      <!-- Source Selection -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <p id="ipset-source-label" class="text-text-primary font-medium mb-4">Ipset Source</p>
        <div class="space-y-2" role="radiogroup" aria-labelledby="ipset-source-label">
          {#each sources as source}
            <label 
              for="ipset-source-{source.id}"
              class="flex items-center gap-3 p-3 rounded-lg cursor-pointer hover:bg-void-200 transition-colors {ipsetInfo?.source === source.id ? 'bg-indigo-500/10 border border-indigo-500/30' : ''}"
            >
              <input
                id="ipset-source-{source.id}"
                type="radio"
                name="ipset-source"
                value={source.id}
                checked={ipsetInfo?.source === source.id}
                onchange={() => handleSourceChange(source.id)}
                class="w-4 h-4 text-indigo-500 bg-void-200 border-glass-border focus:ring-indigo-500/50"
              />
              <div>
                <p class="text-text-primary font-medium">{source.name}</p>
                <p class="text-text-secondary text-sm">{source.description}</p>
              </div>
            </label>
          {/each}
        </div>
      </div>

      <!-- Auto-Update Toggle -->
      <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
        <div id="ipset-auto-update-label">
          <p class="text-text-primary font-medium">Auto-Update</p>
          <p class="text-text-secondary text-sm">
            Automatically update ipset every {ipsetInfo?.auto_update_interval_hours ?? 24} hours
          </p>
        </div>
        <Toggle 
          checked={ipsetInfo?.auto_update ?? false}
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
          <span>Ipset contains IP addresses that are known to be blocked. This helps strategies target the right traffic for DPI bypass.</span>
        </p>
      </div>
    </div>
  {/if}
</div>
