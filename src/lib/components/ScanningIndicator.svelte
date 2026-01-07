<script lang="ts">
  interface Props {
    active?: boolean;
    text?: string;
    variant?: 'dots' | 'bar' | 'pulse';
  }
  
  let { active = false, text = 'Scanning', variant = 'dots' }: Props = $props();
</script>

{#if active}
  <div class="flex items-center gap-2">
    {#if variant === 'dots'}
      <div class="flex gap-1">
        <span class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce" style="animation-delay: 0ms"></span>
        <span class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce" style="animation-delay: 150ms"></span>
        <span class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce" style="animation-delay: 300ms"></span>
      </div>
    {:else if variant === 'bar'}
      <div class="w-32 h-1 bg-zinc-800 rounded-full overflow-hidden">
        <div class="h-full bg-indigo-500 animate-scanning-bar"></div>
      </div>
    {:else if variant === 'pulse'}
      <div class="w-3 h-3 bg-indigo-500 rounded-full animate-ping"></div>
    {/if}
    <span class="text-sm text-zinc-400">{text}</span>
  </div>
{/if}

<style>
  @keyframes scanning-bar {
    0% { width: 0%; margin-left: 0%; }
    50% { width: 30%; margin-left: 35%; }
    100% { width: 0%; margin-left: 100%; }
  }
  .animate-scanning-bar {
    animation: scanning-bar 1.5s ease-in-out infinite;
  }
</style>
