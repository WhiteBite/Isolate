<script lang="ts">
  /**
   * EmptyState — компонент для отображения пустого состояния
   * Используется когда список/таблица не содержит данных
   */
  import type { Snippet } from 'svelte';
  
  type Size = 'sm' | 'md' | 'lg';
  
  interface Props {
    icon?: string;
    title?: string;
    description?: string;
    actionText?: string;
    onaction?: () => void;
    size?: Size;
    /** Кастомный контент вместо иконки */
    iconSlot?: Snippet;
    /** Кастомные действия */
    actions?: Snippet;
    class?: string;
  }
  
  let { 
    icon = '📭', 
    title = 'No data', 
    description,
    actionText,
    onaction,
    size = 'md',
    iconSlot,
    actions,
    class: className = ''
  }: Props = $props();
  
  const sizeClasses: Record<Size, { icon: string; title: string; desc: string; btn: string }> = {
    sm: { icon: 'text-2xl', title: 'text-sm', desc: 'text-xs', btn: 'px-3 py-1.5 text-xs' },
    md: { icon: 'text-4xl', title: 'text-base', desc: 'text-sm', btn: 'px-4 py-2 text-sm' },
    lg: { icon: 'text-5xl', title: 'text-lg', desc: 'text-base', btn: 'px-5 py-2.5 text-base' }
  };
</script>

<div class="flex flex-col items-center justify-center py-12 px-4 text-center {className}">
  <!-- Icon -->
  <div class="mb-4">
    {#if iconSlot}
      {@render iconSlot()}
    {:else}
      <span class="{sizeClasses[size].icon}">{icon}</span>
    {/if}
  </div>
  
  <!-- Title -->
  <h3 class="{sizeClasses[size].title} font-medium text-zinc-200 mb-1">
    {title}
  </h3>
  
  <!-- Description -->
  {#if description}
    <p class="{sizeClasses[size].desc} text-zinc-400 max-w-sm">
      {description}
    </p>
  {/if}
  
  <!-- Actions -->
  <div class="mt-5">
    {#if actions}
      {@render actions()}
    {:else if actionText && onaction}
      <button
        onclick={onaction}
        class="{sizeClasses[size].btn} bg-violet-500/20 border border-violet-500/30 rounded-lg
               text-violet-400 font-medium hover:bg-violet-500/30 hover:border-violet-500/40
               transition-all duration-200"
      >
        {actionText}
      </button>
    {/if}
  </div>
</div>
