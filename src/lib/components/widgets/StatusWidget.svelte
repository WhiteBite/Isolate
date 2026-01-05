<script lang="ts">
  import BigToggleButton from '../BigToggleButton.svelte';

  interface Props {
    active?: boolean;
    loading?: boolean;
    onToggle?: () => void;
  }

  let { 
    active = false, 
    loading = false,
    onToggle 
  }: Props = $props();

  function handleClick() {
    onToggle?.();
  }

  const statusText = $derived(
    loading 
      ? 'Connecting...' 
      : active 
        ? 'Protection Active' 
        : 'Protection Disabled'
  );

  const statusColor = $derived(
    loading 
      ? 'text-neon-yellow' 
      : active 
        ? 'text-neon-green' 
        : 'text-text-muted'
  );
</script>

<div class="flex flex-col items-center justify-center h-full gap-4 relative">
  <!-- Glow effect when active -->
  {#if active && !loading}
    <div 
      class="absolute inset-0 rounded-xl bg-neon-green/5 blur-2xl animate-pulse-glow pointer-events-none"
    ></div>
  {/if}

  <div class="relative z-10">
    <BigToggleButton 
      {active} 
      {loading} 
      onClick={handleClick}
    />
  </div>

  <div class="text-center relative z-10">
    <p class="text-sm font-medium {statusColor} transition-colors duration-300">
      {statusText}
    </p>
    {#if active && !loading}
      <p class="text-xs text-text-muted mt-1">
        All traffic is protected
      </p>
    {/if}
  </div>
</div>

<style>
  @keyframes glow-pulse {
    0%, 100% {
      opacity: 0.3;
    }
    50% {
      opacity: 0.6;
    }
  }
</style>
