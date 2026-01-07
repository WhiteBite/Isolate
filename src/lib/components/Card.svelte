<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    padding?: 'none' | 'sm' | 'md' | 'lg';
    class?: string;
    children?: Snippet;
  }

  let { 
    title,
    padding = 'md',
    class: className = '',
    children 
  }: Props = $props();

  const paddingClasses = {
    none: '',
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-6'
  };
</script>

<div class="bg-zinc-900/40 rounded-xl border border-white/5 shadow-card hover:border-white/10 transition-all duration-200 {className}">
  {#if title}
    <div class="px-4 py-3 border-b border-white/5">
      <h3 class="text-lg font-semibold text-white">{title}</h3>
    </div>
  {/if}
  <div class={paddingClasses[padding]}>
    {#if children}
      {@render children()}
    {/if}
  </div>
</div>
