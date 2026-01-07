<script lang="ts">
  import { fly, fade, scale } from 'svelte/transition';
  import type { NetworkRule } from './types';
  import type { ProxyConfig } from '$lib/api';

  interface Props {
    open: boolean;
    onclose: () => void;
    onadd: (rule: Omit<NetworkRule, 'id' | 'priority'>) => void;
    gateways: ProxyConfig[];
  }

  let { open, onclose, onadd, gateways }: Props = $props();

  // Rule templates
  interface RuleTemplate {
    name: string;
    source: 'domain' | 'app' | 'ip';
    sourceValue: string;
    action: 'direct' | 'proxy' | 'block' | 'dpi-bypass';
    icon: string;
    color: string;
  }

  const ruleTemplates: RuleTemplate[] = [
    { 
      name: 'YouTube DPI Bypass', 
      source: 'domain', 
      sourceValue: 'youtube.com', 
      action: 'dpi-bypass',
      icon: '▶',
      color: 'red'
    },
    { 
      name: 'YouTube Video DPI', 
      source: 'domain', 
      sourceValue: '*.googlevideo.com', 
      action: 'dpi-bypass',
      icon: '🎬',
      color: 'red'
    },
    { 
      name: 'Discord Proxy', 
      source: 'domain', 
      sourceValue: 'discord.com', 
      action: 'proxy',
      icon: '💬',
      color: 'indigo'
    },
    { 
      name: 'Discord CDN', 
      source: 'domain', 
      sourceValue: '*.discord.gg', 
      action: 'proxy',
      icon: '📎',
      color: 'indigo'
    },
    { 
      name: 'Telegram Proxy', 
      source: 'domain', 
      sourceValue: '*.telegram.org', 
      action: 'proxy',
      icon: '✈',
      color: 'sky'
    },
    { 
      name: 'Twitter/X Proxy', 
      source: 'domain', 
      sourceValue: '*.x.com', 
      action: 'proxy',
      icon: '𝕏',
      color: 'zinc'
    },
    { 
      name: 'Block Ads', 
      source: 'domain', 
      sourceValue: '*.doubleclick.net', 
      action: 'block',
      icon: '🚫',
      color: 'amber'
    },
    { 
      name: 'Block Trackers', 
      source: 'domain', 
      sourceValue: '*.analytics.google.com', 
      action: 'block',
      icon: '👁',
      color: 'amber'
    },
  ];

  // Template animation state
  let appliedTemplate = $state<string | null>(null);

  // Form state
  let name = $state('');
  let source = $state<'domain' | 'app' | 'ip'>('domain');
  let sourceValue = $state('');
  let action = $state<'direct' | 'proxy' | 'block' | 'dpi-bypass'>('direct');
  let proxyId = $state<string | undefined>(undefined);
  let enabled = $state(true);

  // Apply template to form
  function applyTemplate(template: RuleTemplate) {
    name = template.name;
    source = template.source;
    sourceValue = template.sourceValue;
    action = template.action;
    
    // Animation feedback
    appliedTemplate = template.name;
    setTimeout(() => {
      appliedTemplate = null;
    }, 600);
  }

  // Get template color classes
  function getTemplateColorClasses(color: string, isApplied: boolean): string {
    const colors: Record<string, { bg: string; border: string; text: string; hover: string }> = {
      red: { 
        bg: 'bg-red-500/10', 
        border: 'border-red-500/20', 
        text: 'text-red-400',
        hover: 'hover:bg-red-500/20 hover:border-red-500/30'
      },
      indigo: { 
        bg: 'bg-indigo-500/10', 
        border: 'border-indigo-500/20', 
        text: 'text-indigo-400',
        hover: 'hover:bg-indigo-500/20 hover:border-indigo-500/30'
      },
      sky: { 
        bg: 'bg-sky-500/10', 
        border: 'border-sky-500/20', 
        text: 'text-sky-400',
        hover: 'hover:bg-sky-500/20 hover:border-sky-500/30'
      },
      zinc: { 
        bg: 'bg-zinc-500/10', 
        border: 'border-zinc-500/20', 
        text: 'text-zinc-400',
        hover: 'hover:bg-zinc-500/20 hover:border-zinc-500/30'
      },
      amber: { 
        bg: 'bg-amber-500/10', 
        border: 'border-amber-500/20', 
        text: 'text-amber-400',
        hover: 'hover:bg-amber-500/20 hover:border-amber-500/30'
      },
    };
    
    const c = colors[color] || colors.indigo;
    
    if (isApplied) {
      return `${c.bg} ${c.border} ${c.text} ring-2 ring-${color}-500/50 scale-95`;
    }
    
    return `${c.bg} ${c.border} ${c.text} ${c.hover}`;
  }

  // Validation
  let isValid = $derived(
    name.trim().length > 0 && 
    sourceValue.trim().length > 0 &&
    (action !== 'proxy' || proxyId !== undefined)
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

  // Reset form
  function resetForm() {
    name = '';
    source = 'domain';
    sourceValue = '';
    action = 'direct';
    proxyId = undefined;
    enabled = true;
  }

  // Handle submit
  function handleSubmit() {
    if (!isValid) return;

    onadd({
      name: name.trim(),
      enabled,
      source,
      sourceValue: sourceValue.trim(),
      action,
      proxyId: action === 'proxy' ? proxyId : undefined
    });

    resetForm();
    onclose();
  }

  // Handle close
  function handleClose() {
    resetForm();
    onclose();
  }

  // Handle Escape key
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      handleClose();
    }
  }

  // Handle backdrop click
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      handleClose();
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

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Add Rule"
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
          <div class="w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center">
            <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </div>
          <h2 class="text-lg font-semibold text-white">Add Rule</h2>
        </div>
        <button
          type="button"
          onclick={handleClose}
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
        <!-- Quick Templates -->
        <div class="space-y-3">
          <div class="flex items-center gap-2">
            <svg class="w-4 h-4 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            <span class="text-sm font-medium text-zinc-400">Quick Templates</span>
          </div>
          <div class="grid grid-cols-2 gap-2">
            {#each ruleTemplates as template (template.name)}
              <button
                type="button"
                onclick={() => applyTemplate(template)}
                class="group flex items-center gap-2 px-3 py-2 rounded-lg border text-left
                       transition-all duration-200 transform
                       {getTemplateColorClasses(template.color, appliedTemplate === template.name)}"
              >
                <span class="text-base shrink-0 transition-transform duration-200 group-hover:scale-110">
                  {template.icon}
                </span>
                <div class="flex-1 min-w-0">
                  <p class="text-xs font-medium truncate">{template.name}</p>
                  <p class="text-[10px] text-zinc-500 truncate font-mono">{template.sourceValue}</p>
                </div>
                {#if appliedTemplate === template.name}
                  <svg 
                    class="w-4 h-4 text-emerald-400 shrink-0" 
                    fill="currentColor" 
                    viewBox="0 0 20 20"
                    in:scale={{ duration: 200 }}
                  >
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                {/if}
              </button>
            {/each}
          </div>
          <p class="text-xs text-zinc-600">Click a template to fill the form below</p>
        </div>

        <!-- Divider -->
        <div class="relative">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-white/5"></div>
          </div>
          <div class="relative flex justify-center">
            <span class="px-3 bg-zinc-950 text-xs text-zinc-600">or create custom rule</span>
          </div>
        </div>

        <!-- Rule Name -->
        <div class="space-y-2">
          <label for="rule-name" class="block text-sm font-medium text-zinc-400">Rule Name</label>
          <input
            id="rule-name"
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
          <label for="source-type-domain" class="block text-sm font-medium text-zinc-400">Match Type</label>
          <div class="grid grid-cols-3 gap-2">
            {#each sourceOptions as option, i}
              <button
                id={i === 0 ? 'source-type-domain' : undefined}
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
          <label for="source-value" class="block text-sm font-medium text-zinc-400">
            {#if source === 'domain'}
              Domain Pattern
            {:else if source === 'app'}
              Application Name
            {:else}
              IP Address / CIDR
            {/if}
          </label>
          <input
            id="source-value"
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
          <label for="action-direct" class="block text-sm font-medium text-zinc-400">Action</label>
          <div class="space-y-2">
            {#each actionOptions as option, i}
              <button
                id={i === 0 ? 'action-direct' : undefined}
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
            <label for="gateway-select" class="block text-sm font-medium text-zinc-400">Gateway</label>
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
                id="gateway-select"
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
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-white/5 shrink-0 space-y-3">
        <button
          type="button"
          onclick={handleSubmit}
          disabled={!isValid}
          class="w-full px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:bg-zinc-800 disabled:text-zinc-600
                 text-white text-sm font-medium rounded-lg transition-colors 
                 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 disabled:cursor-not-allowed"
        >
          Add Rule
        </button>
        <button
          type="button"
          onclick={handleClose}
          class="w-full px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-sm font-medium 
                 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-zinc-500/50"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
