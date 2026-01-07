<script lang="ts">
  import type { Snippet } from 'svelte';
  import { getContext } from 'svelte';
  import { browser } from '$app/environment';
  import { BENTO_GRID_CONTEXT_KEY, type BentoGridContext } from './bento-grid-context';

  interface Props {
    colspan?: 1 | 2 | 3 | 4;
    rowspan?: 1 | 2;
    title?: string;
    icon?: string;
    children?: Snippet;
    /** Unique widget ID for drag-n-drop ordering */
    widgetId?: string;
    /** Widget index in the grid */
    index?: number;
    /** Disable entrance animation */
    noAnimation?: boolean;
  }

  let { 
    colspan = 1, 
    rowspan = 1,
    title,
    icon,
    children,
    widgetId,
    index = 0,
    noAnimation = false
  }: Props = $props();

  // Staggered entrance animation
  // svelte-ignore state_referenced_locally
  let isVisible = $state(!browser || noAnimation);
  
  $effect(() => {
    if (browser && !noAnimation) {
      // Staggered delay based on index (100ms per widget)
      const delay = index * 100;
      const timer = setTimeout(() => {
        isVisible = true;
      }, delay);
      
      return () => clearTimeout(timer);
    }
  });

  const colspanClasses = {
    1: 'col-span-1',
    2: 'col-span-2',
    3: 'col-span-3',
    4: 'col-span-4'
  };

  const rowspanClasses = {
    1: 'row-span-1',
    2: 'row-span-2'
  };

  // Get grid context for drag state (with HMR safety)
  let gridContext: BentoGridContext | undefined;
  try {
    gridContext = getContext<BentoGridContext | undefined>(BENTO_GRID_CONTEXT_KEY);
  } catch {
    // HMR may cause lifecycle_outside_component error
  }
  const isDraggableGrid = gridContext?.draggable ?? false;

  // Local drag state
  let isDraggingThis = $state(false);

  // Drag handlers using HTML5 Drag and Drop API
  function handleDragStart(e: DragEvent) {
    if (!widgetId || !isDraggableGrid) return;
    
    isDraggingThis = true;
    gridContext?.setDragging(true);
    gridContext?.setDraggedId(widgetId);
    
    // Set drag data
    e.dataTransfer?.setData('text/plain', widgetId);
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDragEnd() {
    isDraggingThis = false;
    gridContext?.setDragging(false);
    gridContext?.setDraggedId(null);
    gridContext?.setDragOverId(null);
  }

  function handleDragOver(e: DragEvent) {
    if (!widgetId || !isDraggableGrid) return;
    
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
    
    // Only set drag over if not dragging this widget
    if (gridContext?.draggedId !== widgetId) {
      gridContext?.setDragOverId(widgetId);
    }
  }

  function handleDragLeave() {
    if (gridContext?.dragOverId === widgetId) {
      gridContext?.setDragOverId(null);
    }
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    if (!widgetId || !isDraggableGrid) return;
    
    gridContext?.handleDrop(widgetId);
    gridContext?.setDragOverId(null);
  }

  // Keyboard navigation handlers
  function handleKeyDown(e: KeyboardEvent) {
    if (!widgetId || !isDraggableGrid) return;
    
    const isMoving = gridContext?.movingId === widgetId;
    
    switch (e.key) {
      case 'Enter':
      case ' ':
        e.preventDefault();
        gridContext?.handleKeyboardSelect(widgetId);
        break;
        
      case 'Escape':
        if (gridContext?.movingId !== null) {
          e.preventDefault();
          gridContext?.cancelKeyboardMove();
        }
        break;
        
      case 'ArrowUp':
        if (isMoving) {
          e.preventDefault();
          gridContext?.handleKeyboardMove(widgetId, 'up');
        }
        break;
        
      case 'ArrowDown':
        if (isMoving) {
          e.preventDefault();
          gridContext?.handleKeyboardMove(widgetId, 'down');
        }
        break;
        
      case 'ArrowLeft':
        if (isMoving) {
          e.preventDefault();
          gridContext?.handleKeyboardMove(widgetId, 'left');
        }
        break;
        
      case 'ArrowRight':
        if (isMoving) {
          e.preventDefault();
          gridContext?.handleKeyboardMove(widgetId, 'right');
        }
        break;
    }
  }

  // Computed states
  let isDropTarget = $derived(gridContext?.dragOverId === widgetId && !isDraggingThis);
  let isMovingThis = $derived(gridContext?.movingId === widgetId);
  let isSwapTarget = $derived(gridContext?.movingId !== null && gridContext?.movingId !== widgetId);
</script>

<div 
  class="
    bento-widget
    {colspanClasses[colspan]} 
    {rowspanClasses[rowspan]}
    bg-zinc-900/40
    backdrop-blur-md
    border border-white/5 border-t-white/10
    rounded-xl 
    p-5
    transition-all duration-200
    hover:border-white/10
    {!isDraggingThis ? 'hover:-translate-y-0.5' : ''}
    {isVisible ? 'widget-visible' : 'widget-hidden'}
  "
  class:is-dragging={isDraggingThis}
  class:is-drop-target={isDropTarget}
  class:is-moving={isMovingThis}
  class:is-swap-target={isSwapTarget}
  draggable={isDraggableGrid}
  ondragstart={handleDragStart}
  ondragend={handleDragEnd}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  onkeydown={handleKeyDown}
  role={isDraggableGrid ? 'option' : 'article'}
  aria-label={title || 'Dashboard widget'}
  aria-grabbed={isDraggableGrid ? isDraggingThis : undefined}
  aria-selected={isDraggableGrid ? isMovingThis : undefined}
  tabindex={isDraggableGrid ? 0 : undefined}
  data-widget-id={widgetId}
  data-widget-index={index}
  style="--stagger-delay: {index * 100}ms"
>
  {#if title || icon || isDraggableGrid}
    <div class="flex items-center gap-2.5 mb-4 pb-3 border-b border-white/5">
      <!-- Drag Handle Indicator -->
      {#if isDraggableGrid}
        <button 
          type="button"
          class="drag-handle flex items-center justify-center w-5 h-5 -ml-1 rounded hover:bg-white/5 cursor-grab active:cursor-grabbing transition-colors {isMovingThis ? 'bg-purple-500/20 text-purple-400' : ''}"
          aria-label={isMovingThis ? 'Moving widget. Use arrow keys to reorder, Enter to swap with another, Escape to cancel.' : 'Press Enter or Space to select for keyboard reordering'}
          aria-roledescription="draggable"
          tabindex="-1"
        >
          {#if isMovingThis}
            <!-- Moving indicator icon -->
            <svg class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <path d="M8 2l3 3H9v3h3V6l3 3-3 3v-2H9v3h2l-3 3-3-3h2V10H4v2l-3-3 3-3v2h3V5H5l3-3z"/>
            </svg>
          {:else}
            <!-- Default drag handle icon -->
            <svg class="w-3.5 h-3.5 text-zinc-400" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <circle cx="4" cy="4" r="1.5"/>
              <circle cx="4" cy="8" r="1.5"/>
              <circle cx="4" cy="12" r="1.5"/>
              <circle cx="10" cy="4" r="1.5"/>
              <circle cx="10" cy="8" r="1.5"/>
              <circle cx="10" cy="12" r="1.5"/>
            </svg>
          {/if}
        </button>
      {/if}
      
      {#if icon}
        <span class="text-base opacity-60">{icon}</span>
      {/if}
      {#if title}
        <h3 class="text-[10px] text-zinc-400 uppercase tracking-widest font-semibold">{title}</h3>
      {/if}
    </div>
  {/if}
  
  <div class="h-full">
    {#if children}
      {@render children()}
    {/if}
  </div>
</div>

<style>
  .bento-widget {
    position: relative;
    z-index: 1;
  }
  
  /* Entrance animation */
  .widget-hidden {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
  
  .widget-visible {
    opacity: 1;
    transform: translateY(0) scale(1);
    animation: widget-enter 0.5s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
  
  @keyframes widget-enter {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
  
  .bento-widget.is-dragging {
    opacity: 0.6;
    transform: scale(0.98);
    z-index: 100;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
    border-color: rgba(255, 255, 255, 0.15) !important;
  }
  
  .bento-widget.is-drop-target {
    border-color: rgba(34, 211, 238, 0.5) !important;
    background: rgba(34, 211, 238, 0.08);
    transform: scale(1.02);
  }
  
  /* Keyboard navigation: widget selected for moving */
  .bento-widget.is-moving {
    border-color: rgba(168, 85, 247, 0.6) !important;
    box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.3), 0 8px 24px rgba(168, 85, 247, 0.2);
    transform: scale(1.02);
    z-index: 50;
  }
  
  /* Keyboard navigation: potential swap target */
  .bento-widget.is-swap-target:focus {
    border-color: rgba(34, 211, 238, 0.5) !important;
    box-shadow: 0 0 0 2px rgba(34, 211, 238, 0.2);
  }
  
  /* Focus styles for keyboard navigation */
  .bento-widget:focus {
    outline: none;
    border-color: rgba(255, 255, 255, 0.2) !important;
  }
  
  .bento-widget:focus-visible {
    outline: 2px solid rgba(168, 85, 247, 0.5);
    outline-offset: 2px;
  }
  
  .drag-handle {
    touch-action: none;
  }
  
  .drag-handle:hover svg {
    color: rgba(255, 255, 255, 0.7);
  }
</style>
