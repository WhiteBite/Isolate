<script lang="ts">
  import { page } from '$app/stores';
  import { fly, fade, scale, blur } from 'svelte/transition';
  import type { Snippet } from 'svelte';
  import type { TransitionVariant, StaggerConfig } from './page-transition-types';
  
  interface Props {
    children: Snippet;
    /** Animation variant */
    variant?: TransitionVariant;
    /** Animation duration in ms */
    duration?: number;
    /** Slide distance in pixels (for slide variants) */
    slideDistance?: number;
    /** Scale factor (for scale/zoom variants, 0-1) */
    scaleFactor?: number;
    /** Blur amount in pixels (for blur variant) */
    blurAmount?: number;
    /** Stagger configuration for child animations */
    stagger?: StaggerConfig;
  }
  
  let { 
    children,
    variant = 'slide',
    duration = 280,
    slideDistance = 12,
    scaleFactor = 0.96,
    blurAmount = 4,
    stagger
  }: Props = $props();
  
  // Custom easing functions for smoother animations
  // Smooth deceleration curve
  const easeOutQuart = (t: number) => 1 - Math.pow(1 - t, 4);
  // Smooth acceleration-deceleration
  const easeInOutCubic = (t: number) => t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  // Slight overshoot for playful feel
  const easeOutBack = (t: number) => {
    const c1 = 1.70158;
    const c3 = c1 + 1;
    return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
  };
  
  // Key for re-render on page change
  let key = $derived($page.url.pathname);
  
  // Compute transition parameters based on variant
  let inTransition = $derived.by(() => {
    const baseDelay = 40;
    
    switch (variant) {
      case 'fade':
        return {
          transition: fade,
          params: { 
            duration, 
            delay: baseDelay,
            easing: easeOutQuart 
          }
        };
      
      case 'slide':
        return {
          transition: fly,
          params: { 
            x: slideDistance, 
            y: 0,
            duration, 
            delay: baseDelay,
            easing: easeOutQuart 
          }
        };
      
      case 'slide-up':
        return {
          transition: fly,
          params: { 
            y: slideDistance, 
            duration, 
            delay: baseDelay,
            easing: easeOutQuart 
          }
        };
      
      case 'slide-down':
        return {
          transition: fly,
          params: { 
            y: -slideDistance, 
            duration, 
            delay: baseDelay,
            easing: easeOutQuart 
          }
        };
      
      case 'scale':
        return {
          transition: scale,
          params: { 
            start: scaleFactor, 
            duration, 
            delay: baseDelay,
            easing: easeOutBack 
          }
        };
      
      case 'zoom':
        return {
          transition: scale,
          params: { 
            start: scaleFactor * 0.9, 
            opacity: 0,
            duration: duration * 1.1, 
            delay: baseDelay,
            easing: easeOutBack 
          }
        };
      
      case 'blur':
        return {
          transition: blur,
          params: { 
            amount: blurAmount, 
            duration: duration * 1.2, 
            delay: baseDelay,
            easing: easeInOutCubic 
          }
        };
      
      default:
        return {
          transition: fly,
          params: { 
            y: slideDistance, 
            duration, 
            delay: baseDelay,
            easing: easeOutQuart 
          }
        };
    }
  });
  
  let outTransition = $derived.by(() => {
    const outDuration = Math.floor(duration * 0.5);
    
    switch (variant) {
      case 'blur':
        return {
          transition: blur,
          params: { 
            amount: blurAmount * 0.5, 
            duration: outDuration,
            easing: easeInOutCubic 
          }
        };
      
      case 'scale':
      case 'zoom':
        return {
          transition: scale,
          params: { 
            start: scaleFactor, 
            duration: outDuration,
            easing: easeInOutCubic 
          }
        };
      
      default:
        return {
          transition: fade,
          params: { 
            duration: outDuration,
            easing: easeInOutCubic 
          }
        };
    }
  });
  
  // CSS custom properties for stagger animation
  let staggerStyles = $derived.by(() => {
    if (!stagger?.enabled) return '';
    const delay = stagger.delay ?? 50;
    return `--stagger-delay: ${delay}ms;`;
  });
  
  let wrapperClass = $derived(
    `page-transition-wrapper${stagger?.enabled ? ' stagger-children' : ''}`
  );
</script>

{#key key}
  <div 
    in:inTransition.transition={inTransition.params}
    out:outTransition.transition={outTransition.params}
    class={wrapperClass}
    style={staggerStyles}
  >
    {@render children()}
  </div>
{/key}

<style>
  .page-transition-wrapper {
    height: 100%;
    width: 100%;
  }
  
  /* Staggered animation for child elements */
  .stagger-children :global(> *) {
    animation: stagger-fade-in 0.3s ease-out backwards;
  }
  
  .stagger-children :global(> *:nth-child(1)) { animation-delay: calc(var(--stagger-delay, 50ms) * 0); }
  .stagger-children :global(> *:nth-child(2)) { animation-delay: calc(var(--stagger-delay, 50ms) * 1); }
  .stagger-children :global(> *:nth-child(3)) { animation-delay: calc(var(--stagger-delay, 50ms) * 2); }
  .stagger-children :global(> *:nth-child(4)) { animation-delay: calc(var(--stagger-delay, 50ms) * 3); }
  .stagger-children :global(> *:nth-child(5)) { animation-delay: calc(var(--stagger-delay, 50ms) * 4); }
  .stagger-children :global(> *:nth-child(6)) { animation-delay: calc(var(--stagger-delay, 50ms) * 5); }
  .stagger-children :global(> *:nth-child(7)) { animation-delay: calc(var(--stagger-delay, 50ms) * 6); }
  .stagger-children :global(> *:nth-child(8)) { animation-delay: calc(var(--stagger-delay, 50ms) * 7); }
  .stagger-children :global(> *:nth-child(n+9)) { animation-delay: calc(var(--stagger-delay, 50ms) * 8); }
  
  @keyframes stagger-fade-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
