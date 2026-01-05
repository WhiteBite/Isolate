<script lang="ts">
  import '../app.css';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { appStatus } from '$lib/stores';
  import { toasts } from '$lib/stores/toast';
  import { 
    ToastContainer, 
    CommandPalette, 
    Sidebar,
    TerminalPanel
  } from '$lib/components';
  
  let { children } = $props();
  let isReady = $state(true);
  let isOnboarding = $state(false);
  let sidebarCollapsed = $state(false);
  let initialized = $state(false);
  
  // Page transition state
  let currentPath = $state('');
  let isTransitioning = $state(false);
  
  // Track page changes for transitions
  $effect(() => {
    const newPath = $page.url.pathname;
    if (currentPath && currentPath !== newPath) {
      isTransitioning = true;
      // Reset transition after animation completes
      setTimeout(() => {
        isTransitioning = false;
      }, 50);
    }
    currentPath = newPath;
  });

  async function checkOnboarding() {
    if (!browser || initialized) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Ждём готовности бэкенда с retry
      let backendReady = false;
      for (let i = 0; i < 10; i++) {
        try {
          backendReady = await invoke<boolean>('is_backend_ready');
          if (backendReady) break;
        } catch {
          // Команда может не существовать, продолжаем
        }
        await new Promise(r => setTimeout(r, 200));
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
    } catch (e) {
      console.error('Failed to check onboarding status:', e);
    }
  }

  // Заменяем onMount на $effect для инициализации
  $effect(() => {
    checkOnboarding();
  });

  $effect(() => {
    if (browser) {
      isOnboarding = $page.url.pathname === '/onboarding';
    }
  });

  // Keyboard shortcuts Ctrl+1-4 для переключения панелей
  $effect(() => {
    if (!browser) return;

    const handleKeydown = (e: KeyboardEvent) => {
      // Игнорируем если фокус в input/textarea
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        return;
      }

      // Ctrl+1-4 для навигации
      if (e.ctrlKey && !e.shiftKey && !e.altKey && !e.metaKey) {
        const routes: Record<string, string> = {
          '1': '/',           // Dashboard
          '2': '/services',   // Services
          '3': '/routing',    // Routing
          '4': '/proxies'     // Proxies
        };

        if (routes[e.key]) {
          e.preventDefault();
          goto(routes[e.key]);
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
      <div 
        class="page-transition"
        class:page-enter={isTransitioning}
      >
        {@render children()}
      </div>
    </main>
  {:else}
    <!-- Main Three-Pane Layout -->
    <div class="flex h-screen bg-zinc-950 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-indigo-900/15 via-zinc-950 to-zinc-950 overflow-hidden">
      <!-- Sidebar -->
      <Sidebar bind:collapsed={sidebarCollapsed} />

      <!-- Main Content Area with Resizable Panels -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <!-- Top Bar -->
        <header class="h-12 backdrop-blur-xl bg-zinc-950/80 border-b border-white/5 flex items-center justify-between px-4 shrink-0">
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
                <span class="text-zinc-500 text-xs">Inactive</span>
              </div>
            {/if}
          </div>

          <!-- Right: Quick Actions -->
          <div class="flex items-center gap-2">
            <button
              onclick={() => {
                if (browser) {
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
              <kbd class="px-1.5 py-0.5 text-[10px] bg-zinc-900 rounded border border-white/5 text-zinc-500">⌘K</kbd>
            </button>
          </div>
        </header>

        <!-- Content Area -->
        <main class="flex-1 overflow-auto">
          <div 
            class="page-transition"
            class:page-enter={isTransitioning}
          >
            {@render children()}
          </div>
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

<!-- Global Components -->
<CommandPalette />
<TerminalPanel />
<ToastContainer />

<style>
  /* Page transition styles */
  .page-transition {
    animation: pageEnter 250ms ease-out forwards;
  }
  
  .page-enter {
    animation: none;
    opacity: 0;
    transform: translateY(8px);
  }
  
  @keyframes pageEnter {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
