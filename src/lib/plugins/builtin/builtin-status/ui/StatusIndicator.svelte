<script lang="ts">
  /**
   * Status Indicator Widget (Builtin Plugin)
   * Shows current connection and protection status
   * Glass & Void design with Indigo-500 primary
   */
  import { browser } from '$app/environment';
  
  let status = $state<'protected' | 'unprotected' | 'loading'>('loading');
  let strategyName = $state<string | null>(null);
  
  const statusText = $derived(
    status === 'protected' ? 'Protected' :
    status === 'unprotected' ? 'Unprotected' :
    'Checking...'
  );
  
  const statusColor = $derived(
    status === 'protected' ? 'text-emerald-400' :
    status === 'unprotected' ? 'text-zinc-400' :
    'text-indigo-400'
  );
  
  const glowColor = $derived(
    status === 'protected' ? 'shadow-[0_0_20px_rgba(16,185,129,0.3)]' :
    status === 'unprotected' ? '' :
    'shadow-[0_0_20px_rgba(99,102,241,0.3)]'
  );
  
  const iconBg = $derived(
    status === 'protected' ? 'bg-emerald-500/10 border-emerald-500/30' :
    status === 'unprotected' ? 'bg-zinc-800/50 border-zinc-700/50' :
    'bg-indigo-500/10 border-indigo-500/30'
  );

  $effect(() => {
    if (!browser) return;
    
    let unsubscribe: (() => void) | undefined;
    let interval: ReturnType<typeof setInterval> | undefined;
    
    async function checkStatus() {
      const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
      if (!isTauri) {
        // Browser preview - show demo state
        status = 'unprotected';
        return;
      }
      
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const ready = await invoke<boolean>('is_backend_ready');
        if (!ready) {
          status = 'loading';
          return;
        }
        
        const result = await invoke<{
          is_active: boolean;
          current_strategy: string | null;
          current_strategy_name: string | null;
        }>('get_status');
        
        status = result.is_active ? 'protected' : 'unprotected';
        strategyName = result.current_strategy_name;
      } catch (e) {
        console.error('[StatusIndicator] Failed to get status:', e);
        status = 'unprotected';
      }
    }
    
    // Initial check
    checkStatus();
    
    // Poll every 5 seconds
    interval = setInterval(checkStatus, 5000);
    
    // Listen for strategy events
    (async () => {
      const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
      if (!isTauri) return;
      
      try {
        const { listen } = await import('@tauri-apps/api/event');
        
        const unlistenApplied = await listen('strategy:applied', (event) => {
          const payload = event.payload as { strategy_name?: string };
          status = 'protected';
          strategyName = payload.strategy_name ?? null;
        });
        
        const unlistenStopped = await listen('strategy:stopped', () => {
          status = 'unprotected';
          strategyName = null;
        });
        
        unsubscribe = () => {
          unlistenApplied();
          unlistenStopped();
        };
      } catch (e) {
        console.error('[StatusIndicator] Failed to setup listeners:', e);
      }
    })();
    
    return () => {
      if (interval) clearInterval(interval);
      if (unsubscribe) unsubscribe();
    };
  });
</script>

<div class="flex flex-col items-center justify-center p-4 h-full bg-void-50/50 rounded-xl border border-glass-border">
  <!-- Status Icon with Glow -->
  <div class="relative mb-3">
    {#if status === 'protected'}
      <div class="absolute inset-0 bg-emerald-500/20 rounded-full blur-xl animate-pulse"></div>
    {:else if status === 'loading'}
      <div class="absolute inset-0 bg-indigo-500/20 rounded-full blur-xl animate-pulse"></div>
    {/if}
    
    <div class="relative w-16 h-16 rounded-full {iconBg} border flex items-center justify-center {glowColor} transition-all duration-500">
      {#if status === 'protected'}
        <svg class="w-8 h-8 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
          <path d="M9 12l2 2 4-4"/>
        </svg>
      {:else if status === 'unprotected'}
        <svg class="w-8 h-8 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
          <path d="M12 8v4"/>
          <path d="M12 16h.01"/>
        </svg>
      {:else}
        <svg class="w-8 h-8 text-indigo-400 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" stroke-dasharray="32" stroke-dashoffset="8"/>
        </svg>
      {/if}
    </div>
  </div>
  
  <!-- Status Text -->
  <div class="text-center">
    <p class="text-sm font-medium {statusColor} transition-colors duration-300">
      {statusText}
    </p>
    {#if status === 'protected' && strategyName}
      <p class="text-xs text-zinc-400 mt-1 truncate max-w-[120px]">
        {strategyName}
      </p>
    {:else if status === 'unprotected'}
      <p class="text-xs text-zinc-400 mt-1">
        Click to enable
      </p>
    {/if}
  </div>
</div>
