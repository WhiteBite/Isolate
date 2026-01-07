<script lang="ts">
  import type { Snippet } from 'svelte';
  import { setContext, hasContext } from 'svelte';
  import { BENTO_GRID_CONTEXT_KEY, type BentoGridContext } from './bento-grid-context';
  import { logger } from '$lib/utils/logger';

  interface Props {
    columns?: 1 | 2 | 3 | 4;
    gap?: 2 | 3 | 4 | 6 | 8;
    children?: Snippet;
    /** Enable drag-n-drop reordering */
    draggable?: boolean;
    /** Current widget order (array of widget IDs) */
    order?: string[];
    /** Callback when widget order changes */
    onReorder?: (order: string[]) => void;
  }

  let { 
    columns = 4, 
    gap = 4,
    children,
    draggable = false,
    order = [],
    onReorder
  }: Props = $props();

  const columnClasses: Record<number, string> = {
    1: 'grid-cols-1',
    2: 'grid-cols-2',
    3: 'grid-cols-3',
    4: 'grid-cols-4'
  };

  const gapClasses: Record<number, string> = {
    2: 'gap-2',
    3: 'gap-3',
    4: 'gap-4',
    6: 'gap-6',
    8: 'gap-8'
  };

  // Reactive drag state
  let isDragging = $state(false);
  let draggedId = $state<string | null>(null);
  let dragOverId = $state<string | null>(null);
  
  // Keyboard navigation state
  let movingId = $state<string | null>(null);

  // Handle drop - reorder widgets
  function handleDrop(targetId: string) {
    if (!draggedId || draggedId === targetId || !onReorder) return;
    
    const currentOrder = [...order];
    const draggedIndex = currentOrder.indexOf(draggedId);
    const targetIndex = currentOrder.indexOf(targetId);
    
    if (draggedIndex === -1 || targetIndex === -1) return;
    
    // Remove dragged item and insert at target position
    currentOrder.splice(draggedIndex, 1);
    currentOrder.splice(targetIndex, 0, draggedId);
    
    onReorder(currentOrder);
  }

  // Keyboard navigation: select widget for moving
  function handleKeyboardSelect(widgetId: string) {
    if (movingId === null) {
      // Start moving this widget
      movingId = widgetId;
    } else if (movingId === widgetId) {
      // Deselect if clicking same widget
      movingId = null;
    } else {
      // Swap widgets
      const currentOrder = [...order];
      const movingIndex = currentOrder.indexOf(movingId);
      const targetIndex = currentOrder.indexOf(widgetId);
      
      if (movingIndex !== -1 && targetIndex !== -1 && onReorder) {
        [currentOrder[movingIndex], currentOrder[targetIndex]] = [currentOrder[targetIndex], currentOrder[movingIndex]];
        onReorder(currentOrder);
      }
      movingId = null;
    }
  }

  // Keyboard navigation: move widget in direction
  function handleKeyboardMove(widgetId: string, direction: 'up' | 'down' | 'left' | 'right') {
    if (movingId !== widgetId || !onReorder) return;
    
    const currentOrder = [...order];
    const currentIndex = currentOrder.indexOf(widgetId);
    if (currentIndex === -1) return;
    
    let targetIndex: number;
    
    // Calculate target index based on direction and grid columns
    switch (direction) {
      case 'left':
      case 'up':
        targetIndex = currentIndex - 1;
        break;
      case 'right':
      case 'down':
        targetIndex = currentIndex + 1;
        break;
    }
    
    // Bounds check
    if (targetIndex < 0 || targetIndex >= currentOrder.length) return;
    
    // Swap with adjacent widget
    [currentOrder[currentIndex], currentOrder[targetIndex]] = [currentOrder[targetIndex], currentOrder[currentIndex]];
    onReorder(currentOrder);
  }

  // Cancel keyboard move mode
  function cancelKeyboardMove() {
    movingId = null;
  }

  // Context object with reactive getters (using function wrapper for reactivity)
  const gridContext: BentoGridContext = {
    get draggable() { return draggable; },
    get isDragging() { return isDragging; },
    setDragging: (value: boolean) => { isDragging = value; },
    get draggedId() { return draggedId; },
    setDraggedId: (id: string | null) => { draggedId = id; },
    get dragOverId() { return dragOverId; },
    setDragOverId: (id: string | null) => { dragOverId = id; },
    handleDrop,
    // Keyboard navigation
    get movingId() { return movingId; },
    setMovingId: (id: string | null) => { movingId = id; },
    handleKeyboardMove,
    handleKeyboardSelect,
    cancelKeyboardMove,
    get order() { return order; }
  };
  
  // Set context with HMR safety - wrap in try-catch for HMR edge cases
  try {
    setContext(BENTO_GRID_CONTEXT_KEY, gridContext);
  } catch (e) {
    // HMR may cause lifecycle_outside_component error, ignore it
    logger.debug('BentoGrid', 'Context setup skipped (HMR)', e);
  }
</script>

<div 
  class="grid {columnClasses[columns]} {gapClasses[gap]} auto-rows-[minmax(140px,auto)] {draggable ? 'bento-grid-draggable' : ''}"
  class:is-dragging={isDragging}
  class:is-keyboard-moving={movingId !== null}
  role={draggable ? 'listbox' : undefined}
  aria-label={draggable ? 'Reorderable widget grid. Press Enter or Space to select a widget, then use arrow keys to move it.' : undefined}
  aria-multiselectable="false"
>
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .bento-grid-draggable {
    position: relative;
  }
  
  .is-dragging {
    user-select: none;
  }
</style>
