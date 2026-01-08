<script lang="ts">
  import { aiPilotStore, type AIPilotAction } from '$lib/stores/aiPilot.svelte';

  // Derived состояния
  let isEnabled = $derived(aiPilotStore.enabled);
  let currentInterval = $derived(aiPilotStore.interval);
  let isChecking = $derived(aiPilotStore.isChecking);
  let lastCheck = $derived(aiPilotStore.lastCheck);
  let history = $derived(aiPilotStore.recentHistory);

  // Опции интервала
  const intervalOptions: { value: 30 | 60 | 120; label: string }[] = [
    { value: 30, label: '30 мин' },
    { value: 60, label: '1 час' },
    { value: 120, label: '2 часа' }
  ];

  function formatTime(date: Date): string {
    return date.toLocaleTimeString('ru-RU', { 
      hour: '2-digit', 
      minute: '2-digit' 
    });
  }

  function formatDate(date: Date): string {
    const today = new Date();
    const isToday = date.toDateString() === today.toDateString();
    
    if (isToday) {
      return `Сегодня, ${formatTime(date)}`;
    }
    
    return date.toLocaleDateString('ru-RU', {
      day: 'numeric',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function handleToggle() {
    aiPilotStore.toggle();
  }

  function handleIntervalChange(value: 30 | 60 | 120) {
    aiPilotStore.setInterval(value);
  }

  function handleManualCheck() {
    aiPilotStore.runCheck();
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h3 class="text-lg font-semibold text-white flex items-center gap-2">
        <span class="text-xl">🤖</span>
        AI Pilot
      </h3>
      <p class="text-sm text-white/50 mt-1">
        Автоматическая оптимизация стратегий
      </p>
    </div>
    
    <!-- Status badge -->
    <div 
      class="px-3 py-1.5 rounded-full text-sm font-medium
             {isEnabled ? 'bg-green-500/20 text-green-400' : 'bg-white/10 text-white/50'}"
    >
      {#if isChecking}
        <span class="flex items-center gap-2">
          <span class="w-2 h-2 bg-blue-400 rounded-full animate-pulse"></span>
          Проверка...
        </span>
      {:else if isEnabled}
        <span class="flex items-center gap-2">
          <span class="w-2 h-2 bg-green-400 rounded-full"></span>
          Активен
        </span>
      {:else}
        Выключен
      {/if}
    </div>
  </div>

  <!-- Main toggle -->
  <div 
    class="p-4 rounded-xl bg-white/5 border border-white/10
           hover:bg-white/[0.07] transition-colors"
  >
    <div class="flex items-center justify-between">
      <div>
        <div class="font-medium text-white">Фоновая оптимизация</div>
        <div class="text-sm text-white/50 mt-0.5">
          Автоматически переключать стратегии при проблемах
        </div>
      </div>
      
      <!-- Toggle switch -->
      <button
        type="button"
        role="switch"
        aria-checked={isEnabled}
        class="relative w-14 h-8 rounded-full transition-colors duration-200
               {isEnabled ? 'bg-blue-500' : 'bg-white/20'}
               focus:outline-none focus:ring-2 focus:ring-blue-500/50"
        onclick={handleToggle}
      >
        <span 
          class="absolute top-1 left-1 w-6 h-6 rounded-full bg-white shadow-lg
                 transition-transform duration-200
                 {isEnabled ? 'translate-x-6' : 'translate-x-0'}"
        ></span>
      </button>
    </div>
  </div>

  <!-- Interval selector -->
  {#if isEnabled}
    <div class="p-4 rounded-xl bg-white/5 border border-white/10">
      <div class="text-sm font-medium text-white/80 mb-3">
        Интервал проверки
      </div>
      
      <div class="flex gap-2">
        {#each intervalOptions as option (option.value)}
          <button
            type="button"
            class="flex-1 py-2 px-3 rounded-lg text-sm font-medium
                   transition-all duration-200
                   {currentInterval === option.value 
                     ? 'bg-blue-500 text-white' 
                     : 'bg-white/10 text-white/60 hover:bg-white/15 hover:text-white'}"
            onclick={() => handleIntervalChange(option.value)}
          >
            {option.label}
          </button>
        {/each}
      </div>
      
      {#if lastCheck}
        <div class="mt-3 text-xs text-white/40">
          Последняя проверка: {formatDate(lastCheck)}
        </div>
      {/if}
    </div>

    <!-- Manual check button -->
    <button
      type="button"
      class="w-full py-3 px-4 rounded-xl font-medium
             bg-white/10 hover:bg-white/15 text-white
             border border-white/10 hover:border-white/20
             disabled:opacity-50 disabled:cursor-not-allowed
             transition-all duration-200"
      onclick={handleManualCheck}
      disabled={isChecking}
    >
      {#if isChecking}
        <span class="flex items-center justify-center gap-2">
          <span class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
          Проверяю...
        </span>
      {:else}
        Проверить сейчас
      {/if}
    </button>
  {/if}

  <!-- History log -->
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h4 class="text-sm font-medium text-white/80">
        История действий
      </h4>
      {#if history.length > 0}
        <button
          type="button"
          class="text-xs text-white/40 hover:text-white/60 transition-colors"
          onclick={() => aiPilotStore.clearHistory()}
        >
          Очистить
        </button>
      {/if}
    </div>
    
    {#if history.length === 0}
      <div class="p-6 rounded-xl bg-white/5 border border-white/10 text-center">
        <div class="text-2xl mb-2">📋</div>
        <div class="text-sm text-white/50">
          {#if isEnabled}
            Пока нет действий. AI Pilot сообщит о переключениях.
          {:else}
            Включите AI Pilot для автоматической оптимизации.
          {/if}
        </div>
      </div>
    {:else}
      <div class="space-y-2 max-h-64 overflow-y-auto">
        {#each history as action (action.id)}
          <div 
            class="p-3 rounded-lg bg-white/5 border border-white/10
                   hover:bg-white/[0.07] transition-colors"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="font-medium text-white text-sm">
                    {action.serviceName}
                  </span>
                  <span class="text-white/30">→</span>
                  <span class="text-blue-400 text-sm">
                    {action.newStrategy}
                  </span>
                </div>
                <div class="text-xs text-white/40 mt-1 line-clamp-1">
                  {action.reason}
                </div>
              </div>
              
              <div class="text-xs text-white/30 whitespace-nowrap">
                {formatTime(action.timestamp)}
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
