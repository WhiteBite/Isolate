<script lang="ts">
  import { badgeVariants, badgeDotColors, type BadgeVariant } from '$lib/styles/theme';

  interface Props {
    variant?: BadgeVariant;
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

  const sizeClasses = {
    sm: 'px-2 py-0.5 text-xs',
    md: 'px-2.5 py-1 text-sm'
  };
</script>

<span class="inline-flex items-center gap-1.5 font-medium rounded-full border {badgeVariants[variant]} {sizeClasses[size]}">
  {#if dot}
    <span class="w-1.5 h-1.5 rounded-full {badgeDotColors[variant]}"></span>
  {/if}
  {label}
</span>
