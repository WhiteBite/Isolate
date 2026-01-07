<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  interface Props {
    context: PluginContext;
  }
  
  let { context }: Props = $props();
  
  // ============================================
  // State
  // ============================================
  
  let value = $state(0);
  let loading = $state(false);
  let lastUpdate = $state<Date | null>(null);
  let history = $state<{ value: number; time: Date }[]>([]);
  
  // ============================================
  // Lifecycle
  // ============================================
  
  // Загрузка данных при монтировании
  $effect(() => {
    loadSavedData();
    
    // Подписка на события
    const unsubscribe = context.events.on('status-changed', handleStatusChange);
    
    // Cleanup при размонтировании
    return () => unsubscribe();
  });
  
  // ============================================
  // Data Management
  // ============================================
  
  async function loadSavedData() {
    loading = true;
    try {
      const saved = await context.storage.get<typeof history>('widget-history');
      if (saved) {
        history = saved.slice(-10);
        if (history.length > 0) {
          value = history[history.length - 1].value;
          lastUpdate = new Date(history[history.length - 1].time);
        }
      }
    } catch (error) {
      console.error('Failed to load saved data:', error);
    } finally {
      loading = false;
    }
  }
  
  async function saveData() {
    try {
      await context.storage.set('widget-history', history.slice(-10));
    } catch (error) {
      console.error('Failed to save data:', error);
    }
  }
  
  // ============================================
  // Actions
  // ============================================
  
  async function refresh() {
    if (loading) return;
    
    loading = true;
    context.events.emit('my-widget-refresh-started');
    
    try {
      // Пример: получение данных через HTTP
      // const response = await context.http.get('https://api.example.com/data');
      // const data = await response.json();
      // value = data.value;
      
      // Для демонстрации: случайное значение
      value = Math.floor(Math.random() * 100);
      lastUpdate = new Date();
      
      // Сохранение в историю
      history = [...history, { value, time: lastUpdate }].slice(-10);
      await saveData();
      
      context.events.emit('my-widget-refresh-completed', { value });
    } catch (error) {
      context.events.emit('my-widget-refresh-error', { error: String(error) });
    } finally {
      loading = false;
    }
  }
  
  function handleStatusChange(data: any) {
    // Обработка события изменения статуса
    console.log('Status changed:', data);
  }
  
  // ============================================
  // Helpers
  // ============================================
  
  function getValueColor(val: number): string {
    if (val >= 80) return 'text-emerald-400';
    if (val >= 50) return 'text-cyan-400';
    if (val >= 20) return 'text-amber-400';
    return 'text-red-400';
  }
  
  function formatTime(date: Date | null): string {
    if (!date) return '—';
    return date.toLocaleTimeString();
  }
</script>

<!-- ============================================ -->
<!-- Template -->
<!-- ============================================ -->

<div class="widget p-4 bg-zinc-900/40 backdrop-blur-md border border-white/5 rounded-xl">
  <!-- Header -->
  <div class="flex items-center justify-between mb-4">
    <div class="flex items-center gap-2">
      <span class="text-lg">📊</span>
      <h3 class="text-xs text-zinc-400 uppercase tracking-wider font-semibold">
        My Widget
      </h3>
    </div>
    
    <button
      onclick={refresh}
      disabled={loading}
      class="px-3 py-1.5 text-xs font-medium rounded-lg transition-all
        {loading 
          ? 'bg-zinc-800 text-zinc-500 cursor-not-allowed' 
          : 'bg-cyan-500/20 text-cyan-400 hover:bg-cyan-500/30 border border-cyan-500/30'}"
    >
      {loading ? 'Loading...' : 'Refresh'}
    </button>
  </div>
  
  <!-- Main Content -->
  <div class="text-center py-4">
    <div class="text-4xl font-bold {getValueColor(value)}">
      {value}
    </div>
    <div class="text-xs text-zinc-500 mt-1">
      Current Value
    </div>
  </div>
  
  <!-- Progress indicator -->
  {#if loading}
    <div class="mt-4">
      <div class="h-1 bg-zinc-800 rounded-full overflow-hidden">
        <div 
          class="h-full bg-gradient-to-r from-cyan-500 to-emerald-500 animate-pulse" 
          style="width: 100%"
        ></div>
      </div>
    </div>
  {/if}
  
  <!-- Footer -->
  {#if lastUpdate && !loading}
    <div class="mt-4 pt-3 border-t border-white/5">
      <p class="text-[10px] text-zinc-500 text-center">
        Last updated: {formatTime(lastUpdate)}
      </p>
    </div>
  {/if}
</div>

<!-- ============================================ -->
<!-- Styles -->
<!-- ============================================ -->

<style>
  .widget {
    min-height: 150px;
  }
</style>
