<script lang="ts">
  import Skeleton from './Skeleton.svelte';

  interface Props {
    lines?: number;
    showHeader?: boolean;
    showAvatar?: boolean;
    class?: string;
  }

  let {
    lines = 3,
    showHeader = true,
    showAvatar = false,
    class: className = ''
  }: Props = $props();
</script>

<div class="skeleton-card bg-void-200/50 rounded-xl p-4 border border-void-100/50 {className}">
  {#if showHeader}
    <div class="flex items-center gap-3 mb-4">
      {#if showAvatar}
        <Skeleton width="40px" height="40px" rounded="full" />
      {/if}
      <div class="flex-1 space-y-2">
        <Skeleton width="60%" height="16px" rounded="md" />
        {#if showAvatar}
          <Skeleton width="40%" height="12px" rounded="md" />
        {/if}
      </div>
    </div>
  {/if}

  <div class="space-y-2">
    {#each Array(lines) as _, i}
      <Skeleton 
        width={i === lines - 1 ? '75%' : '100%'} 
        height="14px" 
        rounded="md" 
      />
    {/each}
  </div>
</div>

<style>
  .skeleton-card {
    backdrop-filter: blur(8px);
  }
</style>
