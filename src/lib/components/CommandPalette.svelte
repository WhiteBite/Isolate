<script lang="ts">
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { createEventDispatcher } from 'svelte';

  interface Command {
    id: string;
    label: string;
    category: 'navigation' | 'actions' | 'settings';
    shortcut?: string;
    icon?: string;
    action: () => void | Promise<void>;
  }

  let isOpen = $state(false);
  let query = $state('');
  let selectedIndex = $state(0);
  let inputRef: HTMLInputElement | null = $state(null);

  const dispatch = createEventDispatcher();

  // Команды
  const commands: Command[] = [
    // Navigation
    {
      id: 'nav-dashboard',
      label: 'Go to Dashboard',
      category: 'navigation',
      shortcut: '⌘1',
      icon: 'home',
      action: () => navigate('/')
    },
    {
      id: 'nav-diagnostics',
      label: 'Go to Diagnostics',
      category: 'navigation',
      shortcut: '⌘2',
      icon: 'wand',
      action: () => navigate('/diagnostics')
    },
    {
      id: 'nav-proxies',
      label: 'Go to Proxies',
      category: 'navigation',
      shortcut: '⌘3',
      icon: 'globe',
      action: () => navigate('/proxies')
    },
    {
      id: 'nav-settings',
      label: 'Go to Settings',
      category: 'navigation',
      shortcut: '⌘4',
      icon: 'cog',
      action: () => navigate('/settings')
    },
    // Actions
    {
      id: 'action-start',
      label: 'Start Protection',
      category: 'actions',
      icon: 'play',
      action: () => invokeCommand('apply_recommended_strategy')
    },
    {
      id: 'action-stop',
      label: 'Stop Protection',
      category: 'actions',
      icon: 'stop',
      action: () => invokeCommand('stop_strategy')
    },
    {
      id: 'action-add-proxy',
      label: 'Add Proxy',
      category: 'actions',
      icon: 'plus',
      action: () => emitEvent('open-add-proxy')
    },
    {
      id: 'action-test',
      label: 'Test Connection',
      category: 'actions',
      icon: 'test',
      action: () => invokeCommand('check_all_registry_services')
    },
    {
      id: 'action-panic',
      label: 'Panic Reset',
      category: 'actions',
      shortcut: '⌘⇧R',
      icon: 'alert',
      action: () => invokeCommand('panic_reset')
    },
    // Settings
    {
      id: 'settings-theme',
      label: 'Toggle Theme',
      category: 'settings',
      icon: 'theme',
      action: () => emitEvent('toggle-theme')
    }
  ];

  // Fuzzy search
  function fuzzyMatch(text: string, pattern: string): boolean {
    if (!pattern) return true;
    const lowerText = text.toLowerCase();
    const lowerPattern = pattern.toLowerCase();
    
    // Simple contains check + fuzzy
    if (lowerText.includes(lowerPattern)) return true;
    
    let patternIdx = 0;
    for (let i = 0; i < lowerText.length && patternIdx < lowerPattern.length; i++) {
      if (lowerText[i] === lowerPattern[patternIdx]) {
        patternIdx++;
      }
    }
    return patternIdx === lowerPattern.length;
  }

  // Filtered commands
  let filteredCommands = $derived.by(() => {
    if (!query.trim()) return commands;
    return commands.filter(cmd => fuzzyMatch(cmd.label, query));
  });

  // Grouped by category
  let groupedCommands = $derived.by(() => {
    const groups: Record<string, Command[]> = {
      navigation: [],
      actions: [],
      settings: []
    };
    
    for (const cmd of filteredCommands) {
      groups[cmd.category].push(cmd);
    }
    
    return groups;
  });

  // Flat list for keyboard navigation
  let flatList = $derived.by(() => {
    return [
      ...groupedCommands.navigation,
      ...groupedCommands.actions,
      ...groupedCommands.settings
    ];
  });

  // Category labels
  const categoryLabels: Record<string, string> = {
    navigation: 'Navigation',
    actions: 'Actions',
    settings: 'Settings'
  };

  // Navigation helper
  function navigate(path: string) {
    close();
    goto(path);
  }

  // Invoke Tauri command
  async function invokeCommand(command: string) {
    close();
    if (!browser) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke(command);
    } catch (e) {
      console.error(`Command ${command} failed:`, e);
    }
  }

  // Emit custom event
  function emitEvent(eventName: string) {
    close();
    dispatch(eventName);
    // Also dispatch on window for global listeners
    if (browser) {
      window.dispatchEvent(new CustomEvent(eventName));
    }
  }

  // Open/Close
  function open() {
    isOpen = true;
    query = '';
    selectedIndex = 0;
    // Focus input after render
    setTimeout(() => inputRef?.focus(), 10);
  }

  function close() {
    isOpen = false;
    query = '';
    selectedIndex = 0;
  }

  // Execute selected command
  function executeSelected() {
    const cmd = flatList[selectedIndex];
    if (cmd) {
      cmd.action();
    }
  }

  // Keyboard handling
  function handleKeydown(e: KeyboardEvent) {
    // Global: Ctrl+K to open
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      if (isOpen) {
        close();
      } else {
        open();
      }
      return;
    }

    // Only handle when open
    if (!isOpen) return;

    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        close();
        break;
      case 'ArrowDown':
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, flatList.length - 1);
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        e.preventDefault();
        executeSelected();
        break;
    }
  }

  // Backdrop click
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      close();
    }
  }

  // Reset selection when query changes
  $effect(() => {
    query; // dependency
    selectedIndex = 0;
  });

  // Scroll selected item into view
  $effect(() => {
    if (browser && isOpen) {
      const el = document.querySelector(`[data-command-index="${selectedIndex}"]`);
      el?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  });

  // Setup keyboard listener with cleanup
  $effect(() => {
    if (!browser) return;
    
    window.addEventListener('keydown', handleKeydown);
    
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });

  // Get icon SVG
  function getIcon(icon: string | undefined): string {
    switch (icon) {
      case 'home':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />';
      case 'wand':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />';
      case 'globe':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />';
      case 'cog':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />';
      case 'play':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />';
      case 'stop':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />';
      case 'plus':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />';
      case 'test':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />';
      case 'alert':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />';
      case 'theme':
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />';
      default:
        return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />';
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[100] flex items-start justify-center pt-[15vh]"
    style="background: rgba(5, 5, 5, 0.85); backdrop-filter: blur(8px);"
    onclick={handleBackdropClick}
  >
    <div
      class="w-full max-w-xl bg-[#0d0d0d]/95 rounded-2xl border border-[#2a2f4a]/50 shadow-2xl overflow-hidden"
      style="box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.05);"
      role="dialog"
      aria-modal="true"
      aria-label="Command Palette"
    >
      <!-- Search Input -->
      <div class="relative border-b border-[#2a2f4a]/50">
        <div class="absolute left-4 top-1/2 -translate-y-1/2 text-[#606060]">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <input
          bind:this={inputRef}
          bind:value={query}
          type="text"
          placeholder="Type a command or search..."
          class="w-full bg-transparent text-white text-lg px-12 py-4 outline-none placeholder:text-[#505050]"
          autocomplete="off"
          spellcheck="false"
        />
        <div class="absolute right-4 top-1/2 -translate-y-1/2">
          <kbd class="px-2 py-1 text-xs text-[#606060] bg-[#1a1a1a] rounded border border-[#2a2a2a]">ESC</kbd>
        </div>
      </div>

      <!-- Results -->
      <div class="max-h-[400px] overflow-y-auto py-2" style="scrollbar-width: thin; scrollbar-color: #2a2f4a transparent;">
        {#if flatList.length === 0}
          <div class="px-4 py-8 text-center text-[#606060]">
            <p>No commands found</p>
          </div>
        {:else}
          {#each ['navigation', 'actions', 'settings'] as category}
            {#if groupedCommands[category].length > 0}
              <!-- Category Header -->
              <div class="px-4 py-2">
                <span class="text-xs font-medium text-[#505050] uppercase tracking-wider">
                  {categoryLabels[category]}
                </span>
              </div>
              
              <!-- Commands in category -->
              {#each groupedCommands[category] as cmd, i}
                {@const globalIndex = flatList.indexOf(cmd)}
                <button
                  data-command-index={globalIndex}
                  onclick={() => cmd.action()}
                  onmouseenter={() => selectedIndex = globalIndex}
                  class="w-full flex items-center gap-3 px-4 py-3 text-left transition-colors duration-75
                         {selectedIndex === globalIndex 
                           ? 'bg-[#00d4ff]/10 text-white' 
                           : 'text-[#a0a0a0] hover:bg-[#1a1a1a]'}"
                >
                  <!-- Icon -->
                  <div class="w-8 h-8 flex items-center justify-center rounded-lg bg-[#1a1a1a] {selectedIndex === globalIndex ? 'bg-[#00d4ff]/20' : ''}">
                    <svg class="w-4 h-4 {selectedIndex === globalIndex ? 'text-[#00d4ff]' : 'text-[#707070]'}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      {@html getIcon(cmd.icon)}
                    </svg>
                  </div>
                  
                  <!-- Label -->
                  <span class="flex-1 font-medium">{cmd.label}</span>
                  
                  <!-- Shortcut -->
                  {#if cmd.shortcut}
                    <kbd class="px-2 py-1 text-xs text-[#505050] bg-[#1a1a1a] rounded border border-[#2a2a2a]">
                      {cmd.shortcut}
                    </kbd>
                  {/if}
                </button>
              {/each}
            {/if}
          {/each}
        {/if}
      </div>

      <!-- Footer hint -->
      <div class="px-4 py-3 border-t border-[#2a2f4a]/50 flex items-center gap-4 text-xs text-[#505050]">
        <span class="flex items-center gap-1">
          <kbd class="px-1.5 py-0.5 bg-[#1a1a1a] rounded border border-[#2a2a2a]">↑</kbd>
          <kbd class="px-1.5 py-0.5 bg-[#1a1a1a] rounded border border-[#2a2a2a]">↓</kbd>
          <span class="ml-1">to navigate</span>
        </span>
        <span class="flex items-center gap-1">
          <kbd class="px-1.5 py-0.5 bg-[#1a1a1a] rounded border border-[#2a2a2a]">↵</kbd>
          <span class="ml-1">to select</span>
        </span>
        <span class="flex items-center gap-1">
          <kbd class="px-1.5 py-0.5 bg-[#1a1a1a] rounded border border-[#2a2a2a]">esc</kbd>
          <span class="ml-1">to close</span>
        </span>
      </div>
    </div>
  </div>
{/if}
