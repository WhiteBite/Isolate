<script lang="ts">
  import { onMount } from 'svelte';
  
  let status = $state<'idle' | 'active' | 'optimizing'>('idle');
  let currentStrategy = $state<string | null>(null);

  onMount(() => {
    // TODO: Load initial state from Tauri
  });

  async function handleOptimize() {
    status = 'optimizing';
    // TODO: Call Tauri command
  }

  async function handleToggle() {
    // TODO: Toggle strategy on/off
  }
</script>

<div class="flex flex-col items-center justify-center min-h-screen p-8">
  <div class="w-full max-w-md space-y-8">
    <!-- Header -->
    <div class="text-center">
      <h1 class="text-4xl font-bold text-primary-400">Isolate</h1>
      <p class="mt-2 text-gray-400">Автоматический обход DPI-блокировок</p>
    </div>

    <!-- Status Card -->
    <div class="bg-gray-800 rounded-2xl p-6 space-y-4">
      <div class="flex items-center justify-between">
        <span class="text-gray-400">Статус</span>
        <span class="flex items-center gap-2">
          {#if status === 'active'}
            <span class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></span>
            <span class="text-green-400">Активен</span>
          {:else if status === 'optimizing'}
            <span class="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></span>
            <span class="text-yellow-400">Оптимизация...</span>
          {:else}
            <span class="w-2 h-2 bg-gray-500 rounded-full"></span>
            <span class="text-gray-400">Неактивен</span>
          {/if}
        </span>
      </div>

      {#if currentStrategy}
        <div class="flex items-center justify-between">
          <span class="text-gray-400">Стратегия</span>
          <span class="text-white">{currentStrategy}</span>
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class="space-y-3">
      <button
        onclick={handleOptimize}
        disabled={status === 'optimizing'}
        class="w-full py-4 px-6 bg-primary-600 hover:bg-primary-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-xl font-medium transition-colors"
      >
        {status === 'optimizing' ? 'Оптимизация...' : 'Оптимизировать'}
      </button>

      {#if status === 'active'}
        <button
          onclick={handleToggle}
          class="w-full py-3 px-6 bg-gray-700 hover:bg-gray-600 rounded-xl font-medium transition-colors"
        >
          Отключить
        </button>
      {/if}
    </div>

    <!-- Services -->
    <div class="bg-gray-800 rounded-2xl p-6">
      <h2 class="text-lg font-medium mb-4">Сервисы</h2>
      <div class="space-y-3">
        {#each ['Discord', 'YouTube', 'Telegram'] as service}
          <div class="flex items-center justify-between">
            <span>{service}</span>
            <span class="w-2 h-2 bg-gray-500 rounded-full"></span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>
