<script lang="ts">
  interface Props {
    value: number; // 0-100
    label?: string;
    showPercent?: boolean;
    color?: 'cyan' | 'green' | 'red' | 'yellow';
  }

  let { 
    value, 
    label, 
    showPercent = false,
    color = 'cyan'
  }: Props = $props();

  const clampedValue = $derived(Math.min(100, Math.max(0, value)));

  const colorClasses = {
    cyan: 'bg-[#00d4ff]',
    green: 'bg-[#00ff88]',
    red: 'bg-[#ff3333]',
    yellow: 'bg-[#ffaa00]'
  };
</script>

<div class="w-full">
  {#if label || showPercent}
    <div class="flex justify-between items-center mb-2">
      {#if label}
        <span class="text-sm text-gray-300">{label}</span>
      {/if}
      {#if showPercent}
        <span class="text-sm text-gray-400">{Math.round(clampedValue)}%</span>
      {/if}
    </div>
  {/if}
  <div class="w-full h-2 bg-gray-700 rounded-full overflow-hidden">
    <div
      class="h-full {colorClasses[color]} rounded-full transition-all duration-300 ease-out"
      style="width: {clampedValue}%"
    ></div>
  </div>
</div>
