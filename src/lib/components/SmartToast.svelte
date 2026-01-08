<script lang="ts">
  import { humanizeError, getErrorSuggestion } from '$lib/utils/errorMessages';
  import { fly, fade } from 'svelte/transition';

  interface ToastAction {
    label: string;
    onclick: () => void;
  }

  interface Props {
    type?: 'success' | 'error' | 'warning' | 'info' | 'progress';
    message: string;
    rawError?: string;
    progress?: number;
    actions?: ToastAction[];
    duration?: number;
    onClose?: () => void;
  }

  let { 
    type = 'info', 
    message, 
    rawError, 
    progress = 0, 
    actions = [], 
    duration = 5000,
    onClose 
  }: Props = $props();

  const humanMessage = $derived(rawError ? humanizeError(rawError) : message);
  const suggestion = $derived(rawError ? getErrorSuggestion(rawError) : null);

  let showDetails = $state(false);
  let visible = $state(true);

  const icons: Record<string, string> = {
    success: '✓',
    error: '✕',
    warning: '⚠',
    info: 'ℹ',
    progress: '◐'
  };

  const colors: Record<string, { bg: string; border: string; text: string; icon: string }> = {
    success: {
      bg: 'bg-emerald-500/10',
      border: 'border-emerald-500/30',
      text: 'text-emerald-300',
      icon: 'text-emerald-400'
    },
    error: {
      bg: 'bg-red-500/10',
      border: 'border-red-500/30',
      text: 'text-red-300',
      icon: 'text-red-400'
    },
    warning: {
      bg: 'bg-amber-500/10',
      border: 'border-amber-500/30',
      text: 'text-amber-300',
      icon: 'text-amber-400'
    },
    info: {
      bg: 'bg-blue-500/10',
      border: 'border-blue-500/30',
      text: 'text-blue-300',
      icon: 'text-blue-400'
    },
    progress: {
      bg: 'bg-violet-500/10',
      border: 'border-violet-500/30',
      text: 'text-violet-300',
      icon: 'text-violet-400'
    }
  };

  const style = $derived(colors[type]);

  function handleClose() {
    visible = false;
    setTimeout(() => onClose?.(), 200);
  }

  // Auto-close для не-progress тостов
  $effect(() => {
    if (type !== 'progress' && duration > 0) {
      const timer = setTimeout(handleClose, duration);
      return () => clearTimeout(timer);
    }
  });
</script>

{#if visible}
  <div
    class="relative max-w-sm w-full {style.bg} {style.border} border rounded-xl p-4 shadow-xl backdrop-blur-sm"
    role="alert"
    aria-live="polite"
    in:fly={{ y: -20, duration: 200 }}
    out:fade={{ duration: 150 }}
  >
    <!-- Header -->
    <div class="flex items-start gap-3">
      <!-- Icon -->
      <div class="flex-shrink-0 w-6 h-6 flex items-center justify-center {style.icon} text-lg"
           class:animate-spin={type === 'progress'}>
        {icons[type]}
      </div>

      <!-- Content -->
      <div class="flex-1 min-w-0">
        <p class="{style.text} text-sm font-medium">{humanMessage}</p>
        
        {#if suggestion}
          <p class="mt-1 text-xs text-white/50">{suggestion}</p>
        {/if}

        {#if rawError && rawError !== humanMessage}
          <button
            type="button"
            class="mt-1 text-xs text-white/40 hover:text-white/60 transition-colors"
            onclick={() => showDetails = !showDetails}
          >
            {showDetails ? 'Скрыть детали' : 'Показать детали'}
          </button>
          
          {#if showDetails}
            <pre class="mt-2 p-2 bg-black/30 rounded text-xs text-white/50 overflow-x-auto max-h-24 overflow-y-auto">{rawError}</pre>
          {/if}
        {/if}

        <!-- Progress bar -->
        {#if type === 'progress'}
          <div class="mt-3 h-1.5 bg-white/10 rounded-full overflow-hidden">
            <div 
              class="h-full bg-violet-500 rounded-full transition-all duration-300 ease-out"
              style="width: {progress}%"
            ></div>
          </div>
          <p class="mt-1 text-xs text-white/40 text-right">{progress}%</p>
        {/if}

        <!-- Actions -->
        {#if actions.length > 0}
          <div class="mt-3 flex flex-wrap gap-2">
            {#each actions as action}
              <button
                type="button"
                class="px-3 py-1.5 text-xs font-medium rounded-lg bg-white/10 hover:bg-white/20 text-white/80 hover:text-white transition-colors"
                onclick={action.onclick}
              >
                {action.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Close button -->
      <button
        type="button"
        class="flex-shrink-0 w-6 h-6 flex items-center justify-center text-white/40 hover:text-white/70 transition-colors rounded-lg hover:bg-white/10"
        onclick={handleClose}
        aria-label="Закрыть"
      >
        ✕
      </button>
    </div>

    <!-- Auto-close progress indicator -->
    {#if type !== 'progress' && duration > 0}
      <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-white/5 rounded-b-xl overflow-hidden">
        <div 
          class="h-full {style.icon.replace('text-', 'bg-')} opacity-50"
          style="animation: shrink {duration}ms linear forwards"
        ></div>
      </div>
    {/if}
  </div>
{/if}

<style>
  @keyframes shrink {
    from { width: 100%; }
    to { width: 0%; }
  }
</style>
