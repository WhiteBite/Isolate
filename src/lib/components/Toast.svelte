<script lang="ts">
  import type { Toast } from '$lib/stores/toast';
  import { toasts } from '$lib/stores/toast';
  import { fly, fade } from 'svelte/transition';

  interface Props {
    toast: Toast;
  }

  let { toast }: Props = $props();

  const typeClasses = {
    success: 'border-l-green-500',
    error: 'border-l-red-500',
    warning: 'border-l-yellow-500',
    info: 'border-l-blue-500'
  };

  const iconPaths = {
    success: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
    error: 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z',
    warning: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z',
    info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
  };

  const iconColors = {
    success: 'text-green-500',
    error: 'text-red-500',
    warning: 'text-yellow-500',
    info: 'text-blue-500'
  };

  function handleClose() {
    toasts.dismiss(toast.id);
  }
</script>

<div
  class="bg-surface border border-white/10 rounded-lg border-l-4 {typeClasses[toast.type]} p-4 shadow-lg flex items-start gap-3 min-w-[300px] max-w-[400px]"
  in:fly={{ x: 100, duration: 300 }}
  out:fade={{ duration: 200 }}
  role="alert"
>
  <!-- Icon -->
  <svg
    class="w-5 h-5 flex-shrink-0 {iconColors[toast.type]}"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d={iconPaths[toast.type]}
    />
  </svg>

  <!-- Message -->
  <p class="text-sm text-white/90 flex-1">{toast.message}</p>

  <!-- Close button -->
  <button
    onclick={handleClose}
    class="text-white/40 hover:text-white/80 transition-colors flex-shrink-0"
    aria-label="Close notification"
  >
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
    </svg>
  </button>
</div>
