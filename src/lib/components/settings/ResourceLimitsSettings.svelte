<script lang="ts">
  /**
   * ResourceLimitsSettings Component
   * 
   * UI for configuring resource limits for child processes.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { browser } from '$app/environment';
  import Button from '$lib/components/Button.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import type { ResourceLimits, ResourceUsage, CpuPriority } from '$lib/api/resources';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let limits = $state<ResourceLimits>({
    memory_limit_mb: 0,
    cpu_priority: 'normal',
    max_connections: 0,
    monitoring_enabled: true
  });
  let usage = $state<ResourceUsage | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' } | null>(null);
  let isTauri = $state(false);
  let usageInterval = $state<ReturnType<typeof setInterval> | null>(null);

  // CPU priority options
  const cpuPriorityOptions: { value: CpuPriority; label: string; description: string }[] = [
    { value: 'idle', label: 'Idle', description: 'Lowest priority, runs only when system is idle' },
    { value: 'below_normal', label: 'Below Normal', description: 'Lower than normal priority' },
    { value: 'normal', label: 'Normal', description: 'Default system priority' },
    { value: 'above_normal', label: 'Above Normal', description: 'Higher than normal priority' },
    { value: 'high', label: 'High', description: 'High priority, may affect system responsiveness' },
    { value: 'realtime', label: 'Realtime', description: 'Highest priority (use with caution!)' }
  ];

  // Derived
  let memoryPercent = $derived(
    usage && limits.memory_limit_mb > 0 
      ? Math.min(100, (usage.memory_mb / limits.memory_limit_mb) * 100)
      : 0
  );

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadLimits();
      startUsagePolling();
    } else {
      loading = false;
    }

    return () => {
      if (usageInterval) {
        clearInterval(usageInterval);
      }
    };
  });

  async function loadLimits() {
    if (!browser || !isTauri) return;
    
    loading = true;
    try {
      const { getResourceLimits, getResourceUsage } = await import('$lib/api/resources');
      limits = await getResourceLimits();
      usage = await getResourceUsage();
    } catch (e) {
      console.error('Failed to load resource limits:', e);
      showMessage('Failed to load settings', 'error');
    } finally {
      loading = false;
    }
  }

  function startUsagePolling() {
    usageInterval = setInterval(async () => {
      if (!isTauri || !limits.monitoring_enabled) return;
      try {
        const { getResourceUsage } = await import('$lib/api/resources');
        usage = await getResourceUsage();
      } catch (e) {
        // Silently ignore polling errors
      }
    }, 2000);
  }

  async function handleSave() {
    if (!browser || !isTauri || saving) return;
    
    saving = true;
    message = null;
    
    try {
      const { saveResourceLimits } = await import('$lib/api/resources');
      await saveResourceLimits(limits);
      showMessage('Settings saved successfully', 'success');
    } catch (e) {
      console.error('Failed to save resource limits:', e);
      showMessage(`Failed to save: ${e}`, 'error');
    } finally {
      saving = false;
    }
  }

  async function handleReset() {
    if (!browser || !isTauri) return;
    
    try {
      const { resetResourceLimits, getResourceLimits } = await import('$lib/api/resources');
      await resetResourceLimits();
      limits = await getResourceLimits();
      showMessage('Reset to defaults', 'success');
    } catch (e) {
      console.error('Failed to reset resource limits:', e);
      showMessage(`Failed to reset: ${e}`, 'error');
    }
  }

  function showMessage(text: string, type: 'success' | 'error') {
    message = { text, type };
    setTimeout(() => { message = null; }, 3000);
  }

  function formatUptime(secs: number): string {
    if (secs < 60) return `${secs}s`;
    if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
    const hours = Math.floor(secs / 3600);
    const mins = Math.floor((secs % 3600) / 60);
    return `${hours}h ${mins}m`;
  }
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-text-primary">Resource Limits</h2>
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
      <!-- Current Usage Stats -->
      {#if usage && limits.monitoring_enabled}
        <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
          <p class="text-text-primary font-medium mb-4">Current Usage</p>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="text-center">
              <p class="text-2xl font-bold text-indigo-400">{usage.memory_mb.toFixed(1)}</p>
              <p class="text-text-muted text-sm">Memory (MB)</p>
            </div>
            <div class="text-center">
              <p class="text-2xl font-bold text-emerald-400">{usage.cpu_percent.toFixed(1)}%</p>
              <p class="text-text-muted text-sm">CPU</p>
            </div>
            <div class="text-center">
              <p class="text-2xl font-bold text-amber-400">{usage.process_count}</p>
              <p class="text-text-muted text-sm">Processes</p>
            </div>
            <div class="text-center">
              <p class="text-2xl font-bold text-cyan-400">{formatUptime(usage.uptime_secs)}</p>
              <p class="text-text-muted text-sm">Uptime</p>
            </div>
          </div>
          
          {#if limits.memory_limit_mb > 0}
            <div class="mt-4">
              <div class="flex justify-between text-sm mb-1">
                <span class="text-text-muted">Memory Usage</span>
                <span class="text-text-secondary">{usage.memory_mb.toFixed(1)} / {limits.memory_limit_mb} MB</span>
              </div>
              <div class="h-2 bg-void-200 rounded-full overflow-hidden">
                <div 
                  class="h-full transition-all duration-300 {memoryPercent > 90 ? 'bg-red-500' : memoryPercent > 70 ? 'bg-amber-500' : 'bg-indigo-500'}"
                  style="width: {memoryPercent}%"
                ></div>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Memory Limit Slider -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <div class="flex items-center justify-between mb-4">
          <div>
            <label for="memory-limit-slider" class="text-text-primary font-medium block">Memory Limit</label>
            <p id="memory-limit-hint" class="text-text-secondary text-sm">Maximum memory for child processes</p>
          </div>
          <span class="text-indigo-400 font-mono" aria-live="polite">
            {limits.memory_limit_mb === 0 ? 'Unlimited' : `${limits.memory_limit_mb} MB`}
          </span>
        </div>
        <input
          id="memory-limit-slider"
          type="range"
          min="0"
          max="1024"
          step="64"
          bind:value={limits.memory_limit_mb}
          aria-describedby="memory-limit-hint"
          aria-valuemin="0"
          aria-valuemax="1024"
          aria-valuenow={limits.memory_limit_mb}
          aria-valuetext={limits.memory_limit_mb === 0 ? 'Unlimited' : `${limits.memory_limit_mb} megabytes`}
          class="w-full h-2 bg-void-200 rounded-lg appearance-none cursor-pointer accent-indigo-500"
        />
        <div class="flex justify-between text-xs text-text-muted mt-2" aria-hidden="true">
          <span>Unlimited</span>
          <span>256 MB</span>
          <span>512 MB</span>
          <span>1 GB</span>
        </div>
      </div>

      <!-- CPU Priority -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <label for="cpu-priority-select" class="text-text-primary font-medium mb-4 block">CPU Priority</label>
        <select
          id="cpu-priority-select"
          bind:value={limits.cpu_priority}
          aria-label="Select CPU priority for child processes"
          class="w-full bg-void-200 text-text-primary rounded-lg px-4 py-3 border border-glass-border focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/20 focus:outline-none cursor-pointer"
        >
          {#each cpuPriorityOptions as option}
            <option value={option.value}>{option.label} - {option.description}</option>
          {/each}
        </select>
        
        {#if limits.cpu_priority === 'realtime'}
          <div class="mt-3 p-3 bg-amber-500/10 rounded-lg border border-amber-500/20">
            <p class="text-amber-400 text-sm flex items-start gap-2">
              <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
              </svg>
              <span>Realtime priority can make your system unresponsive. Use only if you know what you're doing.</span>
            </p>
          </div>
        {/if}
      </div>

      <!-- Monitoring Toggle -->
      <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
        <div id="resource-monitoring-label">
          <p class="text-text-primary font-medium">Resource Monitoring</p>
          <p class="text-text-secondary text-sm">Track memory and CPU usage in real-time</p>
        </div>
        <Toggle 
          checked={limits.monitoring_enabled}
          onchange={(checked) => limits.monitoring_enabled = checked}
          aria-labelledby="resource-monitoring-label"
        />
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-3 pt-4" role="group" aria-label="Resource limits actions">
        <Button 
          variant="primary" 
          onclick={handleSave}
          loading={saving}
          disabled={saving}
          aria-label="Apply resource limit settings"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
          </svg>
          Apply
        </Button>
        <Button 
          variant="secondary" 
          onclick={handleReset}
          disabled={saving}
          aria-label="Reset resource limits to default values"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          Reset to Defaults
        </Button>
      </div>

      <!-- Info Box -->
      <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
        <p class="text-indigo-400 text-sm flex items-start gap-2">
          <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span>Resource limits apply to winws and sing-box processes. Setting memory limits too low may cause connection issues.</span>
        </p>
      </div>
    </div>
  {/if}
</div>
