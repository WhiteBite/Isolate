<script lang="ts">
  /**
   * Global Error Boundary
   * 
   * Catches unhandled errors at the route level and displays
   * a user-friendly error page with recovery options.
   */
  import { page } from '$app/stores';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  
  // Type assertion for error with stack
  interface ErrorWithStack extends Error {
    stack?: string;
  }
  
  let showStack = $state(false);
  
  function getErrorMessage(): string {
    return $page.error?.message || 'An unexpected error occurred';
  }
  
  function getErrorStack(): string | null {
    const error = $page.error as ErrorWithStack | null;
    return error?.stack || null;
  }
  
  function handleGoHome() {
    goto('/');
  }
  
  function handleReload() {
    if (browser) {
      window.location.reload();
    }
  }
  
  function handleGoBack() {
    if (browser) {
      window.history.back();
    }
  }
  
  async function handleReportError() {
    // Copy error details to clipboard for reporting
    if (browser) {
      const errorDetails = `
Error: ${getErrorMessage()}
Status: ${$page.status}
URL: ${$page.url.pathname}
Stack: ${getErrorStack() || 'N/A'}
      `.trim();
      
      try {
        await navigator.clipboard.writeText(errorDetails);
        // Could show a toast here
      } catch {
        console.error('Failed to copy error details');
      }
    }
  }
</script>

<div class="min-h-screen bg-zinc-950 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-red-900/10 via-zinc-950 to-zinc-950 flex items-center justify-center p-8">
  <div class="bg-zinc-900/80 backdrop-blur-xl rounded-2xl p-8 border border-red-500/20 max-w-2xl w-full shadow-2xl shadow-red-500/5">
    <!-- Header -->
    <div class="flex items-center gap-4 mb-6">
      <div class="w-14 h-14 rounded-2xl bg-red-500/10 border border-red-500/20 flex items-center justify-center">
        <svg class="w-7 h-7 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
        </svg>
      </div>
      <div>
        <h1 class="text-2xl font-bold text-white">
          {#if $page.status === 404}
            Page Not Found
          {:else if $page.status === 500}
            Server Error
          {:else}
            Error {$page.status}
          {/if}
        </h1>
        <p class="text-zinc-400 text-sm mt-0.5">
          {#if $page.status === 404}
            The page you're looking for doesn't exist
          {:else}
            Something went wrong while processing your request
          {/if}
        </p>
      </div>
    </div>
    
    <!-- Error Details -->
    <div class="bg-zinc-950/50 rounded-xl p-4 mb-6 border border-white/5">
      <div class="flex items-start gap-3">
        <div class="w-5 h-5 rounded-full bg-red-500/20 flex items-center justify-center flex-shrink-0 mt-0.5">
          <svg class="w-3 h-3 text-red-400" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
          </svg>
        </div>
        <div class="flex-1 min-w-0">
          <p class="text-red-300 text-sm font-mono break-words">{getErrorMessage()}</p>
          
          {#if getErrorStack()}
            <button
              onclick={() => showStack = !showStack}
              class="mt-3 text-xs text-zinc-400 hover:text-zinc-300 transition-colors flex items-center gap-1"
            >
              <svg 
                class="w-3 h-3 transition-transform {showStack ? 'rotate-90' : ''}" 
                fill="none" 
                stroke="currentColor" 
                viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
              </svg>
              {showStack ? 'Hide' : 'Show'} stack trace
            </button>
            
            {#if showStack}
              <pre class="mt-2 p-3 bg-zinc-900/50 rounded-lg text-xs text-zinc-400 overflow-auto max-h-48 whitespace-pre-wrap border border-white/5">{getErrorStack()}</pre>
            {/if}
          {/if}
        </div>
      </div>
    </div>
    
    <!-- Actions -->
    <div class="flex flex-wrap gap-3">
      <button 
        onclick={handleGoHome}
        class="px-5 py-2.5 bg-indigo-500 hover:bg-indigo-400 text-white rounded-xl font-medium transition-all
               shadow-lg shadow-indigo-500/20 hover:shadow-indigo-500/30"
      >
        Go to Dashboard
      </button>
      <button 
        onclick={handleGoBack}
        class="px-5 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-white rounded-xl font-medium transition-colors
               border border-white/5"
      >
        Go Back
      </button>
      <button 
        onclick={handleReload}
        class="px-5 py-2.5 bg-zinc-800/50 hover:bg-zinc-800 text-zinc-300 rounded-xl font-medium transition-colors
               border border-white/5"
      >
        Reload Page
      </button>
      <button 
        onclick={handleReportError}
        class="px-5 py-2.5 text-zinc-400 hover:text-zinc-300 rounded-xl font-medium transition-colors
               flex items-center gap-2"
        title="Copy error details to clipboard"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
        </svg>
        Copy Error
      </button>
    </div>
    
    <!-- Help Text -->
    <p class="mt-6 text-xs text-zinc-400 text-center">
      If this problem persists, try restarting the application or check the logs for more details.
    </p>
  </div>
</div>
