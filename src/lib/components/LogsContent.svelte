<script lang="ts">
  import { tick } from 'svelte';
  import { logs, logFilters, filteredLogs, logSources, type LogLevel } from '$lib/stores/logs';
  import { browser } from '$app/environment';

  // State
  let autoScroll = $state(true);
  let logsContainer = $state<HTMLDivElement | undefined>(undefined);

  // Level colors
  const levelColors: Record<LogLevel, string> = {
    error: 'text-red-400',
    warn: 'text-amber-400',
    info: 'text-zinc-300',
    debug: 'text-zinc-400',
    success: 'text-emerald-400'
  };

  const levelBgColors: Record<LogLevel, string> = {
    error: 'bg-red-500/10 border-red-500/20',
    warn: 'bg-amber-500/10 border-amber-500/20',
    info: 'bg-zinc-500/10 border-zinc-500/20',
    debug: 'bg-zinc-800/50 border-zinc-700/30',
    success: 'bg-emerald-500/10 border-emerald-500/20'
  };

  // Source badge colors
  const sourceColors = [
    'bg-cyan-500/10 text-cyan-400 border-cyan-500/20',
    'bg-blue-500/10 text-blue-400 border-blue-500/20',
    'bg-violet-500/10 text-violet-400 border-violet-500/20',
    'bg-pink-500/10 text-pink-400 border-pink-500/20',
    'bg-orange-500/10 text-orange-400 border-orange-500/20',
  ];

  function getSourceColor(source: string): string {
    let hash = 0;
    for (let i = 0; i < source.length; i++) {
      hash = source.charCodeAt(i) + ((hash << 5) - hash);
    }
    return sourceColors[Math.abs(hash) % sourceColors.length];
  }

  // Format timestamp
  function formatTime(date: Date): string {
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }

  // Clear logs
  function clearLogs() {
    logs.clear();
  }

  // Copy logs to clipboard
  async function copyLogs() {
    const logsText = $filteredLogs
      .map(log => `[${formatTime(log.timestamp)}] [${log.level.toUpperCase()}] [${log.source}] ${log.message}`)
      .join('\n');
    
    try {
      await navigator.clipboard.writeText(logsText);
    } catch (err) {
      console.error('Failed to copy logs:', err);
    }
  }

  // Auto-scroll to bottom
  async function scrollToBottom() {
    if (autoScroll && logsContainer) {
      await tick();
      logsContainer.scrollTop = logsContainer.scrollHeight;
    }
  }

  // Handle scroll - disable auto-scroll if user scrolls up
  function handleScroll() {
    if (!logsContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = logsContainer;
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 50;
    if (!isAtBottom && autoScroll) {
      autoScroll = false;
    }
  }

  // Watch for new logs and auto-scroll
  $effect(() => {
    $filteredLogs;
    scrollToBottom();
  });
</script>

<div class="flex flex-col h-full bg-zinc-950">
  <!-- Toolbar -->
  <div class="flex items-center justify-between px-4 py-2 border-b border-white/5 bg-zinc-900/60 shrink-0">
    <div class="flex items-center gap-4">
      <!-- Level Filter -->
      <select
        bind:value={$logFilters.level}
        class="px-2 py-1 text-xs bg-zinc-800/80 border border-white/5 rounded
               text-zinc-400 focus:outline-none focus:border-white/10"
        aria-label="Filter by log level"
      >
        <option value="all">All Levels</option>
        <option value="error">Error</option>
        <option value="warn">Warning</option>
        <option value="info">Info</option>
        <option value="debug">Debug</option>
        <option value="success">Success</option>
      </select>

      <!-- Source Filter -->
      <select
        bind:value={$logFilters.source}
        class="px-2 py-1 text-xs bg-zinc-800/80 border border-white/5 rounded
               text-zinc-400 focus:outline-none focus:border-white/10"
        aria-label="Filter by log source"
      >
        <option value="all">All Sources</option>
        {#each $logSources as source}
          <option value={source}>{source}</option>
        {/each}
      </select>

      <!-- Search -->
      <div class="relative">
        <svg class="absolute left-2 top-1/2 -translate-y-1/2 w-3 h-3 text-zinc-400" 
             fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          type="text"
          bind:value={$logFilters.search}
          placeholder="Search logs..."
          class="pl-7 pr-2 py-1 text-xs bg-zinc-800/80 border border-white/5 rounded
                 text-zinc-300 placeholder:text-zinc-400
                 focus:outline-none focus:border-white/10 w-40"
          aria-label="Search logs"
        />
      </div>
    </div>

    <div class="flex items-center gap-2">
      <!-- Auto-scroll toggle -->
      <button
        onclick={() => { autoScroll = !autoScroll; if (autoScroll) scrollToBottom(); }}
        class="flex items-center gap-1.5 px-2 py-1 text-xs rounded transition-colors
               {autoScroll 
                 ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' 
                 : 'bg-zinc-800/50 text-zinc-400 border border-white/5 hover:text-zinc-300'}"
        aria-label="Toggle auto-scroll"
        aria-pressed={autoScroll}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M19 14l-7 7m0 0l-7-7m7 7V3" />
        </svg>
        Auto
      </button>

      <!-- Copy button -->
      <button
        onclick={copyLogs}
        class="p-1.5 text-zinc-400 hover:text-zinc-300 
               hover:bg-zinc-800 rounded transition-colors"
        aria-label="Copy logs to clipboard"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      </button>

      <!-- Clear button -->
      <button
        onclick={clearLogs}
        class="p-1.5 text-zinc-400 hover:text-red-400 
               hover:bg-red-500/10 rounded transition-colors"
        aria-label="Clear all logs"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    </div>
  </div>

  <!-- Logs Container -->
  <div
    bind:this={logsContainer}
    onscroll={handleScroll}
    class="flex-1 overflow-y-auto overflow-x-hidden font-mono text-[10px]"
    role="log"
    aria-label="Application logs"
    aria-live="polite"
    aria-relevant="additions"
  >
    {#if $filteredLogs.length === 0}
      <div class="flex items-center justify-center h-full text-zinc-400">
        <div class="text-center">
          <svg class="w-8 h-8 mx-auto mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p>No logs yet</p>
        </div>
      </div>
    {:else}
      {#each $filteredLogs as log (log.id)}
        <div class="flex items-start gap-3 px-4 py-1.5 hover:bg-zinc-900/50 border-b border-white/[0.02]" role="listitem">
          <!-- Timestamp -->
          <span class="text-zinc-400 shrink-0 w-16" aria-label="Time">
            {formatTime(log.timestamp)}
          </span>

          <!-- Level Badge -->
          <span class="shrink-0 px-1.5 py-0.5 text-[10px] uppercase font-semibold rounded border
                      {levelBgColors[log.level]} {levelColors[log.level]}"
                role="status"
                aria-label="Level: {log.level}">
            {log.level}
          </span>

          <!-- Source Badge -->
          <span class="shrink-0 px-1.5 py-0.5 text-[10px] rounded border {getSourceColor(log.source)}"
                aria-label="Source: {log.source}">
            {log.source}
          </span>

          <!-- Message -->
          <span class="{levelColors[log.level]} break-all flex-1">
            {log.message}
          </span>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Status Bar -->
  <div class="flex items-center justify-between px-4 py-1 text-[10px] text-zinc-400 
              border-t border-white/5 bg-zinc-900/40 shrink-0"
       role="status"
       aria-live="polite">
    <span>{$filteredLogs.length} entries</span>
  </div>
</div>
