<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Array of items to render */
    items: T[];
    /** Height of each item in pixels */
    itemHeight: number;
    /** Number of items to render outside visible area */
    overscan?: number;
    /** Container height in pixels (or use class for CSS height) */
    height?: number;
    /** Additional CSS classes for container */
    class?: string;
    /** Render function for each item */
    renderItem: Snippet<[item: T, index: number]>;
    /** Optional empty state */
    empty?: Snippet;
  }

  let {
    items,
    itemHeight,
    overscan = 3,
    height = 400,
    class: className = '',
    renderItem,
    empty
  }: Props = $props();

  let scrollTop = $state(0);
  let containerRef = $state<HTMLElement | null>(null);
  let containerHeight = $state(height);

  // Update container height when ref changes or height prop changes
  $effect(() => {
    if (containerRef) {
      containerHeight = containerRef.clientHeight || height;
    } else {
      containerHeight = height;
    }
  });

  // Calculate visible range
  const totalHeight = $derived(items.length * itemHeight);
  
  const startIndex = $derived(
    Math.max(0, Math.floor(scrollTop / itemHeight) - overscan)
  );
  
  const visibleCount = $derived(
    Math.ceil(containerHeight / itemHeight) + 2 * overscan
  );
  
  const endIndex = $derived(
    Math.min(items.length, startIndex + visibleCount)
  );
  
  const offsetTop = $derived(startIndex * itemHeight);
  
  const visibleItems = $derived(
    items.slice(startIndex, endIndex).map((item, i) => ({
      item,
      index: startIndex + i
    }))
  );

  function handleScroll(event: Event) {
    const target = event.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  export function scrollToIndex(index: number, behavior: ScrollBehavior = 'auto') {
    if (containerRef) {
      const top = Math.max(0, Math.min(index * itemHeight, totalHeight - containerHeight));
      containerRef.scrollTo({ top, behavior });
    }
  }

  export function scrollToTop() {
    scrollToIndex(0);
  }

  export function scrollToBottom() {
    scrollToIndex(items.length - 1);
  }
</script>

<div
  bind:this={containerRef}
  class="virtual-list-container overflow-auto {className}"
  style:height="{height}px"
  onscroll={handleScroll}
>
  {#if items.length === 0}
    {#if empty}
      {@render empty()}
    {:else}
      <div class="flex items-center justify-center h-full text-gray-500">
        No items
      </div>
    {/if}
  {:else}
    <!-- Spacer for total scroll height -->
    <div class="virtual-list-spacer relative" style:height="{totalHeight}px">
      <!-- Visible items container -->
      <div 
        class="virtual-list-items absolute left-0 right-0"
        style:transform="translateY({offsetTop}px)"
      >
        {#each visibleItems as { item, index } (index)}
          <div 
            class="virtual-list-item"
            style:height="{itemHeight}px"
          >
            {@render renderItem(item, index)}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .virtual-list-container {
    will-change: scroll-position;
  }
  
  .virtual-list-items {
    will-change: transform;
  }
  
  .virtual-list-item {
    overflow: hidden;
  }
</style>
