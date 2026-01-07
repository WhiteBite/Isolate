<script lang="ts">
  interface Props {
    currentStrategyName: string | null;
    isActive: boolean;
    onStop: () => void;
  }

  let { currentStrategyName, isActive, onStop }: Props = $props();
</script>

<div class="flex items-center justify-between">
  <div>
    <h1 class="text-3xl font-bold text-text-primary">Strategies</h1>
    <p class="text-text-secondary mt-1">Manage bypass strategies</p>
  </div>
  
  {#if isActive && currentStrategyName}
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-2 px-4 py-2 bg-neon-green/10 rounded-xl border border-neon-green/20">
        <div class="w-2 h-2 rounded-full bg-neon-green animate-pulse shadow-glow-green"></div>
        <span class="text-neon-green font-medium">{currentStrategyName}</span>
      </div>
      <button
        onclick={onStop}
        class="px-4 py-2 bg-void-100 hover:bg-void-200 text-text-primary rounded-xl transition-all duration-200 border border-glass-border"
      >
        Disable
      </button>
    </div>
  {/if}
</div>
