<script lang="ts">
  /**
   * StrategyCompositionSettings Component
   * 
   * UI for configuring strategy composition rules (Service → Strategy mapping).
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { browser } from '$app/environment';
  import Button from '$lib/components/Button.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import type { CompositionConfig, CompositionRule } from '$lib/api/composition';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let config = $state<CompositionConfig>({
    enabled: false,
    rules: [],
    fallback_strategy_id: null
  });
  let loading = $state(true);
  let saving = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' } | null>(null);
  let isTauri = $state(false);
  
  // Available services and strategies for dropdowns
  let availableServices = $state<{ id: string; name: string }[]>([]);
  let availableStrategies = $state<{ id: string; name: string }[]>([]);
  
  // Add rule modal state
  let showAddModal = $state(false);
  let newRule = $state<{ service_id: string; strategy_id: string }>({
    service_id: '',
    strategy_id: ''
  });
  
  // Drag state
  let draggedIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  // Derived
  let enabledRulesCount = $derived(config.rules.filter(r => r.enabled).length);
  let fallbackStrategyName = $derived(
    availableStrategies.find(s => s.id === config.fallback_strategy_id)?.name ?? 'None'
  );

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadData();
    } else {
      loading = false;
    }
  });

  async function loadData() {
    if (!browser || !isTauri) return;
    
    loading = true;
    try {
      const { getCompositionConfig } = await import('$lib/api/composition');
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Load composition config
      config = await getCompositionConfig();
      
      // Load available services and strategies
      try {
        availableServices = await invoke<{ id: string; name: string }[]>('get_services_list');
      } catch {
        availableServices = [];
      }
      
      try {
        availableStrategies = await invoke<{ id: string; name: string }[]>('get_strategies_list');
      } catch {
        availableStrategies = [];
      }
    } catch (e) {
      console.error('Failed to load composition config:', e);
      showMessage('Failed to load settings', 'error');
    } finally {
      loading = false;
    }
  }

  async function handleToggleEnabled(enabled: boolean) {
    if (!browser || !isTauri) return;
    
    try {
      const { toggleComposition } = await import('$lib/api/composition');
      await toggleComposition(enabled);
      config.enabled = enabled;
    } catch (e) {
      console.error('Failed to toggle composition:', e);
      showMessage(`Failed to toggle: ${e}`, 'error');
    }
  }

  async function handleAddRule() {
    if (!browser || !isTauri || !newRule.service_id || !newRule.strategy_id) return;
    
    saving = true;
    try {
      const { addCompositionRule } = await import('$lib/api/composition');
      
      const service = availableServices.find(s => s.id === newRule.service_id);
      const strategy = availableStrategies.find(s => s.id === newRule.strategy_id);
      
      const rule = await addCompositionRule({
        service_id: newRule.service_id,
        service_name: service?.name ?? newRule.service_id,
        strategy_id: newRule.strategy_id,
        strategy_name: strategy?.name ?? newRule.strategy_id,
        priority: config.rules.length,
        enabled: true
      });
      
      config.rules = [...config.rules, rule];
      showAddModal = false;
      newRule = { service_id: '', strategy_id: '' };
      showMessage('Rule added', 'success');
    } catch (e) {
      console.error('Failed to add rule:', e);
      showMessage(`Failed to add rule: ${e}`, 'error');
    } finally {
      saving = false;
    }
  }

  async function handleDeleteRule(ruleId: string) {
    if (!browser || !isTauri) return;
    
    try {
      const { deleteCompositionRule } = await import('$lib/api/composition');
      await deleteCompositionRule(ruleId);
      config.rules = config.rules.filter(r => r.id !== ruleId);
      showMessage('Rule deleted', 'success');
    } catch (e) {
      console.error('Failed to delete rule:', e);
      showMessage(`Failed to delete: ${e}`, 'error');
    }
  }

  async function handleToggleRule(rule: CompositionRule) {
    if (!browser || !isTauri) return;
    
    try {
      const { updateCompositionRule } = await import('$lib/api/composition');
      const updatedRule = { ...rule, enabled: !rule.enabled };
      await updateCompositionRule(updatedRule);
      config.rules = config.rules.map(r => r.id === rule.id ? updatedRule : r);
    } catch (e) {
      console.error('Failed to toggle rule:', e);
      showMessage(`Failed to toggle rule: ${e}`, 'error');
    }
  }

  async function handleSetFallback(strategyId: string | null) {
    if (!browser || !isTauri) return;
    
    try {
      const { setFallbackStrategy } = await import('$lib/api/composition');
      await setFallbackStrategy(strategyId);
      config.fallback_strategy_id = strategyId;
      showMessage('Fallback strategy updated', 'success');
    } catch (e) {
      console.error('Failed to set fallback:', e);
      showMessage(`Failed to set fallback: ${e}`, 'error');
    }
  }

  // Drag and drop handlers
  function handleDragStart(index: number) {
    draggedIndex = index;
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  async function handleDrop(targetIndex: number) {
    if (draggedIndex === null || draggedIndex === targetIndex) {
      draggedIndex = null;
      dragOverIndex = null;
      return;
    }

    // Reorder locally
    const rules = [...config.rules];
    const [removed] = rules.splice(draggedIndex, 1);
    rules.splice(targetIndex, 0, removed);
    
    // Update priorities
    const reorderedRules = rules.map((r, i) => ({ ...r, priority: i }));
    config.rules = reorderedRules;
    
    draggedIndex = null;
    dragOverIndex = null;

    // Save to backend
    if (isTauri) {
      try {
        const { reorderCompositionRules } = await import('$lib/api/composition');
        await reorderCompositionRules(reorderedRules.map(r => r.id));
      } catch (e) {
        console.error('Failed to reorder rules:', e);
        showMessage(`Failed to reorder: ${e}`, 'error');
      }
    }
  }

  function handleDragEnd() {
    draggedIndex = null;
    dragOverIndex = null;
  }

  function showMessage(text: string, type: 'success' | 'error') {
    message = { text, type };
    setTimeout(() => { message = null; }, 3000);
  }
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-text-primary">Strategy Composition</h2>
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
      <!-- Enable Toggle -->
      <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
        <div id="enable-composition-label">
          <p class="text-text-primary font-medium">Enable Composition</p>
          <p class="text-text-secondary text-sm">Use different strategies for different services</p>
        </div>
        <Toggle 
          checked={config.enabled}
          onchange={handleToggleEnabled}
          aria-labelledby="enable-composition-label"
        />
      </div>

      <!-- Rules Table -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <div class="flex items-center justify-between mb-4">
          <div>
            <p class="text-text-primary font-medium">Composition Rules</p>
            <p class="text-text-secondary text-sm">{enabledRulesCount} active rules</p>
          </div>
          <Button 
            variant="primary" 
            size="sm"
            onclick={() => showAddModal = true}
            disabled={!config.enabled}
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
            </svg>
            Add Rule
          </Button>
        </div>

        {#if config.rules.length === 0}
          <div class="text-center py-8 text-text-muted">
            <svg class="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>
            </svg>
            <p>No rules configured</p>
            <p class="text-sm mt-1">Add rules to map services to specific strategies</p>
          </div>
        {:else}
          <div class="space-y-2">
            {#each config.rules as rule, index (rule.id)}
              <div
                draggable="true"
                ondragstart={() => handleDragStart(index)}
                ondragover={(e) => handleDragOver(e, index)}
                ondragleave={handleDragLeave}
                ondrop={() => handleDrop(index)}
                ondragend={handleDragEnd}
                class="flex items-center gap-3 p-3 rounded-lg transition-all duration-200 cursor-move
                  {dragOverIndex === index ? 'bg-indigo-500/20 border-indigo-500/50' : 'bg-void-200 hover:bg-void-300'}
                  {draggedIndex === index ? 'opacity-50' : ''}
                  {!rule.enabled ? 'opacity-60' : ''}
                  border border-transparent"
              >
                <!-- Drag Handle -->
                <div class="text-text-muted hover:text-text-secondary cursor-grab active:cursor-grabbing" aria-label="Drag to reorder rule" role="img">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8h16M4 16h16"/>
                  </svg>
                </div>

                <!-- Priority Badge -->
                <span class="w-6 h-6 flex items-center justify-center bg-void-100 rounded text-xs text-text-muted font-mono">
                  {index + 1}
                </span>

                <!-- Service → Strategy -->
                <div class="flex-1 flex items-center gap-2">
                  <span class="text-text-primary font-medium">{rule.service_name}</span>
                  <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"/>
                  </svg>
                  <span class="text-indigo-400">{rule.strategy_name}</span>
                </div>

                <!-- Toggle -->
                <Toggle 
                  checked={rule.enabled}
                  onchange={() => handleToggleRule(rule)}
                  aria-label="Toggle rule {rule.service_name} to {rule.strategy_name}"
                />

                <!-- Delete -->
                <button
                  onclick={() => handleDeleteRule(rule.id)}
                  class="p-1.5 text-text-muted hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
                  aria-label="Delete rule {rule.service_name} to {rule.strategy_name}"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                  </svg>
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Fallback Strategy -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <label for="fallback-strategy" class="text-text-primary font-medium mb-2 block">Fallback Strategy</label>
        <p id="fallback-strategy-hint" class="text-text-secondary text-sm mb-4">Strategy to use for services without specific rules</p>
        <select
          id="fallback-strategy"
          value={config.fallback_strategy_id ?? ''}
          onchange={(e) => handleSetFallback(e.currentTarget.value || null)}
          disabled={!config.enabled}
          aria-describedby="fallback-strategy-hint"
          class="w-full bg-void-200 text-text-primary rounded-lg px-4 py-3 border border-glass-border focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/20 focus:outline-none cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <option value="">None (use global strategy)</option>
          {#each availableStrategies as strategy}
            <option value={strategy.id}>{strategy.name}</option>
          {/each}
        </select>
      </div>

      <!-- Info Box -->
      <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
        <p class="text-indigo-400 text-sm flex items-start gap-2">
          <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span>Strategy composition allows you to use different bypass strategies for different services. Drag rules to change priority — higher rules take precedence.</span>
        </p>
      </div>
    </div>
  {/if}
</div>

<!-- Add Rule Modal -->
{#if showAddModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center" role="dialog" aria-modal="true" aria-labelledby="add-rule-modal-title">
    <!-- Backdrop -->
    <div 
      class="absolute inset-0 bg-black/60 backdrop-blur-sm"
      onclick={() => showAddModal = false}
      onkeydown={(e) => e.key === 'Escape' && (showAddModal = false)}
      role="button"
      tabindex="0"
      aria-label="Close modal"
    ></div>
    
    <!-- Modal -->
    <div class="relative bg-void-50 rounded-2xl border border-glass-border shadow-2xl w-full max-w-md p-6 m-4">
      <h3 id="add-rule-modal-title" class="text-lg font-semibold text-text-primary mb-4">Add Composition Rule</h3>
      
      <div class="space-y-4">
        <!-- Service Select -->
        <div>
          <label for="new-rule-service" class="block text-sm text-text-secondary mb-2">Service</label>
          <select
            id="new-rule-service"
            bind:value={newRule.service_id}
            class="w-full bg-void-200 text-text-primary rounded-lg px-4 py-3 border border-glass-border focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/20 focus:outline-none"
          >
            <option value="">Select a service...</option>
            {#each availableServices as service}
              <option value={service.id}>{service.name}</option>
            {/each}
          </select>
        </div>

        <!-- Strategy Select -->
        <div>
          <label for="new-rule-strategy" class="block text-sm text-text-secondary mb-2">Strategy</label>
          <select
            id="new-rule-strategy"
            bind:value={newRule.strategy_id}
            class="w-full bg-void-200 text-text-primary rounded-lg px-4 py-3 border border-glass-border focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/20 focus:outline-none"
          >
            <option value="">Select a strategy...</option>
            {#each availableStrategies as strategy}
              <option value={strategy.id}>{strategy.name}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex items-center justify-end gap-3 mt-6">
        <Button variant="secondary" onclick={() => showAddModal = false}>
          Cancel
        </Button>
        <Button 
          variant="primary" 
          onclick={handleAddRule}
          loading={saving}
          disabled={!newRule.service_id || !newRule.strategy_id || saving}
        >
          Add Rule
        </Button>
      </div>
    </div>
  </div>
{/if}
