<script lang="ts">
  /**
   * HostsSettings Component
   * 
   * UI for managing Windows hosts file modifications for Discord bypass.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { browser } from '$app/environment';
  import Button from '$lib/components/Button.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import type { HostsStatus, HostsOperationResult } from '$lib/api/hosts';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let status = $state<HostsStatus | null>(null);
  let loading = $state(true);
  let operating = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' | 'warning' } | null>(null);
  let isTauri = $state(false);

  // Derived
  let formattedBackupDate = $derived(
    status?.backup_timestamp 
      ? new Date(status.backup_timestamp).toLocaleString() 
      : 'No backup'
  );

  let formattedLastModified = $derived(
    status?.last_modified 
      ? new Date(status.last_modified).toLocaleString() 
      : 'Unknown'
  );

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadStatus();
    } else {
      loading = false;
    }
  });

  async function loadStatus() {
    if (!browser || !isTauri) return;
    
    loading = true;
    try {
      const { getHostsStatus } = await import('$lib/api/hosts');
      status = await getHostsStatus();
    } catch (e) {
      console.error('Failed to load hosts status:', e);
      showMessage('Failed to load hosts status', 'error');
    } finally {
      loading = false;
    }
  }

  async function handleToggleDiscord(enabled: boolean) {
    if (!browser || !isTauri || operating) return;
    
    operating = true;
    message = null;
    
    try {
      const { enableDiscordHosts, disableDiscordHosts } = await import('$lib/api/hosts');
      const result: HostsOperationResult = enabled 
        ? await enableDiscordHosts() 
        : await disableDiscordHosts();
      
      if (result.success) {
        showMessage(
          enabled 
            ? `Discord hosts enabled (${result.entries_affected} entries)` 
            : 'Discord hosts disabled',
          'success'
        );
        await loadStatus();
      } else if (result.requires_admin) {
        showMessage('Administrator privileges required', 'warning');
      } else {
        showMessage(result.error || 'Operation failed', 'error');
      }
    } catch (e) {
      console.error('Failed to toggle Discord hosts:', e);
      showMessage(`Failed: ${e}`, 'error');
    } finally {
      operating = false;
    }
  }

  async function handleBackup() {
    if (!browser || !isTauri || operating) return;
    
    operating = true;
    
    try {
      const { backupHostsFile } = await import('$lib/api/hosts');
      const result = await backupHostsFile();
      
      if (result.success) {
        showMessage('Backup created successfully', 'success');
        await loadStatus();
      } else {
        showMessage(result.error || 'Backup failed', 'error');
      }
    } catch (e) {
      console.error('Failed to backup hosts:', e);
      showMessage(`Backup failed: ${e}`, 'error');
    } finally {
      operating = false;
    }
  }

  async function handleRestore() {
    if (!browser || !isTauri || operating || !status?.backup_exists) return;
    
    operating = true;
    
    try {
      const { restoreHostsFile } = await import('$lib/api/hosts');
      const result = await restoreHostsFile();
      
      if (result.success) {
        showMessage('Hosts file restored from backup', 'success');
        await loadStatus();
      } else if (result.requires_admin) {
        showMessage('Administrator privileges required', 'warning');
      } else {
        showMessage(result.error || 'Restore failed', 'error');
      }
    } catch (e) {
      console.error('Failed to restore hosts:', e);
      showMessage(`Restore failed: ${e}`, 'error');
    } finally {
      operating = false;
    }
  }

  async function handleFlushDns() {
    if (!browser || !isTauri || operating) return;
    
    operating = true;
    
    try {
      const { flushDnsCache } = await import('$lib/api/hosts');
      await flushDnsCache();
      showMessage('DNS cache flushed', 'success');
    } catch (e) {
      console.error('Failed to flush DNS:', e);
      showMessage(`Failed to flush DNS: ${e}`, 'error');
    } finally {
      operating = false;
    }
  }

  function showMessage(text: string, type: 'success' | 'error' | 'warning') {
    message = { text, type };
    setTimeout(() => { message = null; }, 4000);
  }
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-text-primary">Hosts Manager</h2>
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
      <!-- Admin Warning -->
      {#if status && !status.is_writable}
        <div class="p-4 bg-amber-500/10 rounded-xl border border-amber-500/20">
          <p class="text-amber-400 text-sm flex items-start gap-2">
            <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
            </svg>
            <span><strong>Administrator privileges required.</strong> Run Isolate as administrator to modify the hosts file.</span>
          </p>
        </div>
      {/if}

      <!-- Discord Hosts Toggle -->
      <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
        <div id="discord-hosts-label">
          <p class="text-text-primary font-medium">Discord Hosts</p>
          <p class="text-text-secondary text-sm">
            {status?.discord_enabled 
              ? `Enabled (${status.discord_entries_count} entries)` 
              : 'Add IP mappings for Discord domains'}
          </p>
        </div>
        <Toggle 
          checked={status?.discord_enabled ?? false}
          onchange={handleToggleDiscord}
          disabled={operating || (status && !status.is_writable)}
          aria-labelledby="discord-hosts-label"
        />
      </div>

      <!-- Status Info -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <p class="text-text-primary font-medium mb-4">Hosts File Status</p>
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-text-muted">Status:</span>
            <span class="ml-2 {status?.is_writable ? 'text-emerald-400' : 'text-amber-400'}">
              {status?.is_writable ? 'Writable' : 'Read-only'}
            </span>
          </div>
          <div>
            <span class="text-text-muted">Last Modified:</span>
            <span class="text-text-secondary ml-2">{formattedLastModified}</span>
          </div>
          <div>
            <span class="text-text-muted">Backup:</span>
            <span class="text-text-secondary ml-2">
              {status?.backup_exists ? formattedBackupDate : 'None'}
            </span>
          </div>
          <div>
            <span class="text-text-muted">Discord Entries:</span>
            <span class="text-text-secondary ml-2">{status?.discord_entries_count ?? 0}</span>
          </div>
        </div>
      </div>

      <!-- Backup/Restore Actions -->
      <div class="flex items-center gap-3" role="group" aria-label="Hosts file actions">
        <Button 
          variant="secondary" 
          onclick={handleBackup}
          disabled={operating}
          aria-label="Create backup of hosts file"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"/>
          </svg>
          Backup
        </Button>
        <Button 
          variant="secondary" 
          onclick={handleRestore}
          disabled={operating || !status?.backup_exists}
          aria-label="Restore hosts file from backup"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          Restore
        </Button>
        <Button 
          variant="ghost" 
          onclick={handleFlushDns}
          disabled={operating}
          aria-label="Flush DNS cache"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          Flush DNS
        </Button>
      </div>

      <!-- Info Box -->
      <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
        <p class="text-indigo-400 text-sm flex items-start gap-2">
          <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span>Discord hosts modification adds IP mappings to bypass DPI for Discord. This requires administrator privileges and modifies the Windows hosts file at <code class="bg-void-200 px-1 rounded">C:\Windows\System32\drivers\etc\hosts</code>.</span>
        </p>
      </div>
    </div>
  {/if}
</div>
