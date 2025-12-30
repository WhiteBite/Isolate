<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  
  let { children } = $props();
  let isReady = $state(false);
  let isOnboarding = $state(false);

  const menuItems = [
    { path: '/', icon: 'home', label: 'Dashboard' },
    { path: '/proxies', icon: 'server', label: 'Proxies' },
    { path: '/routing', icon: 'git-branch', label: 'Routing' },
    { path: '/strategies', icon: 'zap', label: 'Strategies' },
    { path: '/testing', icon: 'activity', label: 'Testing' },
    { path: '/settings', icon: 'settings', label: 'Settings' },
    { path: '/logs', icon: 'file-text', label: 'Logs' },
  ];

  onMount(async () => {
    if (!browser) {
      isReady = true;
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const onboardingComplete = await invoke<boolean>('get_setting', { key: 'onboarding_complete' }).catch(() => false);
      
      const currentPath = window.location.pathname;
      
      if (!onboardingComplete && currentPath !== '/onboarding') {
        isOnboarding = true;
        goto('/onboarding');
      } else if (onboardingComplete && currentPath === '/onboarding') {
        goto('/');
      }
      
      isOnboarding = currentPath === '/onboarding';
    } catch (e) {
      console.error('Failed to check onboarding status:', e);
    }
    
    isReady = true;
  });

  // Update isOnboarding when page changes
  $effect(() => {
    if (browser) {
      isOnboarding = $page.url.pathname === '/onboarding';
    }
  });

  function isActive(path: string): boolean {
    if (path === '/') {
      return $page.url.pathname === '/';
    }
    return $page.url.pathname.startsWith(path);
  }
</script>

{#if isReady}
  {#if isOnboarding}
    <!-- Onboarding without sidebar -->
    <main class="min-h-screen bg-[#0a0e27]">
      {@render children()}
    </main>
  {:else}
    <!-- Main layout with sidebar -->
    <div class="flex min-h-screen bg-[#0a0e27]">
      <!-- Sidebar -->
      <aside class="w-64 bg-[#1a1f3a] flex flex-col border-r border-[#2a2f4a]">
        <!-- Logo -->
        <div class="p-6 border-b border-[#2a2f4a]">
          <h1 class="text-2xl font-bold text-[#00d4ff]">Isolate</h1>
          <p class="text-xs text-[#a0a0a0] mt-1">DPI Bypass Tool</p>
        </div>

        <!-- Navigation -->
        <nav class="flex-1 p-4 space-y-1">
          {#each menuItems as item}
            <a
              href={item.path}
              class="flex items-center gap-3 px-4 py-3 rounded-lg transition-all duration-200 {isActive(item.path) 
                ? 'bg-[#2a2f4a] text-[#00d4ff]' 
                : 'text-[#a0a0a0] hover:bg-[#2a2f4a]/50 hover:text-white'}"
            >
              {#if item.icon === 'home'}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                </svg>
              {:else if item.icon === 'server'}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                </svg>
              {:else if item.icon === 'git-branch'}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              {:else if item.icon === 'zap'}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              {:else if item.icon === 'activity'}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              {:else if item.icon === 'settings'}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
              {:else if item.icon === 'file-text'}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              {/if}
              <span class="font-medium">{item.label}</span>
            </a>
          {/each}
        </nav>

        <!-- Footer -->
        <div class="p-4 border-t border-[#2a2f4a]">
          <div class="text-xs text-[#a0a0a0]">
            <span>v0.1.0</span>
          </div>
        </div>
      </aside>

      <!-- Main Content -->
      <main class="flex-1 overflow-auto">
        {@render children()}
      </main>
    </div>
  {/if}
{:else}
  <main class="min-h-screen bg-[#0a0e27] flex items-center justify-center">
    <div class="w-8 h-8 border-2 border-[#00d4ff] border-t-transparent rounded-full animate-spin"></div>
  </main>
{/if}
