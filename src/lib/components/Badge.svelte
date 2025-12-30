<script lang="ts">
  interface Props {
    variant?: 'active' | 'inactive' | 'error' | 'warning' | 'info';
    size?: 'sm' | 'md';
    dot?: boolean;
    label: string;
  }

  let { 
    variant = 'info', 
    size = 'md',
    dot = false,
    label 
  }: Props = $props();

  const variantClasses = {
    active: 'bg-green-500/10 text-green-400 border-green-500/30',
    inactive: 'bg-gray-500/10 text-gray-400 border-gray-500/30',
    error: 'bg-red-500/10 text-red-400 border-red-500/30',
    warning: 'bg-yellow-500/10 text-yellow-400 border-yellow-500/30',
    info: 'bg-cyan-500/10 text-cyan-400 border-cyan-500/30'
  };

  const dotColors = {
    active: 'bg-green-400',
    inactive: 'bg-gray-400',
    error: 'bg-red-400',
    warning: 'bg-yellow-400',
    info: 'bg-cyan-400'
  };

  const sizeClasses = {
    sm: 'px-2 py-0.5 text-xs',
    md: 'px-2.5 py-1 text-sm'
  };
</script>

<span class="inline-flex items-center gap-1.5 font-medium rounded-full border {variantClasses[variant]} {sizeClasses[size]}">
  {#if dot}
    <span class="w-1.5 h-1.5 rounded-full {dotColors[variant]}"></span>
  {/if}
  {label}
</span>
