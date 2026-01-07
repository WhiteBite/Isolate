<script lang="ts">
  import { ProxyCard } from '$lib/components';
  import type { ProxyConfig } from '$lib/api';

  interface Props {
    proxies: ProxyConfig[];
    loading: boolean;
    error: string | null;
    draggedIndex: number | null;
    dragOverIndex: number | null;
    onEdit: (proxy: ProxyConfig) => void;
    onDelete: (id: string) => void;
    onToggle: (id: string) => void;
    onCopy: (proxy: ProxyConfig) => void;
    onShare: (proxy: ProxyConfig) => void;
    onContextMenu: (event: MouseEvent, proxy: ProxyConfig) => void;
    onDragStart: (e: DragEvent, index: number) => void;
    onDragOver: (e: DragEvent, index: number) => void;
    onDragLeave: () => void;
    onDrop: (e: DragEvent, index: number) => void;
    onDragEnd: () => void;
    onReorder?: (fromIndex: number, toIndex: number) => void;
  }

  let {
    proxies,
    loading,
    error,
    draggedIndex,
    dragOverIndex,
    onEdit,
    onDelete,
    onToggle,
    onCopy,
    onShare,
    onContextMenu,
    onDragStart,
    onDragOver,
    onDragLeave,
    onDrop,
    onDragEnd,
    onReorder
  }: Props = $props();

  // Live region for screen reader announcements
  let liveAnnouncement = $state('');
  
  function announce(message: string) {
    liveAnnouncement = '';
    setTimeout(() => { liveAnnouncement = message; }, 50);
  }

  // Keyboard handler for proxy items
  function handleKeyDown(e: KeyboardEvent, index: number) {
    const proxy = proxies[index];
    
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        if (e.ctrlKey && index > 0) {
          onReorder?.(index, index - 1);
          announce(`Proxy moved to position ${index}`);
          setTimeout(() => focusItem(index - 1), 0);
        } else if (index > 0) {
          focusItem(index - 1);
        }
        break;
        
      case 'ArrowDown':
        e.preventDefault();
        if (e.ctrlKey && index < proxies.length - 1) {
          onReorder?.(index, index + 1);
          announce(`Proxy moved to position ${index + 2}`);
          setTimeout(() => focusItem(index + 1), 0);
        } else if (index < proxies.length - 1) {
          focusItem(index + 1);
        }
        break;
        
      case 'Home':
        e.preventDefault();
        if (e.ctrlKey && index > 0) {
          onReorder?.(index, 0);
          announce('Proxy moved to first position');
          setTimeout(() => focusItem(0), 0);
        } else {
          focusItem(0);
        }
        break;
        
      case 'End':
        e.preventDefault();
        const lastIndex = proxies.length - 1;
        if (e.ctrlKey && index < lastIndex) {
          onReorder?.(index, lastIndex);
          announce('Proxy moved to last position');
          setTimeout(() => focusItem(lastIndex), 0);
        } else {
          focusItem(lastIndex);
        }
        break;
        
      case 'Enter':
        e.preventDefault();
        onEdit(proxy);
        break;
        
      case 'Delete':
      case 'Backspace':
        e.preventDefault();
        onDelete(proxy.id);
        announce('Proxy deleted');
        break;
        
      case ' ':
        e.preventDefault();
        onToggle(proxy.id);
        announce(proxy.active ? 'Proxy deactivated' : 'Proxy activated');
        break;
    }
  }
  
  function focusItem(index: number) {
    const items = document.querySelectorAll('[data-proxy-item]');
    (items[index] as HTMLElement)?.focus();
  }

  function getAriaLabel(proxy: ProxyConfig, index: number): string {
    const status = proxy.active ? 'active' : 'inactive';
    return `Proxy ${index + 1} of ${proxies.length}: ${proxy.name}, ${proxy.protocol}, ${status}. Press Ctrl+Arrow to reorder.`;
  }

  function getProtocolDisplay(protocol: string): string {
    const map: Record<string, string> = {
      vless: 'VLESS',
      vmess: 'VMess',
      shadowsocks: 'Shadowsocks',
      trojan: 'Trojan',
      socks5: 'SOCKS5',
      http: 'HTTP'
    };
    return map[protocol] || protocol.toUpperCase();
  }
</script>

{#snippet skeletonLoader()}
  <div class="space-y-3">
    {#each Array(3) as _}
      <div class="animate-pulse bg-void-50 border border-glass-border rounded-xl p-4">
        <div class="flex items-center gap-4">
          <div class="w-10 h-10 bg-void-100 rounded-lg"></div>
          <div class="flex-1 space-y-2">
            <div class="h-4 bg-void-100 rounded w-1/3"></div>
            <div class="h-3 bg-void-100 rounded w-1/2"></div>
          </div>
          <div class="w-20 h-8 bg-void-100 rounded-lg"></div>
        </div>
      </div>
    {/each}
  </div>
{/snippet}

{#snippet emptyState()}
  <div class="text-center py-12 bg-void-50 border border-glass-border rounded-xl">
    <div class="text-4xl mb-4">🌐</div>
    <p class="text-lg text-text-primary">No proxies added</p>
    <p class="text-sm mt-2 text-text-muted">Click Add or paste a link (Ctrl+V)</p>
  </div>
{/snippet}

<!-- Live region for announcements -->
<div role="status" aria-live="polite" aria-atomic="true" class="sr-only">
  {liveAnnouncement}
</div>

<div 
  role="listbox" 
  aria-label="Proxy list. Use arrow keys to navigate, Ctrl+Arrow to reorder."
  aria-describedby="proxy-list-instructions"
  class="space-y-3"
>
  <div id="proxy-list-instructions" class="sr-only">
    Use Tab to enter the list, Arrow keys to navigate between proxies, 
    Ctrl+Arrow Up or Down to reorder, Enter to edit, Space to toggle, Delete to remove.
  </div>
  {#if loading}
    {@render skeletonLoader()}
  {:else if error}
    <div role="alert" class="text-center py-12 text-red-400">{error}</div>
  {:else if proxies.length === 0}
    {@render emptyState()}
  {:else}
    {#each proxies as proxy, i (proxy.id)}
      <div 
        role="option"
        tabindex="0"
        data-proxy-item
        aria-label={getAriaLabel(proxy, i)}
        aria-roledescription="reorderable proxy"
        aria-selected={draggedIndex === i}
        aria-grabbed={draggedIndex === i}
        draggable="true"
        ondragstart={(e) => onDragStart(e, i)}
        ondragover={(e) => onDragOver(e, i)}
        ondragleave={onDragLeave}
        ondrop={(e) => onDrop(e, i)}
        ondragend={onDragEnd}
        onkeydown={(e) => handleKeyDown(e, i)}
        oncontextmenu={(e) => onContextMenu(e, proxy)}
        class="transform transition-all duration-200 hover:-translate-y-0.5 outline-none
               focus-visible:ring-2 focus-visible:ring-cyan-500/70 focus-visible:rounded-xl
               {draggedIndex === i ? 'opacity-40 scale-[0.98]' : ''}
               {dragOverIndex === i && draggedIndex !== i ? 'ring-2 ring-indigo-500/40 rounded-xl' : ''}" 
        style="animation-delay: {i * 50}ms"
      >
        <div class="flex items-center gap-3">
          <!-- Drag Handle -->
          <div 
            class="flex-shrink-0 cursor-grab active:cursor-grabbing p-2 rounded-lg
                   text-text-muted hover:text-text-secondary hover:bg-void-100 transition-all duration-200"
            title="Drag to reorder"
            aria-hidden="true"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M4 8h16M4 16h16" />
            </svg>
          </div>
          
          <!-- Proxy Card -->
          <div class="flex-1">
            <ProxyCard
              id={proxy.id}
              name={proxy.name}
              server={proxy.server}
              port={proxy.port}
              protocol={getProtocolDisplay(proxy.protocol)}
              country={proxy.country}
              ping={proxy.ping}
              active={proxy.active}
              onEdit={() => onEdit(proxy)}
              onDelete={() => onDelete(proxy.id)}
              onToggle={() => onToggle(proxy.id)}
              onCopy={() => onCopy(proxy)}
              onShare={() => onShare(proxy)}
            />
          </div>
        </div>
      </div>
    {/each}
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
