<script lang="ts">
  import type { RoutingRule, ProxyConfig } from './types';
  import RuleCard from './RuleCard.svelte';
  import { RoutingSkeleton } from '$lib/components/skeletons';

  interface Props {
    rules: RoutingRule[];
    proxies: ProxyConfig[];
    loading: boolean;
    draggedIndex: number | null;
    dragOverIndex: number | null;
    ontoggle: (rule: RoutingRule) => void;
    onedit: (rule: RoutingRule) => void;
    ondelete: (rule: RoutingRule) => void;
    onadd: () => void;
    ondragstart: (e: DragEvent, index: number) => void;
    ondragover: (e: DragEvent, index: number) => void;
    ondragleave: () => void;
    ondrop: (e: DragEvent, index: number) => void;
    ondragend: () => void;
    oncontextmenu: (e: MouseEvent, rule: RoutingRule) => void;
    onreorder?: (fromIndex: number, toIndex: number) => void;
  }

  let {
    rules,
    proxies,
    loading,
    draggedIndex,
    dragOverIndex,
    ontoggle,
    onedit,
    ondelete,
    onadd,
    ondragstart,
    ondragover,
    ondragleave,
    ondrop,
    ondragend,
    oncontextmenu,
    onreorder,
  }: Props = $props();

  // Live region for screen reader announcements
  let liveAnnouncement = $state('');
  
  function announce(message: string) {
    liveAnnouncement = '';
    setTimeout(() => { liveAnnouncement = message; }, 50);
  }

  function focusItem(index: number) {
    const items = document.querySelectorAll('[data-rule-item]');
    (items[index] as HTMLElement)?.focus();
  }
</script>

<!-- Live region for announcements -->
<div role="status" aria-live="polite" aria-atomic="true" class="sr-only">
  {liveAnnouncement}
</div>

{#if loading}
  <RoutingSkeleton />
{:else if rules.length === 0}
  <!-- Empty State -->
  <div class="text-center py-20">
    <div class="w-20 h-20 mx-auto mb-6 rounded-2xl bg-zinc-900/60 border border-white/5 
                flex items-center justify-center">
      <svg class="w-10 h-10 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
              d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
      </svg>
    </div>
    <h3 class="text-xl font-semibold text-white mb-2">No routing rules yet</h3>
    <p class="text-zinc-400 mb-6 max-w-md mx-auto">
      Create rules to control how traffic flows through your network
    </p>
    <button
      onclick={onadd}
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
  <div 
    role="listbox" 
    aria-label="Routing rules. Use arrow keys to navigate, Ctrl+Arrow to reorder."
    aria-describedby="rule-list-instructions"
    class="space-y-4"
  >
    <div id="rule-list-instructions" class="sr-only">
      Use Tab to enter the list, Arrow keys to navigate between rules, 
      Ctrl+Arrow Up or Down to reorder, Enter to edit, Space to toggle, Delete to remove.
    </div>
    {#each rules as rule, i (rule.id)}
      <RuleCard
        {rule}
        index={i}
        totalRules={rules.length}
        {proxies}
        isDragging={draggedIndex === i}
        isDropTarget={dragOverIndex === i && draggedIndex !== i}
        {ontoggle}
        {onedit}
        {ondelete}
        {ondragstart}
        {ondragover}
        {ondragleave}
        {ondrop}
        {ondragend}
        {oncontextmenu}
        {onreorder}
        onfocusitem={focusItem}
        onannounce={announce}
      />
    {/each}
  </div>
{/if}

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
