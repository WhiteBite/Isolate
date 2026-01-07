<script lang="ts">
  /**
   * AutoRecoverySettings Component
   * 
   * Настройки автоматического переключения на backup стратегию при сбоях.
   * Включает:
   * - Toggle для включения/выключения
   * - Настройка max_failures (slider 1-10)
   * - Настройка cooldown_secs (slider 10-120)
   * - Список backup стратегий с приоритетом
   * - Текущий статус (failures count, current backup)
   */
  import { browser } from '$app/environment';
  import Toggle from '$lib/components/Toggle.svelte';
  import type { FailoverStatus, FailoverConfig } from '$lib/api/failover';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let status = $state<FailoverStatus | null>(null);
  let config = $state<FailoverConfig | null>(null);
  let learnedStrategies = $state<string[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' | 'warning' } | null>(null);
  let isTauri = $state(false);

  // Local config state for editing
  let localMaxFailures = $state(3);
  let localCooldownSecs = $state(60);

  // Polling interval
  const POLL_INTERVAL = 5000;
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  // Derived - compute values directly, not functions
  let failoverProgress = $derived(
    status && status.maxFailures > 0 
      ? Math.min(100, (status.failureCount / status.maxFailures) * 100) 
      : 0
  );

  let statusColor = $derived(
    !status?.enabled ? 'gray' :
    failoverProgress === 0 ? 'green' :
    failoverProgress < 66 ? 'yellow' : 'red'
  );

  let cooldownFormatted = $derived(
    !status?.cooldownRemaining || status.cooldownRemaining <= 0 ? null :
    status.cooldownRemaining < 60 ? `${status.cooldownRemaining}s` :
    (() => {
      const mins = Math.floor(status.cooldownRemaining / 60);
      const secs = status.cooldownRemaining % 60;
      return secs > 0 ? `${mins}m ${secs}s` : `${mins}m`;
    })()
  );

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadData();
      startPolling();
    } else {
      loading = false;
    }
    
    return () => stopPolling();
  });

  function startPolling() {
    pollTimer = setInterval(loadStatus, POLL_INTERVAL);
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  async function loadData() {
    loading = true;
    try {
      await Promise.all([loadStatus(), loadConfig(), loadLearnedStrategies()]);
    } finally {
      loading = false;
    }
  }

  async function loadStatus() {
    if (!browser || !isTauri) return;
    
    try {
      const { getFailoverStatus } = await import('$lib/api/failover');
      status = await getFailoverStatus();
    } catch (e) {
      console.error('Failed to load failover status:', e);
    }
  }

  async function loadConfig() {
    if (!browser || !isTauri) return;
    
    try {
      const { getFailoverConfig } = await import('$lib/api/failover');
      config = await getFailoverConfig();
      if (config) {
        localMaxFailures = config.maxFailures;
        localCooldownSecs = config.cooldownSecs;
      }
    } catch (e) {
      console.error('Failed to load failover config:', e);
    }
  }

  async function loadLearnedStrategies() {
    if (!browser || !isTauri) return;
    
    try {
      const { getLearnedStrategies } = await import('$lib/api/failover');
      learnedStrategies = await getLearnedStrategies();
    } catch (e) {
      console.error('Failed to load learned strategies:', e);
    }
  }

  async function handleToggleEnabled(enabled: boolean) {
    if (!browser || !isTauri || saving) return;
    
    saving = true;
    message = null;
    
    try {
      const { setFailoverEnabled } = await import('$lib/api/failover');
      await setFailoverEnabled(enabled);
      await loadStatus();
      showMessage(enabled ? 'Auto-recovery enabled' : 'Auto-recovery disabled', 'success');
    } catch (e) {
      console.error('Failed to toggle failover:', e);
      showMessage(`Failed: ${e}`, 'error');
    } finally {
      saving = false;
    }
  }

  async function handleSaveConfig() {
    if (!browser || !isTauri || saving) return;
    
    saving = true;
    message = null;
    
    try {
      const { setFailoverConfig } = await import('$lib/api/failover');
      await setFailoverConfig(localMaxFailures, localCooldownSecs);
      await loadConfig();
      showMessage('Configuration saved', 'success');
    } catch (e) {
      console.error('Failed to save failover config:', e);
      showMessage(`Failed to save: ${e}`, 'error');
    } finally {
      saving = false;
    }
  }

  async function handleManualFailover() {
    if (!browser || !isTauri || saving) return;
    
    saving = true;
    message = null;
    
    try {
      const { triggerManualFailover } = await import('$lib/api/failover');
      const backupId = await triggerManualFailover();
      
      if (backupId) {
        showMessage(`Switching to: ${backupId}`, 'success');
      } else {
        showMessage('No backup strategy available', 'warning');
      }
      
      await loadStatus();
    } catch (e) {
      console.error('Failed to trigger failover:', e);
      showMessage(`Failed: ${e}`, 'error');
    } finally {
      saving = false;
    }
  }

  async function handleResetState() {
    if (!browser || !isTauri || saving || !status?.currentStrategy) return;
    
    saving = true;
    message = null;
    
    try {
      const { resetFailoverState } = await import('$lib/api/failover');
      await resetFailoverState(status.currentStrategy);
      await loadStatus();
      showMessage('Failover state reset', 'success');
    } catch (e) {
      console.error('Failed to reset failover state:', e);
      showMessage(`Failed: ${e}`, 'error');
    } finally {
      saving = false;
    }
  }

  function showMessage(text: string, type: 'success' | 'error' | 'warning') {
    message = { text, type };
    setTimeout(() => { message = null; }, 4000);
  }

  // Config has changed
  let configChanged = $derived(
    config && (localMaxFailures !== config.maxFailures || localCooldownSecs !== config.cooldownSecs)
  );
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <div>
      <h2 class="text-xl font-semibold text-text-primary flex items-center gap-2">
        <svg class="w-5 h-5 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
        </svg>
        Auto Recovery
      </h2>
      <p class="text-text-secondary text-sm mt-1">Automatically switch to backup strategy on failure</p>
    </div>
    {#if message}
      <span class="text-sm animate-pulse {message.type === 'error' ? 'text-red-400' : message.type === 'warning' ? 'text-amber-400' : 'text-indigo-400'}">
        {message.text}
      </span>
    {/if}
  </div>
  
  {#if loading}
    <div class="flex items-center justify-center py-12">
      <svg class="w-8 h-8 animate-spin text-indigo-500" fill="none" viewBox="0 0 24 24" aria-label="Loading">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
    </div>
  {:else}
    <div class="space-y-4">
      <!-- Enable Toggle -->
      <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
        <div id="auto-recovery-label">
          <p class="text-text-primary font-medium">Enable Auto Recovery</p>
          <p class="text-text-secondary text-sm">
            {status?.enabled 
              ? 'Active — will switch on failures' 
              : 'Disabled — manual control only'}
          </p>
        </div>
        <Toggle 
          checked={status?.enabled ?? false}
          onchange={handleToggleEnabled}
          disabled={saving}
          aria-labelledby="auto-recovery-label"
        />
      </div>

      {#if status?.enabled}
        <!-- Current Status -->
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <p class="text-text-primary font-medium mb-4">Current Status</p>
          
          <div class="grid grid-cols-2 gap-4">
            <!-- Failure Progress -->
            <div>
              <div class="flex items-center justify-between mb-2">
                <span class="text-text-muted text-sm">Failures</span>
                <span class="text-text-primary font-mono text-sm">
                  {status?.failureCount ?? 0} / {status?.maxFailures ?? 0}
                </span>
              </div>
              <div class="h-2 bg-void-200 rounded-full overflow-hidden" role="progressbar" aria-valuenow={failoverProgress} aria-valuemin={0} aria-valuemax={100}>
                <div 
                  class="h-full transition-all duration-300 {failoverProgress === 0 ? 'bg-emerald-500' : failoverProgress < 66 ? 'bg-amber-500' : 'bg-red-500'}"
                  style="width: {failoverProgress}%"
                ></div>
              </div>
            </div>

            <!-- Status Indicator -->
            <div class="flex items-center gap-3">
              <div class="w-3 h-3 rounded-full {statusColor === 'green' ? 'bg-emerald-500' : statusColor === 'yellow' ? 'bg-amber-500' : statusColor === 'red' ? 'bg-red-500' : 'bg-zinc-500'} {statusColor === 'green' ? 'animate-pulse' : ''}"></div>
              <div>
                <p class="text-text-primary text-sm font-medium">
                  {#if statusColor === 'green'}
                    Healthy
                  {:else if statusColor === 'yellow'}
                    Degraded
                  {:else if statusColor === 'red'}
                    Critical
                  {:else}
                    Inactive
                  {/if}
                </p>
                {#if cooldownFormatted}
                  <p class="text-text-muted text-xs">Cooldown: {cooldownFormatted}</p>
                {/if}
              </div>
            </div>
          </div>

          <!-- Current & Next Strategy -->
          <div class="mt-4 pt-4 border-t border-glass-border grid grid-cols-2 gap-4">
            <div>
              <span class="text-text-muted text-xs uppercase tracking-wider">Current Strategy</span>
              <p class="text-text-primary font-mono text-sm mt-1 truncate" title={status?.currentStrategy ?? 'None'}>
                {status?.currentStrategy ?? 'None'}
              </p>
            </div>
            <div>
              <span class="text-text-muted text-xs uppercase tracking-wider">Next Backup</span>
              <p class="text-text-secondary font-mono text-sm mt-1 truncate" title={status?.nextBackup ?? 'Auto-select'}>
                {status?.nextBackup ?? 'Auto-select'}
              </p>
            </div>
          </div>

          <!-- Last Error -->
          {#if status?.lastFailureReason}
            <div class="mt-4 p-3 bg-red-500/10 rounded-lg border border-red-500/20">
              <p class="text-red-400 text-xs flex items-start gap-2">
                <svg class="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                </svg>
                <span class="line-clamp-2">{status.lastFailureReason}</span>
              </p>
            </div>
          {/if}
        </div>

        <!-- Configuration -->
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <div class="flex items-center justify-between mb-4">
            <p class="text-text-primary font-medium">Configuration</p>
            {#if configChanged}
              <button
                onclick={handleSaveConfig}
                disabled={saving}
                class="px-3 py-1.5 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white text-sm rounded-lg font-medium transition-colors"
                aria-label="Save configuration changes"
              >
                Save Changes
              </button>
            {/if}
          </div>

          <div class="space-y-6">
            <!-- Max Failures Slider -->
            <div>
              <div class="flex items-center justify-between mb-2">
                <label for="max-failures-slider" class="text-text-secondary text-sm">Max failures before switch</label>
                <span class="text-text-primary font-mono text-sm bg-void-200 px-2 py-0.5 rounded">{localMaxFailures}</span>
              </div>
              <input
                id="max-failures-slider"
                type="range"
                min="1"
                max="10"
                step="1"
                bind:value={localMaxFailures}
                class="w-full h-2 bg-void-200 rounded-lg appearance-none cursor-pointer accent-amber-500"
                aria-valuemin={1}
                aria-valuemax={10}
                aria-valuenow={localMaxFailures}
              />
              <div class="flex justify-between text-xs text-text-muted mt-1">
                <span>1 (aggressive)</span>
                <span>10 (tolerant)</span>
              </div>
            </div>

            <!-- Cooldown Slider -->
            <div>
              <div class="flex items-center justify-between mb-2">
                <label for="cooldown-slider" class="text-text-secondary text-sm">Cooldown before retry</label>
                <span class="text-text-primary font-mono text-sm bg-void-200 px-2 py-0.5 rounded">{localCooldownSecs}s</span>
              </div>
              <input
                id="cooldown-slider"
                type="range"
                min="10"
                max="120"
                step="10"
                bind:value={localCooldownSecs}
                class="w-full h-2 bg-void-200 rounded-lg appearance-none cursor-pointer accent-amber-500"
                aria-valuemin={10}
                aria-valuemax={120}
                aria-valuenow={localCooldownSecs}
              />
              <div class="flex justify-between text-xs text-text-muted mt-1">
                <span>10s (fast)</span>
                <span>120s (slow)</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Learned Strategies -->
        {#if learnedStrategies.length > 0}
          <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
            <p class="text-text-primary font-medium mb-3">Backup Strategies</p>
            <p class="text-text-muted text-xs mb-3">
              Strategies that worked successfully in the past (used as backups in order)
            </p>
            <div class="space-y-2 max-h-40 overflow-y-auto" role="list" aria-label="Backup strategies list">
              {#each learnedStrategies as strategy, index}
                <div 
                  class="flex items-center gap-3 p-2 bg-void-200 rounded-lg border border-glass-border"
                  role="listitem"
                >
                  <span class="w-6 h-6 flex items-center justify-center bg-void-300 rounded text-text-muted text-xs font-mono">
                    {index + 1}
                  </span>
                  <span class="text-text-primary text-sm font-mono truncate flex-1" title={strategy}>
                    {strategy}
                  </span>
                  {#if status?.currentStrategy === strategy}
                    <span class="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 text-xs rounded-full">
                      Active
                    </span>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Actions -->
        <div class="flex items-center gap-3" role="group" aria-label="Failover actions">
          <button
            onclick={handleManualFailover}
            disabled={saving || !status?.currentStrategy}
            class="flex-1 px-4 py-2.5 bg-amber-500/10 hover:bg-amber-500/20 disabled:opacity-50 disabled:cursor-not-allowed text-amber-400 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
            aria-label="Manually switch to backup strategy"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
            </svg>
            Switch Now
          </button>
          <button
            onclick={handleResetState}
            disabled={saving || !status?.currentStrategy || status?.failureCount === 0}
            class="px-4 py-2.5 bg-void-200 hover:bg-void-300 disabled:opacity-50 disabled:cursor-not-allowed text-text-secondary rounded-lg font-medium transition-colors flex items-center gap-2"
            aria-label="Reset failure counter"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
            </svg>
            Reset
          </button>
        </div>
      {/if}

      <!-- Info Box -->
      <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
        <p class="text-indigo-400 text-sm flex items-start gap-2">
          <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span>
            Auto Recovery monitors your active strategy and automatically switches to a backup 
            when consecutive failures are detected. Backup strategies are selected from your 
            previously successful strategies.
          </span>
        </p>
      </div>
    </div>
  {/if}
</div>
