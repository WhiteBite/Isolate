<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    padding?: 'none' | 'sm' | 'md' | 'lg';
    children?: Snippet;
  }

  let { 
    title,
    padding = 'md',
    children 
  }: Props = $props();

  const paddingClasses = {
    none: '',
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-6'
  };
</script>

<div class="bg-[#1a1f3a] rounded-xl border border-[#2a2f4a] shadow-lg">
  {#if title}
    <div class="px-4 py-3 border-b border-[#2a2f4a]">
      <h3 class="text-lg font-semibold text-white">{title}</h3>
    </div>
  {/if}
  <div class={paddingClasses[padding]}>
    {#if children}
      {@render children()}
    {/if}
  </div>
</div>
