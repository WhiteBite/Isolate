<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';

  interface LogEntry {
    timestamp: string;
    level: 'info' | 'warn' | 'error' | 'debug';
    module: string;
    message: string;
  }

  let logs = $state<LogEntry[]>([]);
  let filter = $state<string>('all');
  let searchQuery = $state('');
  let autoScroll = $state(true);
  let unlisten: (() => void) | null = null;
  
  // Filter options
  const levels = ['all', 'info', 'warn', 'error', 'debug'];
  
  $effect(() => {
    // Auto-scroll to bottom when new logs arrive
    if (autoScroll && browser) {
      const container = document.getElementById('logs-container');
      if (container) {
        container.scrollTop = container.scrollHeight;
      }
    }
  });

  onMount(() => {
    if (!browser) return;
    
    // Load logs asynchronously
    (async () => {
      const { listen } = await import('@tauri-apps/api/event');
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Load existing logs
      try {
        const existingLogs = await invoke<LogEntry[]>('get_logs');
        logs = existingLogs;
      } catch (e) {
        console.error('Failed to load logs:', e);
      }
      
      // Subscribe to new log events
      unlisten = await listen('log:entry', (event) => {
        const entry = event.payload as LogEntry;
        logs = [...logs, entry];
      });
    })();
    
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  });

  function getFilteredLogs() {
    return logs.filter(log => {
      const matchesLevel = filter === 'all' || log.level === filter;
      const matchesSearch = !searchQuery || 
        log.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
        log.module.toLowerCase().includes(searchQuery.toLowerCase());
      return matchesLevel && matchesSearch;
    });
  }

  async function exportLogs() {
    if (!browser) return;
    
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('export_logs');
  }

  async function clearLogs() {
    logs = [];
  }

  function getLevelColor(level: string): string {
    switch (level) {
      case 'error': return 'text-red-400';
      case 'warn': return 'text-yellow-400';
      case 'info': return 'text-blue-400';
      case 'debug': return 'text-gray-400';
      default: return 'text-gray-300';
    }
  }
</script>

<div class="flex flex-col h-screen p-4">
  <!-- Header -->
  <div class="flex items-center justify-between mb-4">
    <div class="flex items-center gap-4">
      <button onclick={() => goto('/')} class="p-2 hover:bg-gray-800 rounded-lg" aria-label="Назад">
        <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h1 class="text-xl font-bold">Логи</h1>
    </div>
    
    <div class="flex items-center gap-2">
      <button onclick={exportLogs} class="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm">
        Экспорт
      </button>
      <button onclick={clearLogs} class="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm">
        Очистить
      </button>
    </div>
  </div>
  
  <!-- Filters -->
  <div class="flex items-center gap-4 mb-4">
    <input
      type="text"
      bind:value={searchQuery}
      placeholder="Поиск..."
      class="flex-1 px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-sm"
    />
    
    <select bind:value={filter} class="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-sm">
      {#each levels as level}
        <option value={level}>{level === 'all' ? 'Все' : level.toUpperCase()}</option>
      {/each}
    </select>
    
    <label class="flex items-center gap-2 text-sm">
      <input type="checkbox" bind:checked={autoScroll} class="rounded" />
      Автопрокрутка
    </label>
  </div>
  
  <!-- Logs -->
  <div id="logs-container" class="flex-1 overflow-y-auto bg-gray-900 rounded-lg p-4 font-mono text-sm">
    {#each getFilteredLogs() as log}
      <div class="flex gap-2 py-1 hover:bg-gray-800">
        <span class="text-gray-500 shrink-0">{log.timestamp}</span>
        <span class={`shrink-0 w-12 ${getLevelColor(log.level)}`}>[{log.level.toUpperCase()}]</span>
        <span class="text-gray-400 shrink-0">{log.module}:</span>
        <span class="text-gray-200">{log.message}</span>
      </div>
    {:else}
      <div class="text-gray-500 text-center py-8">Нет логов</div>
    {/each}
  </div>
</div>
