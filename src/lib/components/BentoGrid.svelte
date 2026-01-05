<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    columns?: 1 | 2 | 3 | 4;
    gap?: 2 | 3 | 4 | 6 | 8;
    children?: Snippet;
  }

  let { 
    columns = 4, 
    gap = 4,
    children 
  }: Props = $props();

  const columnClasses = {
    1: 'grid-cols-1',
    2: 'grid-cols-2',
    3: 'grid-cols-3',
    4: 'grid-cols-4'
  };

  const gapClasses = {
    2: 'gap-2',
    3: 'gap-3',
    4: 'gap-4',
    6: 'gap-6',
    8: 'gap-8'
  };
</script>

<div class="grid {columnClasses[columns]} {gapClasses[gap]} auto-rows-[minmax(140px,auto)]">
  {#if children}
    {@render children()}
  {/if}
</div>
