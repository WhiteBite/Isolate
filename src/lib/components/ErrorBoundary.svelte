<script lang="ts">
  /**
   * Error Boundary Component
   * 
   * Catches and displays errors in a user-friendly way.
   * Can be used to wrap sections of the UI that might fail.
   * 
   * @example
   * ```svelte
   * <ErrorBoundary>
   *   <RiskyComponent />
   * </ErrorBoundary>
   * ```
   */
  
  import { browser } from '$app/environment';
  
  interface Props {
    /** Error to display (if controlled externally) */
    error?: Error | string | null;
    /** Title for the error card */
    title?: string;
    /** Whether to show stack trace */
    showStack?: boolean;
    /** Custom retry handler */
    onRetry?: () => void;
    /** Children to render when no error */
    children?: import('svelte').Snippet;
  }
  
  let { 
    error = null, 
    title = 'Something went wrong',
    showStack = false,
    onRetry,
    children 
  }: Props = $props();
  
  let internalError = $state<Error | null>(null);
  
  // Combine external and internal errors
  let displayError = $derived(error || internalError);
  
  function getErrorMessage(err: Error | string | null): string {
    if (!err) return 'Unknown error';
    if (typeof err === 'string') return err;
    return err.message || 'Unknown error';
  }
  
  function getErrorStack(err: Error | string | null): string | null {
    if (!err || typeof err === 'string') return null;
    return err.stack || null;
  }
  
  function handleRetry() {
    internalError = null;
    if (onRetry) {
      onRetry();
    } else if (browser) {
      window.location.reload();
    }
  }
  
  function handleDismiss() {
    internalError = null;
  }
</script>

{#if displayError}
  <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-6">
    <div class="flex items-start gap-4">
      <!-- Error Icon -->
      <div class="flex-shrink-0 w-10 h-10 rounded-full bg-red-500/20 flex items-center justify-center">
        <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width="2" 
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
          />
        </svg>
      </div>
      
      <!-- Error Content -->
      <div class="flex-1 min-w-0">
        <h3 class="text-lg font-semibold text-red-400 mb-1">{title}</h3>
        <p class="text-zinc-400 text-sm break-words">{getErrorMessage(displayError)}</p>
        
        {#if showStack && getErrorStack(displayError)}
          <details class="mt-4">
            <summary class="text-zinc-400 text-xs cursor-pointer hover:text-zinc-300 transition-colors">
              Show stack trace
            </summary>
            <pre class="mt-2 p-3 bg-zinc-900/50 rounded-lg text-xs text-zinc-400 overflow-auto max-h-48 whitespace-pre-wrap">
              {getErrorStack(displayError)}
            </pre>
          </details>
        {/if}
        
        <!-- Actions -->
        <div class="flex gap-2 mt-4">
          <button
            onclick={handleRetry}
            class="px-4 py-2 text-sm font-medium text-white bg-red-500/20 hover:bg-red-500/30 
                   border border-red-500/30 rounded-lg transition-colors"
          >
            {onRetry ? 'Retry' : 'Reload'}
          </button>
          <button
            onclick={handleDismiss}
            class="px-4 py-2 text-sm font-medium text-zinc-400 hover:text-zinc-300 
                   bg-zinc-800/50 hover:bg-zinc-800 border border-white/5 rounded-lg transition-colors"
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>
  </div>
{:else if children}
  {@render children()}
{/if}
