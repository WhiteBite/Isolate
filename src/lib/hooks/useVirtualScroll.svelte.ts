/**
 * Virtual scroll hook for efficient rendering of large lists
 * Uses Svelte 5 runes for reactivity
 */
export interface VirtualScrollOptions {
  /** Total number of items */
  itemCount: number;
  /** Height of each item in pixels */
  itemHeight: number;
  /** Number of items to render outside visible area */
  overscan?: number;
  /** Container height (if not using ref) */
  containerHeight?: number;
}

export interface VirtualScrollResult<T> {
  /** Items to render (slice of original array) */
  visibleItems: T[];
  /** Start index of visible items */
  startIndex: number;
  /** End index of visible items */
  endIndex: number;
  /** Total height of all items */
  totalHeight: number;
  /** Offset from top for first visible item */
  offsetTop: number;
  /** Handle scroll event */
  onScroll: (event: Event) => void;
  /** Scroll to specific index */
  scrollToIndex: (index: number, behavior?: ScrollBehavior) => void;
}

export function useVirtualScroll<T>(
  items: () => T[],
  options: () => VirtualScrollOptions
): VirtualScrollResult<T> {
  let scrollTop = $state(0);
  let containerRef = $state<HTMLElement | null>(null);

  const opts = $derived(options());
  const allItems = $derived(items());
  
  const containerHeight = $derived(
    opts.containerHeight ?? containerRef?.clientHeight ?? 400
  );
  
  const overscan = $derived(opts.overscan ?? 3);
  
  const totalHeight = $derived(allItems.length * opts.itemHeight);
  
  const startIndex = $derived(
    Math.max(0, Math.floor(scrollTop / opts.itemHeight) - overscan)
  );
  
  const visibleCount = $derived(
    Math.ceil(containerHeight / opts.itemHeight) + 2 * overscan
  );
  
  const endIndex = $derived(
    Math.min(allItems.length, startIndex + visibleCount)
  );
  
  const offsetTop = $derived(startIndex * opts.itemHeight);
  
  const visibleItems = $derived(
    allItems.slice(startIndex, endIndex)
  );

  function onScroll(event: Event) {
    const target = event.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  function scrollToIndex(index: number, behavior: ScrollBehavior = 'auto') {
    if (containerRef) {
      const top = index * opts.itemHeight;
      containerRef.scrollTo({ top, behavior });
    }
  }

  function setContainerRef(el: HTMLElement | null) {
    containerRef = el;
  }

  return {
    get visibleItems() { return visibleItems; },
    get startIndex() { return startIndex; },
    get endIndex() { return endIndex; },
    get totalHeight() { return totalHeight; },
    get offsetTop() { return offsetTop; },
    onScroll,
    scrollToIndex,
    setContainerRef
  } as VirtualScrollResult<T> & { setContainerRef: (el: HTMLElement | null) => void };
}

/**
 * Simplified virtual scroll for fixed-height containers
 */
export function useSimpleVirtualScroll<T>(
  items: T[],
  itemHeight: number,
  containerHeight: number,
  overscan = 3
) {
  let scrollTop = $state(0);

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
  
  const visibleItems = $derived(items.slice(startIndex, endIndex));

  return {
    get visibleItems() { return visibleItems; },
    get startIndex() { return startIndex; },
    get endIndex() { return endIndex; },
    get totalHeight() { return totalHeight; },
    get offsetTop() { return offsetTop; },
    setScrollTop: (value: number) => { scrollTop = value; }
  };
}
