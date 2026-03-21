<script lang="ts">
  /**
   * LoadingState — компонент для отображения состояния загрузки
   * Поддерживает скелетон и спиннер режимы
   */
  
  type Variant = 'spinner' | 'skeleton' | 'dots';
  type Size = 'sm' | 'md' | 'lg';
  
  interface Props {
    variant?: Variant;
    size?: Size;
    text?: string;
    /** Количество строк скелетона */
    lines?: number;
    class?: string;
  }
  
  let { variant = 'spinner', size = 'md', text, lines = 3, class: className = '' }: Props = $props();
  
  const sizeClasses: Record<Size, { spinner: string; text: string; skeleton: string }> = {
    sm: { spinner: 'w-4 h-4', text: 'text-xs', skeleton: 'h-3' },
    md: { spinner: 'w-6 h-6', text: 'text-sm', skeleton: 'h-4' },
    lg: { spinner: 'w-8 h-8', text: 'text-base', skeleton: 'h-5' }
  };
</script>

<div class="flex flex-col items-center justify-center py-8 {className}">
  {#if variant === 'spinner'}
    <div class="{sizeClasses[size].spinner} border-2 border-zinc-600 border-t-violet-500 rounded-full animate-spin"></div>
    {#if text}
      <p class="{sizeClasses[size].text} text-zinc-400 mt-3">{text}</p>
    {/if}
    
  {:else if variant === 'dots'}
    <div class="flex gap-1.5">
      {#each [0, 1, 2] as i}
        <div 
          class="w-2 h-2 bg-violet-500 rounded-full animate-bounce"
          style="animation-delay: {i * 150}ms"
        ></div>
      {/each}
    </div>
    {#if text}
      <p class="{sizeClasses[size].text} text-zinc-400 mt-3">{text}</p>
    {/if}
    
  {:else if variant === 'skeleton'}
    <div class="w-full space-y-3">
      {#each Array(lines) as _, i}
        <div 
          class="{sizeClasses[size].skeleton} bg-zinc-800 rounded animate-pulse"
          style="width: {i === lines - 1 ? '60%' : '100%'}"
        ></div>
      {/each}
    </div>
  {/if}
</div>
