<script lang="ts">
  interface ServiceHealth {
    name: string;
    status: 'healthy' | 'degraded' | 'down';
    ping?: number;
    sparkline?: number[];
  }

  interface Props {
    services?: ServiceHealth[];
  }

  let { 
    services = [
      { name: 'YouTube', status: 'healthy', ping: 45 },
      { name: 'Discord', status: 'healthy', ping: 32 },
      { name: 'Telegram', status: 'degraded', ping: 120 },
      { name: 'Twitter/X', status: 'down', ping: undefined }
    ]
  }: Props = $props();

  const statusConfig = {
    healthy: { 
      label: 'OK'
    },
    degraded: { 
      label: 'Slow'
    },
    down: { 
      label: 'Down'
    }
  };

  function getPingColor(ping: number | undefined): string {
    if (ping === undefined) return 'text-text-muted';
    if (ping < 50) return 'text-neon-green';
    if (ping < 100) return 'text-neon-yellow';
    return 'text-neon-red';
  }
</script>

<div class="flex flex-col gap-2 h-full">
  {#each services as service}
    <div 
      class="flex items-center justify-between py-2.5 px-3 rounded-lg 
             bg-zinc-900/30 border border-white/5
             hover:bg-zinc-800/40 hover:border-white/10
             transition-all duration-200 group"
    >
      <div class="flex items-center gap-3">
        <!-- Status indicator with glow -->
        <span 
          class="block w-2 h-2 rounded-full
                 {service.status === 'healthy' ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]' : ''}
                 {service.status === 'degraded' ? 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)] animate-pulse' : ''}
                 {service.status === 'down' ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)]' : ''}"
        ></span>
        
        <!-- Service name -->
        <span class="text-sm text-zinc-200 font-medium">
          {service.name}
        </span>
      </div>

      <div class="flex items-center gap-3">
        <!-- Status text - subtle -->
        <span class="text-[10px] uppercase tracking-wider font-medium
                     {service.status === 'healthy' ? 'text-zinc-500' : ''}
                     {service.status === 'degraded' ? 'text-zinc-500' : ''}
                     {service.status === 'down' ? 'text-zinc-500' : ''}">
          {statusConfig[service.status].label}
        </span>

        <!-- Ping value -->
        {#if service.ping !== undefined}
          <span class="text-xs font-mono text-zinc-500 min-w-[45px] text-right">
            {service.ping}ms
          </span>
        {:else}
          <span class="text-xs font-mono text-zinc-600 min-w-[45px] text-right">
            --
          </span>
        {/if}
      </div>
    </div>
  {/each}
</div>
