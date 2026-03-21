<script lang="ts">
  /**
   * ErrorState — компонент для отображения состояния ошибки
   * Поддерживает retry и кастомные действия
   */
  import type { Snippet } from 'svelte';
  
  type Size = 'sm' | 'md' | 'lg';
  type Variant = 'inline' | 'card' | 'fullscreen';
  
  interface Props {
    error: string | Error | null;
    title?: string;
    onretry?: () => void | Promise<void>;
    retryText?: string;
    size?: Size;
    variant?: Variant;
    /** Показывать ли иконку */
    showIcon?: boolean;
    /** Состояние повторной попытки */
    retrying?: boolean;
    /** Кастомные действия */
    actions?: Snippet;
    class?: string;
  }
  
  let { 
    error,
    title = 'Something went wrong',
    onretry,
    retryText = 'Try again',
    size = 'md',
    variant = 'card',
    showIcon = true,
    retrying = false,
    actions,
    class: className = ''
  }: Props = $props();
  
  let errorMessage = $derived(
    error instanceof Error ? error.message : (error || 'Unknown error')
  );
  
  const sizeClasses: Record<Size, { icon: string; title: string; desc: string; btn: string; iconBox: string }> = {
    sm: { icon: 'w-5 h-5', title: 'text-sm', desc: 'text-xs', btn: 'px-3 py-1.5 text-xs', iconBox: 'w-10 h-10' },
    md: { icon: 'w-6 h-6', title: 'text-base', desc: 'text-sm', btn: 'px-4 py-2 text-sm', iconBox: 'w-12 h-12' },
    lg: { icon: 'w-7 h-7', title: 'text-lg', desc: 'text-base', btn: 'px-5 py-2.5 text-base', iconBox: 'w-14 h-14' }
  };
  
  const variantClasses: Record<Variant, string> = {
    inline: 'py-4',
    card: 'p-6 bg-red-500/10 border border-red-500/30 rounded-2xl',
    fullscreen: 'h-full p-8'
  };
  
  async function handleRetry() {
    if (onretry) {
      await onretry();
    }
  }
</script>

<div class="flex flex-col items-center justify-center text-center {variantClasses[variant]} {className}">
  <!-- Icon -->
  {#if showIcon}
    <div class="{sizeClasses[size].iconBox} mx-auto mb-4 rounded-full bg-red-500/20 border border-red-500/30 flex items-center justify-center">
      <svg class="{sizeClasses[size].icon} text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
    </div>
  {/if}
  
  <!-- Title -->
  <h3 class="{sizeClasses[size].title} font-semibold text-zinc-100 mb-2">
    {title}
  </h3>
  
  <!-- Error message -->
  <p class="{sizeClasses[size].desc} text-red-300 mb-5 max-w-md">
    {errorMessage}
  </p>
  
  <!-- Actions -->
  {#if actions}
    {@render actions()}
  {:else if onretry}
    <button
      onclick={handleRetry}
      disabled={retrying}
      class="{sizeClasses[size].btn} inline-flex items-center gap-2 bg-red-500/20 border border-red-500/30 rounded-xl
             text-red-400 font-medium hover:bg-red-500/30 hover:border-red-500/40
             disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200"
    >
      <svg class="w-4 h-4 {retrying ? 'animate-spin' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
        <path d="M3 3v5h5"/>
        <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
        <path d="M16 16h5v5"/>
      </svg>
      {retrying ? 'Retrying...' : retryText}
    </button>
  {/if}
</div>
