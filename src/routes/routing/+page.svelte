<script lang="ts">
  import { browser } from '$app/environment';
  import { Modal, Spinner } from '$lib/components';
  import { toasts } from '$lib/stores/toast';

  // Types
  interface RoutingRule {
    id: string;
    name: string;
    enabled: boolean;
    source: 'all' | 'app' | 'domain';
    sourceValue?: string;
    action: 'direct' | 'proxy' | 'block';
    proxyId?: string;
  }

  interface ProxyConfig {
    id: string;
    name: string;
    protocol: string;
  }

  // State
  let rules = $state<RoutingRule[]>([]);
  let proxies = $state<ProxyConfig[]>([]);
  let loading = $state(true);
  let showAddModal = $state(false);
  let editingRule = $state<RoutingRule | null>(null);
  let isTauri = $state(false);
  let initialized = $state(false);

  // Form state
  let formName = $state('');
  let formSource = $state<'all' | 'app' | 'domain'>('domain');
  let formSourceValue = $state('');
  let formAction = $state<'direct' | 'proxy' | 'block'>('direct');
  let formProxyId = $state('');

  // Mock data
  const mockRules: RoutingRule[] = [
    { id: '1', name: 'YouTube Direct', enabled: true, source: 'domain', sourceValue: 'youtube.com', action: 'direct' },
    { id: '2', name: 'Discord via VLESS', enabled: true, source: 'domain', sourceValue: 'discord.com', action: 'proxy', proxyId: 'vless-1' },
    { id: '3', name: 'Block Ads', enabled: false, source: 'domain', sourceValue: '*.doubleclick.net', action: 'block' },
    { id: '4', name: 'Telegram Proxy', enabled: true, source: 'domain', sourceValue: 'telegram.org', action: 'proxy', proxyId: 'vless-1' },
    { id: '5', name: 'Chrome Direct', enabled: true, source: 'app', sourceValue: 'chrome.exe', action: 'direct' },
  ];

  const mockProxies: ProxyConfig[] = [
    { id: 'vless-1', name: 'VLESS Germany', protocol: 'vless' },
    { id: 'vless-2', name: 'VLESS Netherlands', protocol: 'vless' },
  ];

  // Initialize
  $effect(() => {
    if (!browser || initialized) return;
    initialized = true;
    
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    if (!isTauri) {
      rules = mockRules;
      proxies = mockProxies;
      loading = false;
    } else {
      loadData();
    }
  });

  async function loadData(retries = 10) {
    loading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      for (let i = 0; i < retries; i++) {
        try {
          const ready = await invoke<boolean>('is_backend_ready');
          if (ready) break;
        } catch { /* Backend not ready */ }
        if (i < retries - 1) await new Promise(r => setTimeout(r, 200));
      }
      
      const [proxyList] = await Promise.all([
        invoke<ProxyConfig[]>('get_proxies').catch(() => []),
      ]);
      
      proxies = proxyList;
      // For now use mock rules since backend doesn't have routing rules yet
      rules = mockRules;
    } catch (e) {
      console.error('[Routing] Failed to load data:', e);
      rules = mockRules;
      proxies = mockProxies;
    } finally {
      loading = false;
    }
  }

  function openAddModal() {
    editingRule = null;
    formName = '';
    formSource = 'domain';
    formSourceValue = '';
    formAction = 'direct';
    formProxyId = proxies[0]?.id || '';
    showAddModal = true;
  }

  function openEditModal(rule: RoutingRule) {
    editingRule = rule;
    formName = rule.name;
    formSource = rule.source;
    formSourceValue = rule.sourceValue || '';
    formAction = rule.action;
    formProxyId = rule.proxyId || proxies[0]?.id || '';
    showAddModal = true;
  }

  function handleSaveRule() {
    if (!formName.trim()) {
      toasts.error('Введите название правила');
      return;
    }
    if (formSource !== 'all' && !formSourceValue.trim()) {
      toasts.error('Введите значение источника');
      return;
    }
    if (formAction === 'proxy' && !formProxyId) {
      toasts.error('Выберите прокси');
      return;
    }

    const newRule: RoutingRule = {
      id: editingRule?.id || `rule-${Date.now()}`,
      name: formName.trim(),
      enabled: editingRule?.enabled ?? true,
      source: formSource,
      sourceValue: formSource === 'all' ? undefined : formSourceValue.trim(),
      action: formAction,
      proxyId: formAction === 'proxy' ? formProxyId : undefined,
    };

    if (editingRule) {
      rules = rules.map(r => r.id === editingRule!.id ? newRule : r);
      toasts.success('Правило обновлено');
    } else {
      rules = [...rules, newRule];
      toasts.success('Правило добавлено');
    }

    showAddModal = false;
  }

  function toggleRule(rule: RoutingRule) {
    rules = rules.map(r => r.id === rule.id ? { ...r, enabled: !r.enabled } : r);
  }

  function deleteRule(rule: RoutingRule) {
    rules = rules.filter(r => r.id !== rule.id);
    toasts.success('Правило удалено');
  }

  function getActionIcon(action: string): string {
    switch (action) {
      case 'direct': return '🌐';
      case 'proxy': return '🔒';
      case 'block': return '🚫';
      default: return '❓';
    }
  }

  function getActionColor(action: string): string {
    switch (action) {
      case 'direct': return 'emerald';
      case 'proxy': return 'indigo';
      case 'block': return 'red';
      default: return 'zinc';
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
</script>

<div class="h-full overflow-auto bg-gradient-to-br from-zinc-950 to-black">
  <div class="p-8 max-w-5xl mx-auto">
    <!-- Header -->
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold text-white tracking-tight">Routing Rules</h1>
          <p class="text-sm text-zinc-500 mt-2">Configure traffic routing with visual flow rules</p>
        </div>
        <button
          onclick={openAddModal}
          class="flex items-center gap-2 px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 
                 rounded-xl text-white font-medium text-sm transition-all duration-200
                 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-indigo-500/20"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Add Rule
        </button>
      </div>
    </div>

    <!-- Stats Bar -->
    <div class="grid grid-cols-4 gap-4 mb-8">
      <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
        <div class="text-xs text-zinc-500 uppercase tracking-wider mb-1">Total Rules</div>
        <div class="text-2xl font-bold text-white">{rules.length}</div>
      </div>
      <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
        <div class="text-xs text-zinc-500 uppercase tracking-wider mb-1">Active</div>
        <div class="text-2xl font-bold text-emerald-400">{rules.filter(r => r.enabled).length}</div>
      </div>
      <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
        <div class="text-xs text-zinc-500 uppercase tracking-wider mb-1">Proxied</div>
        <div class="text-2xl font-bold text-indigo-400">{rules.filter(r => r.action === 'proxy').length}</div>
      </div>
      <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
        <div class="text-xs text-zinc-500 uppercase tracking-wider mb-1">Blocked</div>
        <div class="text-2xl font-bold text-red-400">{rules.filter(r => r.action === 'block').length}</div>
      </div>
    </div>

    <!-- Rules List -->
    {#if loading}
      <div class="flex items-center justify-center py-20">
        <Spinner />
      </div>
    {:else if rules.length === 0}
      <!-- Empty State -->
      <div class="text-center py-20">
        <div class="w-20 h-20 mx-auto mb-6 rounded-2xl bg-zinc-900/60 border border-white/5 
                    flex items-center justify-center">
          <svg class="w-10 h-10 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                  d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
          </svg>
        </div>
        <h3 class="text-xl font-semibold text-white mb-2">No routing rules yet</h3>
        <p class="text-zinc-500 mb-6 max-w-md mx-auto">
          Create rules to control how traffic flows through your network
        </p>
        <button
          onclick={openAddModal}
          class="inline-flex items-center gap-2 px-5 py-2.5 bg-indigo-500/10 border border-indigo-500/30 
                 text-indigo-400 rounded-xl hover:bg-indigo-500/20 transition-all duration-200"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Create your first rule
        </button>
      </div>
    {:else}
      <div class="space-y-4">
        {#each rules as rule, i (rule.id)}
          {@const actionColor = getActionColor(rule.action)}
          <div 
            class="group relative bg-zinc-900/40 border border-white/5 rounded-2xl overflow-hidden
                   hover:border-white/10 transition-all duration-300
                   {!rule.enabled ? 'opacity-50' : ''}"
            style="animation: slideIn 300ms ease-out {i * 50}ms both"
          >
            <!-- Flow Visualization -->
            <div class="p-5">
              <div class="flex items-center gap-4">
                <!-- Toggle -->
                <button
                  onclick={() => toggleRule(rule)}
                  class="relative w-12 h-6 rounded-full transition-colors duration-200
                         {rule.enabled ? 'bg-emerald-500/20' : 'bg-zinc-800'}"
                >
                  <div class="absolute top-1 left-1 w-4 h-4 rounded-full transition-all duration-200
                              {rule.enabled ? 'translate-x-6 bg-emerald-400' : 'bg-zinc-500'}">
                  </div>
                </button>

                <!-- Rule Name -->
                <div class="flex-1 min-w-0">
                  <h3 class="text-lg font-semibold text-white truncate">{rule.name}</h3>
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onclick={() => openEditModal(rule)}
                    class="p-2 rounded-lg bg-zinc-800/60 border border-white/5 
                           hover:bg-zinc-700/60 hover:border-white/10 transition-colors"
                    title="Edit"
                  >
                    <svg class="w-4 h-4 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </button>
                  <button
                    onclick={() => deleteRule(rule)}
                    class="p-2 rounded-lg bg-red-500/10 border border-red-500/20 
                           hover:bg-red-500/20 hover:border-red-500/30 transition-colors"
                    title="Delete"
                  >
                    <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </div>

              <!-- Flow Cards -->
              <div class="mt-4 flex items-center gap-3">
                <!-- Source Card -->
                <div class="flex-1 p-4 bg-zinc-800/40 border border-white/5 rounded-xl">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg bg-zinc-700/50 flex items-center justify-center text-xl">
                      {getSourceIcon(rule.source)}
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="text-xs text-zinc-500 uppercase tracking-wider">Source</div>
                      <div class="text-sm font-medium text-white truncate">
                        {#if rule.source === 'all'}
                          All Traffic
                        {:else if rule.source === 'app'}
                          App: {rule.sourceValue}
                        {:else}
                          {rule.sourceValue}
                        {/if}
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Arrow -->
                <div class="flex-shrink-0 flex items-center">
                  <svg class="w-8 h-8 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                          d="M13 7l5 5m0 0l-5 5m5-5H6" />
                  </svg>
                </div>

                <!-- Condition Card (optional visual) -->
                <div class="flex-shrink-0 w-12 h-12 rounded-xl bg-zinc-800/40 border border-white/5 
                            flex items-center justify-center">
                  <svg class="w-5 h-5 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                          d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                </div>

                <!-- Arrow -->
                <div class="flex-shrink-0 flex items-center">
                  <svg class="w-8 h-8 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                          d="M13 7l5 5m0 0l-5 5m5-5H6" />
                  </svg>
                </div>

                <!-- Action Card -->
                <div class="flex-1 p-4 rounded-xl border
                            {actionColor === 'emerald' ? 'bg-emerald-500/10 border-emerald-500/20' : ''}
                            {actionColor === 'indigo' ? 'bg-indigo-500/10 border-indigo-500/20' : ''}
                            {actionColor === 'red' ? 'bg-red-500/10 border-red-500/20' : ''}">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg flex items-center justify-center text-xl
                                {actionColor === 'emerald' ? 'bg-emerald-500/20' : ''}
                                {actionColor === 'indigo' ? 'bg-indigo-500/20' : ''}
                                {actionColor === 'red' ? 'bg-red-500/20' : ''}">
                      {getActionIcon(rule.action)}
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="text-xs uppercase tracking-wider
                                  {actionColor === 'emerald' ? 'text-emerald-400/60' : ''}
                                  {actionColor === 'indigo' ? 'text-indigo-400/60' : ''}
                                  {actionColor === 'red' ? 'text-red-400/60' : ''}">
                        Action
                      </div>
                      <div class="text-sm font-medium truncate
                                  {actionColor === 'emerald' ? 'text-emerald-400' : ''}
                                  {actionColor === 'indigo' ? 'text-indigo-400' : ''}
                                  {actionColor === 'red' ? 'text-red-400' : ''}">
                        {#if rule.action === 'direct'}
                          Direct Connection
                        {:else if rule.action === 'proxy'}
                          Via {getProxyName(rule.proxyId)}
                        {:else}
                          Blocked
                        {/if}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Bottom accent line -->
            <div class="h-0.5 
                        {actionColor === 'emerald' ? 'bg-gradient-to-r from-emerald-500/50 to-transparent' : ''}
                        {actionColor === 'indigo' ? 'bg-gradient-to-r from-indigo-500/50 to-transparent' : ''}
                        {actionColor === 'red' ? 'bg-gradient-to-r from-red-500/50 to-transparent' : ''}">
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Add/Edit Rule Modal -->
<Modal bind:open={showAddModal} title={editingRule ? 'Edit Rule' : 'Add Routing Rule'}>
  <form onsubmit={(e) => { e.preventDefault(); handleSaveRule(); }} class="space-y-5">
    <!-- Rule Name -->
    <div>
      <label class="block text-sm font-medium text-zinc-400 mb-2">Rule Name</label>
      <input 
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
      <label class="block text-sm font-medium text-zinc-400 mb-2">Traffic Source</label>
      <div class="grid grid-cols-3 gap-2">
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
        <label class="block text-sm font-medium text-zinc-400 mb-2">
          {formSource === 'domain' ? 'Domain Pattern' : 'Application Name'}
        </label>
        <input 
          type="text" 
          bind:value={formSourceValue}
          placeholder={formSource === 'domain' ? 'e.g., youtube.com or *.google.com' : 'e.g., chrome.exe'}
          class="w-full px-4 py-3 bg-zinc-800/60 border border-white/10 rounded-xl
                 text-white placeholder-zinc-500 font-mono
                 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20
                 transition-all duration-200"
        />
        {#if formSource === 'domain'}
          <p class="mt-1.5 text-xs text-zinc-500">Use * for wildcard matching</p>
        {/if}
      </div>
    {/if}

    <!-- Action -->
    <div>
      <label class="block text-sm font-medium text-zinc-400 mb-2">Action</label>
      <div class="space-y-2">
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
              <div class="text-xs text-zinc-500">{option.desc}</div>
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
        <label class="block text-sm font-medium text-zinc-400 mb-2">Select Proxy</label>
        {#if proxies.length === 0}
          <div class="p-4 bg-amber-500/10 border border-amber-500/20 rounded-xl text-center">
            <p class="text-sm text-amber-400">No proxies configured</p>
            <p class="text-xs text-zinc-500 mt-1">Add a proxy in the Proxies section first</p>
          </div>
        {:else}
          <div class="space-y-2">
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
                <div class="w-8 h-8 rounded-lg bg-indigo-500/20 flex items-center justify-center">
                  <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                          d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                  </svg>
                </div>
                <div class="flex-1">
                  <div class="text-sm font-medium text-white">{proxy.name}</div>
                  <div class="text-xs text-zinc-500 uppercase">{proxy.protocol}</div>
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
      <div class="text-xs text-zinc-500 uppercase tracking-wider mb-3">Preview</div>
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
        onclick={() => showAddModal = false}
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
</Modal>

<style>
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
