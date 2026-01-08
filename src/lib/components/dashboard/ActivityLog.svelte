<script lang="ts">
  import { logs, type LogEntry, type LogLevel } from '$lib/stores/logs';

  interface Props {
    maxItems?: number;
  }

  let { maxItems = 5 }: Props = $props();

  // Подписка на store с последними логами
  let allLogs = $state<LogEntry[]>([]);
  
  $effect(() => {
    const unsubscribe = logs.subscribe(value => {
      allLogs = value;
    });
    return unsubscribe;
  });

  // Последние N логов в обратном порядке (новые сверху)
  const recentLogs = $derived(
    [...allLogs].reverse().slice(0, maxItems)
  );

  // Конфигурация иконок и цветов по уровню лога
  const levelConfig: Record<LogLevel, { icon: string; color: string; bgColor: string }> = {
    success: {
      icon: '✓',
      color: 'text-emerald-400',
      bgColor: 'bg-emerald-500/20'
    },
    info: {
      icon: 'ℹ',
      color: 'text-blue-400',
      bgColor: 'bg-blue-500/20'
    },
    warn: {
      icon: '⚠',
      color: 'text-amber-400',
      bgColor: 'bg-amber-500/20'
    },
    error: {
      icon: '✕',
      color: 'text-red-400',
      bgColor: 'bg-red-500/20'
    },
    debug: {
      icon: '⚙',
      color: 'text-slate-400',
      bgColor: 'bg-slate-500/20'
    }
  };

  // Форматирование времени
  function formatTime(date: Date): string {
    return date.toLocaleTimeString('ru-RU', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }
</script>

<div class="flex flex-col gap-2 p-4 rounded-2xl bg-slate-800/30 border border-slate-700/50 max-h-[200px]">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-semibold text-slate-200">Журнал активности</h3>
    <span class="text-xs text-slate-500">
      {allLogs.length} событий
    </span>
  </div>

  <!-- Logs List -->
  {#if recentLogs.length > 0}
    <div class="flex flex-col gap-1.5 overflow-y-auto flex-1">
      {#each recentLogs as log (log.id)}
        {@const config = levelConfig[log.level]}
        <div class="flex items-start gap-2 py-1.5 px-2 rounded-lg hover:bg-slate-700/30 transition-colors">
          <!-- Icon -->
          <div class="flex-shrink-0 w-5 h-5 rounded-full {config.bgColor} flex items-center justify-center">
            <span class="text-xs {config.color}">{config.icon}</span>
          </div>
          
          <!-- Content -->
          <div class="flex-1 min-w-0">
            <p class="text-xs text-slate-300 truncate" title={log.message}>
              {log.message}
            </p>
            <div class="flex items-center gap-2 mt-0.5">
              <span class="text-[10px] text-slate-500">{log.source}</span>
              <span class="text-[10px] text-slate-600">•</span>
              <span class="text-[10px] text-slate-500">{formatTime(log.timestamp)}</span>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="py-4 text-center flex-1 flex items-center justify-center">
      <p class="text-xs text-slate-500">Нет событий</p>
    </div>
  {/if}
</div>
