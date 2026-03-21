<script lang="ts">
  /**
   * StateContainer — универсальная обёртка для автоматического отображения состояний
   * Поддерживает: loading, empty, error, success
   */
  import type { Snippet } from 'svelte';
  import LoadingState from './LoadingState.svelte';
  import EmptyState from './EmptyState.svelte';
  import ErrorState from './ErrorState.svelte';
  
  type Size = 'sm' | 'md' | 'lg';
  
  interface Props {
    /** Состояние загрузки */
    loading?: boolean;
    /** Текст ошибки или объект Error */
    error?: string | Error | null;
    /** Пустое состояние (нет данных) */
    empty?: boolean;
    
    // Loading props
    loadingVariant?: 'spinner' | 'skeleton' | 'dots';
    loadingText?: string;
    loadingLines?: number;
    
    // Empty props
    emptyIcon?: string;
    emptyTitle?: string;
    emptyDescription?: string;
    emptyActionText?: string;
    onemptyaction?: () => void;
    
    // Error props
    errorTitle?: string;
    onretry?: () => void | Promise<void>;
    retryText?: string;
    retrying?: boolean;
    
    // Common
    size?: Size;
    class?: string;
    
    // Slots
    /** Основной контент (success state) */
    children: Snippet;
    /** Кастомный loading */
    loadingSlot?: Snippet;
    /** Кастомный empty */
    emptySlot?: Snippet;
    /** Кастомный error */
    errorSlot?: Snippet;
  }
  
  let {
    loading = false,
    error = null,
    empty = false,
    
    loadingVariant = 'spinner',
    loadingText,
    loadingLines = 3,
    
    emptyIcon = '📭',
    emptyTitle = 'No data',
    emptyDescription,
    emptyActionText,
    onemptyaction,
    
    errorTitle = 'Something went wrong',
    onretry,
    retryText = 'Try again',
    retrying = false,
    
    size = 'md',
    class: className = '',
    
    children,
    loadingSlot,
    emptySlot,
    errorSlot
  }: Props = $props();
</script>

<div class={className}>
  {#if loading}
    <!-- Loading State -->
    {#if loadingSlot}
      {@render loadingSlot()}
    {:else}
      <LoadingState 
        variant={loadingVariant} 
        {size} 
        text={loadingText}
        lines={loadingLines}
      />
    {/if}
    
  {:else if error}
    <!-- Error State -->
    {#if errorSlot}
      {@render errorSlot()}
    {:else}
      <ErrorState 
        {error}
        title={errorTitle}
        {onretry}
        {retryText}
        {retrying}
        {size}
      />
    {/if}
    
  {:else if empty}
    <!-- Empty State -->
    {#if emptySlot}
      {@render emptySlot()}
    {:else}
      <EmptyState
        icon={emptyIcon}
        title={emptyTitle}
        description={emptyDescription}
        actionText={emptyActionText}
        onaction={onemptyaction}
        {size}
      />
    {/if}
    
  {:else}
    <!-- Success State (content) -->
    {@render children()}
  {/if}
</div>
