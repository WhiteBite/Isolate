<script lang="ts">
  import BaseModal from './BaseModal.svelte';

  interface Props {
    open?: boolean;
    onclose?: () => void;
  }

  let { 
    open = $bindable(false), 
    onclose 
  }: Props = $props();

  interface Shortcut {
    keys: string[];
    description: string;
  }

  interface ShortcutGroup {
    title: string;
    icon: string;
    shortcuts: Shortcut[];
  }

  const shortcutGroups: ShortcutGroup[] = [
    {
      title: 'Protection',
      icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />',
      shortcuts: [
        { keys: ['Ctrl', 'S'], description: 'Start/Stop protection' },
        { keys: ['Ctrl', 'R'], description: 'Refresh services' },
      ]
    },
    {
      title: 'Navigation',
      icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />',
      shortcuts: [
        { keys: ['Ctrl', '1'], description: 'Go to Dashboard' },
        { keys: ['Ctrl', '2'], description: 'Go to Services' },
        { keys: ['Ctrl', '3'], description: 'Go to Routing' },
        { keys: ['Ctrl', '4'], description: 'Go to Proxies' },
        { keys: ['Ctrl', ','], description: 'Open Settings' },
        { keys: ['Ctrl', 'M'], description: 'Open Marketplace' },
      ]
    },
    {
      title: 'Actions',
      icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />',
      shortcuts: [
        { keys: ['Ctrl', 'K'], description: 'Open Command Palette' },
        { keys: ['Ctrl', 'Shift', 'P'], description: 'Open Command Palette (alt)' },
        { keys: ['Ctrl', '`'], description: 'Toggle Terminal' },
        { keys: ['Ctrl', 'L'], description: 'Focus on Logs/Terminal' },
      ]
    },
    {
      title: 'System',
      icon: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />',
      shortcuts: [
        { keys: ['Escape'], description: 'Close modals / Cancel' },
        { keys: ['?'], description: 'Show keyboard shortcuts' },
        { keys: ['F1'], description: 'Show keyboard shortcuts (alt)' },
      ]
    }
  ];

  function handleClose() {
    open = false;
    onclose?.();
  }
</script>

<BaseModal bind:open onclose={handleClose} class="w-full max-w-lg bg-[#0d0d0d]/95 border-[#2a2f4a]/50 overflow-hidden" style="box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.05), 0 0 80px -20px rgba(99, 102, 241, 0.15);">
  <div aria-labelledby="shortcuts-title">
      <!-- Header -->
      <div class="relative px-6 py-5 border-b border-[#2a2f4a]/50">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 flex items-center justify-center rounded-xl bg-indigo-500/10 border border-indigo-500/20">
            <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
            </svg>
          </div>
          <div>
            <h2 id="shortcuts-title" class="text-lg font-semibold text-white">Keyboard Shortcuts</h2>
            <p class="text-sm text-zinc-500">Quick actions to navigate faster</p>
          </div>
        </div>
        
        <!-- Close button -->
        <button
          onclick={handleClose}
          class="absolute top-4 right-4 p-2 text-zinc-500 hover:text-white rounded-lg hover:bg-white/5 transition-colors"
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="px-6 py-5 space-y-6 max-h-[60vh] overflow-y-auto" style="scrollbar-width: thin; scrollbar-color: #2a2f4a transparent;">
        {#each shortcutGroups as group}
          <div>
            <!-- Group Header -->
            <div class="flex items-center gap-2 mb-3">
              <svg class="w-4 h-4 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                {@html group.icon}
              </svg>
              <h3 class="text-xs font-medium text-zinc-500 uppercase tracking-wider">{group.title}</h3>
            </div>
            
            <!-- Shortcuts List -->
            <div class="space-y-2">
              {#each group.shortcuts as shortcut}
                <div class="flex items-center justify-between py-2 px-3 rounded-lg bg-zinc-900/50 border border-white/5 hover:border-white/10 transition-colors">
                  <span class="text-sm text-zinc-300">{shortcut.description}</span>
                  <div class="flex items-center gap-1">
                    {#each shortcut.keys as key, i}
                      {#if i > 0}
                        <span class="text-zinc-600 text-xs">+</span>
                      {/if}
                      <kbd class="min-w-[28px] px-2 py-1 text-xs font-medium text-zinc-400 bg-zinc-800/80 rounded border border-zinc-700/50 text-center shadow-sm">
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

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-[#2a2f4a]/50 bg-zinc-900/30">
        <div class="flex items-center justify-between text-xs text-zinc-500">
          <span>Press <kbd class="px-1.5 py-0.5 bg-zinc-800 rounded border border-zinc-700/50 text-zinc-400">?</kbd> anytime to show this help</span>
          <span class="flex items-center gap-1">
            <kbd class="px-1.5 py-0.5 bg-zinc-800 rounded border border-zinc-700/50 text-zinc-400">Esc</kbd>
            <span>to close</span>
          </span>
        </div>
      </div>
    </div>
</BaseModal>
