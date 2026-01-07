<script lang="ts">
  import { getContext } from 'svelte';

  interface Props {
    disabled?: boolean;
  }

  let { disabled = false }: Props = $props();

  // Get context from parent group (with HMR safety)
  type HandleGroupContext = {
    direction: () => 'horizontal' | 'vertical';
    startResize: (handleIndex: number, event: MouseEvent | TouchEvent) => void;
    handleKeyboardResize: (handleIndex: number, event: KeyboardEvent) => void;
    getHandleIndex: (panelId: string) => number;
    isDragging: () => boolean;
    activeHandleIndex: () => number | null;
    panelOrder: () => string[];
  };
  
  let ctx: HandleGroupContext | undefined;
  try {
    ctx = getContext<HandleGroupContext>('resizable-panel-group');
  } catch {
    // HMR may cause lifecycle_outside_component error
  }

  let handleRef = $state<HTMLElement | null>(null);
  let handleIndex = $state(0);
  let isHovered = $state(false);
  let isFocused = $state(false);

  // Calculate handle index based on position in DOM
  $effect(() => {
    if (handleRef && ctx) {
      const parent = handleRef.parentElement;
      if (parent) {
        const handles = parent.querySelectorAll('[data-resizable-handle]');
        handleIndex = Array.from(handles).indexOf(handleRef);
      }
    }
  });

  function onMouseDown(event: MouseEvent) {
    if (disabled || !ctx) return;
    event.preventDefault();
    ctx.startResize(handleIndex, event);
  }

  function onTouchStart(event: TouchEvent) {
    if (disabled || !ctx) return;
    ctx.startResize(handleIndex, event);
  }

  function onKeyDown(event: KeyboardEvent) {
    if (disabled || !ctx) return;
    ctx.handleKeyboardResize(handleIndex, event);
  }

  function onDoubleClick() {
    // Double-click to reset to default sizes (could be implemented)
  }

  let direction = $derived(ctx?.direction() ?? 'horizontal');
  const isActive = $derived(ctx?.activeHandleIndex() === handleIndex && ctx?.isDragging());
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  bind:this={handleRef}
  data-resizable-handle
  class="
    relative flex-shrink-0 
    {direction === 'horizontal' ? 'w-1 cursor-col-resize' : 'h-1 cursor-row-resize'}
    {disabled ? 'cursor-not-allowed opacity-50' : ''}
    group
  "
  role="separator"
  aria-orientation={direction === 'horizontal' ? 'vertical' : 'horizontal'}
  aria-valuenow={handleIndex}
  tabindex={disabled ? -1 : 0}
  onmousedown={onMouseDown}
  ontouchstart={onTouchStart}
  onkeydown={onKeyDown}
  ondblclick={onDoubleClick}
  onmouseenter={() => isHovered = true}
  onmouseleave={() => isHovered = false}
  onfocus={() => isFocused = true}
  onblur={() => isFocused = false}
>
  <!-- Hit area (larger than visual) -->
  <div 
    class="
      absolute 
      {direction === 'horizontal' ? 'inset-y-0 -left-1 -right-1 w-3' : 'inset-x-0 -top-1 -bottom-1 h-3'}
    "
  ></div>
  
  <!-- Visual handle -->
  <div 
    class="
      absolute rounded-full transition-colors duration-150
      {direction === 'horizontal' ? 'inset-y-0 left-0 right-0' : 'inset-x-0 top-0 bottom-0'}
      {isActive ? 'bg-[#6366f1]/50' : isHovered || isFocused ? 'bg-[#6366f1]/30' : 'bg-transparent'}
    "
  ></div>
  
  <!-- Center grip indicator (visible on hover/focus) -->
  <div 
    class="
      absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2
      transition-opacity duration-150
      {isHovered || isFocused || isActive ? 'opacity-100' : 'opacity-0'}
    "
  >
    {#if direction === 'horizontal'}
      <div class="flex flex-col gap-0.5">
        <div class="w-0.5 h-0.5 rounded-full bg-[#6366f1]"></div>
        <div class="w-0.5 h-0.5 rounded-full bg-[#6366f1]"></div>
        <div class="w-0.5 h-0.5 rounded-full bg-[#6366f1]"></div>
      </div>
    {:else}
      <div class="flex flex-row gap-0.5">
        <div class="w-0.5 h-0.5 rounded-full bg-[#6366f1]"></div>
        <div class="w-0.5 h-0.5 rounded-full bg-[#6366f1]"></div>
        <div class="w-0.5 h-0.5 rounded-full bg-[#6366f1]"></div>
      </div>
    {/if}
  </div>
</div>
