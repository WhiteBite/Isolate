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
    active: 'bg-[#00ff88]/10 text-[#00ff88] border-[#00ff88]/30',
    inactive: 'bg-[#a0a0a0]/10 text-[#a0a0a0] border-[#a0a0a0]/30',
    error: 'bg-[#ff3333]/10 text-[#ff3333] border-[#ff3333]/30',
    warning: 'bg-[#ffaa00]/10 text-[#ffaa00] border-[#ffaa00]/30',
    info: 'bg-[#00d4ff]/10 text-[#00d4ff] border-[#00d4ff]/30'
  };

  const dotColors = {
    active: 'bg-[#00ff88]',
    inactive: 'bg-[#a0a0a0]',
    error: 'bg-[#ff3333]',
    warning: 'bg-[#ffaa00]',
    info: 'bg-[#00d4ff]'
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
