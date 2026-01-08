<script lang="ts">
  import { aiPilotStore } from '$lib/stores/aiPilot.svelte';

  // Props
  interface Props {
    onDetails?: () => void;
  }
  
  let { onDetails }: Props = $props();

  // Текущее уведомление
  let notification = $derived(aiPilotStore.currentNotification);
  let isVisible = $derived(notification !== null);

  // Автоскрытие через 10 секунд
  let hideTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (notification) {
      // Сбросить предыдущий таймер
      if (hideTimer) {
        clearTimeout(hideTimer);
      }
      
      // Установить новый таймер на 10 секунд
      hideTimer = setTimeout(() => {
        aiPilotStore.dismissNotification();
      }, 10000);
    }
    
    return () => {
      if (hideTimer) {
        clearTimeout(hideTimer);
      }
    };
  });

  function handleUndo() {
    if (notification) {
      aiPilotStore.undoAction(notification.id);
    }
  }

  function handleDismiss() {
    aiPilotStore.dismissNotification();
  }

  function handleDetails() {
    if (onDetails) {
      onDetails();
    }
    aiPilotStore.dismissNotification();
  }
</script>

{#if isVisible && notification}
  <div 
    class="fixed bottom-6 right-6 z-50 max-w-sm w-full
           animate-in slide-in-from-bottom-4 fade-in duration-300"
    role="alert"
    aria-live="polite"
  >
    <div 
      class="relative p-4 rounded-2xl shadow-2xl
             bg-gradient-to-br from-slate-800 to-slate-900
             border border-white/10 backdrop-blur-xl"
    >
      <!-- Close button -->
      <button
        type="button"
        class="absolute top-3 right-3 p-1 rounded-lg
               text-white/40 hover:text-white hover:bg-white/10
               transition-colors"
        onclick={handleDismiss}
        aria-label="Закрыть уведомление"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>

      <!-- Header -->
      <div class="flex items-start gap-3 pr-6">
        <div 
          class="flex-shrink-0 w-10 h-10 rounded-xl
                 bg-blue-500/20 flex items-center justify-center"
        >
          <span class="text-xl">🤖</span>
        </div>
        
        <div class="flex-1 min-w-0">
          <div class="font-semibold text-white">
            AI Pilot
          </div>
          <div class="text-sm text-white/60 mt-0.5">
            Стратегия переключена
          </div>
        </div>
      </div>

      <!-- Content -->
      <div class="mt-4 p-3 rounded-xl bg-white/5">
        <div class="flex items-center gap-2 text-sm">
          <span class="text-white/80">{notification.serviceName}</span>
          <span class="text-white/30">:</span>
          <span class="text-white/50 line-through">{notification.oldStrategy}</span>
          <span class="text-white/30">→</span>
          <span class="text-blue-400 font-medium">{notification.newStrategy}</span>
        </div>
        
        <div class="text-xs text-white/40 mt-2">
          {notification.reason}
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-2 mt-4">
        <button
          type="button"
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium
                 bg-white/10 hover:bg-white/15 text-white
                 transition-colors"
          onclick={handleUndo}
        >
          Отменить
        </button>
        
        <button
          type="button"
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium
                 bg-blue-500 hover:bg-blue-400 text-white
                 transition-colors"
          onclick={handleDetails}
        >
          Подробнее
        </button>
      </div>

      <!-- Progress bar (auto-hide indicator) -->
      <div class="absolute bottom-0 left-0 right-0 h-1 rounded-b-2xl overflow-hidden">
        <div 
          class="h-full bg-blue-500/50 animate-shrink"
          style="animation-duration: 10s; animation-timing-function: linear;"
        ></div>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes slideInFromBottom {
    from { 
      opacity: 0;
      transform: translateY(1rem); 
    }
    to { 
      opacity: 1;
      transform: translateY(0); 
    }
  }
  
  @keyframes shrink {
    from { width: 100%; }
    to { width: 0%; }
  }
  
  .animate-in {
    animation: slideInFromBottom 0.3s ease-out;
  }
  
  .slide-in-from-bottom-4 {
    animation-name: slideInFromBottom;
  }
  
  .fade-in {
    animation-name: slideInFromBottom;
  }
  
  .duration-300 {
    animation-duration: 300ms;
  }
  
  .animate-shrink {
    animation-name: shrink;
  }
</style>
