<script lang="ts">
  import { goto } from '$app/navigation';

  interface Props {
    onRetry: () => Promise<void>;
  }

  let { onRetry }: Props = $props();

  let isRetrying = $state(false);

  async function handleRetry() {
    isRetrying = true;
    try {
      await onRetry();
    } finally {
      isRetrying = false;
    }
  }

  function goToLogs() {
    goto('/logs');
  }
</script>

<div class="flex flex-col items-center justify-center h-full min-h-[400px] p-8">
  <!-- Error Icon -->
  <div class="relative mb-6">
    <div class="w-24 h-24 rounded-full bg-neon-red/10 flex items-center justify-center">
      <svg 
        class="w-12 h-12 text-neon-red" 
        fill="none" 
        stroke="currentColor" 
        viewBox="0 0 24 24"
      >
        <path 
          stroke-linecap="round" 
          stroke-linejoin="round" 
          stroke-width="2" 
          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
        />
      </svg>
    </div>
    <!-- Pulse ring -->
    <div class="absolute inset-0 rounded-full bg-neon-red/20 animate-ping opacity-75"></div>
  </div>

  <!-- Title -->
  <h2 class="text-xl font-semibold text-white mb-2">
    Backend не готов
  </h2>

  <!-- Description -->
  <p class="text-sm text-zinc-400 text-center max-w-md mb-6">
    Не удалось подключиться к backend-сервису. Это может быть вызвано долгой инициализацией 
    или ошибкой при запуске.
  </p>

  <!-- Actions -->
  <div class="flex flex-col sm:flex-row gap-3">
    <!-- Retry Button -->
    <button
      onclick={handleRetry}
      disabled={isRetrying}
      class="px-6 py-2.5 bg-electric hover:bg-electric/90 disabled:bg-electric/50 
             text-white font-medium rounded-lg transition-all duration-200
             flex items-center justify-center gap-2 min-w-[140px]"
    >
      {#if isRetrying}
        <!-- Spinner -->
        <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle 
            class="opacity-25" 
            cx="12" 
            cy="12" 
            r="10" 
            stroke="currentColor" 
            stroke-width="4"
          />
          <path 
            class="opacity-75" 
            fill="currentColor" 
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
        <span>Подключение...</span>
      {:else}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width="2" 
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
        <span>Повторить</span>
      {/if}
    </button>

    <!-- Logs Link -->
    <button
      onclick={goToLogs}
      class="px-6 py-2.5 bg-void-100 hover:bg-void-200 
             text-zinc-300 font-medium rounded-lg transition-all duration-200
             flex items-center justify-center gap-2"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path 
          stroke-linecap="round" 
          stroke-linejoin="round" 
          stroke-width="2" 
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
        />
      </svg>
      <span>Смотреть логи</span>
    </button>
  </div>

  <!-- Help text -->
  <p class="text-xs text-zinc-400 mt-6 text-center">
    Если проблема повторяется, проверьте логи или перезапустите приложение
  </p>
</div>
