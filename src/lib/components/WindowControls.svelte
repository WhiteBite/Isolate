<script lang="ts">
  import { browser } from '$app/environment';
  
  let isMaximized = $state(false);
  
  // Track window maximize state
  $effect(() => {
    if (!browser) return;
    
    let cleanup: (() => void) | undefined;
    
    (async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        
        // Check initial state
        isMaximized = await appWindow.isMaximized();
        
        // Listen for resize events to track maximize state
        const unlisten = await appWindow.onResized(async () => {
          isMaximized = await appWindow.isMaximized();
        });
        
        cleanup = unlisten;
      } catch (e) {
        console.error('Failed to setup window controls:', e);
      }
    })();
    
    return () => {
      cleanup?.();
    };
  });
  
  async function minimize() {
    if (!browser) return;
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().minimize();
    } catch (e) {
      console.error('Failed to minimize:', e);
    }
  }
  
  async function toggleMaximize() {
    if (!browser) return;
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().toggleMaximize();
    } catch (e) {
      console.error('Failed to toggle maximize:', e);
    }
  }
  
  async function close() {
    if (!browser) return;
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().close();
    } catch (e) {
      console.error('Failed to close:', e);
    }
  }
</script>

<div class="flex items-center gap-1.5">
  <!-- Minimize Button -->
  <button
    onclick={minimize}
    class="group w-[46px] h-8 flex items-center justify-center
           hover:bg-zinc-700/50 transition-colors duration-150"
    title="Minimize"
  >
    <svg 
      class="w-4 h-4 text-zinc-400 group-hover:text-zinc-200 transition-colors" 
      fill="none" 
      viewBox="0 0 24 24" 
      stroke="currentColor"
      stroke-width="2"
    >
      <path stroke-linecap="round" stroke-linejoin="round" d="M20 12H4" />
    </svg>
  </button>
  
  <!-- Maximize/Restore Button -->
  <button
    onclick={toggleMaximize}
    class="group w-[46px] h-8 flex items-center justify-center
           hover:bg-zinc-700/50 transition-colors duration-150"
    title={isMaximized ? 'Restore' : 'Maximize'}
  >
    {#if isMaximized}
      <!-- Restore icon (two overlapping squares) -->
      <svg 
        class="w-3.5 h-3.5 text-zinc-400 group-hover:text-zinc-200 transition-colors" 
        fill="none" 
        viewBox="0 0 24 24" 
        stroke="currentColor"
        stroke-width="2"
      >
        <path stroke-linecap="round" stroke-linejoin="round" d="M8 4h12v12M4 8h12v12H4z" />
      </svg>
    {:else}
      <!-- Maximize icon (single square) -->
      <svg 
        class="w-3.5 h-3.5 text-zinc-400 group-hover:text-zinc-200 transition-colors" 
        fill="none" 
        viewBox="0 0 24 24" 
        stroke="currentColor"
        stroke-width="2"
      >
        <rect x="4" y="4" width="16" height="16" rx="1" />
      </svg>
    {/if}
  </button>
  
  <!-- Close Button -->
  <button
    onclick={close}
    class="group w-[46px] h-8 flex items-center justify-center
           hover:bg-red-500 transition-colors duration-150"
    title="Close"
  >
    <svg 
      class="w-4 h-4 text-zinc-400 group-hover:text-white transition-colors" 
      fill="none" 
      viewBox="0 0 24 24" 
      stroke="currentColor"
      stroke-width="2"
    >
      <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
    </svg>
  </button>
</div>
