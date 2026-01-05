<script lang="ts">
  import { setContext } from 'svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    direction?: 'horizontal' | 'vertical';
    id: string;
    children?: Snippet;
  }

  let { 
    direction = 'horizontal',
    id,
    children 
  }: Props = $props();

  // Panel state management
  let panels = $state<Map<string, { size: number; minSize: number; maxSize: number; collapsible: boolean; collapsed: boolean }>>(new Map());
  let panelOrder = $state<string[]>([]);
  let containerRef = $state<HTMLElement | null>(null);
  let isDragging = $state(false);
  let activeHandleIndex = $state<number | null>(null);

  // Load saved sizes from localStorage
  function loadSavedSizes(): Record<string, number> {
    if (typeof window === 'undefined') return {};
    try {
      const saved = localStorage.getItem(`resizable-panel-${id}`);
      return saved ? JSON.parse(saved) : {};
    } catch {
      return {};
    }
  }

  // Save sizes to localStorage
  function saveSizes() {
    if (typeof window === 'undefined') return;
    const sizes: Record<string, number> = {};
    panels.forEach((panel, panelId) => {
      sizes[panelId] = panel.size;
    });
    localStorage.setItem(`resizable-panel-${id}`, JSON.stringify(sizes));
  }

  // Register a panel
  function registerPanel(panelId: string, defaultSize: number, minSize: number, maxSize: number, collapsible: boolean) {
    const savedSizes = loadSavedSizes();
    const size = savedSizes[panelId] ?? defaultSize;
    
    panels.set(panelId, { 
      size, 
      minSize, 
      maxSize, 
      collapsible,
      collapsed: collapsible && size <= minSize
    });
    panels = new Map(panels);
    
    if (!panelOrder.includes(panelId)) {
      panelOrder = [...panelOrder, panelId];
    }
  }

  // Unregister a panel
  function unregisterPanel(panelId: string) {
    panels.delete(panelId);
    panels = new Map(panels);
    panelOrder = panelOrder.filter(p => p !== panelId);
  }

  // Get panel size
  function getPanelSize(panelId: string): number {
    return panels.get(panelId)?.size ?? 0;
  }

  // Check if panel is collapsed
  function isPanelCollapsed(panelId: string): boolean {
    return panels.get(panelId)?.collapsed ?? false;
  }

  // Start resize
  function startResize(handleIndex: number, event: MouseEvent | TouchEvent) {
    isDragging = true;
    activeHandleIndex = handleIndex;
    
    const clientPos = 'touches' in event ? event.touches[0] : event;
    const startPos = direction === 'horizontal' ? clientPos.clientX : clientPos.clientY;
    
    const leftPanelId = panelOrder[handleIndex];
    const rightPanelId = panelOrder[handleIndex + 1];
    
    if (!leftPanelId || !rightPanelId) return;
    
    const leftPanelData = panels.get(leftPanelId);
    const rightPanelData = panels.get(rightPanelId);
    
    if (!leftPanelData || !rightPanelData || !containerRef) return;
    
    const containerSize = direction === 'horizontal' 
      ? containerRef.offsetWidth 
      : containerRef.offsetHeight;
    
    const startLeftSize = leftPanelData.size;
    const startRightSize = rightPanelData.size;
    
    // Capture panel config at start
    const leftConfig = { ...leftPanelData };
    const rightConfig = { ...rightPanelData };

    function onMove(e: MouseEvent | TouchEvent) {
      const pos = 'touches' in e ? e.touches[0] : e;
      const currentPos = direction === 'horizontal' ? pos.clientX : pos.clientY;
      const delta = ((currentPos - startPos) / containerSize) * 100;
      
      let newLeftSize = startLeftSize + delta;
      let newRightSize = startRightSize - delta;
      
      // Get current panel refs
      const leftPanel = panels.get(leftPanelId);
      const rightPanel = panels.get(rightPanelId);
      if (!leftPanel || !rightPanel) return;
      
      // Apply constraints
      const leftMin = leftConfig.collapsible ? 0 : leftConfig.minSize;
      const rightMin = rightConfig.collapsible ? 0 : rightConfig.minSize;
      
      // Check collapse threshold (collapse if dragged below minSize)
      if (leftConfig.collapsible && newLeftSize < leftConfig.minSize && newLeftSize < startLeftSize) {
        newLeftSize = 0;
        newRightSize = startLeftSize + startRightSize;
        leftPanel.collapsed = true;
      } else if (rightConfig.collapsible && newRightSize < rightConfig.minSize && newRightSize < startRightSize) {
        newRightSize = 0;
        newLeftSize = startLeftSize + startRightSize;
        rightPanel.collapsed = true;
      } else {
        // Normal resize with constraints
        newLeftSize = Math.max(leftMin, Math.min(leftConfig.maxSize, newLeftSize));
        newRightSize = Math.max(rightMin, Math.min(rightConfig.maxSize, newRightSize));
        
        // Ensure total doesn't exceed available space
        const total = startLeftSize + startRightSize;
        if (newLeftSize + newRightSize > total) {
          if (delta > 0) {
            newRightSize = total - newLeftSize;
          } else {
            newLeftSize = total - newRightSize;
          }
        }
        
        leftPanel.collapsed = false;
        rightPanel.collapsed = false;
      }
      
      leftPanel.size = newLeftSize;
      rightPanel.size = newRightSize;
      panels = new Map(panels);
    }

    function onEnd() {
      isDragging = false;
      activeHandleIndex = null;
      saveSizes();
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onEnd);
      document.removeEventListener('touchmove', onMove);
      document.removeEventListener('touchend', onEnd);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    }

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onEnd);
    document.addEventListener('touchmove', onMove);
    document.addEventListener('touchend', onEnd);
    document.body.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize';
    document.body.style.userSelect = 'none';
  }

  // Keyboard resize
  function handleKeyboardResize(handleIndex: number, event: KeyboardEvent) {
    if (!event.ctrlKey) return;
    
    const step = 2; // 2% per keypress
    const leftPanelId = panelOrder[handleIndex];
    const rightPanelId = panelOrder[handleIndex + 1];
    
    if (!leftPanelId || !rightPanelId) return;
    
    const leftPanel = panels.get(leftPanelId);
    const rightPanel = panels.get(rightPanelId);
    
    if (!leftPanel || !rightPanel) return;
    
    let delta = 0;
    
    if (direction === 'horizontal') {
      if (event.key === 'ArrowLeft') delta = -step;
      else if (event.key === 'ArrowRight') delta = step;
    } else {
      if (event.key === 'ArrowUp') delta = -step;
      else if (event.key === 'ArrowDown') delta = step;
    }
    
    if (delta === 0) return;
    
    event.preventDefault();
    
    let newLeftSize = leftPanel.size + delta;
    let newRightSize = rightPanel.size - delta;
    
    // Apply constraints
    const leftMin = leftPanel.collapsible ? 0 : leftPanel.minSize;
    const rightMin = rightPanel.collapsible ? 0 : rightPanel.minSize;
    
    newLeftSize = Math.max(leftMin, Math.min(leftPanel.maxSize, newLeftSize));
    newRightSize = Math.max(rightMin, Math.min(rightPanel.maxSize, newRightSize));
    
    leftPanel.size = newLeftSize;
    rightPanel.size = newRightSize;
    panels = new Map(panels);
    saveSizes();
  }

  // Expand collapsed panel
  function expandPanel(panelId: string) {
    const panel = panels.get(panelId);
    if (!panel || !panel.collapsed) return;
    
    panel.collapsed = false;
    panel.size = panel.minSize + 5; // Expand to slightly above minimum
    panels = new Map(panels);
    saveSizes();
  }

  // Provide context to children
  setContext('resizable-panel-group', {
    direction: () => direction,
    registerPanel,
    unregisterPanel,
    getPanelSize,
    isPanelCollapsed,
    startResize,
    handleKeyboardResize,
    expandPanel,
    getHandleIndex: (panelId: string) => panelOrder.indexOf(panelId),
    isDragging: () => isDragging,
    activeHandleIndex: () => activeHandleIndex,
    panelOrder: () => panelOrder
  });
</script>

<div 
  bind:this={containerRef}
  class="flex h-full w-full {direction === 'horizontal' ? 'flex-row' : 'flex-col'}"
  class:select-none={isDragging}
>
  {#if children}
    {@render children()}
  {/if}
</div>
