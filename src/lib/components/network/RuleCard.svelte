<script lang="ts">
  import type { NetworkRule } from './types';

  interface Props {
    rule: NetworkRule;
    index: number;
    totalRules: number;
    gatewayName?: string;
    isDragging?: boolean;
    isDropTarget?: boolean;
    isSelected?: boolean;
    showCheckbox?: boolean;
    isFocused?: boolean;
    isGrabbed?: boolean;
    ontoggle: (id: string) => void;
    onedit: (rule: NetworkRule) => void;
    ondelete: (id: string) => void;
    onselect?: (id: string) => void;
    onreorder?: (fromIndex: number, toIndex: number) => void;
    onfocuschange?: (index: number) => void;
    onclearselection?: () => void;
    onannounce?: (message: string) => void;
    ondragstart: (e: DragEvent, index: number) => void;
    ondragover: (e: DragEvent, index: number) => void;
    ondragleave: () => void;
    ondrop: (e: DragEvent, index: number) => void;
    ondragend: () => void;
  }

  let {
    rule,
    index,
    totalRules,
    gatewayName,
    isDragging = false,
    isDropTarget = false,
    isSelected = false,
    showCheckbox = false,
    isFocused = false,
    isGrabbed = false,
    ontoggle,
    onedit,
    ondelete,
    onselect,
    onreorder,
    onfocuschange,
    onclearselection,
    onannounce,
    ondragstart,
    ondragover,
    ondragleave,
    ondrop,
    ondragend,
  }: Props = $props();

  // Announce action to screen readers
  function announce(message: string) {
    onannounce?.(message);
  }

  // Keyboard navigation handler
  function handleKeyDown(e: KeyboardEvent) {
    // Escape: Clear selection or cancel grab mode
    if (e.key === 'Escape') {
      e.preventDefault();
      onclearselection?.();
      announce('Selection cleared');
      return;
    }

    // Home: Jump to first rule
    if (e.key === 'Home') {
      e.preventDefault();
      if (e.ctrlKey && index > 0) {
        // Ctrl+Home: Move rule to top
        onreorder?.(index, 0);
        setTimeout(() => onfocuschange?.(0), 0);
        announce(`Rule moved to position 1`);
      } else {
        onfocuschange?.(0);
      }
      return;
    }

    // End: Jump to last rule
    if (e.key === 'End') {
      e.preventDefault();
      if (e.ctrlKey && index < totalRules - 1) {
        // Ctrl+End: Move rule to bottom
        onreorder?.(index, totalRules - 1);
        setTimeout(() => onfocuschange?.(totalRules - 1), 0);
        announce(`Rule moved to position ${totalRules}`);
      } else {
        onfocuschange?.(totalRules - 1);
      }
      return;
    }

    // Ctrl+Arrow Up: Move rule up
    if (e.ctrlKey && e.key === 'ArrowUp' && index > 0) {
      e.preventDefault();
      onreorder?.(index, index - 1);
      // Focus will follow the moved item
      setTimeout(() => onfocuschange?.(index - 1), 0);
      announce(`Rule moved to position ${index}`);
      return;
    }
    
    // Ctrl+Arrow Down: Move rule down
    if (e.ctrlKey && e.key === 'ArrowDown' && index < totalRules - 1) {
      e.preventDefault();
      onreorder?.(index, index + 1);
      // Focus will follow the moved item
      setTimeout(() => onfocuschange?.(index + 1), 0);
      announce(`Rule moved to position ${index + 2}`);
      return;
    }
    
    // Arrow Up: Move focus to previous rule
    if (e.key === 'ArrowUp' && index > 0) {
      e.preventDefault();
      onfocuschange?.(index - 1);
      return;
    }
    
    // Arrow Down: Move focus to next rule
    if (e.key === 'ArrowDown' && index < totalRules - 1) {
      e.preventDefault();
      onfocuschange?.(index + 1);
      return;
    }
    
    // Space: Toggle selection
    if (e.key === ' ' && showCheckbox) {
      e.preventDefault();
      onselect?.(rule.id);
      announce(isSelected ? 'Rule deselected' : 'Rule selected');
      return;
    }
    
    // Enter: Edit rule
    if (e.key === 'Enter') {
      e.preventDefault();
      onedit(rule);
      return;
    }
    
    // Delete or Backspace: Delete rule
    if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      ondelete(rule.id);
      announce('Rule deleted');
      return;
    }
    
    // T: Toggle enabled/disabled
    if (e.key === 't' || e.key === 'T') {
      e.preventDefault();
      ontoggle(rule.id);
      announce(rule.enabled ? 'Rule disabled' : 'Rule enabled');
      return;
    }

    // Ctrl+A: Select all (bubble up to parent)
    if (e.ctrlKey && (e.key === 'a' || e.key === 'A')) {
      // Let it bubble to RuleList
      return;
    }
  }

  // Generate aria-label for the rule
  let ariaLabel = $derived(() => {
    const status = rule.enabled ? 'enabled' : 'disabled';
    const actionLabel = getActionLabel(rule.action);
    const position = `Rule ${index + 1} of ${totalRules}`;
    const target = gatewayName ? ` via ${gatewayName}` : '';
    const selectedStatus = isSelected ? ', selected' : '';
    return `${position}: ${rule.sourceValue} → ${actionLabel}${target}, ${status}${selectedStatus}`;
  });

  // Generate aria-describedby content
  const keyboardHelpId = $derived(`rule-help-${rule.id}`);

  // Source icons
  function getSourceIcon(source: string): string {
    switch (source) {
      case 'domain': return '🌐';
      case 'app': return '📱';
      case 'ip': return '🔢';
      default: return '❓';
    }
  }

  // Action icons
  function getActionIcon(action: string): string {
    switch (action) {
      case 'direct': return '🌍';
      case 'proxy': return '🔒';
      case 'block': return '🚫';
      case 'dpi-bypass': return '🛡️';
      default: return '❓';
    }
  }

  // Action labels
  function getActionLabel(action: string): string {
    switch (action) {
      case 'direct': return 'Direct';
      case 'proxy': return 'Proxy';
      case 'block': return 'Block';
      case 'dpi-bypass': return 'DPI Bypass';
      default: return action;
    }
  }

  // Action styles
  function getActionStyles(action: string): string {
    switch (action) {
      case 'direct': return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
      case 'proxy': return 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20';
      case 'block': return 'bg-red-500/10 text-red-400 border-red-500/20';
      case 'dpi-bypass': return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
      default: return 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20';
    }
  }

  let actionStyles = $derived(getActionStyles(rule.action));
</script>

<div
  role="listitem"
  tabindex="0"
  aria-label={ariaLabel()}
  aria-selected={isSelected}
  aria-disabled={!rule.enabled}
  aria-grabbed={isGrabbed}
  aria-roledescription="reorderable rule"
  aria-describedby={keyboardHelpId}
  draggable="true"
  onkeydown={handleKeyDown}
  ondragstart={(e) => ondragstart(e, index)}
  ondragover={(e) => ondragover(e, index)}
  ondragleave={ondragleave}
  ondrop={(e) => ondrop(e, index)}
  ondragend={ondragend}
  class="group flex items-center gap-3 p-3 bg-zinc-900/50 rounded-xl border transition-all duration-200 outline-none
         {!rule.enabled ? 'opacity-50' : ''}
         {isDragging ? 'opacity-50 scale-[0.98] shadow-2xl shadow-black/50 z-50 border-indigo-500/40' : 'border-white/5 hover:border-white/10'}
         {isDropTarget && !isDragging ? 'border-indigo-500 ring-2 ring-indigo-500/30 bg-indigo-500/5 scale-[1.01]' : ''}
         {isSelected ? 'ring-2 ring-indigo-500/50 border-indigo-500/30 bg-indigo-500/5' : ''}
         {isFocused ? 'ring-2 ring-cyan-500/70 border-cyan-500/50' : ''}
         focus-visible:ring-2 focus-visible:ring-cyan-500/70 focus-visible:border-cyan-500/50"
  style="animation: slideIn 200ms ease-out {index * 30}ms both"
>
  <!-- Hidden keyboard help for screen readers -->
  <span id={keyboardHelpId} class="sr-only">
    Press Enter to edit, Delete to remove, T to toggle, Space to select, 
    Arrow keys to navigate, Ctrl+Arrow to reorder, Home/End to jump to first/last
  </span>
  <!-- Checkbox for multi-select -->
  {#if showCheckbox}
    <button
      type="button"
      onclick={(e) => { e.stopPropagation(); onselect?.(rule.id); }}
      class="flex-shrink-0 w-5 h-5 rounded border-2 flex items-center justify-center transition-all duration-200
             {isSelected 
               ? 'bg-indigo-500 border-indigo-500' 
               : 'border-zinc-600 hover:border-zinc-400 bg-transparent'}"
      aria-label={isSelected ? 'Deselect rule' : 'Select rule'}
    >
      {#if isSelected}
        <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
        </svg>
      {/if}
    </button>
  {/if}

  <!-- Drag handle -->
  <div 
    class="flex-shrink-0 cursor-grab active:cursor-grabbing p-1 rounded text-zinc-400 
           hover:text-zinc-400 hover:bg-zinc-800/60 transition-all duration-200
           {isDragging ? 'cursor-grabbing text-indigo-400' : ''}"
    title="Drag to reorder (or use Ctrl+↑/↓)"
    aria-hidden="true"
  >
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
      <circle cx="9" cy="6" r="1.5" />
      <circle cx="15" cy="6" r="1.5" />
      <circle cx="9" cy="12" r="1.5" />
      <circle cx="15" cy="12" r="1.5" />
      <circle cx="9" cy="18" r="1.5" />
      <circle cx="15" cy="18" r="1.5" />
    </svg>
  </div>
  
  <!-- Source -->
  <div class="flex items-center gap-2 min-w-[140px]">
    <span class="text-lg">{getSourceIcon(rule.source)}</span>
    <span class="text-sm text-white font-medium truncate" title={rule.sourceValue}>
      {rule.sourceValue}
    </span>
  </div>
  
  <!-- Arrow -->
  <svg class="w-5 h-5 text-zinc-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
  </svg>
  
  <!-- Action -->
  <div class="flex-1 flex items-center gap-2 min-w-0">
    <span class="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg border {actionStyles}">
      <span>{getActionIcon(rule.action)}</span>
      <span>{getActionLabel(rule.action)}</span>
    </span>
    {#if rule.proxyId && gatewayName}
      <span class="text-sm text-zinc-400 truncate" title={gatewayName}>
        → {gatewayName}
      </span>
    {/if}
  </div>
  
  <!-- Actions (visible on hover) -->
  <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
    <button
      onclick={() => onedit(rule)}
      class="p-1.5 rounded-lg bg-zinc-800/60 border border-white/5 
             hover:bg-zinc-700/60 hover:border-white/10 transition-colors"
      title="Edit rule"
    >
      <svg class="w-3.5 h-3.5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
      </svg>
    </button>
    <button
      onclick={() => ondelete(rule.id)}
      class="p-1.5 rounded-lg bg-red-500/10 border border-red-500/20 
             hover:bg-red-500/20 hover:border-red-500/30 transition-colors"
      title="Delete rule"
    >
      <svg class="w-3.5 h-3.5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
    </button>
  </div>
  
  <!-- Toggle -->
  <button
    onclick={() => ontoggle(rule.id)}
    aria-label="Toggle rule"
    role="switch"
    aria-checked={rule.enabled}
    class="flex-shrink-0 relative w-10 h-5 rounded-full transition-colors duration-200
           {rule.enabled ? 'bg-emerald-500/20' : 'bg-zinc-800'}"
  >
    <div class="absolute top-0.5 left-0.5 w-4 h-4 rounded-full transition-all duration-200
                {rule.enabled ? 'translate-x-5 bg-emerald-400' : 'bg-zinc-500'}">
    </div>
  </button>
</div>

<style>
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

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
