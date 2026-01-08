<script lang="ts">
  import { browser } from '$app/environment';
  import { page } from '$app/stores';
  import Spinner from '$lib/components/Spinner.svelte';
  import { logs, filteredLogs, logFilters, logSources, type LogEntry, type LogLevel } from '$lib/stores/logs';
  import { t } from '$lib/i18n';

  // State
  let levelFilter = $state<LogLevel | 'all'>('all');
  let sourceFilter = $state<string>('all');
  let searchQuery = $state('');
  let autoScroll = $state(true);
  let exporting = $state(false);
  let clearing = $state(false);

  // Local copies of store values
  let logsValue = $state<LogEntry[]>([]);
  let filteredLogsValue = $state<LogEntry[]>([]);
  let sourcesValue = $state<string[]>([]);

  // Virtual scrolling state
  const ITEM_HEIGHT = 32; // Height of each log row in pixels
  const BUFFER_SIZE = 10; // Extra items to render above/below viewport
  let containerRef: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let containerHeight = $state(600);

  // Calculate visible range for virtual scrolling
  let virtualScrollData = $derived(() => {
    const totalItems = filteredLogsValue.length;
    const totalHeight = totalItems * ITEM_HEIGHT;
    
    const startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - BUFFER_SIZE);
    const visibleCount = Math.ceil(containerHeight / ITEM_HEIGHT) + BUFFER_SIZE * 2;
    const endIndex = Math.min(totalItems, startIndex + visibleCount);
    
    const visibleItems = filteredLogsValue.slice(startIndex, endIndex);
    const offsetY = startIndex * ITEM_HEIGHT;
    
    return {
      totalHeight,
      visibleItems,
      offsetY,
      startIndex,
      endIndex,
      totalItems
    };
  });

  // Use virtualization only when we have many items
  const VIRTUALIZATION_THRESHOLD = 200;
  let useVirtualization = $derived(filteredLogsValue.length > VIRTUALIZATION_THRESHOLD);

  const levelOptions: { value: LogLevel | 'all'; label: string; color: string }[] = [
    { value: 'all', label: t('logs.levels.all'), color: 'text-zinc-400' },
    { value: 'error', label: t('logs.levels.errors'), color: 'text-red-400' },
    { value: 'warn', label: t('logs.levels.warnings'), color: 'text-amber-400' },
    { value: 'info', label: t('logs.levels.info'), color: 'text-cyan-400' },
    { value: 'debug', label: t('logs.levels.debug'), color: 'text-zinc-400' },
    { value: 'success', label: t('logs.levels.success'), color: 'text-emerald-400' }
  ];

  // Read source filter from URL query params on load
  $effect(() => {
    if (!browser) return;
    
    const unsubPage = page.subscribe(($page) => {
      const sourceParam = $page.url.searchParams.get('source');
      if (sourceParam) {
        sourceFilter = sourceParam;
      }
    });
    
    return () => {
      unsubPage();
    };
  });

  // Subscribe to stores
  $effect(() => {
    if (!browser) return;

    const unsubLogs = logs.subscribe(v => { logsValue = v; });
    const unsubFiltered = filteredLogs.subscribe(v => { filteredLogsValue = v; });
    const unsubSources = logSources.subscribe(v => { sourcesValue = v; });

    return () => {
      unsubLogs();
      unsubFiltered();
      unsubSources();
    };
  });

  // Update filters when local state changes
  $effect(() => {
    logFilters.set({
      level: levelFilter,
      source: sourceFilter,
      search: searchQuery
    });
  });

  // Auto-scroll when new logs arrive
  $effect(() => {
    if (autoScroll && filteredLogsValue.length > 0) {
      scrollToBottom();
    }
  });

  function scrollToBottom() {
    requestAnimationFrame(() => {
      const container = document.getElementById('logs-container');
      if (container) {
        container.scrollTop = container.scrollHeight;
        scrollTop = container.scrollTop;
      }
    });
  }

  function handleScroll(e: Event) {
    const target = e.target as HTMLDivElement;
    scrollTop = target.scrollTop;
    
    // Update container height on scroll (in case of resize)
    if (target.clientHeight !== containerHeight) {
      containerHeight = target.clientHeight;
    }
  }

  // Initialize container height
  $effect(() => {
    if (containerRef) {
      containerHeight = containerRef.clientHeight;
      
      // Set up resize observer
      const resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          containerHeight = entry.contentRect.height;
        }
      });
      resizeObserver.observe(containerRef);
      
      return () => resizeObserver.disconnect();
    }
  });

  function clearLogs() {
    clearing = true;
    logs.clear();
    setTimeout(() => { clearing = false; }, 300);
  }

  async function exportLogs() {
    if (!browser || logsValue.length === 0) return;
    exporting = true;

    try {
      const exportData = logsValue.map(log => ({
        timestamp: log.timestamp.toISOString(),
        level: log.level,
        source: log.source,
        message: log.message
      }));

      const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `isolate-logs-${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error('Failed to export logs:', e);
    } finally {
      exporting = false;
    }
  }

  function getLevelBadgeClass(level: LogLevel): string {
    switch (level) {
      case 'error': return 'bg-red-500/20 text-red-400 border-red-500/30';
      case 'warn': return 'bg-amber-500/20 text-amber-400 border-amber-500/30';
      case 'info': return 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30';
      case 'debug': return 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30';
      case 'success': return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30';
      default: return 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30';
    }
  }

  function getLevelIcon(level: LogLevel): string {
    switch (level) {
      case 'error': return '✗';
      case 'warn': return '⚠';
      case 'info': return 'ℹ';
      case 'debug': return '⚙';
      case 'success': return '✓';
      default: return '•';
    }
  }

  function formatTimestamp(date: Date): string {
    return date.toLocaleTimeString('en-US', { 
      hour: '2-digit', 
      minute: '2-digit', 
      second: '2-digit',
      fractionalSecondDigits: 3,
      hour12: false
    });
  }
</script>

<div class="flex flex-col h-full p-8 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-white">{t('logs.title')}</h1>
      <p class="text-zinc-400 mt-1">{t('logs.subtitle')}</p>
    </div>
    
    <div class="flex items-center gap-3">
      <button
        onclick={clearLogs}
        disabled={clearing || logsValue.length === 0}
        class="flex items-center gap-2 px-4 py-2.5 bg-zinc-800/50 hover:bg-zinc-800 
               border border-white/5 hover:border-white/10
               disabled:opacity-50 disabled:cursor-not-allowed rounded-xl font-medium transition-all text-zinc-300"
      >
        {#if clearing}
          <Spinner size="sm" />
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        {/if}
        <span>{t('logs.buttons.clear')}</span>
      </button>
      
      <button
        onclick={exportLogs}
        disabled={exporting || logsValue.length === 0}
        class="flex items-center gap-2 px-4 py-2.5 
               bg-indigo-500 hover:bg-indigo-600
               disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-xl font-medium transition-all"
      >
        {#if exporting}
          <Spinner size="sm" />
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        {/if}
        <span>{t('logs.buttons.export')}</span>
      </button>
    </div>
  </div>

  <!-- Filters -->
  <div class="bg-black/20 backdrop-blur-sm rounded-2xl p-4 border border-white/10">
    <div class="flex flex-wrap items-center gap-4">
      <!-- Level Filter -->
      <div class="flex items-center gap-2">
        <label for="log-level" class="text-zinc-400 text-sm">{t('logs.level')}:</label>
        <select
          id="log-level"
          bind:value={levelFilter}
          class="bg-black/20 border border-white/10 text-white rounded-lg px-3 py-2 text-sm 
                 focus:border-indigo-500 focus:outline-none transition-colors"
        >
          {#each levelOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </div>

      <!-- Source Filter -->
      <div class="flex items-center gap-2">
        <label for="log-module" class="text-zinc-400 text-sm">{t('logs.module')}:</label>
        <select
          id="log-module"
          bind:value={sourceFilter}
          class="bg-black/20 border border-white/10 text-white rounded-lg px-3 py-2 text-sm 
                 focus:border-indigo-500 focus:outline-none transition-colors"
        >
          <option value="all">{t('logs.levels.all')}</option>
          {#each sourcesValue as source}
            <option value={source}>{source}</option>
          {/each}
        </select>
      </div>

      <!-- Search -->
      <div class="flex-1 min-w-[200px]">
        <div class="relative">
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            bind:value={searchQuery}
            placeholder={t('common.search')}
            class="w-full bg-black/20 border border-white/10 text-white rounded-lg pl-10 pr-4 py-2 text-sm 
                   focus:border-indigo-500 focus:outline-none placeholder-zinc-400 transition-colors"
          />
        </div>
      </div>

      <!-- Auto-scroll Toggle -->
      <label class="flex items-center gap-2 cursor-pointer px-3 py-2 rounded-lg hover:bg-white/5 transition-colors">
        <input
          type="checkbox"
          bind:checked={autoScroll}
          class="w-4 h-4 rounded bg-zinc-700 border-zinc-600 text-indigo-500 focus:ring-indigo-500 focus:ring-offset-zinc-900"
        />
        <span class="text-zinc-400 text-sm">{t('logs.autoScroll')}</span>
      </label>
    </div>
  </div>

  <!-- Logs Container -->
  <div 
    id="logs-container"
    bind:this={containerRef}
    onscroll={handleScroll}
    class="flex-1 bg-black/30 backdrop-blur-sm rounded-2xl border border-white/5 overflow-hidden"
  >
    {#if filteredLogsValue.length === 0}
      <div class="flex flex-col items-center justify-center h-full text-zinc-400">
        <div class="w-20 h-20 rounded-2xl bg-zinc-800/30 border border-white/5 flex items-center justify-center mb-4">
          <svg class="w-10 h-10 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>
        <p class="text-lg font-medium text-zinc-400">{t('logs.noLogs')}</p>
        <p class="text-sm mt-1">{t('logs.noLogsDesc')}</p>
      </div>
    {:else if useVirtualization}
      <!-- Virtual scrolling for large lists -->
      <div 
        class="h-full overflow-y-auto p-4 font-mono text-xs"
        style="position: relative;"
      >
        <!-- Spacer to maintain scroll height -->
        <div style="height: {virtualScrollData().totalHeight}px; position: relative;">
          <!-- Visible items positioned absolutely -->
          <div style="position: absolute; top: {virtualScrollData().offsetY}px; left: 0; right: 0;">
            {#each virtualScrollData().visibleItems as log, i (log.id)}
              <div 
                class="flex items-start gap-3 py-1.5 px-3 rounded-lg hover:bg-white/5 transition-colors"
                style="height: {ITEM_HEIGHT}px;"
              >
                <!-- Timestamp -->
                <span class="text-zinc-400 shrink-0 tabular-nums">
                  {formatTimestamp(log.timestamp)}
                </span>
                
                <!-- Level Badge -->
                <span class="shrink-0 px-1.5 py-0.5 text-xs font-medium rounded border {getLevelBadgeClass(log.level)} flex items-center gap-1">
                  <span>{getLevelIcon(log.level)}</span>
                  {log.level.toUpperCase()}
                </span>
                
                <!-- Source -->
                <span class="text-cyan-400 shrink-0">
                  [{log.source}]
                </span>
                
                <!-- Message -->
                <span class="text-zinc-300 break-all leading-relaxed truncate">
                  {log.message}
                </span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {:else}
      <!-- Regular rendering for small lists -->
      <div class="h-full overflow-y-auto p-4 font-mono text-xs">
        {#each filteredLogsValue as log (log.id)}
          <div class="flex items-start gap-3 py-1.5 px-3 rounded-lg hover:bg-white/5 transition-colors">
            <!-- Timestamp -->
            <span class="text-zinc-400 shrink-0 tabular-nums">
              {formatTimestamp(log.timestamp)}
            </span>
            
            <!-- Level Badge -->
            <span class="shrink-0 px-1.5 py-0.5 text-xs font-medium rounded border {getLevelBadgeClass(log.level)} flex items-center gap-1">
              <span>{getLevelIcon(log.level)}</span>
              {log.level.toUpperCase()}
            </span>
            
            <!-- Source -->
            <span class="text-cyan-400 shrink-0">
              [{log.source}]
            </span>
            
            <!-- Message -->
            <span class="text-zinc-300 break-all leading-relaxed">
              {log.message}
            </span>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Status Bar -->
  <div class="flex items-center justify-between text-sm text-zinc-400">
    <span>
      {filteredLogsValue.length} {t('common.of')} {logsValue.length} {t('common.entries')}
      {#if useVirtualization}
        <span class="text-zinc-400 ml-2">({t('logs.virtualized')})</span>
      {/if}
    </span>
    <div class="flex items-center gap-4">
      {#if levelFilter !== 'all'}
        <span class="px-2 py-1 rounded bg-zinc-800/50 text-xs">
          {t('logs.filter')}: {levelFilter.toUpperCase()}
        </span>
      {/if}
      {#if sourceFilter !== 'all'}
        <span class="px-2 py-1 rounded bg-zinc-800/50 text-xs">
          {t('logs.module')}: {sourceFilter}
        </span>
      {/if}
    </div>
  </div>
</div>
