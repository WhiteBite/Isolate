<script lang="ts">
  import PingChart from '../PingChart.svelte';

  interface Props {
    /** История значений latency в ms */
    history?: number[];
    /** Текущее значение latency */
    currentLatency?: number;
    /** Название сервиса/стратегии */
    label?: string;
    /** Максимальное количество точек на графике */
    maxPoints?: number;
  }

  let { 
    history = [],
    currentLatency,
    label = 'Latency',
    maxPoints = 30
  }: Props = $props();

  // Определяем цвет на основе текущего latency
  let chartColor = $derived(
    currentLatency === undefined ? '#6B7280' :
    currentLatency < 50 ? '#22C55E' :  // green
    currentLatency < 100 ? '#EAB308' : // yellow
    currentLatency < 200 ? '#F97316' : // orange
    '#EF4444'                          // red
  );

  // Статус на основе latency
  let statusText = $derived(
    currentLatency === undefined ? 'Нет данных' :
    currentLatency < 50 ? 'Отлично' :
    currentLatency < 100 ? 'Хорошо' :
    currentLatency < 200 ? 'Средне' :
    'Плохо'
  );

  let statusColor = $derived(
    currentLatency === undefined ? 'text-zinc-400' :
    currentLatency < 50 ? 'text-emerald-400' :
    currentLatency < 100 ? 'text-yellow-400' :
    currentLatency < 200 ? 'text-orange-400' :
    'text-red-400'
  );

  // Статистика
  let avgLatency = $derived(
    history.length > 0 
      ? Math.round(history.reduce((a, b) => a + b, 0) / history.length)
      : undefined
  );

  let minLatency = $derived(
    history.length > 0 ? Math.min(...history) : undefined
  );

  let maxLatency = $derived(
    history.length > 0 ? Math.max(...history) : undefined
  );
</script>

<div class="flex flex-col h-full gap-3">
  <!-- Header with current value -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">{label}</span>
      <span class="text-[10px] {statusColor} font-medium">{statusText}</span>
    </div>
    <div class="flex items-baseline gap-1">
      {#if currentLatency !== undefined}
        <span class="text-xl font-bold text-white font-mono">{currentLatency}</span>
        <span class="text-[10px] text-zinc-400">ms</span>
      {:else}
        <span class="text-xl font-bold text-zinc-600 font-mono">--</span>
        <span class="text-[10px] text-zinc-600">ms</span>
      {/if}
    </div>
  </div>

  <!-- Chart -->
  <div class="flex-1 min-h-[60px] rounded-lg bg-zinc-900/30 border border-white/5 p-2">
    {#if history.length > 0}
      <PingChart data={history} {maxPoints} height={56} color={chartColor} />
    {:else}
      <div class="h-full flex flex-col items-center justify-center gap-1">
        <span class="text-xs text-zinc-500">Нет данных</span>
        <span class="text-[10px] text-zinc-600">Запустите сканирование сервисов</span>
      </div>
    {/if}
  </div>

  <!-- Stats footer -->
  <div class="flex items-center justify-between px-1 text-[10px] text-zinc-400">
    <div class="flex items-center gap-3">
      {#if avgLatency !== undefined}
        <span>Avg: <span class="text-zinc-400 font-mono">{avgLatency}ms</span></span>
      {/if}
      {#if minLatency !== undefined}
        <span>Min: <span class="text-emerald-400/70 font-mono">{minLatency}ms</span></span>
      {/if}
      {#if maxLatency !== undefined}
        <span>Max: <span class="text-red-400/70 font-mono">{maxLatency}ms</span></span>
      {/if}
    </div>
    <span class="text-zinc-600">{history.length} samples</span>
  </div>
</div>
