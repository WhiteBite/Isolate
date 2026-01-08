<script lang="ts">
  import '../app.css';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { appStatus, isOptimizing, optimizationProgress } from '$lib/stores';
  import { toasts } from '$lib/stores/toast';
  import { logs } from '$lib/stores/logs';
  import { themeStore } from '$lib/stores/theme';
  import { hotkeysStore, matchesHotkey, type HotkeysState } from '$lib/stores/hotkeys';
  import { initLocale, t } from '$lib/i18n';
  import { 
    ToastContainer, 
    Sidebar,
    PageTransition,
    UpdateNotification,
    KeyboardOverlay
  } from '$lib/components';
  import { lazyComponents, preloadAllLazyComponents } from '$lib/utils/lazyComponent';
  import { waitForBackend, isTauriEnv } from '$lib/hooks/useBackendReady';
  import type { Component } from 'svelte';
  
  let { children } = $props();
  let isReady = $state(true);
  let isOnboarding = $state(false);
  let sidebarCollapsed = $state(false);
  let initialized = $state(false);
  let isCheckingOnboarding = $state(false); // Guard against concurrent checkOnboarding calls
  let showShortcutsModal = $state(false);
  
  // Lazy loading triggers - components load on first use
  let commandPaletteTriggered = $state(false);
  let terminalTriggered = $state(false);
  let shortcutsModalTriggered = $state(false);
  
  // Loaded components for direct rendering
  let CommandPaletteComponent = $state<Component | null>(null);
  let TerminalPanelComponent = $state<Component | null>(null);
  let KeyboardShortcutsModalComponent = $state<Component | null>(null);
  
  // Track app status for shortcuts
  let appStatusValue = $state<{isActive: boolean; currentStrategy: string | null; currentStrategyName: string | null}>({
    isActive: false,
    currentStrategy: null,
    currentStrategyName: null
  });
  
  // Track hotkeys configuration
  let currentHotkeys = $state<HotkeysState>(hotkeysStore.get());
  
  // Subscribe to appStatus store
  $effect(() => {
    const unsubscribe = appStatus.subscribe(v => { appStatusValue = v; });
    return unsubscribe;
  });

  // Subscribe to hotkeys store
  $effect(() => {
    const unsubscribe = hotkeysStore.subscribe(state => { currentHotkeys = state; });
    return unsubscribe;
  });

  // Initialize theme on mount
  $effect(() => {
    if (browser) {
      const cleanup = themeStore.init();
      initLocale(); // Initialize i18n
      return cleanup;
    }
  });

  async function checkOnboarding() {
    // Guard: prevent multiple initializations and concurrent calls
    if (!browser || initialized || isCheckingOnboarding) return;
    isCheckingOnboarding = true;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Wait for backend with exponential backoff
      const backendReady = await waitForBackend({
        maxRetries: 15,
        initialDelay: 100,
        maxDelay: 1000,
      });
      
      if (!backendReady) {
        console.warn('[Layout] Backend not ready after retries');
        initialized = true;
        isCheckingOnboarding = false;
        return;
      }
      
      const result = await invoke<boolean | null>('get_setting', { key: 'onboarding_complete' }).catch(() => null);
      const onboardingComplete = result === true;
      
      const currentPath = window.location.pathname;
      
      if (!onboardingComplete && currentPath !== '/onboarding') {
        isOnboarding = true;
        goto('/onboarding');
      } else if (onboardingComplete && currentPath === '/onboarding') {
        goto('/');
      }
      
      isOnboarding = currentPath === '/onboarding';
      initialized = true;
      isCheckingOnboarding = false;
    } catch (e) {
      console.error('Failed to check onboarding status:', e);
      initialized = true; // Mark as initialized even on error to prevent infinite retries
      isCheckingOnboarding = false;
    }
  }

  // Initialize on mount with guard
  $effect(() => {
    if (!initialized && !isCheckingOnboarding) {
      checkOnboarding();
    }
  });

  // Preload lazy components after app is interactive
  $effect(() => {
    if (browser && initialized) {
      preloadAllLazyComponents();
    }
  });

  // Listen for failover events from backend
  $effect(() => {
    if (!browser || !initialized) return;
    
    let unlisten: (() => void) | undefined;
    let isCleanedUp = false;
    
    (async () => {
      if (!isTauriEnv()) return;
      
      try {
        const { listen } = await import('@tauri-apps/api/event');
        
        // Guard: don't setup listeners if already cleaned up
        if (isCleanedUp) return;
        
        // Listen for failover triggered event
        const unlistenTriggered = await listen<{
          previousStrategy: string;
          newStrategy: string;
          reason: string;
          timestamp: string;
        }>('failover:triggered', (event) => {
          logs.warn('system', `Auto-recovery triggered: ${event.payload.reason}`);
          toasts.warning(`Switching strategy: ${event.payload.reason}`);
        });
        
        // Listen for apply strategy event
        const unlistenApply = await listen<string>('failover:apply_strategy', async (event) => {
          const strategyId = event.payload;
          logs.info('system', `Auto-recovery: applying strategy ${strategyId}`);
          
          try {
            const { invoke } = await import('@tauri-apps/api/core');
            await invoke('apply_strategy', { strategyId });
            
            appStatus.set({
              isActive: true,
              currentStrategy: strategyId,
              currentStrategyName: strategyId
            });
            
            logs.success('system', `Auto-recovery: switched to ${strategyId}`);
            toasts.success(`Switched to backup strategy: ${strategyId}`);
          } catch (e) {
            logs.error('system', `Auto-recovery failed: ${e}`);
            toasts.error(`Failed to apply backup strategy: ${e}`);
          }
        });
        
        // Guard: if cleaned up during async setup, immediately cleanup listeners
        if (isCleanedUp) {
          unlistenTriggered();
          unlistenApply();
          return;
        }
        
        unlisten = () => {
          unlistenTriggered();
          unlistenApply();
        };
      } catch (e) {
        console.error('Failed to setup failover listeners:', e);
      }
    })();
    
    return () => {
      isCleanedUp = true;
      if (unlisten) unlisten();
    };
  });
  
  // Load CommandPalette when triggered (Ctrl+K)
  $effect(() => {
    if (commandPaletteTriggered && !CommandPaletteComponent) {
      lazyComponents.CommandPalette.load().then(c => {
        CommandPaletteComponent = c;
      });
    }
  });
  
  // Load TerminalPanel when triggered (Ctrl+`)
  $effect(() => {
    if (terminalTriggered && !TerminalPanelComponent) {
      lazyComponents.TerminalPanel.load().then(c => {
        TerminalPanelComponent = c;
      });
    }
  });
  
  // Load KeyboardShortcutsModal when triggered (? or F1)
  $effect(() => {
    if (shortcutsModalTriggered && !KeyboardShortcutsModalComponent) {
      lazyComponents.KeyboardShortcutsModal.load().then(c => {
        KeyboardShortcutsModalComponent = c;
      });
    }
  });

  $effect(() => {
    if (browser) {
      isOnboarding = $page.url.pathname === '/onboarding';
    }
  });

  // Toggle protection (start/stop)
  async function toggleProtection() {
    if (!browser) return;
    
    if (!isTauriEnv()) {
      toasts.info('Protection toggle is only available in the desktop app');
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      if (appStatusValue.isActive) {
        // Stop protection
        await invoke('stop_strategy');
        appStatus.set({
          isActive: false,
          currentStrategy: null,
          currentStrategyName: null
        });
        logs.info('system', 'Protection stopped via shortcut');
        toasts.success('Protection stopped');
      } else {
        // Start optimization to find best strategy
        isOptimizing.set(true);
        optimizationProgress.set({
          step: 'initializing',
          progress: 0,
          message: 'Finding best strategy...',
          isComplete: false,
          error: null
        });
        logs.info('system', 'Starting protection via shortcut');
        await invoke('run_optimization_v2');
      }
    } catch (e) {
      logs.error('system', `Toggle protection failed: ${e}`);
      toasts.error(`Failed to toggle protection: ${e}`);
      isOptimizing.set(false);
      optimizationProgress.set({
        step: 'failed',
        progress: 0,
        message: '',
        isComplete: false,
        error: String(e)
      });
    }
  }
  
  // Refresh/rescan services
  async function refreshServices() {
    if (!browser) return;
    
    if (!isTauriEnv()) {
      toasts.info('Service refresh is only available in the desktop app');
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      logs.info('system', 'Refreshing services via shortcut');
      toasts.info('Refreshing services...');
      
      // Check all registry services
      await invoke('check_all_registry_services');
      
      logs.success('system', 'Services refreshed');
      toasts.success('Services refreshed');
    } catch (e) {
      logs.error('system', `Refresh services failed: ${e}`);
      toasts.error(`Failed to refresh services: ${e}`);
    }
  }
  
  // Focus terminal/logs panel
  function focusTerminal() {
    if (!browser) return;
    
    // Trigger lazy loading of TerminalPanel
    terminalTriggered = true;
    
    // Dispatch Ctrl+` to toggle terminal
    window.dispatchEvent(new KeyboardEvent('keydown', { 
      key: '`', 
      ctrlKey: true,
      bubbles: true 
    }));
  }
  
  // Open command palette
  function openCommandPalette() {
    if (!browser) return;
    
    // Trigger lazy loading of CommandPalette
    commandPaletteTriggered = true;
    
    window.dispatchEvent(new KeyboardEvent('keydown', { 
      key: 'k', 
      ctrlKey: true,
      bubbles: true 
    }));
  }

  // Run quick connectivity test
  async function runQuickTest() {
    if (!browser) return;
    
    if (!isTauriEnv()) {
      toasts.info('Quick test is only available in the desktop app');
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      logs.info('system', 'Running quick test via hotkey');
      toasts.info('Running connectivity test...');
      
      // Run quick test command
      const result = await invoke<{ success: boolean; message: string }>('quick_test').catch(() => null);
      
      if (result?.success) {
        logs.success('system', `Quick test passed: ${result.message}`);
        toasts.success(result.message || 'Connection test passed');
      } else {
        logs.warn('system', `Quick test: ${result?.message || 'Test completed'}`);
        toasts.info(result?.message || 'Test completed');
      }
    } catch (e) {
      logs.error('system', `Quick test failed: ${e}`);
      toasts.error(`Quick test failed: ${e}`);
    }
  }

  // Stop all running strategies and processes
  async function stopAll() {
    if (!browser) return;
    
    if (!isTauriEnv()) {
      toasts.info('Stop all is only available in the desktop app');
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      logs.info('system', 'Stopping all processes via hotkey');
      toasts.info('Stopping all processes...');
      
      // Stop current strategy
      await invoke('stop_strategy').catch(() => {});
      
      // Reset app status
      appStatus.set({
        isActive: false,
        currentStrategy: null,
        currentStrategyName: null
      });
      
      // Stop optimization if running
      isOptimizing.set(false);
      optimizationProgress.set({
        step: 'idle',
        progress: 0,
        message: '',
        isComplete: false,
        error: null
      });
      
      logs.success('system', 'All processes stopped');
      toasts.success('All processes stopped');
    } catch (e) {
      logs.error('system', `Stop all failed: ${e}`);
      toasts.error(`Failed to stop all: ${e}`);
    }
  }

  // Keyboard shortcuts
  $effect(() => {
    if (!browser) return;

    const handleKeydown = (e: KeyboardEvent) => {
      // Игнорируем если фокус в input/textarea
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        return;
      }

      // User-configurable hotkeys (from hotkeysStore)
      if (matchesHotkey(e, currentHotkeys.toggleStrategy)) {
        e.preventDefault();
        toggleProtection();
        return;
      }
      
      if (matchesHotkey(e, currentHotkeys.openSettings)) {
        e.preventDefault();
        goto('/settings');
        return;
      }
      
      if (matchesHotkey(e, currentHotkeys.quickTest)) {
        e.preventDefault();
        runQuickTest();
        return;
      }

      if (matchesHotkey(e, currentHotkeys.stopAll)) {
        e.preventDefault();
        stopAll();
        return;
      }

      // ? или F1 для открытия справки по горячим клавишам
      if ((e.key === '?' || e.key === 'F1') && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        shortcutsModalTriggered = true;
        showShortcutsModal = true;
        return;
      }
      
      // Ctrl+Shift+P — альтернатива для Command Palette
      if (e.ctrlKey && e.shiftKey && e.key === 'P') {
        e.preventDefault();
        openCommandPalette();
        return;
      }

      // Ctrl+<key> shortcuts (без Shift)
      if (e.ctrlKey && !e.shiftKey && !e.altKey && !e.metaKey) {
        // Ctrl+1-3 для навигации
        const routes: Record<string, string> = {
          '1': '/',           // Dashboard
          '2': '/services',   // Services
          '3': '/network'     // Network (proxies + routing)
        };

        if (routes[e.key]) {
          e.preventDefault();
          goto(routes[e.key]);
          return;
        }
        
        // Ctrl+S — Toggle protection (Start/Stop)
        if (e.key === 's') {
          e.preventDefault();
          toggleProtection();
          return;
        }
        
        // Ctrl+R — Refresh/Rescan services
        if (e.key === 'r') {
          e.preventDefault();
          refreshServices();
          return;
        }
        
        // Ctrl+, — Open Settings
        if (e.key === ',') {
          e.preventDefault();
          goto('/settings');
          return;
        }
        
        // Ctrl+M — Open Marketplace
        if (e.key === 'm') {
          e.preventDefault();
          goto('/marketplace');
          return;
        }
        
        // Ctrl+L — Focus on Logs/Terminal
        if (e.key === 'l') {
          e.preventDefault();
          focusTerminal();
          return;
        }
      }
    };

    window.addEventListener('keydown', handleKeydown);
    
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

{#if isReady}
  {#if isOnboarding}
    <!-- Onboarding without shell -->
    <main class="min-h-screen bg-void">
      <PageTransition>
        {@render children()}
      </PageTransition>
    </main>
  {:else}
    <!-- Skip to content link for keyboard navigation -->
    <a 
      href="#main-content" 
      class="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-[100] 
             focus:px-4 focus:py-2 focus:bg-indigo-500 focus:text-white focus:rounded-lg 
             focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 focus:ring-offset-zinc-950"
    >
      Skip to main content
    </a>
    
    <!-- Main Three-Pane Layout -->
    <div class="flex h-screen bg-zinc-950 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-indigo-900/15 via-zinc-950 to-zinc-950 overflow-hidden">
      <!-- Sidebar -->
      <Sidebar bind:collapsed={sidebarCollapsed} />

      <!-- Main Content Area with Resizable Panels -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <!-- Top Bar (Draggable) -->
        <header 
          data-tauri-drag-region
          class="h-12 backdrop-blur-xl bg-zinc-950/80 border-b border-white/5 flex items-center justify-between px-4 shrink-0"
        >
          <!-- Left: Breadcrumb / Page Title -->
          <div class="flex items-center gap-2">
            <span class="text-zinc-400 text-sm font-medium">
              {#if $page.url.pathname === '/'}
                Dashboard
              {:else if $page.url.pathname.startsWith('/diagnostics')}
                Diagnostics
              {:else if $page.url.pathname.startsWith('/proxies')}
                Proxies
              {:else if $page.url.pathname.startsWith('/settings')}
                Settings
              {:else if $page.url.pathname.startsWith('/services')}
                Services
              {:else if $page.url.pathname.startsWith('/routing')}
                Routing
              {:else}
                {$page.url.pathname.slice(1)}
              {/if}
            </span>
          </div>

          <!-- Center: Global Status -->
          <div class="flex items-center gap-3">
            {#if $appStatus.isActive}
              <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-emerald-500/10 border border-emerald-500/20">
                <div class="w-1.5 h-1.5 rounded-full bg-emerald-400"></div>
                <span class="text-emerald-400 text-xs font-medium">
                  {$appStatus.currentStrategyName || 'Protected'}
                </span>
              </div>
            {:else}
              <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-800/50 border border-white/5">
                <div class="w-1.5 h-1.5 rounded-full bg-zinc-500"></div>
                <span class="text-zinc-400 text-xs">Inactive</span>
              </div>
            {/if}
          </div>

          <!-- Right: Quick Actions + Window Controls -->
          <div class="flex items-center gap-2">
            <button
              onclick={() => {
                if (browser) {
                  commandPaletteTriggered = true;
                  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true }));
                }
              }}
              class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-zinc-800/50 border border-white/5
                     hover:bg-zinc-800 hover:border-white/10 transition-all text-sm text-zinc-400"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <span>Search</span>
              <kbd class="px-1.5 py-0.5 text-[10px] bg-zinc-900 rounded border border-white/5 text-zinc-400">⌘K</kbd>
            </button>
            
            <!-- Window Controls -->
            <div class="flex items-center ml-2 -mr-2">
              <button
                onclick={async () => {
                  if (browser) {
                    const { getCurrentWindow } = await import('@tauri-apps/api/window');
                    getCurrentWindow().minimize();
                  }
                }}
                class="w-10 h-10 flex items-center justify-center text-zinc-400 hover:text-zinc-300 hover:bg-white/5 transition-colors"
                aria-label="Minimize"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
                </svg>
              </button>
              <button
                onclick={async () => {
                  if (browser) {
                    const { getCurrentWindow } = await import('@tauri-apps/api/window');
                    getCurrentWindow().toggleMaximize();
                  }
                }}
                class="w-10 h-10 flex items-center justify-center text-zinc-400 hover:text-zinc-300 hover:bg-white/5 transition-colors"
                aria-label="Maximize"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <rect x="4" y="4" width="16" height="16" rx="2" stroke-width="2" />
                </svg>
              </button>
              <button
                onclick={async () => {
                  if (browser) {
                    const { getCurrentWindow } = await import('@tauri-apps/api/window');
                    getCurrentWindow().close();
                  }
                }}
                class="w-10 h-10 flex items-center justify-center text-zinc-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
                aria-label="Close"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
        </header>

        <!-- Content Area -->
        <main id="main-content" class="flex-1 overflow-auto">
          <PageTransition>
            {@render children()}
          </PageTransition>
        </main>
      </div>
    </div>
  {/if}
{:else}
  <!-- Loading State -->
  <main class="min-h-screen bg-void flex items-center justify-center">
    <div class="flex flex-col items-center gap-4">
      <div class="w-10 h-10 border-2 border-electric border-t-transparent rounded-full animate-spin"></div>
      <span class="text-text-muted text-sm">Loading...</span>
    </div>
  </main>
{/if}

<!-- Global Components (Lazy Loaded) -->
{#if CommandPaletteComponent}
  <CommandPaletteComponent />
{/if}

{#if TerminalPanelComponent}
  <TerminalPanelComponent />
{/if}

<ToastContainer />

<!-- Update Notification (checks GitHub Releases) -->
<UpdateNotification />

{#if KeyboardShortcutsModalComponent}
  <KeyboardShortcutsModalComponent bind:open={showShortcutsModal} />
{/if}

<!-- Keyboard Overlay (shows on Ctrl hold) -->
<KeyboardOverlay />

