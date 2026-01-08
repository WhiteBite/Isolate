<script lang="ts">
  /**
   * AutoRecoverySettings Component
   * 
   * UI for configuring automatic failover/recovery between strategies.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { browser } from '$app/environment';
  import Toggle from '$lib/components/Toggle.svelte';
  import type { 
    FailoverConfig, 
    FailoverStrategyInfo,
    FailoverOperationResult 
  } from '$lib/api/failover';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let enabled = $state(false);
  let primaryStrategyId = $state<string | null>(null);
  let backupStrategyIds = $state<string[]>([]);
  let maxFailures = $state(3);
  let cooldownMinutes = $state(5);
  
  let strategies = $state<FailoverStrategyInfo[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let testing = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' | 'info' } | null>(null);
  let isTauri = $state(false);
  let draggedIndex = $state<number | null>(null);

  // Derived
  let availableForBackup = $derived(
    strategies.filter(s => s.id !== primaryStrategyId && s.is_available)
  );
  
  let selectedBackups = $derived(
    backupStrategyIds
      .map(id => strategies.find(s => s.id === id))
      .filter((s): s is FailoverStrategyInfo => s !== undefined)
  );

  let hasChanges = $derived(false); // TODO: track changes

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadSettings();
    } else {
      loading = false;
    }
  });

  async function loadSettings() {
    if (!browser || !isTauri) return;
    
    loading = true;
    try {
      const { getFailoverConfig, getFailoverStrategies } = await import('$lib/api/failover');
      
      const [config, strats] = await Promise.all([
        getFailoverConfig(),
        getFailoverStrategies()
      ]);
      
      strategies = strats;
      enabled = config.enabled;
      primaryStrategyId = config.primary_strategy_id;
      backupStrategyIds = config.backup_strategy_ids;
      maxFailures = config.max_failures;
      cooldownMinutes = Math.round(config.cooldown_secs / 60);
    } catch (e) {
      console.error('Failed to load failover settings:', e);
      showMessage('Failed to load settings', 'error');
    } finally {
      loading = false;
    }
  }

  async function saveSettings() {
    if (!browser || !isTauri || saving) return;
    
    saving = true;
    message = null;
    
    try {
      const { configureFailover } = await import('$lib/api/failover');
      
      const config: FailoverConfig = {
        enabled,
        primary_strategy_id: primaryStrategyId,
        backup_strategy_ids: backupStrategyIds,
        max_failures: maxFailures,
        cooldown_secs: cooldownMinutes * 60
      };
      
      const result = await configureFailover(config);
      
      if (result.success) {
        showMessage('Settings saved successfully', 'success');
      } else {
        showMessage(result.message || 'Failed to save settings', 'error');
      }
    } catch (e) {
      console.error('Failed to save failover settings:', e);
      showMessage(`Failed to save: ${e}`, 'error');
    } finally {
      saving = false;
    }
  }

  async function handleTestFailover() {
    if (!browser || !isTauri || testing) return;
    
    testing = true;
    message = null;
    
    try {
      const { testFailover } = await import('$lib/api/failover');
      const result = await testFailover();
      
      if (result.success) {
        showMessage(`Test passed: ${result.message}`, 'success');
      } else {
        showMessage(`Test failed: ${result.message}`, 'error');
      }
    } catch (e) {
      console.error('Failover test failed:', e);
      showMessage(`Test error: ${e}`, 'error');
    } finally {
      testing = false;
    }
  }

  function handleToggleEnabled(value: boolean) {
    enabled = value;
  }

  function handlePrimaryChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    primaryStrategyId = select.value || null;
    // Remove from backups if selected as primary
    if (primaryStrategyId) {
      backupStrategyIds = backupStrategyIds.filter(id => id !== primaryStrategyId);
    }
  }

  function handleBackupToggle(strategyId: string) {
    if (backupStrategyIds.includes(strategyId)) {
      backupStrategyIds = backupStrategyIds.filter(id => id !== strategyId);
    } else {
      backupStrategyIds = [...backupStrategyIds, strategyId];
    }
  }

  function handleMaxFailuresChange(e: Event) {
    const input = e.target as HTMLInputElement;
    maxFailures = Math.max(1, Math.min(10, parseInt(input.value) || 3));
  }

  function handleCooldownChange(e: Event) {
    const input = e.target as HTMLInputElement;
    cooldownMinutes = Math.max(1, Math.min(60, parseInt(input.value) || 5));
  }

  // Drag and drop handlers for backup priority
  function handleDragStart(e: DragEvent, index: number) {
    draggedIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === index) return;
    
    const newOrder = [...backupStrategyIds];
    const [removed] = newOrder.splice(draggedIndex, 1);
    newOrder.splice(index, 0, removed);
    backupStrategyIds = newOrder;
    draggedIndex = index;
  }

  function handleDragEnd() {
    draggedIndex = null;
  }

  function showMessage(text: string, type: 'success' | 'error' | 'info') {
    message = { text, type };
    setTimeout(() => { message = null; }, 4000);
  }

  function getStrategyIcon(family: string): string {
    switch (family.toLowerCase()) {
      case 'zapret': return '⚡';
      case 'vless': return '🔒';
      case 'proxy': return '🌐';
      default: return '📦';
    }
  }
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-text-primary">Auto-Recovery</h2>
    {#if message}
      <span class="text-sm animate-pulse {message.type === 'error' ? 'text-red-400' : message.type === 'success' ? 'text-emerald-400' : 'text-indigo-400'}">
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
    <div class="space-y-6">
      <!-- Enable Toggle -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-text-primary font-medium">Enable Auto-Recovery</p>
            <p class="text-text-secondary text-sm">
              Automatically switch to backup strategy when primary fails
            </p>
          </div>
          <Toggle 
            checked={enabled}
            onchange={handleToggleEnabled}
            disabled={saving}
          />
        </div>
      </div>

      {#if enabled}
        <!-- Primary Strategy Selection -->
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <label class="block">
            <span class="text-text-primary font-medium mb-2 block">Primary Strategy</span>
            <select
              value={primaryStrategyId ?? ''}
              onchange={handlePrimaryChange}
              disabled={saving}
              class="w-full px-4 py-2.5 bg-zinc-900/50 border border-white/10 rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
            >
              <option value="">Select primary strategy...</option>
              {#each strategies.filter(s => s.is_available) as strategy}
                <option value={strategy.id}>
                  {getStrategyIcon(strategy.family)} {strategy.name}
                </option>
              {/each}
            </select>
          </label>
        </div>

        <!-- Backup Strategies Selection -->
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <div class="mb-3">
            <p class="text-text-primary font-medium">Backup Strategies</p>
            <p class="text-text-secondary text-sm">
              Select and drag to set priority order
            </p>
          </div>
          
          <!-- Selected backups with drag-n-drop -->
          {#if selectedBackups.length > 0}
            <div class="mb-4 space-y-2">
              <p class="text-xs text-zinc-400 uppercase tracking-wider">Priority Order</p>
              {#each selectedBackups as backup, index}
                <div
                  draggable="true"
                  ondragstart={(e) => handleDragStart(e, index)}
                  ondragover={(e) => handleDragOver(e, index)}
                  ondragend={handleDragEnd}
                  class="flex items-center gap-3 p-3 bg-zinc-900/50 rounded-lg border border-white/10 cursor-move hover:border-indigo-500/30 transition-colors {draggedIndex === index ? 'opacity-50' : ''}"
                >
                  <span class="text-zinc-400 font-mono text-sm w-6">{index + 1}.</span>
                  <svg class="w-4 h-4 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16"/>
                  </svg>
                  <span class="text-lg">{getStrategyIcon(backup.family)}</span>
                  <span class="text-text-primary flex-1">{backup.name}</span>
                  <button
                    onclick={() => handleBackupToggle(backup.id)}
                    class="p-1 text-zinc-400 hover:text-red-400 transition-colors"
                    title="Remove from backups"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                    </svg>
                  </button>
                </div>
              {/each}
            </div>
          {/if}

          <!-- Available strategies to add -->
          <div class="space-y-2">
            <p class="text-xs text-zinc-400 uppercase tracking-wider">Available Strategies</p>
            {#each availableForBackup as strategy}
              {#if !backupStrategyIds.includes(strategy.id)}
                <button
                  onclick={() => handleBackupToggle(strategy.id)}
                  disabled={saving}
                  class="w-full flex items-center gap-3 p-3 bg-zinc-900/30 rounded-lg border border-white/5 hover:border-indigo-500/30 hover:bg-zinc-900/50 transition-colors text-left"
                >
                  <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                  </svg>
                  <span class="text-lg">{getStrategyIcon(strategy.family)}</span>
                  <span class="text-text-secondary">{strategy.name}</span>
                </button>
              {/if}
            {/each}
            {#if availableForBackup.filter(s => !backupStrategyIds.includes(s.id)).length === 0}
              <p class="text-zinc-400 text-sm text-center py-2">No more strategies available</p>
            {/if}
          </div>
        </div>

        <!-- Settings -->
        <div class="grid grid-cols-2 gap-4">
          <!-- Max Failures -->
          <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
            <label class="block">
              <span class="text-text-primary font-medium mb-1 block">Max Failures</span>
              <span class="text-text-secondary text-xs mb-2 block">Before switching to backup</span>
              <input
                type="number"
                min="1"
                max="10"
                value={maxFailures}
                oninput={handleMaxFailuresChange}
                disabled={saving}
                class="w-full px-4 py-2.5 bg-zinc-900/50 border border-white/10 rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
              />
            </label>
          </div>

          <!-- Cooldown -->
          <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
            <label class="block">
              <span class="text-text-primary font-medium mb-1 block">Cooldown</span>
              <span class="text-text-secondary text-xs mb-2 block">Minutes before restore attempt</span>
              <input
                type="number"
                min="1"
                max="60"
                value={cooldownMinutes}
                oninput={handleCooldownChange}
                disabled={saving}
                class="w-full px-4 py-2.5 bg-zinc-900/50 border border-white/10 rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
              />
            </label>
          </div>
        </div>

        <!-- Info Box -->
        <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
          <p class="text-indigo-400 text-sm flex items-start gap-2">
            <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
            <span>
              When the primary strategy fails <strong>{maxFailures} times</strong>, 
              the system will automatically switch to the first available backup. 
              After <strong>{cooldownMinutes} minutes</strong>, it will attempt to restore the primary.
            </span>
          </p>
        </div>
      {/if}

      <!-- Action Buttons -->
      <div class="flex items-center gap-3 pt-2">
        <button
          onclick={saveSettings}
          disabled={saving}
          class="flex-1 px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 disabled:bg-indigo-500/50 text-white font-medium rounded-lg transition-colors flex items-center justify-center gap-2"
        >
          {#if saving}
            <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Saving...
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
            </svg>
            Save Settings
          {/if}
        </button>
        
        {#if enabled}
          <button
            onclick={handleTestFailover}
            disabled={testing || !primaryStrategyId || backupStrategyIds.length === 0}
            class="px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 disabled:bg-zinc-800/50 disabled:text-zinc-400 text-text-primary font-medium rounded-lg transition-colors flex items-center gap-2"
          >
            {#if testing}
              <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Testing...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
              Test Failover
            {/if}
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
