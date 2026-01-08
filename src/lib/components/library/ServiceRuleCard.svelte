<script lang="ts">
  import type { ServiceRule, AccessMethod } from '$lib/stores/library.svelte';
  import { libraryStore } from '$lib/stores/library.svelte';
  import ServiceStatusBadge from './ServiceStatusBadge.svelte';
  import MethodDropdown from './MethodDropdown.svelte';

  interface Props {
    rule: ServiceRule;
    isSelected?: boolean;
    onclick?: () => void;
    'data-rule-id'?: string;
    'data-rule-index'?: number;
  }

  let { rule, isSelected = false, onclick, ...restProps }: Props = $props();

  let showContextMenu = $state(false);
  let contextMenuPos = $state({ x: 0, y: 0 });

  function handleMethodSelect(method: AccessMethod) {
    libraryStore.setRuleMethod(rule.id, method);
  }

  function handleCheck() {
    libraryStore.checkRule(rule.id);
  }

  function handleRemove() {
    libraryStore.removeRule(rule.id);
    showContextMenu = false;
  }

  function handleContextMenu(event: MouseEvent) {
    if (rule.isCustom) {
      event.preventDefault();
      contextMenuPos = { x: event.clientX, y: event.clientY };
      showContextMenu = true;
    }
  }

  function handleClickOutside() {
    showContextMenu = false;
  }

  $effect(() => {
    if (showContextMenu) {
      document.addEventListener('click', handleClickOutside);
    }
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  });
</script>

<div 
  class="group flex items-center gap-4 p-4 bg-zinc-900/50 hover:bg-zinc-800/50 
         border border-zinc-800 hover:border-zinc-700 rounded-xl
         transition-all duration-200 cursor-default
         {isSelected ? 'ring-2 ring-emerald-500/50 border-emerald-500/30 bg-zinc-800/50' : ''}"
  oncontextmenu={handleContextMenu}
  {onclick}
  role="listitem"
  aria-label="{rule.name} - {rule.domain}"
  aria-selected={isSelected}
  data-rule-id={restProps['data-rule-id']}
  data-rule-index={restProps['data-rule-index']}
>
  <!-- Icon -->
  <div class="flex-shrink-0 w-12 h-12 flex items-center justify-center 
              bg-zinc-800 rounded-xl text-2xl">
    {rule.icon}
  </div>

  <!-- Info -->
  <div class="flex-1 min-w-0">
    <div class="flex items-center gap-2">
      <h3 class="text-base font-medium text-white truncate">{rule.name}</h3>
      {#if rule.isCustom}
        <span class="px-1.5 py-0.5 text-xs font-medium text-zinc-400 bg-zinc-800 rounded">
          Custom
        </span>
      {/if}
    </div>
    <div class="flex items-center gap-2 mt-1">
      <span class="text-sm text-zinc-500 truncate">{rule.domain}</span>
      <span class="text-zinc-600">•</span>
      <ServiceStatusBadge status={rule.status} ping={rule.ping} />
    </div>
  </div>

  <!-- Actions -->
  <div class="flex items-center gap-2">
    <!-- Check button -->
    <button
      type="button"
      class="p-2 text-zinc-400 hover:text-white hover:bg-zinc-700 rounded-lg
             opacity-0 group-hover:opacity-100 transition-all duration-150
             disabled:opacity-50 disabled:cursor-not-allowed"
      onclick={handleCheck}
      disabled={rule.status === 'checking'}
      aria-label="Проверить доступность"
      title="Проверить"
    >
      <svg class="w-5 h-5 {rule.status === 'checking' ? 'animate-spin' : ''}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
    </button>

    <!-- Method dropdown -->
    <MethodDropdown 
      currentMethod={rule.currentMethod}
      availableMethods={rule.availableMethods}
      onSelect={handleMethodSelect}
      disabled={rule.status === 'checking'}
    />
  </div>
</div>

<!-- Context menu for custom rules -->
{#if showContextMenu && rule.isCustom}
  <div 
    class="fixed z-50 py-1 bg-zinc-800 border border-zinc-700 rounded-lg shadow-xl"
    style="left: {contextMenuPos.x}px; top: {contextMenuPos.y}px;"
    role="menu"
  >
    <button
      type="button"
      class="w-full flex items-center gap-2 px-4 py-2 text-sm text-left text-red-400 hover:bg-zinc-700"
      onclick={handleRemove}
      role="menuitem"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
      Удалить
    </button>
  </div>
{/if}
