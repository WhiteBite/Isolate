<script lang="ts">
  import type { NetworkRule } from './types';
  import type { ProxyConfig } from '$lib/api';
  import RuleGroupHeader from './RuleGroupHeader.svelte';
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

  // Filter state
  type ActionFilter = 'all' | 'direct' | 'proxy' | 'block' | 'dpi-bypass';
  let activeFilter = $state<ActionFilter>('all');

  // Expanded groups state
  let expandedGroups = $state<Set<string>>(new Set(['direct', 'proxy', 'block', 'dpi-bypass']));

  // Multi-select state
  let selectedRules = $state<Set<string>>(new Set());
  let hasSelection = $derived(selectedRules.size > 0);

  // Live region for screen reader announcements
  let liveAnnouncement = $state('');

  // Action types in display order
  const actionTypes: Array<'direct' | 'proxy' | 'block' | 'dpi-bypass'> = ['direct', 'proxy', 'block', 'dpi-bypass'];

  // Filter buttons config
  const filterConfig = {
    all: { label: 'All', color: 'bg-zinc-500/20 text-zinc-300 border-zinc-500/30' },
    direct: { label: 'Direct', color: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30' },
    proxy: { label: 'Proxy', color: 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30' },
    block: { label: 'Block', color: 'bg-red-500/20 text-red-400 border-red-500/30' },
    'dpi-bypass': { label: 'DPI Bypass', color: 'bg-amber-500/20 text-amber-400 border-amber-500/30' },
  };

  // Group rules by action
  let groupedRules = $derived.by(() => {
    const groups: Record<string, NetworkRule[]> = {
      direct: [],
      proxy: [],
      block: [],
      'dpi-bypass': [],
    };
    
    for (const rule of rules) {
      if (groups[rule.action]) {
        groups[rule.action].push(rule);
      }
    }
    
    return groups;
  });

  // Filtered rules based on active filter
  let filteredRules = $derived(
    activeFilter === 'all' ? rules : rules.filter(r => r.action === activeFilter)
  );

  // Count stats
  let totalCount = $derived(rules.length);
  let filteredCount = $derived(filteredRules.length);

  // Announce message to screen readers
  function announce(message: string) {
    liveAnnouncement = '';
    setTimeout(() => {
      liveAnnouncement = message;
    }, 50);
  }

  // Toggle group expansion
  function toggleGroup(action: string) {
    const newSet = new Set(expandedGroups);
    if (newSet.has(action)) {
      newSet.delete(action);
    } else {
      newSet.add(action);
    }
    expandedGroups = newSet;
  }

  // Expand all groups
  function expandAll() {
    expandedGroups = new Set(actionTypes);
    announce('All groups expanded');
  }

  // Collapse all groups
  function collapseAll() {
    expandedGroups = new Set();
    announce('All groups collapsed');
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

  // Select all visible rules
  function selectAllVisible() {
    selectedRules = new Set(filteredRules.map(r => r.id));
    announce(`Selected ${filteredRules.length} rules`);
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

  // Get enabled count for a group
  function getEnabledCount(action: string): number {
    return groupedRules[action]?.filter(r => r.enabled).length ?? 0;
  }

  // Drag-n-drop state (simplified for grouped view)
  let draggedIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);

  function handleDragStart(e: DragEvent, index: number) {
    draggedIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  function handleDrop(e: DragEvent, index: number) {
    e.preventDefault();
    if (draggedIndex !== null && draggedIndex !== index) {
      onreorder(draggedIndex, index);
    }
    draggedIndex = null;
    dragOverIndex = null;
  }

  function handleDragEnd() {
    draggedIndex = null;
    dragOverIndex = null;
  }
</script>

<div class="bg-zinc-900/30 border border-white/5 rounded-2xl overflow-hidden">
  <!-- Live region for screen reader announcements -->
  <div role="status" aria-live="polite" aria-atomic="true" class="sr-only">
    {liveAnnouncement}
  </div>

  <!-- Header -->
  <div class="flex items-center justify-between p-4 border-b border-white/5">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center">
        <svg class="w-4 h-4 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
        </svg>
      </div>
      <div>
        <h3 class="text-sm font-semibold text-white">Traffic Rules</h3>
        <p class="text-xs text-zinc-400">
          {#if activeFilter === 'all'}
            {totalCount} rule{totalCount !== 1 ? 's' : ''}
          {:else}
            {filteredCount} of {totalCount} rules
          {/if}
        </p>
      </div>
    </div>
    
    <div class="flex items-center gap-2">
      <!-- Expand/Collapse all -->
      <div class="flex items-center gap-1 mr-2">
        <button
          type="button"
          onclick={expandAll}
          class="p-1.5 rounded-lg bg-zinc-800/60 border border-white/5 
                 hover:bg-zinc-700/60 hover:border-white/10 transition-colors"
          title="Expand all groups"
        >
          <svg class="w-3.5 h-3.5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
          </svg>
        </button>
        <button
          type="button"
          onclick={collapseAll}
          class="p-1.5 rounded-lg bg-zinc-800/60 border border-white/5 
                 hover:bg-zinc-700/60 hover:border-white/10 transition-colors"
          title="Collapse all groups"
        >
          <svg class="w-3.5 h-3.5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 9V4.5M9 9H4.5M9 9L3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5l5.25 5.25" />
          </svg>
        </button>
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
  </div>

  <!-- Filter tabs -->
  <div class="flex items-center gap-2 px-4 py-3 border-b border-white/5 bg-zinc-900/20 overflow-x-auto">
    {#each Object.entries(filterConfig) as [key, config]}
      {@const count = key === 'all' ? totalCount : (groupedRules[key]?.length ?? 0)}
      <button
        type="button"
        onclick={() => activeFilter = key as ActionFilter}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border transition-all duration-200 whitespace-nowrap
               {activeFilter === key 
                 ? config.color + ' ring-1 ring-white/10' 
                 : 'bg-zinc-800/40 text-zinc-400 border-zinc-700/50 hover:bg-zinc-800/60 hover:text-zinc-300'}"
      >
        {config.label}
        <span class="px-1.5 py-0.5 rounded-full text-[10px] {activeFilter === key ? 'bg-white/10' : 'bg-zinc-700/50'}">
          {count}
        </span>
      </button>
    {/each}
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
          <div class="p-3 bg-zinc-900/50 rounded-xl border border-white/5 animate-pulse">
            <div class="flex items-center gap-3">
              <div class="w-4 h-4 bg-zinc-800 rounded"></div>
              <div class="w-8 h-8 bg-zinc-800 rounded-lg"></div>
              <div class="flex-1">
                <div class="w-24 h-4 bg-zinc-800 rounded mb-1"></div>
                <div class="w-32 h-3 bg-zinc-800 rounded"></div>
              </div>
              <div class="w-16 h-5 bg-zinc-800 rounded-full"></div>
            </div>
          </div>
        {/each}
      </div>
    {:else if rules.length === 0}
      <!-- Empty state -->
      <div class="text-center py-12">
        <div class="w-16 h-16 mx-auto mb-4 rounded-2xl bg-zinc-900/60 border border-white/5 
                    flex items-center justify-center">
          <svg class="w-8 h-8 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                  d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
          </svg>
        </div>
        <h4 class="text-base font-semibold text-white mb-1">No rules yet</h4>
        <p class="text-sm text-zinc-400 mb-4 max-w-xs mx-auto">
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
    {:else if activeFilter === 'all'}
      <!-- Grouped view -->
      <div class="space-y-4">
        {#each actionTypes as action}
          {@const groupRules = groupedRules[action] ?? []}
          {@const isExpanded = expandedGroups.has(action)}
          
          {#if groupRules.length > 0}
            <div class="space-y-2">
              <RuleGroupHeader
                {action}
                count={groupRules.length}
                enabledCount={getEnabledCount(action)}
                expanded={isExpanded}
                ontoggle={() => toggleGroup(action)}
              />
              
              {#if isExpanded}
                <div 
                  id={`rule-group-${action}`}
                  class="pl-4 space-y-2 border-l-2 border-white/5 ml-4"
                >
                  {#each groupRules as rule, i (rule.id)}
                    {@const globalIndex = rules.findIndex(r => r.id === rule.id)}
                    <RuleCard
                      {rule}
                      index={globalIndex}
                      totalRules={rules.length}
                      gatewayName={getGatewayName(rule.proxyId)}
                      isDragging={draggedIndex === globalIndex}
                      isDropTarget={dragOverIndex === globalIndex && draggedIndex !== globalIndex}
                      isSelected={selectedRules.has(rule.id)}
                      showCheckbox={true}
                      isFocused={false}
                      isGrabbed={false}
                      {ontoggle}
                      {onedit}
                      {ondelete}
                      onselect={toggleSelect}
                      onreorder={(from, to) => onreorder(from, to)}
                      onfocuschange={() => {}}
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
              {/if}
            </div>
          {/if}
        {/each}
        
        <!-- Show empty groups hint if all groups are empty -->
        {#if actionTypes.every(a => (groupedRules[a]?.length ?? 0) === 0)}
          <div class="text-center py-8 text-zinc-500 text-sm">
            No rules in any category
          </div>
        {/if}
      </div>
    {:else}
      <!-- Filtered flat view -->
      <div class="space-y-2">
        {#if filteredRules.length === 0}
          <div class="text-center py-8">
            <p class="text-sm text-zinc-400">No {filterConfig[activeFilter].label.toLowerCase()} rules</p>
          </div>
        {:else}
          {#each filteredRules as rule, i (rule.id)}
            {@const globalIndex = rules.findIndex(r => r.id === rule.id)}
            <RuleCard
              {rule}
              index={globalIndex}
              totalRules={rules.length}
              gatewayName={getGatewayName(rule.proxyId)}
              isDragging={draggedIndex === globalIndex}
              isDropTarget={dragOverIndex === globalIndex && draggedIndex !== globalIndex}
              isSelected={selectedRules.has(rule.id)}
              showCheckbox={true}
              isFocused={false}
              isGrabbed={false}
              {ontoggle}
              {onedit}
              {ondelete}
              onselect={toggleSelect}
              onreorder={(from, to) => onreorder(from, to)}
              onfocuschange={() => {}}
              onclearselection={clearSelection}
              onannounce={announce}
              ondragstart={handleDragStart}
              ondragover={handleDragOver}
              ondragleave={handleDragLeave}
              ondrop={handleDrop}
              ondragend={handleDragEnd}
            />
          {/each}
        {/if}
      </div>
    {/if}
  </div>

  <!-- Footer legend -->
  {#if rules.length > 0}
    <div class="px-4 pb-4">
      <div class="pt-4 border-t border-white/5">
        <div class="flex flex-wrap items-center gap-3 text-xs text-zinc-400">
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
    </div>
  {/if}
</div>

<style>
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
