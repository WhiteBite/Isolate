<script lang="ts">
  import { Modal, JsonEditor } from '$lib/components';
  import type { RoutingRule } from './types';
  import type { ProxyConfig } from '$lib/api/types';
  import { getProxyFlag, getProxyCountryName } from '$lib/utils/countries';

  interface Props {
    open: boolean;
    editingRule: RoutingRule | null;
    proxies: ProxyConfig[];
    onsave: (rule: RoutingRule, isNew: boolean) => void;
    onclose: () => void;
  }

  let { open = $bindable(false), editingRule, proxies, onsave, onclose }: Props = $props();

  // Form state
  let formName = $state('');
  let formSource = $state<'all' | 'app' | 'domain'>('domain');
  let formSourceValue = $state('');
  let formAction = $state<'direct' | 'proxy' | 'block'>('direct');
  let formProxyId = $state('');

  // Tab state
  let activeTab = $state<'form' | 'json'>('form');
  let jsonValue = $state('');
  let jsonSyncError = $state<string | null>(null);

  // Reset form when modal opens
  $effect(() => {
    if (open) {
      if (editingRule) {
        formName = editingRule.name;
        formSource = editingRule.source;
        formSourceValue = editingRule.sourceValue || '';
        formAction = editingRule.action;
        formProxyId = editingRule.proxyId || proxies[0]?.id || '';
      } else {
        formName = '';
        formSource = 'domain';
        formSourceValue = '';
        formAction = 'direct';
        formProxyId = proxies[0]?.id || '';
      }
      activeTab = 'form';
      jsonValue = '';
      jsonSyncError = null;
    }
  });

  // Wildcard validation
  function isValidWildcard(pattern: string): boolean {
    if (!pattern || pattern.trim() === '') return false;
    const trimmed = pattern.trim();
    
    if (trimmed.startsWith('*')) {
      return /^\*\.[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)+$/.test(trimmed);
    }
    
    return /^[a-zA-Z0-9][-a-zA-Z0-9]*(\.[a-zA-Z0-9][-a-zA-Z0-9]*)*$/.test(trimmed);
  }

  function getValidationError(source: string, value: string): string | null {
    if (source === 'all') return null;
    if (!value.trim()) return 'Enter a value';
    
    if (source === 'domain') {
      if (!isValidWildcard(value)) {
        if (value.startsWith('*') && !value.startsWith('*.')) {
          return 'Wildcard must be in format *.domain.com';
        }
        if (value.endsWith('.*')) {
          return 'Wildcard TLD is not supported';
        }
        return 'Invalid domain format';
      }
    }
    
    if (source === 'app') {
      if (!/^[\w\-\.]+\.(exe|app|bin)?$/i.test(value) && !/^[\w\-\.]+$/i.test(value)) {
        return 'Invalid application name';
      }
    }
    
    return null;
  }

  let formValidationError = $derived(getValidationError(formSource, formSourceValue));

  // Convert form to JSON
  function formToJson(): string {
    const rule: Partial<RoutingRule> = {
      name: formName,
      enabled: editingRule?.enabled ?? true,
      source: formSource,
      action: formAction,
    };
    if (formSource !== 'all') {
      rule.sourceValue = formSourceValue;
    }
    if (formAction === 'proxy') {
      rule.proxyId = formProxyId;
    }
    return JSON.stringify(rule, null, 2);
  }

  // Convert JSON to form
  function jsonToForm(json: string): boolean {
    try {
      const parsed = JSON.parse(json);
      if (typeof parsed.name === 'string') formName = parsed.name;
      if (['all', 'app', 'domain'].includes(parsed.source)) formSource = parsed.source;
      if (typeof parsed.sourceValue === 'string') formSourceValue = parsed.sourceValue;
      if (['direct', 'proxy', 'block'].includes(parsed.action)) formAction = parsed.action;
      if (typeof parsed.proxyId === 'string') formProxyId = parsed.proxyId;
      jsonSyncError = null;
      return true;
    } catch (e) {
      jsonSyncError = (e as Error).message;
      return false;
    }
  }

  // Sync JSON when switching tabs
  function handleTabChange(tab: 'form' | 'json') {
    if (tab === 'json') {
      jsonValue = formToJson();
    } else if (activeTab === 'json') {
      jsonToForm(jsonValue);
    }
    activeTab = tab;
  }

  // Handle JSON editor changes
  function handleJsonChange(value: string) {
    jsonValue = value;
    try {
      const parsed = JSON.parse(value);
      if (typeof parsed.name === 'string') formName = parsed.name;
      if (['all', 'app', 'domain'].includes(parsed.source)) formSource = parsed.source;
      if (typeof parsed.sourceValue === 'string') formSourceValue = parsed.sourceValue;
      if (['direct', 'proxy', 'block'].includes(parsed.action)) formAction = parsed.action;
      if (typeof parsed.proxyId === 'string') formProxyId = parsed.proxyId;
      jsonSyncError = null;
    } catch {
      // Invalid JSON, don't sync
    }
  }

  function getActionIcon(action: string): string {
    switch (action) {
      case 'direct': return '🌐';
      case 'proxy': return '🔒';
      case 'block': return '🚫';
      default: return '❓';
    }
  }

  function getSourceIcon(source: string): string {
    switch (source) {
      case 'all': return '🌍';
      case 'app': return '📱';
      case 'domain': return '🔗';
      default: return '❓';
    }
  }

  function getProxyName(proxyId?: string): string {
    if (!proxyId) return '';
    return proxies.find(p => p.id === proxyId)?.name || proxyId;
  }

  function handleSave() {
    if (!formName.trim()) return;
    if (formSource !== 'all' && !formSourceValue.trim()) return;
    
    const validationError = getValidationError(formSource, formSourceValue);
    if (validationError) return;
    
    if (formAction === 'proxy' && !formProxyId) return;

    const isNew = !editingRule;
    const newRule: RoutingRule = {
      id: editingRule?.id || `rule-${Date.now()}`,
      name: formName.trim(),
      enabled: editingRule?.enabled ?? true,
      source: formSource,
      sourceValue: formSource === 'all' ? undefined : formSourceValue.trim(),
      action: formAction,
      proxyId: formAction === 'proxy' ? formProxyId : undefined,
    };

    onsave(newRule, isNew);
    open = false;
  }

  function handleClose() {
    open = false;
    onclose();
  }
</script>

<Modal bind:open title={editingRule ? 'Edit Rule' : 'Add Routing Rule'}>
  <!-- Tabs -->
  <div class="flex gap-1 p-1 bg-zinc-800/40 rounded-xl mb-5">
    <button
      type="button"
      onclick={() => handleTabChange('form')}
      class="flex-1 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200
             {activeTab === 'form' 
               ? 'bg-zinc-700/60 text-white' 
               : 'text-zinc-400 hover:text-zinc-300'}"
    >
      <span class="flex items-center justify-center gap-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        Form
      </span>
    </button>
    <button
      type="button"
      onclick={() => handleTabChange('json')}
      class="flex-1 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200
             {activeTab === 'json' 
               ? 'bg-zinc-700/60 text-white' 
               : 'text-zinc-400 hover:text-zinc-300'}"
    >
      <span class="flex items-center justify-center gap-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
        </svg>
        JSON
      </span>
    </button>
  </div>

  {#if activeTab === 'form'}
  <form onsubmit={(e) => { e.preventDefault(); handleSave(); }} class="space-y-5">
    <!-- Rule Name -->
    <div>
      <label for="rule-name" class="block text-sm font-medium text-zinc-400 mb-2">Rule Name</label>
      <input 
        id="rule-name"
        type="text" 
        bind:value={formName}
        placeholder="e.g., YouTube Direct"
        class="w-full px-4 py-3 bg-zinc-800/60 border border-white/10 rounded-xl
               text-white placeholder-zinc-500
               focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20
               transition-all duration-200"
      />
    </div>

    <!-- Source Type -->
    <div>
      <span id="traffic-source-label" class="block text-sm font-medium text-zinc-400 mb-2">Traffic Source</span>
      <div class="grid grid-cols-3 gap-2" role="group" aria-labelledby="traffic-source-label">
        {#each [
          { value: 'all', label: 'All Traffic', icon: '🌍' },
          { value: 'domain', label: 'Domain', icon: '🔗' },
          { value: 'app', label: 'Application', icon: '📱' }
        ] as option}
          <button
            type="button"
            onclick={() => formSource = option.value as 'all' | 'app' | 'domain'}
            class="p-3 rounded-xl border text-center transition-all duration-200
                   {formSource === option.value 
                     ? 'bg-indigo-500/10 border-indigo-500/30 text-indigo-400' 
                     : 'bg-zinc-800/40 border-white/5 text-zinc-400 hover:border-white/10'}"
          >
            <div class="text-xl mb-1">{option.icon}</div>
            <div class="text-xs font-medium">{option.label}</div>
          </button>
        {/each}
      </div>
    </div>

    <!-- Source Value -->
    {#if formSource !== 'all'}
      <div>
        <label for="source-value" class="block text-sm font-medium text-zinc-400 mb-2">
          {formSource === 'domain' ? 'Domain Pattern' : 'Application Name'}
        </label>
        <input 
          id="source-value"
          type="text" 
          bind:value={formSourceValue}
          placeholder={formSource === 'domain' ? 'e.g., youtube.com or *.google.com' : 'e.g., chrome.exe'}
          class="w-full px-4 py-3 bg-zinc-800/60 border rounded-xl
                 text-white placeholder-zinc-500 font-mono
                 focus:outline-none focus:ring-1 transition-all duration-200
                 {formValidationError && formSourceValue.trim() 
                   ? 'border-red-500/50 focus:border-red-500/50 focus:ring-red-500/20' 
                   : 'border-white/10 focus:border-indigo-500/50 focus:ring-indigo-500/20'}"
        />
        {#if formSource === 'domain'}
          {#if formValidationError && formSourceValue.trim()}
            <p class="mt-1.5 text-xs text-red-400">{formValidationError}</p>
          {:else}
            <p class="mt-1.5 text-xs text-zinc-400">Use *.domain.com for wildcard matching</p>
          {/if}
        {/if}
      </div>
    {/if}

    <!-- Action -->
    <div>
      <span id="action-label" class="block text-sm font-medium text-zinc-400 mb-2">Action</span>
      <div class="space-y-2" role="group" aria-labelledby="action-label">
        {#each [
          { value: 'direct', label: 'Direct Connection', desc: 'Connect directly without proxy', icon: '🌐', color: 'emerald' },
          { value: 'proxy', label: 'Use Proxy', desc: 'Route through selected proxy', icon: '🔒', color: 'indigo' },
          { value: 'block', label: 'Block', desc: 'Block all matching traffic', icon: '🚫', color: 'red' }
        ] as option}
          <label 
            class="flex items-center gap-4 p-4 rounded-xl border cursor-pointer transition-all duration-200
                   {formAction === option.value 
                     ? (option.color === 'emerald' ? 'bg-emerald-500/10 border-emerald-500/30' : 
                        option.color === 'indigo' ? 'bg-indigo-500/10 border-indigo-500/30' : 
                        'bg-red-500/10 border-red-500/30')
                     : 'bg-zinc-800/40 border-white/5 hover:border-white/10'}"
          >
            <input 
              type="radio" 
              name="action" 
              value={option.value}
              bind:group={formAction}
              class="sr-only"
            />
            <div class="w-10 h-10 rounded-lg flex items-center justify-center text-xl
                        {option.color === 'emerald' ? 'bg-emerald-500/20' : ''}
                        {option.color === 'indigo' ? 'bg-indigo-500/20' : ''}
                        {option.color === 'red' ? 'bg-red-500/20' : ''}">
              {option.icon}
            </div>
            <div class="flex-1">
              <div class="text-sm font-medium text-white">{option.label}</div>
              <div class="text-xs text-zinc-400">{option.desc}</div>
            </div>
            <div class="w-5 h-5 rounded-full border-2 flex items-center justify-center
                        {formAction === option.value 
                          ? (option.color === 'emerald' ? 'border-emerald-400' : 
                             option.color === 'indigo' ? 'border-indigo-400' : 'border-red-400')
                          : 'border-zinc-600'}">
              {#if formAction === option.value}
                <div class="w-2.5 h-2.5 rounded-full
                            {option.color === 'emerald' ? 'bg-emerald-400' : ''}
                            {option.color === 'indigo' ? 'bg-indigo-400' : ''}
                            {option.color === 'red' ? 'bg-red-400' : ''}">
                </div>
              {/if}
            </div>
          </label>
        {/each}
      </div>
    </div>

    <!-- Proxy Selection (if action is proxy) -->
    {#if formAction === 'proxy'}
      <div>
        <span id="proxy-label" class="block text-sm font-medium text-zinc-400 mb-2">Select Proxy</span>
        {#if proxies.length === 0}
          <div class="p-4 bg-amber-500/10 border border-amber-500/20 rounded-xl text-center">
            <p class="text-sm text-amber-400">No proxies configured</p>
            <p class="text-xs text-zinc-400 mt-1">Add a proxy in the Proxies section first</p>
          </div>
        {:else}
          <div class="space-y-2" role="group" aria-labelledby="proxy-label">
            {#each proxies as proxy (proxy.id)}
              <label 
                class="flex items-center gap-3 p-3 rounded-xl border cursor-pointer transition-all duration-200
                       {formProxyId === proxy.id 
                         ? 'bg-indigo-500/10 border-indigo-500/30' 
                         : 'bg-zinc-800/40 border-white/5 hover:border-white/10'}"
              >
                <input 
                  type="radio" 
                  name="proxy" 
                  value={proxy.id}
                  bind:group={formProxyId}
                  class="sr-only"
                />
                <span class="text-xl flex-shrink-0" title={getProxyCountryName(proxy.country, proxy.server)}>{getProxyFlag(proxy.country, proxy.server)}</span>
                <div class="flex-1">
                  <div class="text-sm font-medium text-white">{proxy.name}</div>
                  <div class="text-xs text-zinc-400 uppercase">{proxy.protocol}</div>
                </div>
                <div class="w-4 h-4 rounded-full border-2 flex items-center justify-center
                            {formProxyId === proxy.id ? 'border-indigo-400' : 'border-zinc-600'}">
                  {#if formProxyId === proxy.id}
                    <div class="w-2 h-2 rounded-full bg-indigo-400"></div>
                  {/if}
                </div>
              </label>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Preview -->
    <div class="p-4 bg-zinc-900/60 border border-white/5 rounded-xl">
      <div class="text-xs text-zinc-400 uppercase tracking-wider mb-3">Preview</div>
      <div class="flex items-center gap-2 text-sm">
        <span class="text-lg">{getSourceIcon(formSource)}</span>
        <span class="font-mono text-white">
          {formSource === 'all' ? 'All Traffic' : (formSourceValue || '...')}
        </span>
        <svg class="w-4 h-4 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
        </svg>
        <span class="px-2 py-1 rounded-lg text-xs font-medium
                     {formAction === 'direct' ? 'bg-emerald-500/20 text-emerald-400' : ''}
                     {formAction === 'proxy' ? 'bg-indigo-500/20 text-indigo-400' : ''}
                     {formAction === 'block' ? 'bg-red-500/20 text-red-400' : ''}">
          {getActionIcon(formAction)}
          {formAction === 'direct' ? 'Direct' : formAction === 'proxy' ? getProxyName(formProxyId) || 'Proxy' : 'Block'}
        </span>
      </div>
    </div>

    <!-- Buttons -->
    <div class="flex gap-3 pt-2">
      <button
        type="button"
        onclick={handleClose}
        class="flex-1 px-4 py-3 bg-zinc-800/60 border border-white/10 rounded-xl
               text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
      >
        Cancel
      </button>
      <button
        type="submit"
        class="flex-1 px-4 py-3 bg-indigo-500 rounded-xl
               text-white font-medium text-sm hover:bg-indigo-600
               transition-all duration-200 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-indigo-500/20"
      >
        {editingRule ? 'Save Changes' : 'Add Rule'}
      </button>
    </div>
  </form>
  {:else}
  <!-- JSON Tab -->
  <div class="space-y-5">
    <JsonEditor 
      bind:value={jsonValue} 
      onchange={handleJsonChange}
      height="350px"
    />
    
    {#if jsonSyncError}
      <div class="p-3 bg-amber-500/10 border border-amber-500/20 rounded-xl">
        <p class="text-xs text-amber-400">
          ⚠️ JSON has errors. Fix them before saving or switch to Form tab.
        </p>
      </div>
    {/if}

    <!-- Buttons -->
    <div class="flex gap-3 pt-2">
      <button
        type="button"
        onclick={handleClose}
        class="flex-1 px-4 py-3 bg-zinc-800/60 border border-white/10 rounded-xl
               text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={() => { handleTabChange('form'); handleSave(); }}
        disabled={!!jsonSyncError}
        class="flex-1 px-4 py-3 bg-indigo-500 rounded-xl
               text-white font-medium text-sm hover:bg-indigo-600
               transition-all duration-200 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-indigo-500/20
               disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:translate-y-0"
      >
        {editingRule ? 'Save Changes' : 'Add Rule'}
      </button>
    </div>
  </div>
  {/if}
</Modal>
