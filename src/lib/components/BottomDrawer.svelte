<script lang="ts">
  import type { Snippet } from 'svelte';
  import { bottomDrawerStore } from '$lib/stores/bottomDrawer.svelte';
  import { browser } from '$app/environment';

  interface Props {
    title?: string;
    children: Snippet;
  }

  let { title = 'Logs', children }: Props = $props();

  let drawerRef: HTMLDivElement | undefined = $state();
  let startY = $state(0);
  let startHeight = $state(0);

  // Handle drag start
  function handleDragStart(e: MouseEvent | TouchEvent) {
    e.preventDefault();
    bottomDrawerStore.startDrag();
    
    const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
    startY = clientY;
    startHeight = bottomDrawerStore.height;
    
    if (browser) {
      document.addEventListener('mousemove', handleDragMove);
      document.addEventListener('mouseup', handleDragEnd);
      document.addEventListener('touchmove', handleDragMove);
      document.addEventListener('touchend', handleDragEnd);
    }
  }

  // Handle drag move
  function handleDragMove(e: MouseEvent | TouchEvent) {
    if (!bottomDrawerStore.isDragging) return;
    
    const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
    const deltaY = startY - clientY;
    const windowHeight = window.innerHeight;
    const deltaPercent = (deltaY / windowHeight) * 100;
    
    bottomDrawerStore.setHeight(startHeight + deltaPercent);
  }

  // Handle drag end
  function handleDragEnd() {
    bottomDrawerStore.stopDrag();
    
    if (browser) {
      document.removeEventListener('mousemove', handleDragMove);
      document.removeEventListener('mouseup', handleDragEnd);
      document.removeEventListener('touchmove', handleDragMove);
      document.removeEventListener('touchend', handleDragEnd);
    }
  }

  // Close on Escape
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && bottomDrawerStore.isOpen) {
      bottomDrawerStore.close();
    }
  }

  $effect(() => {
    if (browser) {
      window.addEventListener('keydown', handleKeydown);
      return () => window.removeEventListener('keydown', handleKeydown);
    }
  });
</script>

{#if bottomDrawerStore.isOpen}
  <!-- Backdrop (optional, subtle) -->
  <div 
    class="fixed inset-0 z-40 pointer-events-none"
    aria-hidden="true"
  ></div>

  <!-- Drawer -->
  <div
    bind:this={drawerRef}
    class="fixed bottom-0 left-0 right-0 z-50 flex flex-col bg-zinc-900/95 backdrop-blur-xl border-t border-white/10 shadow-2xl transition-transform duration-300 ease-out"
    style="height: {bottomDrawerStore.height}vh; transform: translateY({bottomDrawerStore.isOpen ? '0' : '100%'})"
    role="dialog"
    aria-modal="true"
    aria-label={title}
  >
    <!-- Drag Handle -->
    <div
      class="flex items-center justify-center h-6 cursor-ns-resize group shrink-0 select-none"
      onmousedown={handleDragStart}
      ontouchstart={handleDragStart}
      role="separator"
      aria-orientation="horizontal"
      aria-label="Resize drawer"
      tabindex="0"
      onkeydown={(e) => {
        if (e.key === 'ArrowUp') {
          e.preventDefault();
          bottomDrawerStore.setHeight(bottomDrawerStore.height + 5);
        } else if (e.key === 'ArrowDown') {
          e.preventDefault();
          bottomDrawerStore.setHeight(bottomDrawerStore.height - 5);
        }
      }}
    >
      <div class="w-12 h-1 rounded-full bg-zinc-600 group-hover:bg-zinc-500 transition-colors"></div>
    </div>

    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-white/5 shrink-0">
      <div class="flex items-center gap-2">
        <svg class="w-4 h-4 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="4 17 10 11 4 5"/>
          <line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
        <h2 class="text-sm font-medium text-zinc-200">{title}</h2>
      </div>
      
      <div class="flex items-center gap-1">
        <!-- Minimize button -->
        <button
          onclick={() => bottomDrawerStore.setHeight(bottomDrawerStore.minHeight)}
          class="p-1.5 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-white/5 transition-colors"
          aria-label="Minimize"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20 12H4"/>
          </svg>
        </button>
        
        <!-- Maximize button -->
        <button
          onclick={() => bottomDrawerStore.setHeight(bottomDrawerStore.maxHeight)}
          class="p-1.5 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-white/5 transition-colors"
          aria-label="Maximize"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="4" y="4" width="16" height="16" rx="2"/>
          </svg>
        </button>
        
        <!-- Close button -->
        <button
          onclick={() => bottomDrawerStore.close()}
          class="p-1.5 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-white/5 transition-colors"
          aria-label="Close"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-auto">
      {@render children()}
    </div>
  </div>
{/if}

<style>
  /* Smooth animation for drawer */
  div[role="dialog"] {
    animation: slideUp 0.3s ease-out;
  }
  
  @keyframes slideUp {
    from {
      transform: translateY(100%);
    }
    to {
      transform: translateY(0);
    }
  }
</style>
