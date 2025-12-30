<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import Spinner from '$lib/components/Spinner.svelte';

  interface LogEntry {
    id: string;
    timestamp: string;
    level: 'error' | 'warn' | 'info' | 'debug';
    module: string;
    message: string;
  }

  // State
  let logs = $state<LogEntry[]>([]);
  let filteredLogs = $state<LogEntry[]>([]);
  let levelFilter = $state<'all' | 'error' | 'warn' | 'info' | 'debug'>('all');
  let moduleFilter = $state<string>('all');
  let searchQuery = $state('');
  let dateFilter = $state('');
  let autoScroll = $state(true);
  let loading = $state(true);
  let exporting = $state(false);
  let clearing = $state(false);

  let modules = $state<string[]>([]);
  let unlistenLog: (() => void) | null = null;

  const levelOptions = [
    { value: 'all', label: 'All' },
    { value: 'error', label: 'Error' },
    { value: 'warn', label: 'Warn' },
    { value: 'info', label: 'Info' },
    { value: 'debug', label: 'Debug' }
  ];
</script>

  onMount(async () => {
    if (!browser) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      // Load existing logs
      const existingLogs = await invoke<LogEntry[]>('get_logs', {
        level: levelFilter === 'all' ? null : levelFilter,
        module: moduleFilter === 'all' ? null : moduleFilter,
        search: searchQuery || null,
        date: dateFilter || null
      }).catch(() => []);
      
      logs = existingLogs;
      extractModules();
      applyFilters();

      // Subscribe to new log events
      unlistenLog = await listen('log:entry', (event) => {
        const entry = event.payload as LogEntry;
        logs = [...logs, entry];
        
        // Update modules list
        if (!modules.includes(entry.module)) {
          modules = [...modules, entry.module];
        }
        
        applyFilters();
        
        // Auto-scroll
        if (autoScroll) {
          scrollToBottom();
        }
      });
    } catch (e) {
      console.error('Failed to load logs:', e);
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    unlistenLog?.();
  });

  function extractModules() {
    const uniqueModules = [...new Set(logs.map(l => l.module))];
    modules = uniqueModules.sort();
  }

  function applyFilters() {
    filteredLogs = logs.filter(log => {
      // Level filter
      if (levelFilter !== 'all' && log.level !== levelFilter) return false;
      
      // Module filter
      if (moduleFilter !== 'all' && log.module !== moduleFilter) return false;
      
      // Search filter
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        if (!log.message.toLowerCase().includes(query) && 
            !log.module.toLowerCase().includes(query)) {
          return false;
        }
      }
      
      // Date filter
      if (dateFilter) {
        const logDate = log.timestamp.split('T')[0];
        if (logDate !== dateFilter) return false;
      }
      
      return true;
    });
  }

  function scrollToBottom() {
    requestAnimationFrame(() => {
      const container = document.getElementById('logs-container');
      if (container) {
        container.scrollTop = container.scrollHeight;
      }
    });
  }

  async function loadLogs() {
    if (!browser) return;
    loading = true;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const loadedLogs = await invoke<LogEntry[]>('get_logs', {
        level: levelFilter === 'all' ? null : levelFilter,
        module: moduleFilter === 'all' ? null : moduleFilter,
        search: searchQuery || null,
        date: dateFilter || null
      });
      logs = loadedLogs;
      extractModules();
      applyFilters();
    } catch (e) {
      console.error('Failed to load logs:', e);
    } finally {
      loading = false;
    }
  }

  async function clearLogs() {
    if (!browser) return;
    clearing = true;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('clear_logs');
      logs = [];
      filteredLogs = [];
    } catch (e) {
      console.error('Failed to clear logs:', e);
    } finally {
      clearing = false;
    }
  }

  async function exportLogs() {
    if (!browser) return;
    exporting = true;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('export_logs');
    } catch (e) {
      console.error('Failed to export logs:', e);
    } finally {
      exporting = false;
    }
  }

  function getLevelBadgeClass(level: string): string {
    switch (level) {
      case 'error': return 'bg-red-500/20 text-red-400 border-red-500/30';
      case 'warn': return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
      case 'info': return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
      case 'debug': return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
      default: return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
    }
  }

  // Watch for filter changes
  $effect(() => {
    levelFilter; moduleFilter; searchQuery; dateFilter;
    applyFilters();
  });

</script>

<div class="flex flex-col h-full p-6">
  <!-- Header -->
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-2xl font-bold text-white">Логи</h1>
      <p class="text-[#a0a0a0] mt-1">Журнал событий приложения</p>
    </div>
    
    <div class="flex items-center gap-3">
      <button
        onclick={clearLogs}
        disabled={clearing || logs.length === 0}
        class="flex items-center gap-2 px-4 py-2 bg-[#2a2f4a] hover:bg-[#3a3f5a] disabled:opacity-50 disabled:cursor-not-allowed rounded-lg font-medium transition-colors"
      >
        {#if clearing}
          <Spinner size="sm" color="gray" />
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        {/if}
        <span>Очистить</span>
      </button>
      
      <button
        onclick={exportLogs}
        disabled={exporting || logs.length === 0}
        class="flex items-center gap-2 px-4 py-2 bg-[#00d4ff] hover:bg-[#00b8e6] disabled:opacity-50 disabled:cursor-not-allowed text-[#0a0e27] rounded-lg font-medium transition-colors"
      >
        {#if exporting}
          <Spinner size="sm" color="white" />
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        {/if}
        <span>Экспортировать</span>
      </button>
    </div>
  </div>

  <!-- Filters -->
  <div class="bg-[#1a1f3a] rounded-xl p-4 border border-[#2a2f4a] mb-4">
    <div class="flex flex-wrap items-center gap-4">
      <!-- Level Filter -->
      <div class="flex items-center gap-2">
        <label class="text-[#a0a0a0] text-sm">Уровень:</label>
        <select
          bind:value={levelFilter}
          class="bg-[#0a0e27] border border-[#2a2f4a] text-white rounded-lg px-3 py-2 text-sm focus:ring-[#00d4ff] focus:border-[#00d4ff]"
        >
          {#each levelOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </div>

      <!-- Module Filter -->
      <div class="flex items-center gap-2">
        <label class="text-[#a0a0a0] text-sm">Модуль:</label>
        <select
          bind:value={moduleFilter}
          class="bg-[#0a0e27] border border-[#2a2f4a] text-white rounded-lg px-3 py-2 text-sm focus:ring-[#00d4ff] focus:border-[#00d4ff]"
        >
          <option value="all">Все</option>
          {#each modules as mod}
            <option value={mod}>{mod}</option>
          {/each}
        </select>
      </div>

      <!-- Search -->
      <div class="flex-1 min-w-[200px]">
        <div class="relative">
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[#a0a0a0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Поиск..."
            class="w-full bg-[#0a0e27] border border-[#2a2f4a] text-white rounded-lg pl-10 pr-4 py-2 text-sm focus:ring-[#00d4ff] focus:border-[#00d4ff] placeholder-[#a0a0a0]"
          />
        </div>
      </div>

      <!-- Date Filter -->
      <div class="flex items-center gap-2">
        <label class="text-[#a0a0a0] text-sm">Дата:</label>
        <input
          type="date"
          bind:value={dateFilter}
          class="bg-[#0a0e27] border border-[#2a2f4a] text-white rounded-lg px-3 py-2 text-sm focus:ring-[#00d4ff] focus:border-[#00d4ff]"
        />
      </div>

      <!-- Auto-scroll Toggle -->
      <label class="flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={autoScroll}
          class="w-4 h-4 rounded bg-[#2a2f4a] border-[#3a3f5a] text-[#00d4ff] focus:ring-[#00d4ff]"
        />
        <span class="text-[#a0a0a0] text-sm">Автопрокрутка</span>
      </label>
    </div>
  </div>

  <!-- Logs Container -->
  <div 
    id="logs-container"
    class="flex-1 bg-[#0a0e27] rounded-xl border border-[#2a2f4a] overflow-hidden"
  >
    {#if loading}
      <div class="flex items-center justify-center h-full">
        <Spinner size="lg" color="cyan" />
      </div>
    {:else if filteredLogs.length === 0}
      <div class="flex flex-col items-center justify-center h-full text-[#a0a0a0]">
        <svg class="w-16 h-16 mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <p class="text-lg">Нет логов</p>
        <p class="text-sm mt-1">Логи появятся здесь при работе приложения</p>
      </div>
    {:else}
      <div class="h-full overflow-y-auto p-4 font-mono text-sm space-y-1">
        {#each filteredLogs as log (log.id || log.timestamp + log.message)}
          <div class="flex items-start gap-3 py-2 px-3 rounded-lg hover:bg-[#1a1f3a]/50 transition-colors group">
            <!-- Timestamp -->
            <span class="text-[#606080] shrink-0 text-xs mt-0.5">
              {log.timestamp}
            </span>
            
            <!-- Level Badge -->
            <span class="shrink-0 px-2 py-0.5 text-xs font-medium rounded border {getLevelBadgeClass(log.level)}">
              {log.level.toUpperCase()}
            </span>
            
            <!-- Module -->
            <span class="text-[#00d4ff] shrink-0 text-xs mt-0.5">
              [{log.module}]
            </span>
            
            <!-- Message -->
            <span class="text-white/90 break-all">
              {log.message}
            </span>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Status Bar -->
  <div class="flex items-center justify-between mt-4 text-sm text-[#a0a0a0]">
    <span>
      {filteredLogs.length} из {logs.length} записей
    </span>
    <span>
      {#if levelFilter !== 'all'}
        Фильтр: {levelFilter.toUpperCase()}
      {/if}
    </span>
  </div>
</div>
