<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    type: 'success' | 'error' | 'warning' | 'info';
    message: string;
    duration?: number;
    onclose?: () => void;
  }

  let { 
    type, 
    message, 
    duration = 3000,
    onclose 
  }: Props = $props();

  let visible = $state(true);

  const typeConfig = {
    success: {
      bg: 'bg-green-500/10 border-green-500/50',
      text: 'text-green-400',
      icon: 'M5 13l4 4L19 7'
    },
    error: {
      bg: 'bg-red-500/10 border-red-500/50',
      text: 'text-red-400',
      icon: 'M6 18L18 6M6 6l12 12'
    },
    warning: {
      bg: 'bg-yellow-500/10 border-yellow-500/50',
      text: 'text-yellow-400',
      icon: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z'
    },
    info: {
      bg: 'bg-cyan-500/10 border-cyan-500/50',
      text: 'text-cyan-400',
      icon: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
    }
  };

  const config = $derived(typeConfig[type]);

  onMount(() => {
    if (duration > 0) {
      const timer = setTimeout(() => {
        visible = false;
        onclose?.();
      }, duration);
      return () => clearTimeout(timer);
    }
  });

  function handleClose() {
    visible = false;
    onclose?.();
  }
</script>

{#if visible}
  <div
    class="flex items-center gap-3 px-4 py-3 rounded-lg border {config.bg} shadow-lg animate-in slide-in-from-top-2"
    role="alert"
  >
    <svg class="w-5 h-5 shrink-0 {config.text}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={config.icon} />
    </svg>
    <p class="flex-1 text-sm text-white">{message}</p>
    <button
      onclick={handleClose}
      class="p-1 text-gray-400 hover:text-white rounded transition-colors"
      aria-label="Dismiss"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
{/if}
