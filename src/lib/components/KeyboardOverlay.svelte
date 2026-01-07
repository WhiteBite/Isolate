<script lang="ts">
  /**
   * KeyboardOverlay - Shows keyboard shortcuts when Ctrl is held for 500ms
   * Displays all available hotkeys grouped by category with beautiful styling
   */
  import { browser } from '$app/environment';
  import { hotkeysStore, formatHotkey, HOTKEY_ACTIONS, type HotkeysState } from '$lib/stores/hotkeys';

  // Overlay visibility state
  let visible = $state(false);
  let ctrlHoldTimer: ReturnType<typeof setTimeout> | null = null;
  
  // Track hotkeys configuration
  let currentHotkeys = $state<HotkeysState>(hotkeysStore.get());
  
  // Subscribe to hotkeys store
  $effect(() => {
    const unsubscribe = hotkeysStore.subscribe(state => { currentHotkeys = state; });
    return unsubscribe;
  });

  // Shortcut groups for display
  interface ShortcutItem {
    keys: string[];
    description: string;
  }

  interface ShortcutGroup {
    title: string;
    icon: string;
    shortcuts: ShortcutItem[];
  }

  // Build shortcut groups from hotkeys store and static shortcuts
  let shortcutGroups = $derived<ShortcutGroup[]>([
    {
      title: 'Protection',
      icon: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z',
      shortcuts: [
        { 
          keys: formatHotkey(currentHotkeys.toggleStrategy).split('+'), 
          description: HOTKEY_ACTIONS.find(a => a.id === 'toggleStrategy')?.description || 'Toggle Protection' 
        },
        { 
          keys: formatHotkey(currentHotkeys.quickTest).split('+'), 
          description: HOTKEY_ACTIONS.find(a => a.id === 'quickTest')?.description || 'Quick Test' 
        },
        { 
          keys: formatHotkey(currentHotkeys.stopAll).split('+'), 
          description: HOTKEY_ACTIONS.find(a => a.id === 'stopAll')?.description || 'Stop All' 
        },
        { keys: ['Ctrl', 'R'], description: 'Refresh services' },
      ]
    },
    {
      title: 'Navigation',
      icon: 'M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7',
      shortcuts: [
        { keys: ['Ctrl', '1'], description: 'Dashboard' },
        { keys: ['Ctrl', '2'], description: 'Services' },
        { keys: ['Ctrl', '3'], description: 'Network' },
        { 
          keys: formatHotkey(currentHotkeys.openSettings).split('+'), 
          description: 'Settings' 
        },
        { keys: ['Ctrl', 'M'], description: 'Marketplace' },
      ]
    },
    {
      title: 'Actions',
      icon: 'M13 10V3L4 14h7v7l9-11h-7z',
      shortcuts: [
        { keys: ['Ctrl', 'K'], description: 'Command Palette' },
        { keys: ['Ctrl', '`'], description: 'Toggle Terminal' },
        { keys: ['Ctrl', 'L'], description: 'Focus Logs' },
      ]
    },
    {
      title: 'Help',
      icon: 'M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
      shortcuts: [
        { keys: ['?'], description: 'Show all shortcuts' },
        { keys: ['Esc'], description: 'Close / Cancel' },
      ]
    }
  ]);

  // Handle keydown - start timer when Ctrl is pressed
  function handleKeyDown(e: KeyboardEvent) {
    // Only trigger on Ctrl key press (not when already holding)
    if (e.key === 'Control' && !e.repeat && !visible) {
      ctrlHoldTimer = setTimeout(() => {
        visible = true;
      }, 500);
    }
    
    // Hide if any other key is pressed while Ctrl is held
    if (visible && e.key !== 'Control') {
      visible = false;
    }
  }

  // Handle keyup - hide overlay and clear timer
  function handleKeyUp(e: KeyboardEvent) {
    if (e.key === 'Control') {
      if (ctrlHoldTimer) {
        clearTimeout(ctrlHoldTimer);
        ctrlHoldTimer = null;
      }
      visible = false;
    }
  }

  // Handle window blur - hide overlay when window loses focus
  function handleBlur() {
    if (ctrlHoldTimer) {
      clearTimeout(ctrlHoldTimer);
      ctrlHoldTimer = null;
    }
    visible = false;
  }

  // Setup event listeners
  $effect(() => {
    if (!browser) return;

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    window.addEventListener('blur', handleBlur);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
      window.removeEventListener('blur', handleBlur);
      if (ctrlHoldTimer) {
        clearTimeout(ctrlHoldTimer);
      }
    };
  });
</script>

{#if visible}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
    aria-label="Keyboard shortcuts overlay"
  >
    <!-- Overlay Container -->
    <div 
      class="relative w-full max-w-3xl mx-4 p-6 rounded-2xl bg-zinc-900/95 border border-white/10 shadow-2xl animate-in zoom-in-95 duration-150"
      style="box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.05), 0 0 100px -20px rgba(99, 102, 241, 0.2);"
    >
      <!-- Header -->
      <div class="flex items-center justify-center gap-3 mb-6">
        <div class="flex items-center gap-2 px-4 py-2 rounded-full bg-indigo-500/10 border border-indigo-500/20">
          <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
          </svg>
          <span class="text-sm font-medium text-indigo-300">Keyboard Shortcuts</span>
        </div>
      </div>

      <!-- Shortcuts Grid -->
      <div class="grid grid-cols-2 gap-6">
        {#each shortcutGroups as group}
          <div class="space-y-3">
            <!-- Group Header -->
            <div class="flex items-center gap-2">
              <svg class="w-4 h-4 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={group.icon} />
              </svg>
              <h3 class="text-xs font-semibold text-zinc-400 uppercase tracking-wider">{group.title}</h3>
            </div>
            
            <!-- Shortcuts List -->
            <div class="space-y-1.5">
              {#each group.shortcuts as shortcut}
                <div class="flex items-center justify-between py-1.5 px-2.5 rounded-lg bg-zinc-800/50 border border-white/5">
                  <span class="text-sm text-zinc-300">{shortcut.description}</span>
                  <div class="flex items-center gap-1">
                    {#each shortcut.keys as key, i}
                      {#if i > 0}
                        <span class="text-zinc-600 text-[10px]">+</span>
                      {/if}
                      <kbd class="min-w-[24px] px-1.5 py-0.5 text-[11px] font-medium text-zinc-300 bg-zinc-700/80 rounded border border-zinc-600/50 text-center shadow-sm">
                        {key}
                      </kbd>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <!-- Footer Hint -->
      <div class="mt-6 pt-4 border-t border-white/5 text-center">
        <span class="text-xs text-zinc-500">
          Release <kbd class="px-1.5 py-0.5 mx-1 text-[10px] bg-zinc-800 rounded border border-zinc-700/50 text-zinc-400">Ctrl</kbd> to close
        </span>
      </div>

      <!-- Decorative glow -->
      <div class="absolute -inset-px rounded-2xl bg-gradient-to-b from-indigo-500/10 via-transparent to-transparent pointer-events-none"></div>
    </div>
  </div>
{/if}

<style>
  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  
  @keyframes zoom-in-95 {
    from { 
      opacity: 0;
      transform: scale(0.95);
    }
    to { 
      opacity: 1;
      transform: scale(1);
    }
  }
  
  .animate-in {
    animation-fill-mode: both;
  }
  
  .fade-in {
    animation-name: fade-in;
  }
  
  .zoom-in-95 {
    animation-name: zoom-in-95;
  }
  
  .duration-150 {
    animation-duration: 150ms;
  }
</style>
