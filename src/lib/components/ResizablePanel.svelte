<script lang="ts">
  import { getContext } from 'svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    defaultSize?: number;
    minSize?: number;
    maxSize?: number;
    collapsible?: boolean;
    id?: string;
    children?: Snippet;
  }

  let { 
    defaultSize = 50,
    minSize = 10,
    maxSize = 100,
    collapsible = false,
    id,
    children 
  }: Props = $props();

  // Generate unique ID if not provided - use $derived for reactivity
  let panelId = $derived(id ?? `panel-${Math.random().toString(36).slice(2, 9)}`);

  // Get context from parent group
  const ctx = getContext<{
    direction: () => 'horizontal' | 'vertical';
    registerPanel: (id: string, defaultSize: number, minSize: number, maxSize: number, collapsible: boolean) => void;
    unregisterPanel: (id: string) => void;
    getPanelSize: (id: string) => number;
    isPanelCollapsed: (id: string) => boolean;
    expandPanel: (id: string) => void;
  }>('resizable-panel-group');

  let size = $state(defaultSize);
  let collapsed = $state(false);

  // Register panel on mount and cleanup on destroy
  $effect(() => {
    const currentId = panelId;
    const currentDefaultSize = defaultSize;
    if (ctx) {
      ctx.registerPanel(currentId, currentDefaultSize, minSize, maxSize, collapsible);
    }
    
    return () => {
      if (ctx) {
        ctx.unregisterPanel(currentId);
      }
    };
  });

  // Reactive size from context
  $effect(() => {
    const currentId = panelId;
    if (ctx) {
      size = ctx.getPanelSize(currentId);
      collapsed = ctx.isPanelCollapsed(currentId);
    }
  });

  function handleExpand() {
    const currentId = panelId;
    if (ctx && collapsed) {
      ctx.expandPanel(currentId);
    }
  }

  let direction = $derived(ctx?.direction() ?? 'horizontal');
</script>

<div 
  class="relative overflow-hidden transition-[flex-basis] duration-75"
  style="flex: {collapsed ? '0 0 0%' : `${size} 1 0%`}; {direction === 'horizontal' ? 'min-width' : 'min-height'}: {collapsed ? '0' : 'auto'};"
  data-panel-id={panelId}
  data-collapsed={collapsed}
>
  {#if collapsed && collapsible}
    <!-- Collapsed state indicator -->
    <button
      class="absolute inset-0 flex items-center justify-center bg-[#1a1f3a]/50 hover:bg-[#1a1f3a]/70 transition-colors cursor-pointer z-10"
      onclick={handleExpand}
      aria-label="Expand panel"
    >
      <svg 
        class="w-4 h-4 text-white/50 {direction === 'horizontal' ? '' : 'rotate-90'}" 
        fill="none" 
        stroke="currentColor" 
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
      </svg>
    </button>
  {:else}
    <div class="h-full w-full overflow-auto">
      {#if children}
        {@render children()}
      {/if}
    </div>
  {/if}
</div>
