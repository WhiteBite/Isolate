<script lang="ts">
  /**
   * Health Monitor Widget (Builtin Plugin)
   * Shows system and connection health metrics
   * Glass & Void design with Indigo-500 primary
   */
  import { browser } from '$app/environment';
  import { mockHealthServices } from '$lib/mocks';
  
  interface ServiceHealth {
    id: string;
    name: string;
    status: 'healthy' | 'degraded' | 'down' | 'unknown';
    ping?: number;
  }
  
  let services = $state<ServiceHealth[]>([]);
  let isLoading = $state(true);
  let lastCheck = $state<Date | null>(null);
  
  const overallHealth = $derived.by(() => {
    if (isLoading || services.length === 0) return 'loading' as const;
    
    const healthyCount = services.filter(s => s.status === 'healthy').length;
    const downCount = services.filter(s => s.status === 'down').length;
    
    if (downCount > services.length / 2) return 'error' as const;
    if (healthyCount === services.length) return 'good' as const;
    return 'warning' as const;
  });
  
  const healthColor = $derived(
    overallHealth === 'good' ? 'text-emerald-400' :
    overallHealth === 'warning' ? 'text-amber-400' :
    overallHealth === 'error' ? 'text-red-400' :
    'text-indigo-400'
  );
  
  const avgPing = $derived.by(() => {
    const pings = services.filter(s => s.ping !== undefined).map(s => s.ping!);
    if (pings.length === 0) return null;
    return Math.round(pings.reduce((a, b) => a + b, 0) / pings.length);
  });

  $effect(() => {
    if (!browser) return;
    
    let interval: ReturnType<typeof setInterval> | undefined;
    
    async function checkHealth() {
      const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
      if (!isTauri) {
        // Browser preview - show demo data
        services = [...mockHealthServices];
        isLoading = false;
        lastCheck = new Date();
        return;
      }
      
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const ready = await invoke<boolean>('is_backend_ready');
        if (!ready) {
          return;
        }
        
        const results = await invoke<{
          service_id: string;
          service_name: string;
          accessible: boolean;
          avg_latency_ms: number | null;
          success_rate: number;
        }[]>('check_all_registry_services');
        
        services = results.map(r => ({
          id: r.service_id,
          name: r.service_name,
          status: r.accessible 
            ? (r.success_rate >= 0.8 ? 'healthy' : 'degraded')
            : 'down',
          ping: r.avg_latency_ms ?? undefined
        }));
        
        lastCheck = new Date();
      } catch (e) {
        console.error('[HealthMonitor] Failed to check health:', e);
      } finally {
        isLoading = false;
      }
    }
    
    // Initial check
    checkHealth();
    
    // Poll every 30 seconds
    interval = setInterval(checkHealth, 30000);
    
    return () => {
      if (interval) clearInterval(interval);
    };
  });
  
  function getStatusDot(status: ServiceHealth['status']): string {
    switch (status) {
      case 'healthy': return 'bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.6)]';
      case 'degraded': return 'bg-amber-500 shadow-[0_0_6px_rgba(245,158,11,0.5)] animate-pulse';
      case 'down': return 'bg-red-500 shadow-[0_0_6px_rgba(239,68,68,0.5)]';
      default: return 'bg-zinc-500';
    }
  }
  
  function getPingColor(ping: number | undefined): string {
    if (ping === undefined) return 'text-zinc-600';
    if (ping < 50) return 'text-emerald-400';
    if (ping < 100) return 'text-amber-400';
    return 'text-red-400';
  }
</script>

<div class="flex flex-col h-full p-3 bg-void-50/50 rounded-xl border border-glass-border">
  <!-- Header -->
  <div class="flex items-center justify-between mb-3">
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full {
        overallHealth === 'good' ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]' :
        overallHealth === 'warning' ? 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)]' :
        overallHealth === 'error' ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)]' :
        'bg-indigo-500 animate-pulse'
      }"></div>
      <span class="text-xs font-medium {healthColor}">
        {#if isLoading}
          Checking...
        {:else if overallHealth === 'good'}
          All Systems OK
        {:else if overallHealth === 'warning'}
          Some Issues
        {:else}
          Services Down
        {/if}
      </span>
    </div>
    
    {#if avgPing !== null}
      <span class="text-xs font-mono text-zinc-400">
        ~{avgPing}ms
      </span>
    {/if}
  </div>
  
  <!-- Services List -->
  <div class="flex-1 space-y-1.5 overflow-auto">
    {#if isLoading}
      {#each [1, 2, 3] as _}
        <div class="flex items-center justify-between py-2 px-2.5 rounded-lg bg-zinc-900/30 border border-white/5 animate-pulse">
          <div class="flex items-center gap-2">
            <div class="w-1.5 h-1.5 rounded-full bg-zinc-700"></div>
            <div class="w-16 h-3 bg-zinc-800 rounded"></div>
          </div>
          <div class="w-8 h-3 bg-zinc-800 rounded"></div>
        </div>
      {/each}
    {:else if services.length === 0}
      <div class="flex items-center justify-center h-full text-xs text-zinc-600">
        No services configured
      </div>
    {:else}
      {#each services.slice(0, 4) as service}
        <div class="flex items-center justify-between py-2 px-2.5 rounded-lg bg-zinc-900/30 border border-white/5 hover:bg-zinc-800/40 hover:border-white/10 transition-all duration-200">
          <div class="flex items-center gap-2">
            <span class="block w-1.5 h-1.5 rounded-full {getStatusDot(service.status)}"></span>
            <span class="text-xs text-zinc-300 font-medium truncate max-w-[80px]">
              {service.name}
            </span>
          </div>
          
          <span class="text-[10px] font-mono {getPingColor(service.ping)}">
            {service.ping !== undefined ? `${service.ping}ms` : '--'}
          </span>
        </div>
      {/each}
      
      {#if services.length > 4}
        <div class="text-center text-[10px] text-zinc-600 pt-1">
          +{services.length - 4} more
        </div>
      {/if}
    {/if}
  </div>
  
  <!-- Footer -->
  {#if lastCheck}
    <div class="mt-2 pt-2 border-t border-glass-border">
      <span class="text-[10px] text-zinc-600">
        Last check: {lastCheck.toLocaleTimeString()}
      </span>
    </div>
  {/if}
</div>
