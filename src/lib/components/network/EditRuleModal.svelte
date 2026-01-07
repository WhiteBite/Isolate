<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import type { NetworkRule } from './types';
  import type { ProxyConfig } from '$lib/api';

  interface Props {
    open: boolean;
    onclose: () => void;
    onsave: (rule: NetworkRule) => void;
    rule: NetworkRule | null;
    gateways: ProxyConfig[];
  }

  let { open, onclose, onsave, rule, gateways }: Props = $props();

  // Form state
  let name = $state('');
  let source = $state<'domain' | 'app' | 'ip'>('domain');
  let sourceValue = $state('');
  let action = $state<'direct' | 'proxy' | 'block' | 'dpi-bypass'>('direct');
  let proxyId = $state<string | undefined>(undefined);
  let enabled = $state(true);

  // Sync form with rule prop
  $effect(() => {
    if (rule && open) {
      name = rule.name;
      source = rule.source;
      sourceValue = rule.sourceValue;
      action = rule.action;
      proxyId = rule.proxyId;
      enabled = rule.enabled;
    }
  });

  // Validation
  let isValid = $derived(
    name.trim().length > 0 && 
    sourceValue.trim().length > 0 &&
    (action !== 'proxy' || proxyId !== undefined)
  );

  // Check if form has changes
  let hasChanges = $derived(
    rule !== null && (
      name !== rule.name ||
      source !== rule.source ||
      sourceValue !== rule.sourceValue ||
      action !== rule.action ||
      proxyId !== rule.proxyId ||
      enabled !== rule.enabled
    )
  );

  // Source type options
  const sourceOptions = [
    { value: 'domain', label: 'Domain', placeholder: 'youtube.com, *.google.com' },
    { value: 'app', label: 'Application', placeholder: 'chrome.exe, firefox.exe' },
    { value: 'ip', label: 'IP / CIDR', placeholder: '192.168.1.0/24, 10.0.0.1' }
  ] as const;

  // Action options
  const actionOptions = [
    { value: 'direct', label: 'Direct', description: 'Connect directly without proxy', icon: '→' },
    { value: 'proxy', label: 'Proxy', description: 'Route through selected gateway', icon: '⇄' },
    { value: 'block', label: 'Block', description: 'Block all connections', icon: '✕' },
    { value: 'dpi-bypass', label: 'DPI Bypass', description: 'Use DPI bypass strategy', icon: '⚡' }
  ] as const;

  // Get current source placeholder
  let currentPlaceholder = $derived(
    sourceOptions.find(o => o.value === source)?.placeholder || ''
  );

  // Handle submit
  function handleSubmit() {
    if (!isValid || !rule) return;

    onsave({
      ...rule,
      name: name.trim(),
      enabled,
      source,
      sourceValue: sourceValue.trim(),
      action,
      proxyId: action === 'proxy' ? proxyId : undefined
    });

    onclose();
  }

  // Handle Escape key
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onclose();
    }
  }

  // Handle backdrop click
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }

  // Reset proxyId when action changes away from proxy
  $effect(() => {
    if (action !== 'proxy') {
      proxyId = undefined;
    } else if (gateways.length > 0 && !proxyId) {
      proxyId = gateways[0].id;
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open && rule}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Edit Rule"
    tabindex="-1"
    class="fixed inset-0 z-50"
    onkeydown={handleKeydown}
  >
    <!-- Backdrop -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      transition:fade={{ duration: 200 }}
      onclick={handleBackdropClick}
    ></div>

    <!-- Drawer Panel -->
    <div
      class="absolute right-0 top-0 h-full w-[420px] bg-zinc-950 border-l border-white/5 shadow-2xl flex flex-col"
      transition:fly={{ x: 420, duration: 300, opacity: 1 }}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-white/5 shrink-0">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-amber-500/10 flex items-center justify-center">
            <svg class="w-4 h-4 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </div>
          <h2 class="text-lg font-semibold text-white">Edit Rule</h2>
        </div>
        <button
          type="button"
          onclick={onclose}
          class="p-2 hover:bg-white/5 rounded-lg transition-colors text-zinc-400 hover:text-white"
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4 space-y-5">
        <!-- Enabled Toggle -->
        <div class="flex items-center justify-between p-4 bg-zinc-900/50 rounded-xl border border-white/5">
          <div class="space-y-1">
            <p class="text-sm font-medium text-white">Rule Enabled</p>
            <p class="text-xs text-zinc-500">Toggle to enable or disable this rule</p>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={enabled}
            aria-label="Toggle rule enabled"
            onclick={() => enabled = !enabled}
            class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200
                   {enabled ? 'bg-indigo-600' : 'bg-zinc-700'}"
          >
            <span
              class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200
                     {enabled ? 'translate-x-6' : 'translate-x-1'}"
            ></span>
          </button>
        </div>

        <!-- Rule Name -->
        <div class="space-y-2">
          <label for="edit-rule-name" class="block text-sm font-medium text-zinc-400">Rule Name</label>
          <input
            id="edit-rule-name"
            type="text"
            bind:value={name}
            placeholder="My Rule"
            class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                   placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                   focus:border-indigo-500/50 transition-colors"
          />
        </div>

        <!-- Source Type -->
        <div class="space-y-2">
          <span id="edit-match-type-label" class="block text-sm font-medium text-zinc-400">Match Type</span>
          <div class="grid grid-cols-3 gap-2" role="group" aria-labelledby="edit-match-type-label">
            {#each sourceOptions as option}
              <button
                type="button"
                onclick={() => source = option.value}
                class="px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200
                       {source === option.value 
                         ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30' 
                         : 'bg-zinc-900/50 text-zinc-400 border border-white/5 hover:bg-zinc-800/50 hover:text-zinc-300'}"
              >
                {option.label}
              </button>
            {/each}
          </div>
        </div>

        <!-- Source Value -->
        <div class="space-y-2">
          <label for="edit-source-value" class="block text-sm font-medium text-zinc-400">
            {#if source === 'domain'}
              Domain Pattern
            {:else if source === 'app'}
              Application Name
            {:else}
              IP Address / CIDR
            {/if}
          </label>
          <input
            id="edit-source-value"
            type="text"
            bind:value={sourceValue}
            placeholder={currentPlaceholder}
            class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                   placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 
                   focus:border-indigo-500/50 transition-colors font-mono"
          />
          <p class="text-xs text-zinc-600">
            {#if source === 'domain'}
              Use * for wildcards: *.youtube.com
            {:else if source === 'app'}
              Enter executable name without path
            {:else}
              Supports CIDR notation: 10.0.0.0/8
            {/if}
          </p>
        </div>

        <!-- Action -->
        <div class="space-y-2">
          <span id="edit-action-label" class="block text-sm font-medium text-zinc-400">Action</span>
          <div class="space-y-2" role="group" aria-labelledby="edit-action-label">
            {#each actionOptions as option}
              <button
                type="button"
                onclick={() => action = option.value}
                class="w-full flex items-center gap-3 p-3 rounded-lg text-left transition-all duration-200
                       {action === option.value 
                         ? 'bg-indigo-500/10 border border-indigo-500/30' 
                         : 'bg-zinc-900/50 border border-white/5 hover:bg-zinc-800/50'}"
              >
                <div class="w-8 h-8 rounded-lg flex items-center justify-center text-lg
                            {action === option.value ? 'bg-indigo-500/20 text-indigo-300' : 'bg-zinc-800 text-zinc-500'}">
                  {option.icon}
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-medium {action === option.value ? 'text-white' : 'text-zinc-300'}">
                    {option.label}
                  </p>
                  <p class="text-xs text-zinc-500 truncate">{option.description}</p>
                </div>
                {#if action === option.value}
                  <svg class="w-5 h-5 text-indigo-400 shrink-0" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                  </svg>
                {/if}
              </button>
            {/each}
          </div>
        </div>

        <!-- Gateway Selection (only for proxy action) -->
        {#if action === 'proxy'}
          <div class="space-y-2" transition:fly={{ y: -10, duration: 200 }}>
            <label for="edit-gateway-select" class="block text-sm font-medium text-zinc-400">Gateway</label>
            {#if gateways.length === 0}
              <div class="p-4 bg-amber-500/5 rounded-xl border border-amber-500/10">
                <div class="flex gap-3">
                  <svg class="w-5 h-5 text-amber-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                  <p class="text-sm text-amber-300">No gateways available. Add a gateway first.</p>
                </div>
              </div>
            {:else}
              <select
                id="edit-gateway-select"
                bind:value={proxyId}
                class="w-full bg-zinc-900 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm
                       focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50
                       transition-colors cursor-pointer appearance-none"
                style="background-image: url('data:image/svg+xml;charset=UTF-8,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 24 24%27 fill=%27none%27 stroke=%27%2371717a%27 stroke-width=%272%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27%3e%3cpolyline points=%276 9 12 15 18 9%27%3e%3c/polyline%3e%3c/svg%3e'); background-repeat: no-repeat; background-position: right 0.75rem center; background-size: 1rem;"
              >
                {#each gateways as gateway}
                  <option value={gateway.id}>{gateway.name} ({gateway.protocol})</option>
                {/each}
              </select>
            {/if}
          </div>
        {/if}

        <!-- Rule Info -->
        <div class="p-4 bg-zinc-900/30 rounded-xl border border-white/5">
          <div class="flex items-center gap-2 text-xs text-zinc-500">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>Rule ID: {rule.id}</span>
            <span class="text-zinc-700">•</span>
            <span>Priority: {rule.priority}</span>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-white/5 shrink-0 space-y-3">
        <button
          type="button"
          onclick={handleSubmit}
          disabled={!isValid || !hasChanges}
          class="w-full px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-800 disabled:text-zinc-600
                 text-white text-sm font-medium rounded-lg transition-colors 
                 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 disabled:cursor-not-allowed"
        >
          {hasChanges ? 'Save Changes' : 'No Changes'}
        </button>
        <button
          type="button"
          onclick={onclose}
          class="w-full px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-sm font-medium 
                 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-zinc-500/50"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
