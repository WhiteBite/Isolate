<script lang="ts">
  /**
   * Universal Skeleton loader component
   * Variants: text, circle, card, widget, button, custom
   */
  type SkeletonVariant = 'text' | 'circle' | 'card' | 'widget' | 'button' | 'custom';
  type RoundedSize = 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full';

  interface Props {
    /** Preset variant for common use cases */
    variant?: SkeletonVariant;
    /** Custom width (used with 'custom' variant or to override) */
    width?: string;
    /** Custom height (used with 'custom' variant or to override) */
    height?: string;
    /** Border radius */
    rounded?: RoundedSize;
    /** Enable shimmer animation */
    animate?: boolean;
    /** Number of text lines (for 'text' variant) */
    lines?: number;
    /** Additional CSS classes */
    class?: string;
  }

  let {
    variant = 'custom',
    width,
    height,
    rounded,
    animate = true,
    lines = 1,
    class: className = ''
  }: Props = $props();

  // Variant presets
  const variantPresets: Record<SkeletonVariant, { width: string; height: string; rounded: RoundedSize }> = {
    text: { width: '100%', height: '16px', rounded: 'md' },
    circle: { width: '40px', height: '40px', rounded: 'full' },
    card: { width: '100%', height: '120px', rounded: 'xl' },
    widget: { width: '100%', height: '100%', rounded: 'xl' },
    button: { width: '100px', height: '36px', rounded: 'lg' },
    custom: { width: '100%', height: '20px', rounded: 'md' }
  };

  const roundedClasses: Record<RoundedSize, string> = {
    none: 'rounded-none',
    sm: 'rounded-sm',
    md: 'rounded-md',
    lg: 'rounded-lg',
    xl: 'rounded-xl',
    '2xl': 'rounded-2xl',
    full: 'rounded-full'
  };

  // Computed values with overrides
  let preset = $derived(variantPresets[variant]);
  let finalWidth = $derived(width ?? preset.width);
  let finalHeight = $derived(height ?? preset.height);
  let finalRounded = $derived(rounded ?? preset.rounded);

  // Text variant line widths for natural look
  const lineWidths = ['100%', '95%', '85%', '90%', '75%'];
</script>

{#if variant === 'text' && lines > 1}
  <!-- Multi-line text skeleton -->
  <div class="flex flex-col gap-2 {className}" style="width: {finalWidth};">
    {#each Array(lines) as _, i}
      <div
        class="skeleton bg-zinc-800/60 {roundedClasses[finalRounded]}"
        class:animate-shimmer={animate}
        style="width: {lineWidths[i % lineWidths.length]}; height: {finalHeight};"
      ></div>
    {/each}
  </div>
{:else}
  <!-- Single skeleton element -->
  <div
    class="skeleton bg-zinc-800/60 {roundedClasses[finalRounded]} {className}"
    class:animate-shimmer={animate}
    style="width: {finalWidth}; height: {finalHeight};"
  ></div>
{/if}

<style>
  .skeleton {
    position: relative;
    overflow: hidden;
  }

  .animate-shimmer {
    background: linear-gradient(
      90deg,
      rgba(39, 39, 42, 0.6) 0%,
      rgba(63, 63, 70, 0.4) 50%,
      rgba(39, 39, 42, 0.6) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
  }

  @keyframes shimmer {
    0% {
      background-position: -200% 0;
    }
    100% {
      background-position: 200% 0;
    }
  }
</style>
