<script lang="ts">
  import Skeleton from './Skeleton.svelte';

  interface Props {
    count?: number;
    showIcon?: boolean;
    iconSize?: string;
    gap?: string;
    class?: string;
  }

  let {
    count = 5,
    showIcon = true,
    iconSize = '32px',
    gap = '12px',
    class: className = ''
  }: Props = $props();
</script>

<div class="skeleton-list flex flex-col {className}" style="gap: {gap};">
  {#each Array(count) as _, i}
    <div class="skeleton-list-item flex items-center gap-3 p-3 bg-void-200/30 rounded-lg border border-void-100/30">
      {#if showIcon}
        <Skeleton width={iconSize} height={iconSize} rounded="lg" />
      {/if}
      <div class="flex-1 space-y-2">
        <Skeleton width="70%" height="14px" rounded="md" />
        <Skeleton width="50%" height="12px" rounded="md" />
      </div>
      <Skeleton width="60px" height="24px" rounded="md" />
    </div>
  {/each}
</div>

<style>
  .skeleton-list-item {
    backdrop-filter: blur(4px);
  }
</style>
