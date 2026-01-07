<script lang="ts">
  import type { RoutingRule, ProxyConfig } from './types';

  interface Props {
    rule: RoutingRule;
    index: number;
    totalRules?: number;
    proxies: ProxyConfig[];
    isDragging?: boolean;
    isDropTarget?: boolean;
    ontoggle: (rule: RoutingRule) => void;
    onedit: (rule: RoutingRule) => void;
    ondelete: (rule: RoutingRule) => void;
    ondragstart: (e: DragEvent, index: number) => void;
    ondragover: (e: DragEvent, index: number) => void;
    ondragleave: () => void;
    ondrop: (e: DragEvent, index: number) => void;
    ondragend: () => void;
    oncontextmenu: (e: MouseEvent, rule: RoutingRule) => void;
    onreorder?: (fromIndex: number, toIndex: number) => void;
    onfocusitem?: (index: number) => void;
    onannounce?: (message: string) => void;
  }

  let {
    rule,
    index,
    totalRules = 1,
    proxies,
    isDragging = false,
    isDropTarget = false,
    ontoggle,
    onedit,
    ondelete,
    ondragstart,
    ondragover,
    ondragleave,
    ondrop,
    ondragend,
    oncontextmenu,
    onreorder,
    onfocusitem,
    onannounce,
  }: Props = $props();

  function getActionIcon(action: string): string {
    switch (action) {
      case 'direct': return '🌐';
      case 'proxy': return '🔒';
      case 'block': return '🚫';
      default: return '❓';
    }
  }

  function getActionColor(action: string): string {
    switch (action) {
      case 'direct': return 'emerald';
      case 'proxy': return 'indigo';
      case 'block': return 'red';
      default: return 'zinc';
    }
  }

  function getSourceIcon(source: string): string {
    switch (source) {
      case 'all': return '🌍';
      case 'app': return '📱';
      case 'domain': return '🔗';
      default: return '❓';
    }
  }

  function getProxyName(proxyId?: string): string {
    if (!proxyId) return '';
    return proxies.find(p => p.id === proxyId)?.name || proxyId;
  }

  function announce(message: string) {
    onannounce?.(message);
  }

  function getAriaLabel(): string {
    const status = rule.enabled ? 'enabled' : 'disabled';
    const actionText = rule.action === 'proxy' 
      ? `via ${getProxyName(rule.proxyId)}` 
      : rule.action;
    return `Rule ${index + 1} of ${totalRules}: ${rule.name}, ${status}, action: ${actionText}. Press Ctrl+Arrow to reorder.`;
  }

  // Keyboard handler for rule items
  function handleKeyDown(e: KeyboardEvent) {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        if (e.ctrlKey && index > 0) {
          onreorder?.(index, index - 1);
          announce(`Rule moved to position ${index}`);
          setTimeout(() => onfocusitem?.(index - 1), 0);
        } else if (index > 0) {
          onfocusitem?.(index - 1);
        }
        break;
        
      case 'ArrowDown':
        e.preventDefault();
        if (e.ctrlKey && index < totalRules - 1) {
          onreorder?.(index, index + 1);
          announce(`Rule moved to position ${index + 2}`);
          setTimeout(() => onfocusitem?.(index + 1), 0);
        } else if (index < totalRules - 1) {
          onfocusitem?.(index + 1);
        }
        break;
        
      case 'Home':
        e.preventDefault();
        if (e.ctrlKey && index > 0) {
          onreorder?.(index, 0);
          announce('Rule moved to first position');
          setTimeout(() => onfocusitem?.(0), 0);
        } else {
          onfocusitem?.(0);
        }
        break;
        
      case 'End':
        e.preventDefault();
        const lastIndex = totalRules - 1;
        if (e.ctrlKey && index < lastIndex) {
          onreorder?.(index, lastIndex);
          announce('Rule moved to last position');
          setTimeout(() => onfocusitem?.(lastIndex), 0);
        } else {
          onfocusitem?.(lastIndex);
        }
        break;
        
      case 'Enter':
        e.preventDefault();
        onedit(rule);
        break;
        
      case 'Delete':
      case 'Backspace':
        e.preventDefault();
        ondelete(rule);
        announce('Rule deleted');
        break;
        
      case ' ':
        e.preventDefault();
        ontoggle(rule);
        announce(rule.enabled ? 'Rule disabled' : 'Rule enabled');
        break;
    }
  }

  let actionColor = $derived(getActionColor(rule.action));
</script>

<div 
  role="option"
  tabindex="0"
  data-rule-item
  aria-label={getAriaLabel()}
  aria-roledescription="reorderable rule"
  aria-selected={isDragging}
  aria-grabbed={isDragging}
  draggable="true"
  ondragstart={(e) => ondragstart(e, index)}
  ondragover={(e) => ondragover(e, index)}
  ondragleave={ondragleave}
  ondrop={(e) => ondrop(e, index)}
  ondragend={ondragend}
  oncontextmenu={(e) => oncontextmenu(e, rule)}
  onkeydown={handleKeyDown}
  class="group relative bg-zinc-900/40 border rounded-2xl overflow-hidden
         transition-all duration-200 ease-out outline-none
         focus-visible:ring-2 focus-visible:ring-cyan-500/70
         {!rule.enabled ? 'opacity-50' : ''}
         {isDragging ? 'dragging-rule opacity-50 scale-[0.98] shadow-2xl shadow-black/50 z-50' : 'hover:border-white/10'}
         {isDropTarget && !isDragging ? 'drop-target border-indigo-500 ring-2 ring-indigo-500/30 bg-indigo-500/5 scale-[1.01]' : 'border-white/5'}"
  style="animation: slideIn 300ms ease-out {index * 50}ms both"
>
  <!-- Flow Visualization -->
  <div class="p-5">
    <div class="flex items-center gap-4">
      <!-- Drag Handle -->
      <div 
        class="drag-handle flex-shrink-0 cursor-grab active:cursor-grabbing p-1.5 -ml-1 rounded-lg
               text-zinc-600 hover:text-zinc-400 hover:bg-zinc-800/60 transition-all duration-200
               {isDragging ? 'cursor-grabbing text-indigo-400' : ''}"
        title="Drag to reorder (or use Ctrl+↑/↓)"
        aria-hidden="true"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M4 8h16M4 16h16" />
        </svg>
      </div>

      <!-- Toggle -->
      <button
        onclick={() => ontoggle(rule)}
        aria-label="Toggle rule"
        role="switch"
        aria-checked={rule.enabled}
        class="relative w-12 h-6 rounded-full transition-colors duration-200
               {rule.enabled ? 'bg-emerald-500/20' : 'bg-zinc-800'}"
      >
        <div class="absolute top-1 left-1 w-4 h-4 rounded-full transition-all duration-200
                    {rule.enabled ? 'translate-x-6 bg-emerald-400' : 'bg-zinc-500'}">
        </div>
      </button>

      <!-- Rule Name -->
      <div class="flex-1 min-w-0">
        <h3 class="text-lg font-semibold text-white truncate">{rule.name}</h3>
      </div>

      <!-- Actions -->
      <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity">
        <button
          onclick={() => onedit(rule)}
          class="p-2 rounded-lg bg-zinc-800/60 border border-white/5 
                 hover:bg-zinc-700/60 hover:border-white/10 transition-colors
                 focus-visible:ring-2 focus-visible:ring-cyan-500/70 outline-none"
          title="Edit"
          aria-label="Edit rule"
        >
          <svg class="w-4 h-4 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
        </button>
        <button
          onclick={() => ondelete(rule)}
          class="p-2 rounded-lg bg-red-500/10 border border-red-500/20 
                 hover:bg-red-500/20 hover:border-red-500/30 transition-colors
                 focus-visible:ring-2 focus-visible:ring-red-500/70 outline-none"
          title="Delete"
          aria-label="Delete rule"
        >
          <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Flow Cards -->
    <div class="mt-4 flex items-center gap-3">
      <!-- Source Card -->
      <div class="flex-1 p-4 bg-black/40 border border-white/5 rounded-xl">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-zinc-700/50 flex items-center justify-center text-xl">
            {getSourceIcon(rule.source)}
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs text-zinc-400 uppercase tracking-wider">Source</div>
            <div class="text-sm font-medium text-white truncate">
              {#if rule.source === 'all'}
                All Traffic
              {:else if rule.source === 'app'}
                App: {rule.sourceValue}
              {:else}
                {rule.sourceValue}
              {/if}
            </div>
          </div>
        </div>
      </div>

      <!-- Arrow -->
      <div class="flex-shrink-0 flex items-center" aria-hidden="true">
        <svg class="w-10 h-10 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                d="M13 7l5 5m0 0l-5 5m5-5H6" />
        </svg>
      </div>

      <!-- Condition Card (optional visual) -->
      <div class="flex-shrink-0 w-12 h-12 rounded-xl bg-black/40 border border-white/5 
                  flex items-center justify-center" aria-hidden="true">
        <svg class="w-5 h-5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
        </svg>
      </div>

      <!-- Arrow -->
      <div class="flex-shrink-0 flex items-center" aria-hidden="true">
        <svg class="w-10 h-10 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                d="M13 7l5 5m0 0l-5 5m5-5H6" />
        </svg>
      </div>

      <!-- Action Card -->
      <div class="flex-1 p-4 rounded-xl border
                  {actionColor === 'emerald' ? 'bg-emerald-500/10 border-emerald-500/20' : ''}
                  {actionColor === 'indigo' ? 'bg-indigo-500/10 border-indigo-500/20' : ''}
                  {actionColor === 'red' ? 'bg-red-500/10 border-red-500/20' : ''}">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg flex items-center justify-center text-xl
                      {actionColor === 'emerald' ? 'bg-emerald-500/20' : ''}
                      {actionColor === 'indigo' ? 'bg-indigo-500/20' : ''}
                      {actionColor === 'red' ? 'bg-red-500/20' : ''}">
            {getActionIcon(rule.action)}
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-xs uppercase tracking-wider
                        {actionColor === 'emerald' ? 'text-emerald-400/60' : ''}
                        {actionColor === 'indigo' ? 'text-indigo-400/60' : ''}
                        {actionColor === 'red' ? 'text-red-400/60' : ''}">
              Action
            </div>
            <div class="text-sm font-medium truncate
                        {actionColor === 'emerald' ? 'text-emerald-400' : ''}
                        {actionColor === 'indigo' ? 'text-indigo-400' : ''}
                        {actionColor === 'red' ? 'text-red-400' : ''}">
              {#if rule.action === 'direct'}
                Direct Connection
              {:else if rule.action === 'proxy'}
                Via {getProxyName(rule.proxyId)}
              {:else}
                Blocked
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Bottom accent line -->
  <div class="h-0.5 
              {actionColor === 'emerald' ? 'bg-gradient-to-r from-emerald-500/50 to-transparent' : ''}
              {actionColor === 'indigo' ? 'bg-gradient-to-r from-indigo-500/50 to-transparent' : ''}
              {actionColor === 'red' ? 'bg-gradient-to-r from-red-500/50 to-transparent' : ''}"
       aria-hidden="true">
  </div>
</div>

<style>
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .dragging-rule {
    cursor: grabbing !important;
    border-color: rgba(99, 102, 241, 0.4) !important;
  }

  .drop-target {
    animation: pulse-border 1s ease-in-out infinite;
  }

  @keyframes pulse-border {
    0%, 100% {
      box-shadow: 0 0 0 0 rgba(99, 102, 241, 0.3);
    }
    50% {
      box-shadow: 0 0 0 4px rgba(99, 102, 241, 0.1);
    }
  }

  .drag-handle {
    touch-action: none;
  }

  .drag-handle:hover svg {
    transform: scale(1.1);
  }
</style>
