<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  
  let { children } = $props();
  let isReady = $state(false);

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
        goto('/onboarding');
      } else if (onboardingComplete && currentPath === '/onboarding') {
        goto('/');
      }
    } catch (e) {
      console.error('Failed to check onboarding status:', e);
    }
    
    isReady = true;
  });
</script>

{#if isReady}
  <main class="min-h-screen">
    {@render children()}
  </main>
{:else}
  <main class="min-h-screen flex items-center justify-center">
    <div class="w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full animate-spin"></div>
  </main>
{/if}
