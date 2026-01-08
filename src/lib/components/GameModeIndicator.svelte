<script lang="ts">
  interface Props {
    isActive: boolean;
    gameName?: string;
    compact?: boolean;
  }

  let { isActive, gameName, compact = false }: Props = $props();
</script>

{#if isActive}
  <div
    class="game-mode-indicator flex items-center gap-2 rounded-full transition-all duration-300 bg-purple-500/20 border border-purple-500/30"
    class:px-3={!compact}
    class:py-1.5={!compact}
    class:p-2={compact}
    title="Game Mode: {gameName || 'Активен'}"
    role="status"
    aria-label="Игровой режим {gameName ? `для ${gameName}` : 'активен'}"
  >
    <!-- Pulsing glow effect -->
    <div class="absolute inset-0 rounded-full bg-purple-500/20 animate-pulse-slow"></div>
    
    <!-- Icon with animation -->
    <span class="relative text-lg animate-bounce-subtle" aria-hidden="true">🎮</span>
    
    {#if !compact && gameName}
      <span class="relative text-sm font-medium text-purple-300 truncate max-w-[120px]">
        {gameName}
      </span>
    {/if}

    <!-- Active indicator dot -->
    <span class="relative flex h-2 w-2" aria-hidden="true">
      <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-purple-400 opacity-75"></span>
      <span class="relative inline-flex rounded-full h-2 w-2 bg-purple-500"></span>
    </span>
  </div>
{/if}

<style>
  .game-mode-indicator {
    position: relative;
    overflow: hidden;
  }

  @keyframes pulse-slow {
    0%, 100% {
      opacity: 0.2;
    }
    50% {
      opacity: 0.4;
    }
  }

  @keyframes bounce-subtle {
    0%, 100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-2px);
    }
  }

  .animate-pulse-slow {
    animation: pulse-slow 3s ease-in-out infinite;
  }

  .animate-bounce-subtle {
    animation: bounce-subtle 2s ease-in-out infinite;
  }
</style>
