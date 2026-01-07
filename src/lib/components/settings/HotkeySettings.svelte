<script lang="ts">
  /**
   * HotkeySettings Component
   * 
   * UI for configuring keyboard shortcuts.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { 
    hotkeysStore, 
    formatHotkey, 
    parseKeyboardEvent,
    HOTKEY_ACTIONS,
    type HotkeyConfig,
    type HotkeysState
  } from '$lib/stores/hotkeys';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let hotkeys = $state<HotkeysState>(hotkeysStore.get());
  let recordingHotkey = $state<keyof HotkeysState | null>(null);
  let hotkeyConflict = $state<string | null>(null);

  // Subscribe to hotkeys store
  $effect(() => {
    const unsubscribe = hotkeysStore.subscribe(state => {
      hotkeys = state;
    });
    return unsubscribe;
  });

  // Hotkey recording functions
  function startRecordingHotkey(action: keyof HotkeysState) {
    recordingHotkey = action;
    hotkeyConflict = null;
  }

  function stopRecordingHotkey() {
    recordingHotkey = null;
    hotkeyConflict = null;
  }

  function handleHotkeyKeydown(e: KeyboardEvent) {
    if (!recordingHotkey) return;
    
    e.preventDefault();
    e.stopPropagation();
    
    // Handle Escape to cancel
    if (e.key === 'Escape') {
      stopRecordingHotkey();
      return;
    }
    
    const config = parseKeyboardEvent(e);
    if (!config) return; // Ignore modifier-only presses
    
    // Require at least one modifier key
    if (!config.ctrlKey && !config.altKey && !config.shiftKey) {
      hotkeyConflict = 'Hotkey must include Ctrl, Alt, or Shift';
      return;
    }
    
    // Check for conflicts
    const conflict = hotkeysStore.hasConflict(recordingHotkey, config);
    if (conflict) {
      const conflictAction = HOTKEY_ACTIONS.find(a => a.id === conflict);
      hotkeyConflict = `Conflicts with "${conflictAction?.label || conflict}"`;
      return;
    }
    
    // Save the new hotkey
    hotkeysStore.setHotkey(recordingHotkey, config);
    stopRecordingHotkey();
  }

  function resetHotkey(action: keyof HotkeysState) {
    hotkeysStore.resetHotkey(action);
  }

  function resetAllHotkeys() {
    hotkeysStore.resetToDefaults();
  }
</script>

<div class={className}>
  <h2 class="text-xl font-semibold text-text-primary mb-6">Keyboard Shortcuts</h2>
  
  <div class="space-y-4">
    <!-- Hotkey list -->
    {#each HOTKEY_ACTIONS as action}
      <div class="flex items-center justify-between p-4 bg-void-100 rounded-xl border border-glass-border">
        <div class="flex-1">
          <p class="text-text-primary font-medium">{action.label}</p>
          <p class="text-text-secondary text-sm">{action.description}</p>
        </div>
        
        <div class="flex items-center gap-2">
          {#if recordingHotkey === action.id}
            <!-- Recording mode -->
            <div class="flex items-center gap-2">
              <input
                type="text"
                readonly
                placeholder="Press keys..."
                onkeydown={handleHotkeyKeydown}
                onblur={stopRecordingHotkey}
                class="w-40 bg-indigo-500/10 text-indigo-400 rounded-lg px-4 py-2 border-2 border-indigo-500 focus:outline-none text-center font-mono animate-pulse"
              />
              <button
                onclick={stopRecordingHotkey}
                class="p-2 text-text-muted hover:text-text-primary transition-colors"
                aria-label="Cancel"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                </svg>
              </button>
            </div>
          {:else}
            <!-- Display mode -->
            <button
              onclick={() => startRecordingHotkey(action.id)}
              class="px-4 py-2 bg-void-200 hover:bg-void-300 text-text-primary rounded-lg font-mono text-sm border border-glass-border transition-colors min-w-[140px]"
            >
              {formatHotkey(hotkeys[action.id])}
            </button>
            <button
              onclick={() => resetHotkey(action.id)}
              class="p-2 text-text-muted hover:text-indigo-400 transition-colors"
              aria-label="Reset to default"
              title="Reset to default"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
              </svg>
            </button>
          {/if}
        </div>
      </div>
      
      {#if recordingHotkey === action.id && hotkeyConflict}
        <div class="ml-4 -mt-2 mb-2 text-red-400 text-sm flex items-center gap-2">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
          </svg>
          {hotkeyConflict}
        </div>
      {/if}
    {/each}

    <!-- Reset all button -->
    <div class="pt-4 border-t border-glass-border">
      <button
        onclick={resetAllHotkeys}
        class="px-4 py-2 bg-void-200 hover:bg-void-300 text-text-secondary hover:text-text-primary rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
        </svg>
        Reset All to Defaults
      </button>
    </div>

    <!-- Help text -->
    <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
      <p class="text-indigo-400 text-sm flex items-start gap-2">
        <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span>Click on a shortcut to change it. Press the new key combination, then release. Shortcuts must include Ctrl, Alt, or Shift. Press Escape to cancel.</span>
      </p>
    </div>
  </div>
</div>
