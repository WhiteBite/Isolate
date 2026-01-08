<script lang="ts">
  interface Props {
    latency: number | null;
    testing?: boolean;
    showValue?: boolean;
  }
  
  let { latency, testing = false, showValue = true }: Props = $props();
  
  // Determine color based on latency
  let colorClass = $derived.by(() => {
    if (testing) return 'text-gray-400';
    if (latency === null) return 'text-gray-500';
    if (latency < 100) return 'text-green-400';
    if (latency < 300) return 'text-yellow-400';
    return 'text-red-400';
  });
  
  let dotColorClass = $derived.by(() => {
    if (testing) return 'bg-gray-400';
    if (latency === null) return 'bg-gray-500';
    if (latency < 100) return 'bg-green-400';
    if (latency < 300) return 'bg-yellow-400';
    return 'bg-red-400';
  });
  
  let displayValue = $derived.by(() => {
    if (testing) return 'Testing...';
    if (latency === null) return '—';
    return `${latency}ms`;
  });
</script>

<div class="inline-flex items-center gap-1.5 {colorClass}">
  <!-- Animated dot -->
  <span class="relative flex h-2 w-2">
    {#if testing}
      <span class="animate-ping absolute inline-flex h-full w-full rounded-full {dotColorClass} opacity-75"></span>
    {/if}
    <span class="relative inline-flex rounded-full h-2 w-2 {dotColorClass}"></span>
  </span>
  
  {#if showValue}
    <span class="text-sm font-medium tabular-nums">
      {displayValue}
    </span>
  {/if}
</div>
