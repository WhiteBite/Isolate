<script lang="ts">
  import VirtualList from './VirtualList.svelte';

  interface Props {
    /** Array of host strings */
    hosts: string[];
    /** Height of each item in pixels */
    itemHeight?: number;
    /** Container height in pixels */
    height?: number;
    /** Enable selection */
    selectable?: boolean;
    /** Selected hosts */
    selected?: Set<string>;
    /** Enable grouping by first letter */
    grouped?: boolean;
    /** Additional CSS classes */
    class?: string;
    /** Callback when host is selected */
    onSelect?: (host: string) => void;
    /** Callback when host is deleted */
    onDelete?: (host: string) => void;
  }

  let {
    hosts,
    itemHeight = 36,
    height = 400,
    selectable = false,
    selected = new Set<string>(),
    grouped = false,
    class: className = '',
    onSelect,
    onDelete
  }: Props = $props();

  // Process hosts for display
  interface HostItem {
    type: 'host' | 'header';
    value: string;
    letter?: string;
  }

  const processedItems = $derived.by((): HostItem[] => {
    if (!grouped) {
      return hosts.map(h => ({ type: 'host' as const, value: h }));
    }

    const items: HostItem[] = [];
    let currentLetter = '';
    
    const sorted = [...hosts].sort((a, b) => a.localeCompare(b));
    
    for (const host of sorted) {
      const letter = host[0]?.toUpperCase() || '#';
      if (letter !== currentLetter) {
        currentLetter = letter;
        items.push({ type: 'header', value: letter, letter });
      }
      items.push({ type: 'host', value: host });
    }
    
    return items;
  });

  function handleSelect(host: string) {
    if (selectable && onSelect) {
      onSelect(host);
    }
  }

  function handleDelete(host: string, event: Event) {
    event.stopPropagation();
    if (onDelete) {
      onDelete(host);
    }
  }

  function isSelected(host: string): boolean {
    return selected.has(host);
  }
</script>

<VirtualList
  items={processedItems}
  {itemHeight}
  {height}
  class={className}
>
  {#snippet renderItem(item, index)}
    {#if item.type === 'header'}
      <div class="flex items-center px-3 bg-gray-100 dark:bg-gray-800 font-semibold text-sm text-gray-600 dark:text-gray-400 h-full">
        {item.value}
      </div>
    {:else}
      <button
        type="button"
        class="flex items-center justify-between w-full px-3 h-full text-left hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors
          {isSelected(item.value) ? 'bg-blue-50 dark:bg-blue-900/30' : ''}"
        onclick={() => handleSelect(item.value)}
        disabled={!selectable}
      >
        <span class="truncate font-mono text-sm">{item.value}</span>
        
        {#if onDelete}
          <button
            type="button"
            class="ml-2 p-1 text-gray-400 hover:text-red-500 transition-colors"
            onclick={(e) => handleDelete(item.value, e)}
            aria-label="Delete {item.value}"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        {/if}
      </button>
    {/if}
  {/snippet}

  {#snippet empty()}
    <div class="flex items-center justify-center h-full text-gray-500">
      No hosts
    </div>
  {/snippet}
</VirtualList>
