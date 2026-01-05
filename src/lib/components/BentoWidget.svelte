<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    colspan?: 1 | 2 | 3 | 4;
    rowspan?: 1 | 2;
    title?: string;
    icon?: string;
    children?: Snippet;
  }

  let { 
    colspan = 1, 
    rowspan = 1,
    title,
    icon,
    children 
  }: Props = $props();

  const colspanClasses = {
    1: 'col-span-1',
    2: 'col-span-2',
    3: 'col-span-3',
    4: 'col-span-4'
  };

  const rowspanClasses = {
    1: 'row-span-1',
    2: 'row-span-2'
  };
</script>

<div 
  class="
    {colspanClasses[colspan]} 
    {rowspanClasses[rowspan]}
    bg-zinc-900/40
    backdrop-blur-md
    border border-white/5 border-t-white/10
    rounded-xl 
    p-5
    transition-all duration-200
    hover:border-white/10
    hover:-translate-y-0.5
  "
>
  {#if title || icon}
    <div class="flex items-center gap-2.5 mb-4 pb-3 border-b border-white/5">
      {#if icon}
        <span class="text-base opacity-60">{icon}</span>
      {/if}
      {#if title}
        <h3 class="text-[10px] text-zinc-500 uppercase tracking-widest font-semibold">{title}</h3>
      {/if}
    </div>
  {/if}
  
  <div class="h-full">
    {#if children}
      {@render children()}
    {/if}
  </div>
</div>
