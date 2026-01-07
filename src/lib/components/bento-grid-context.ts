// BentoGrid context module
// Separated to avoid lifecycle_outside_component errors with Svelte 5 HMR

export interface BentoGridContext {
  draggable: boolean;
  isDragging: boolean;
  draggedId: string | null;
  dragOverId: string | null;
  setDragging: (value: boolean) => void;
  setDraggedId: (id: string | null) => void;
  setDragOverId: (id: string | null) => void;
  handleDrop: (targetId: string) => void;
  // Keyboard navigation
  movingId: string | null;
  setMovingId: (id: string | null) => void;
  handleKeyboardMove: (widgetId: string, direction: 'up' | 'down' | 'left' | 'right') => void;
  handleKeyboardSelect: (widgetId: string) => void;
  cancelKeyboardMove: () => void;
  order: string[];
}

export const BENTO_GRID_CONTEXT_KEY = 'bento-grid';
