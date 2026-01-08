<script lang="ts">
  import type { ProtectionStatus } from '$lib/stores/dashboard.svelte';

  interface Props {
    status: ProtectionStatus;
    onclick?: () => void;
  }

  let { status, onclick }: Props = $props();

  const statusConfig = $derived({
    protected: {
      color: 'from-emerald-500 to-green-600',
      ringColor: 'ring-emerald-400/50',
      shadowColor: 'shadow-emerald-500/30',
      icon: '🛡️',
      label: 'Защита активна',
      animation: 'animate-pulse-slow'
    },
    bypassing: {
      color: 'from-amber-500 to-yellow-600',
      ringColor: 'ring-amber-400/50',
      shadowColor: 'shadow-amber-500/30',
      icon: '⚡',
      label: 'Обход DPI',
      animation: 'animate-wave'
    },
    issues: {
      color: 'from-red-500 to-rose-600',
      ringColor: 'ring-red-400/50',
      shadowColor: 'shadow-red-500/30',
      icon: '⚠️',
      label: 'Есть проблемы',
      animation: 'animate-shake'
    },
    disabled: {
      color: 'from-slate-500 to-gray-600',
      ringColor: 'ring-slate-400/30',
      shadowColor: 'shadow-slate-500/20',
      icon: '○',
      label: 'Отключено',
      animation: ''
    }
  }[status]);
</script>

<button
  type="button"
  class="relative group cursor-pointer focus:outline-none focus-visible:ring-4 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900 {statusConfig.ringColor} rounded-full transition-transform hover:scale-105 active:scale-95"
  {onclick}
>
  <!-- Outer glow -->
  <div 
    class="absolute inset-0 rounded-full bg-gradient-to-br {statusConfig.color} opacity-20 blur-xl scale-110 {statusConfig.animation}"
  ></div>
  
  <!-- Main shield circle -->
  <div 
    class="relative w-48 h-48 rounded-full bg-gradient-to-br {statusConfig.color} shadow-2xl {statusConfig.shadowColor} flex items-center justify-center ring-4 {statusConfig.ringColor} {statusConfig.animation}"
  >
    <!-- Inner circle -->
    <div class="absolute inset-4 rounded-full bg-slate-900/40 backdrop-blur-sm"></div>
    
    <!-- Icon -->
    <span class="relative text-6xl select-none">{statusConfig.icon}</span>
  </div>
  
  <!-- Status label -->
  <div class="absolute -bottom-8 left-1/2 -translate-x-1/2 whitespace-nowrap">
    <span class="text-sm font-medium text-slate-300">{statusConfig.label}</span>
  </div>
</button>

<style>
  @keyframes pulse-slow {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.85;
      transform: scale(1.02);
    }
  }

  @keyframes wave {
    0%, 100% {
      transform: scale(1);
    }
    25% {
      transform: scale(1.03) rotate(1deg);
    }
    75% {
      transform: scale(1.03) rotate(-1deg);
    }
  }

  @keyframes shake {
    0%, 100% {
      transform: translateX(0);
    }
    10%, 30%, 50%, 70%, 90% {
      transform: translateX(-2px);
    }
    20%, 40%, 60%, 80% {
      transform: translateX(2px);
    }
  }

  :global(.animate-pulse-slow) {
    animation: pulse-slow 3s ease-in-out infinite;
  }

  :global(.animate-wave) {
    animation: wave 2s ease-in-out infinite;
  }

  :global(.animate-shake) {
    animation: shake 0.8s ease-in-out infinite;
  }
</style>
