<script lang="ts">
  import { getServiceIconSvg } from '$lib/utils/icons';
  import { PingChart } from '$lib/components';
  import { ServiceLogs } from '$lib/components';
  import { ServiceHealthChart } from '$lib/components/services';
  import { logs, type LogEntry } from '$lib/stores/logs';
  import { get } from 'svelte/store';

  interface ServiceWithStatus {
    id: string;
    name: string;
    status: 'working' | 'blocked' | 'unknown' | 'checking' | 'error';
    ping?: number;
    category: string;
    isCustom?: boolean;
    error?: string | null;
  }

  interface Props {
    service: ServiceWithStatus | null;
    pingHistory: number[];
    oncheck: (id: string) => void;
    ondelete: (service: ServiceWithStatus) => void;
    onconfigure: (service: ServiceWithStatus) => void;
  }

  let { service, pingHistory, oncheck, ondelete, onconfigure }: Props = $props();

  // Detail panel tabs
  let activeTab: 'overview' | 'logs' = $state('overview');

  // Reset tab when service changes
  $effect(() => {
    if (service) {
      activeTab = 'overview';
    }
  });

  function getIcon(id: string) {
    return getServiceIconSvg(id);
  }

  function getStatusColor(status: string): string {
    if (status === 'working') return 'bg-emerald-500';
    if (status === 'blocked') return 'bg-red-500';
    if (status === 'error') return 'bg-rose-600';
    if (status === 'checking') return 'bg-indigo-500';
    return 'bg-amber-500';
  }

  function getStatusBadge(status: string): string {
    if (status === 'working') return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
    if (status === 'blocked') return 'bg-red-500/10 text-red-400 border-red-500/20';
    if (status === 'error') return 'bg-rose-500/10 text-rose-400 border-rose-500/20';
    if (status === 'checking') return 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20';
    return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
  }

  function getStatusText(status: string): string {
    if (status === 'working') return 'Working';
    if (status === 'blocked') return 'Blocked';
    if (status === 'error') return 'Error';
    if (status === 'checking') return 'Checking...';
    return 'Unknown';
  }

  function getPingColor(ping?: number): string {
    if (!ping) return 'text-zinc-400';
    if (ping < 50) return 'text-emerald-400';
    if (ping < 150) return 'text-amber-400';
    return 'text-red-400';
  }

  function getPingChartColor(history: number[]): string {
    if (history.length === 0) return '#22C55E';
    const avg = history.reduce((a, b) => a + b, 0) / history.length;
    if (avg < 50) return '#22C55E';
    if (avg < 150) return '#F59E0B';
    return '#EF4444';
  }

  // Get logs count for selected service
  let selectedLogsCount = $derived.by(() => {
    if (!service) return 0;
    const allLogs = get(logs);
    return allLogs.filter(log => log.source === service.id).length;
  });

  // Get recent logs for selected service (last 10)
  let recentLogs = $derived.by(() => {
    if (!service) return [];
    const allLogs = get(logs);
    return allLogs
      .filter(log => log.source === service.id)
      .slice(-10);
  });
</script>

<div class="flex-1 overflow-y-auto bg-zinc-950">
  {#if service}
    {@const icon = getIcon(service.id)}
    <div class="p-6 max-w-3xl">
      <!-- Header -->
      <div class="flex items-start gap-5 mb-6">
        <div class="relative">
          <div class="w-20 h-20 rounded-2xl bg-zinc-900/60 border border-white/5 flex items-center justify-center">
            <svg class="w-10 h-10 {icon.color}" viewBox="0 0 24 24" fill="currentColor">
              <path d={icon.path}/>
            </svg>
          </div>
          {#if service.status === 'checking'}
            <div class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full bg-indigo-500 border-2 border-zinc-950 flex items-center justify-center">
              <svg class="w-3 h-3 animate-spin text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
              </svg>
            </div>
          {:else if service.status === 'error'}
            <div class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full bg-rose-500 border-2 border-zinc-950 flex items-center justify-center">
              <svg class="w-3 h-3 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </div>
          {:else}
            <div class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full {getStatusColor(service.status)} 
                        border-2 border-zinc-950 {service.status === 'working' ? 'animate-pulse' : ''}"></div>
          {/if}
        </div>
        <div class="flex-1">
          <h1 class="text-2xl font-bold text-zinc-100 mb-2">{service.name}</h1>
          <div class="flex items-center gap-3">
            <span class="px-3 py-1 rounded-lg text-xs font-medium border {getStatusBadge(service.status)}">
              {getStatusText(service.status)}
            </span>
            <span class="text-sm text-zinc-400 capitalize">{service.category}</span>
          </div>
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex gap-1 mb-6 p-1 bg-zinc-900/40 rounded-xl border border-white/5">
        <button
          onclick={() => activeTab = 'overview'}
          class="flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all
                 {activeTab === 'overview' 
                   ? 'bg-zinc-800 text-zinc-100 shadow-sm' 
                   : 'text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800/50'}"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7"/>
            <rect x="14" y="3" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/>
            <rect x="3" y="14" width="7" height="7"/>
          </svg>
          Overview
        </button>
        <button
          onclick={() => activeTab = 'logs'}
          class="flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all
                 {activeTab === 'logs' 
                   ? 'bg-zinc-800 text-zinc-100 shadow-sm' 
                   : 'text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800/50'}"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
          </svg>
          Logs
          {#if selectedLogsCount > 0}
            <span class="px-1.5 py-0.5 text-[10px] rounded-full bg-zinc-700 text-zinc-400">
              {selectedLogsCount}
            </span>
          {/if}
        </button>
      </div>

      {#if activeTab === 'overview'}
        <!-- Stats Grid -->
        <div class="grid grid-cols-3 gap-4 mb-6">
          <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl hover:border-white/10 transition-colors">
            <div class="text-xs text-zinc-400 uppercase tracking-wider mb-2">Latency</div>
            <div class="flex items-baseline gap-1">
              {#if service.ping}
                <span class="text-2xl font-bold {getPingColor(service.ping)}">{service.ping}</span>
                <span class="text-sm text-zinc-400">ms</span>
              {:else}
                <span class="text-2xl font-bold text-zinc-400">—</span>
              {/if}
            </div>
          </div>

          <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl hover:border-white/10 transition-colors">
            <div class="text-xs text-zinc-400 uppercase tracking-wider mb-2">Status</div>
            <div class="text-xl font-semibold {service.status === 'working' ? 'text-emerald-400' : service.status === 'blocked' ? 'text-red-400' : service.status === 'error' ? 'text-rose-400' : service.status === 'checking' ? 'text-indigo-400' : 'text-amber-400'}">
              {getStatusText(service.status)}
            </div>
            {#if service.status === 'error' && service.error}
              <div class="mt-2 text-xs text-rose-400/80 truncate" title={service.error}>
                {service.error}
              </div>
            {/if}
          </div>

          <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl hover:border-white/10 transition-colors">
            <div class="text-xs text-zinc-400 uppercase tracking-wider mb-2">Category</div>
            <div class="text-xl font-semibold text-zinc-100 capitalize">{service.category}</div>
          </div>
        </div>

        <!-- Ping History Chart -->
        <div class="p-5 bg-zinc-900/30 border border-white/5 rounded-xl mb-6">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-medium text-zinc-100">Latency History</h3>
            {#if pingHistory.length > 0}
              <span class="text-xs text-zinc-400">
                {pingHistory.length} measurements
              </span>
            {/if}
          </div>
          <PingChart 
            data={pingHistory} 
            maxPoints={30} 
            height={80}
            color={getPingChartColor(pingHistory)}
          />
          {#if pingHistory.length === 0}
            <p class="text-xs text-zinc-400 text-center mt-2">
              No data yet. Click "Re-check" to start collecting latency data.
            </p>
          {:else}
            <div class="flex justify-between mt-3 text-xs text-zinc-400">
              <span>Min: {Math.min(...pingHistory)}ms</span>
              <span>Avg: {Math.round(pingHistory.reduce((a, b) => a + b, 0) / pingHistory.length)}ms</span>
              <span>Max: {Math.max(...pingHistory)}ms</span>
            </div>
          {/if}
        </div>

        <!-- Service Health History -->
        <div class="p-5 bg-zinc-900/30 border border-white/5 rounded-xl mb-6">
          <ServiceHealthChart serviceId={service.id} hours={24} showStats={true} />
        </div>

        <!-- Connection Details -->
        <div class="p-5 bg-zinc-900/30 border border-white/5 rounded-xl mb-6">
          <h3 class="text-sm font-medium text-zinc-100 mb-4">Connection Details</h3>
          <div class="space-y-3">
            <div class="flex items-center justify-between text-sm">
              <span class="text-zinc-400">DNS Resolution</span>
              <span class="text-zinc-300">{service.status === 'blocked' ? 'Failed' : service.status === 'unknown' ? '—' : 'OK'}</span>
            </div>
            <div class="flex items-center justify-between text-sm">
              <span class="text-zinc-400">TCP Connection</span>
              <span class="text-zinc-300">{service.status === 'blocked' ? 'Blocked' : service.status === 'unknown' ? '—' : 'Established'}</span>
            </div>
            <div class="flex items-center justify-between text-sm">
              <span class="text-zinc-400">TLS Handshake</span>
              <span class="text-zinc-300">{service.status === 'blocked' ? 'Failed' : service.status === 'unknown' ? '—' : 'Success'}</span>
            </div>
          </div>
        </div>

        <!-- Recent Logs -->
        <div class="p-5 bg-zinc-900/30 border border-white/5 rounded-xl mb-6">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-sm font-medium text-zinc-100 flex items-center gap-2">
              <svg class="w-4 h-4 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
              </svg>
              Recent Logs
              {#if selectedLogsCount > 0}
                <span class="px-1.5 py-0.5 text-[10px] rounded-full bg-zinc-800 text-zinc-400">
                  {selectedLogsCount}
                </span>
              {/if}
            </h3>
            <a
              href="/logs?source={service.id}"
              class="flex items-center gap-1 text-xs text-indigo-400 hover:text-indigo-300 transition-colors"
            >
              View All
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M5 12h14M12 5l7 7-7 7"/>
              </svg>
            </a>
          </div>
          
          {#if recentLogs.length === 0}
            <div class="flex items-center justify-center py-6 text-zinc-400">
              <div class="text-center">
                <svg class="w-6 h-6 mx-auto mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                        d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <p class="text-xs">No logs yet</p>
                <p class="text-[10px] text-zinc-700 mt-1">Check the service to generate logs</p>
              </div>
            </div>
          {:else}
            <div class="space-y-1 font-mono text-[11px]">
              {#each recentLogs as log (log.id)}
                <div class="flex items-start gap-2 px-2 py-1.5 rounded-lg bg-zinc-900/50 hover:bg-zinc-800/50 transition-colors">
                  <span class="text-zinc-400 shrink-0">
                    {log.timestamp.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                  </span>
                  <span class="shrink-0 px-1 py-0.5 text-[10px] uppercase font-semibold rounded
                              {log.level === 'error' ? 'bg-red-500/10 text-red-400' : 
                               log.level === 'warn' ? 'bg-amber-500/10 text-amber-400' : 
                               log.level === 'success' ? 'bg-emerald-500/10 text-emerald-400' : 
                               log.level === 'debug' ? 'bg-zinc-800/50 text-zinc-400' : 
                               'bg-zinc-500/10 text-zinc-300'}">
                    {log.level.slice(0, 3)}
                  </span>
                  <span class="break-all flex-1
                              {log.level === 'error' ? 'text-red-400' : 
                               log.level === 'warn' ? 'text-amber-400' : 
                               log.level === 'success' ? 'text-emerald-400' : 
                               log.level === 'debug' ? 'text-zinc-400' : 
                               'text-zinc-300'}">
                    {log.message}
                  </span>
                </div>
              {/each}
            </div>
            {#if selectedLogsCount > 10}
              <p class="text-[10px] text-zinc-400 text-center mt-3">
                Showing last 10 of {selectedLogsCount} logs
              </p>
            {/if}
          {/if}
        </div>

        <!-- Actions -->
        <div class="flex gap-3">
          <button
            onclick={() => oncheck(service.id)}
            class="flex-1 flex items-center justify-center gap-2 px-4 py-3 
                   bg-indigo-500 hover:bg-indigo-600 rounded-xl
                   text-white font-medium text-sm
                   transition-all duration-200 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-indigo-500/20"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
              <path d="M3 3v5h5"/>
            </svg>
            Re-check
          </button>

          {#if service.isCustom}
            <button
              onclick={() => ondelete(service)}
              class="flex items-center justify-center gap-2 px-4 py-3 
                     bg-red-500/10 border border-red-500/20 rounded-xl
                     text-red-400 font-medium text-sm
                     hover:bg-red-500/20 hover:border-red-500/30
                     transition-all duration-200 hover:-translate-y-0.5"
            >
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
              Remove
            </button>
          {:else}
            <button
              onclick={() => onconfigure(service)}
              class="flex-1 flex items-center justify-center gap-2 px-4 py-3 
                     bg-zinc-900/60 border border-white/5 rounded-xl
                     text-zinc-100 font-medium text-sm
                     hover:bg-zinc-800/60 hover:border-white/10
                     transition-all duration-200 hover:-translate-y-0.5"
            >
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
              </svg>
              Configure
            </button>
          {/if}
        </div>
      {:else}
        <!-- Logs Tab -->
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <p class="text-sm text-zinc-400">
              Activity logs for <span class="text-zinc-200 font-medium">{service.name}</span>
            </p>
            <button
              onclick={() => oncheck(service.id)}
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium
                     bg-indigo-500/10 border border-indigo-500/20 rounded-lg
                     text-indigo-400 hover:bg-indigo-500/20 transition-colors"
            >
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                <path d="M3 3v5h5"/>
              </svg>
              Check Now
            </button>
          </div>
          
          <ServiceLogs source={service.id} maxHeight={400} />
          
          <p class="text-xs text-zinc-400 text-center">
            Logs are filtered by service ID. Check the service to generate new log entries.
          </p>
        </div>
      {/if}
    </div>
  {:else}
    <!-- Empty State -->
    <div class="h-full flex flex-col items-center justify-center text-center p-6">
      <div class="w-20 h-20 rounded-2xl bg-zinc-900/40 border border-white/5 flex items-center justify-center mb-4">
        <svg class="w-10 h-10 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
          <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
          <line x1="6" y1="6" x2="6.01" y2="6"/>
          <line x1="6" y1="18" x2="6.01" y2="18"/>
        </svg>
      </div>
      <h2 class="text-lg font-medium text-zinc-100 mb-2">Select a service</h2>
      <p class="text-sm text-zinc-400 max-w-xs">
        Choose a service from the list to view its status and configuration.
      </p>
    </div>
  {/if}
</div>
