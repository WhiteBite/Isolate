<script lang="ts">
  import type { NetworkRule, ProxyConfig } from './types';
  import RuleCard from './RuleCard.svelte';

  interface Props {
    rules: NetworkRule[];
    gateways: ProxyConfig[];
    loading: boolean;
    onadd: () => void;
    onedit: (rule: NetworkRule) => void;
    ondelete: (id: string) => void;
    ontoggle: (id: string) => void;
    onreorder: (fromIndex: number, toIndex: number) => void;
    onbulkdelete?: (ids: string[]) => void;
    onbulkenable?: (ids: string[]) => void;
    onbulkdisable?: (ids: string[]) => void;
  }

  let {
    rules,
    gateways,
    loading,
    onadd,
    onedit,
    ondelete,
    ontoggle,
    onreorder,
    onbulkdelete,
    onbulkenable,
    onbulkdisable,
  }: Props = $props();

  // Drag-n-drop state
  let draggedIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  // Keyboard focus state
  let focusedIndex = $state<number | null>(null);
  let ruleElements = $state<HTMLElement[]>([]);

  // Multi-select state
  let selectedRules = $state<Set<string>>(new Set());
  let selectAll = $derived(selectedRules.size === rules.length && rules.length > 0);
  let hasSelection = $derived(selectedRules.size > 0);

  // Live region for screen reader announcements
  let liveAnnouncement = $state('');

  // Announce message to screen readers
  function announce(message: string) {
    // Clear first to ensure re-announcement of same message
    liveAnnouncement = '';
    setTimeout(() => {
      liveAnnouncement = message;
    }, 50);
  }

  // Handle focus change from keyboard navigation
  function handleFocusChange(index: number) {
    focusedIndex = index;
    // Focus the element at the new index
    const listContainer = document.querySelector('[role="list"][aria-label="Traffic rules"]');
    if (listContainer) {
      const items = listContainer.querySelectorAll('[role="listitem"]');
      const targetItem = items[index] as HTMLElement;
      if (targetItem) {
        targetItem.focus();
      }
    }
  }

  // Handle keyboard reorder
  function handleKeyboardReorder(fromIndex: number, toIndex: number) {
    onreorder(fromIndex, toIndex);
    focusedIndex = toIndex;
  }

  // Toggle single rule selection
  function toggleSelect(id: string) {
    const newSet = new Set(selectedRules);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    selectedRules = newSet;
  }

  // Toggle select all
  function toggleSelectAll() {
    if (selectAll) {
      selectedRules = new Set();
      announce('All rules deselected');
    } else {
      selectedRules = new Set(rules.map(r => r.id));
      announce(`All ${rules.length} rules selected`);
    }
  }

  // Clear selection
  function clearSelection() {
    selectedRules = new Set();
    announce('Selection cleared');
  }

  // Bulk actions
  function handleBulkDelete() {
    if (onbulkdelete && selectedRules.size > 0) {
      const count = selectedRules.size;
      onbulkdelete(Array.from(selectedRules));
      selectedRules = new Set();
      announce(`${count} rules deleted`);
    }
  }

  function handleBulkEnable() {
    if (onbulkenable && selectedRules.size > 0) {
      const count = selectedRules.size;
      onbulkenable(Array.from(selectedRules));
      selectedRules = new Set();
      announce(`${count} rules enabled`);
    }
  }

  function handleBulkDisable() {
    if (onbulkdisable && selectedRules.size > 0) {
      const count = selectedRules.size;
      onbulkdisable(Array.from(selectedRules));
      selectedRules = new Set();
      announce(`${count} rules disabled`);
    }
  }

  // Get gateway name by ID
  function getGatewayName(proxyId?: string): string | undefined {
    if (!proxyId) return undefined;
    return gateways.find(g => g.id === proxyId)?.name;
  }

  // Handle list-level keyboard events
  function handleListKeyDown(e: KeyboardEvent) {
    // Ctrl+A: Select all
    if (e.ctrlKey && (e.key === 'a' || e.key === 'A')) {
      e.preventDefault();
      toggleSelectAll();
      return;
    }
  }

  // Drag handlers
  function handleDragStart(e: DragEvent, index: number) {
    draggedIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', String(index));
    }
    announce(`Grabbed rule ${index + 1}. Use arrow keys to move, Enter to drop.`);
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  function handleDrop(e: DragEvent, index: number) {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === index) {
      draggedIndex = null;
      dragOverIndex = null;
      return;
    }
    
    const fromPos = draggedIndex + 1;
    const toPos = index + 1;
    onreorder(draggedIndex, index);
    announce(`Rule moved from position ${fromPos} to position ${toPos}`);
    
    draggedIndex = null;
    dragOverIndex = null;
  }

  function handleDragEnd() {
    if (draggedIndex !== null) {
      announce('Drag cancelled');
    }
    draggedIndex = null;
    dragOverIndex = null;
  }
</script>

<div class="bg-zinc-900/30 border border-white/5 rounded-2xl overflow-hidden">
  <!-- Live region for screen reader announcements -->
  <div 
    role="status" 
    aria-live="polite" 
    aria-atomic="true" 
    class="sr-only"
  >
    {liveAnnouncement}
  </div>

  <!-- Header -->
  <div class="flex items-center justify-between p-4 border-b border-white/5">
    <div class="flex items-center gap-3">
      <!-- Select All Checkbox -->
      {#if rules.length > 0}
        <button
          type="button"
          onclick={toggleSelectAll}
          class="flex-shrink-0 w-5 h-5 rounded border-2 flex items-center justify-center transition-all duration-200
                 {selectAll 
                   ? 'bg-indigo-500 border-indigo-500' 
                   : hasSelection 
                     ? 'bg-indigo-500/50 border-indigo-500' 
                     : 'border-zinc-600 hover:border-zinc-400 bg-transparent'}"
          aria-label={selectAll ? 'Deselect all' : 'Select all'}
          title={selectAll ? 'Deselect all' : 'Select all'}
        >
          {#if selectAll}
            <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
            </svg>
          {:else if hasSelection}
            <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M20 12H4" />
            </svg>
          {/if}
        </button>
      {/if}
      
      <div class="w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center">
        <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
        </svg>
      </div>
      <div>
        <h3 class="text-sm font-semibold text-white">Traffic Rules</h3>
        <p class="text-xs text-zinc-500">{rules.length} rule{rules.length !== 1 ? 's' : ''}</p>
      </div>
    </div>
    <button
      onclick={onadd}
      class="flex items-center gap-1.5 px-3 py-1.5 bg-indigo-500/10 border border-indigo-500/30 
             text-indigo-400 rounded-lg text-xs font-medium
             hover:bg-indigo-500/20 hover:border-indigo-500/40 transition-all duration-200"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      Add Rule
    </button>
  </div>

  <!-- Bulk Actions Bar -->
  {#if hasSelection}
    <div class="flex items-center justify-between gap-3 px-4 py-2.5 bg-indigo-500/10 border-b border-indigo-500/20">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-indigo-300">
          {selectedRules.size} selected
        </span>
        <button
          onclick={clearSelection}
          class="text-xs text-zinc-400 hover:text-white transition-colors"
        >
          Clear
        </button>
      </div>
      <div class="flex items-center gap-2">
        <button
          onclick={handleBulkEnable}
          disabled={!onbulkenable}
          class="flex items-center gap-1.5 px-2.5 py-1.5 bg-emerald-500/10 border border-emerald-500/30 
                 text-emerald-400 rounded-lg text-xs font-medium
                 hover:bg-emerald-500/20 hover:border-emerald-500/40 transition-all duration-200
                 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          Enable
        </button>
        <button
          onclick={handleBulkDisable}
          disabled={!onbulkdisable}
          class="flex items-center gap-1.5 px-2.5 py-1.5 bg-amber-500/10 border border-amber-500/30 
                 text-amber-400 rounded-lg text-xs font-medium
                 hover:bg-amber-500/20 hover:border-amber-500/40 transition-all duration-200
                 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
          </svg>
          Disable
        </button>
        <button
          onclick={handleBulkDelete}
          disabled={!onbulkdelete}
          class="flex items-center gap-1.5 px-2.5 py-1.5 bg-red-500/10 border border-red-500/30 
                 text-red-400 rounded-lg text-xs font-medium
                 hover:bg-red-500/20 hover:border-red-500/40 transition-all duration-200
                 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          Delete
        </button>
      </div>
    </div>
  {/if}

  <!-- Content -->
  <div class="p-4">
    {#if loading}
      <!-- Loading skeleton -->
      <div class="space-y-3">
        {#each Array(3) as _, i}
          <div class="flex items-center gap-3 p-3 bg-zinc-900/50 rounded-xl border border-white/5 animate-pulse">
            <div class="w-4 h-8 bg-zinc-800 rounded"></div>
            <div class="w-6 h-6 bg-zinc-800 rounded"></div>
            <div class="w-24 h-4 bg-zinc-800 rounded"></div>
            <div class="w-5 h-5 bg-zinc-800 rounded"></div>
            <div class="flex-1">
              <div class="w-20 h-6 bg-zinc-800 rounded-lg"></div>
            </div>
            <div class="w-10 h-5 bg-zinc-800 rounded-full"></div>
          </div>
        {/each}
      </div>
    {:else if rules.length === 0}
      <!-- Empty state -->
      <div class="text-center py-12">
        <div class="w-16 h-16 mx-auto mb-4 rounded-2xl bg-zinc-900/60 border border-white/5 
                    flex items-center justify-center">
          <svg class="w-8 h-8 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                  d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
          </svg>
        </div>
        <h4 class="text-base font-semibold text-white mb-1">No rules yet</h4>
        <p class="text-sm text-zinc-500 mb-4 max-w-xs mx-auto">
          Create rules to control how traffic flows through your network
        </p>
        <button
          onclick={onadd}
          class="inline-flex items-center gap-2 px-4 py-2 bg-indigo-500/10 border border-indigo-500/30 
                 text-indigo-400 rounded-xl text-sm font-medium
                 hover:bg-indigo-500/20 transition-all duration-200"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Create your first rule
        </button>
      </div>
    {:else}
      <!-- Rules list -->
      <div 
        role="list" 
        aria-label="Traffic rules"
        aria-describedby="keyboard-help"
        onkeydown={handleListKeyDown}
        class="space-y-2"
      >
        {#each rules as rule, i (rule.id)}
          <RuleCard
            {rule}
            index={i}
            totalRules={rules.length}
            gatewayName={getGatewayName(rule.proxyId)}
            isDragging={draggedIndex === i}
            isDropTarget={dragOverIndex === i && draggedIndex !== i}
            isSelected={selectedRules.has(rule.id)}
            showCheckbox={true}
            isFocused={focusedIndex === i}
            isGrabbed={draggedIndex === i}
            {ontoggle}
            {onedit}
            {ondelete}
            onselect={toggleSelect}
            onreorder={handleKeyboardReorder}
            onfocuschange={handleFocusChange}
            onclearselection={clearSelection}
            onannounce={announce}
            ondragstart={handleDragStart}
            ondragover={handleDragOver}
            ondragleave={handleDragLeave}
            ondrop={handleDrop}
            ondragend={handleDragEnd}
          />
        {/each}
      </div>

      <!-- Keyboard shortcuts hint -->
      <div id="keyboard-help" class="mt-3 px-2 py-1.5 bg-zinc-900/30 rounded-lg border border-white/5">
        <p class="text-xs text-zinc-500">
          <span class="text-zinc-400">Keyboard:</span> 
          ↑↓ navigate • Ctrl+↑↓ reorder • Home/End jump • Space select • Ctrl+A select all • Enter edit • Delete remove • T toggle • Esc clear
        </p>
      </div>

      <!-- Flow legend -->
      <div class="mt-4 pt-4 border-t border-white/5">
        <div class="flex flex-wrap items-center gap-3 text-xs text-zinc-500">
          <span class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
            Direct
          </span>
          <span class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-indigo-400"></span>
            Proxy
          </span>
          <span class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-red-400"></span>
            Block
          </span>
          <span class="flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full bg-amber-400"></span>
            DPI Bypass
          </span>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Screen reader only class */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
