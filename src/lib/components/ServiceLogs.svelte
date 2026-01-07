<script lang="ts">
  import { tick } from 'svelte';
  import { logs, type LogEntry, type LogLevel } from '$lib/stores/logs';
  import { derived } from 'svelte/store';

  // Props
  interface Props {
    source: string;
    maxHeight?: number;
    showHeader?: boolean;
    maxLogs?: number;
  }

  let {
    source,
    maxHeight = 200,
    showHeader = true,
    maxLogs = 50
  }: Props = $props();

  // State
  let autoScroll = $state(true);
  let logsContainer: HTMLDivElement;

  // Derived store for logs filtered by source (limited to maxLogs)
  const sourceLogs = derived(logs, ($logs) => {
    const filtered = $logs.filter(log => log.source === source);
    // Return only the last maxLogs entries
    return filtered.slice(-maxLogs);
  });

  // Level colors
  const levelColors: Record<LogLevel, string> = {
    error: 'text-red-400',
    warn: 'text-amber-400',
    info: 'text-zinc-300',
    debug: 'text-zinc-400',
    success: 'text-emerald-400'
  };

  const levelBgColors: Record<LogLevel, string> = {
    error: 'bg-red-500/10',
    warn: 'bg-amber-500/10',
    info: 'bg-zinc-500/10',
    debug: 'bg-zinc-800/50',
    success: 'bg-emerald-500/10'
  };

  // Format timestamp
  function formatTime(date: Date): string {
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
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
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 30;
    if (!isAtBottom && autoScroll) {
      autoScroll = false;
    }
  }

  // Copy logs to clipboard
  async function copyLogs() {
    const logsText = $sourceLogs
      .map(log => `[${formatTime(log.timestamp)}] [${log.level.toUpperCase()}] ${log.message}`)
      .join('\n');
    
    try {
      await navigator.clipboard.writeText(logsText);
    } catch (err) {
      console.error('Failed to copy logs:', err);
    }
  }

  // Clear logs for this source (just visual, doesn't affect store)
  // Note: We can't clear specific source logs from the store without modifying it
  // So we'll just provide a copy function

  // Watch for new logs and auto-scroll
  $effect(() => {
    $sourceLogs;
    scrollToBottom();
  });
</script>

<div class="flex flex-col bg-zinc-950 rounded-xl border border-white/5 overflow-hidden">
  <!-- Header -->
  {#if showHeader}
    <div class="flex items-center justify-between px-3 py-2 border-b border-white/5 bg-zinc-900/60">
      <div class="flex items-center gap-2 text-xs font-medium text-zinc-400">
        <svg class="w-3.5 h-3.5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        <span>Logs</span>
        <span class="px-1.5 py-0.5 bg-zinc-800 rounded text-[10px] text-zinc-400">
          {$sourceLogs.length}
        </span>
      </div>

      <div class="flex items-center gap-1">
        <!-- Auto-scroll toggle -->
        <button
          onclick={() => { autoScroll = !autoScroll; if (autoScroll) scrollToBottom(); }}
          class="flex items-center gap-1 px-1.5 py-1 text-[10px] rounded transition-colors
                 {autoScroll 
                   ? 'bg-indigo-500/20 text-indigo-400' 
                   : 'bg-zinc-800/50 text-zinc-400 hover:text-zinc-300'}"
          title="Auto-scroll"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </button>

        <!-- Copy button -->
        <button
          onclick={copyLogs}
          class="p-1 text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800 rounded transition-colors"
          title="Copy logs"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
          </svg>
        </button>
      </div>
    </div>
  {/if}

  <!-- Logs Container -->
  <div
    bind:this={logsContainer}
    onscroll={handleScroll}
    class="overflow-y-auto overflow-x-hidden font-mono text-[11px] leading-relaxed
           [&::-webkit-scrollbar]:w-1 [&::-webkit-scrollbar-track]:bg-transparent
           [&::-webkit-scrollbar-thumb]:bg-zinc-700 [&::-webkit-scrollbar-thumb]:rounded-full
           [&::-webkit-scrollbar-thumb:hover]:bg-zinc-600"
    style="max-height: {maxHeight}px;"
  >
    {#if $sourceLogs.length === 0}
      <div class="flex items-center justify-center py-8 text-zinc-600">
        <div class="text-center">
          <svg class="w-6 h-6 mx-auto mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-xs">No logs for this service</p>
        </div>
      </div>
    {:else}
      {#each $sourceLogs as log (log.id)}
        <div class="flex items-start gap-2 px-3 py-1 hover:bg-zinc-900/50 border-b border-white/[0.02]">
          <!-- Timestamp -->
          <span class="text-zinc-600 shrink-0 w-14">
            {formatTime(log.timestamp)}
          </span>

          <!-- Level Badge -->
          <span class="shrink-0 px-1 py-0.5 text-[9px] uppercase font-semibold rounded
                      {levelBgColors[log.level]} {levelColors[log.level]}">
            {log.level.slice(0, 3)}
          </span>

          <!-- Message -->
          <span class="{levelColors[log.level]} break-all flex-1">
            {log.message}
          </span>
        </div>
      {/each}
    {/if}
  </div>
</div>
