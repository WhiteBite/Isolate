<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Array of items to render */
    items: T[];
    /** Number of columns in the grid */
    columns?: number;
    /** Height of each row in pixels */
    rowHeight?: number;
    /** Gap between items in pixels */
    gap?: number;
    /** Additional CSS class for the container */
    class?: string;
    /** Render snippet for each item */
    children: Snippet<[T, number]>;
  }

  let {
    items,
    columns = 3,
    rowHeight = 100,
    gap = 16,
    class: className = '',
    children
  }: Props = $props();

  // Container element reference
  let containerRef = $state<HTMLElement | null>(null);
  let scrollTop = $state(0);
  let containerHeight = $state(400);

  // Number of items to render outside visible area for smooth scrolling
  const overscan = 2;

  // Calculate total rows
  const totalRows = $derived(Math.ceil(items.length / columns));

  // Total height of all content
  const totalHeight = $derived(totalRows * rowHeight + (totalRows - 1) * gap);

  // Calculate visible range
  const effectiveRowHeight = $derived(rowHeight + gap);
  
  const startRow = $derived(
    Math.max(0, Math.floor(scrollTop / effectiveRowHeight) - overscan)
  );
  
  const visibleRowCount = $derived(
    Math.ceil(containerHeight / effectiveRowHeight) + 2 * overscan
  );
  
  const endRow = $derived(
    Math.min(totalRows, startRow + visibleRowCount)
  );

  // Calculate visible items
  const startIndex = $derived(startRow * columns);
  const endIndex = $derived(Math.min(items.length, endRow * columns));
  
  const visibleItems = $derived(
    items.slice(startIndex, endIndex).map((item, i) => ({
      item,
      index: startIndex + i
    }))
  );

  // Offset for positioning visible items
  const offsetTop = $derived(startRow * effectiveRowHeight);

  // Handle scroll event
  function handleScroll(event: Event) {
    const target = event.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  // Update container height on resize
  $effect(() => {
    if (!containerRef) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerHeight = entry.contentRect.height;
      }
    });

    observer.observe(containerRef);

    return () => observer.disconnect();
  });

  // Calculate item position in grid
  function getItemStyle(index: number): string {
    const localIndex = index - startIndex;
    const row = Math.floor(localIndex / columns);
    const col = localIndex % columns;
    
    const top = row * effectiveRowHeight;
    const left = col * (100 / columns);
    const width = 100 / columns;
    
    return `
      position: absolute;
      top: ${top}px;
      left: calc(${left}% + ${col > 0 ? gap / 2 : 0}px);
      width: calc(${width}% - ${gap - gap / columns}px);
      height: ${rowHeight}px;
    `;
  }
</script>

<div
  bind:this={containerRef}
  class="relative overflow-auto {className}"
  onscroll={handleScroll}
>
  <!-- Spacer to maintain scroll height -->
  <div style="height: {totalHeight}px; position: relative;">
    <!-- Visible items container -->
    <div
      style="position: absolute; top: {offsetTop}px; left: 0; right: 0;"
    >
      {#each visibleItems as { item, index } (index)}
        <div style={getItemStyle(index)}>
          {@render children(item, index)}
        </div>
      {/each}
    </div>
  </div>
</div>
