<script lang="ts" generics="T extends import('svelte').Component">
  import type { Component, Snippet } from 'svelte';
  import Skeleton from './Skeleton.svelte';

  interface Props {
    /** Lazy loader from createLazyLoader */
    loader: {
      load: () => Promise<T>;
      isLoaded: () => boolean;
      getCached: () => T | null;
    };
    /** Whether to trigger loading */
    shouldLoad?: boolean;
    /** Props to pass to the loaded component */
    componentProps?: Record<string, unknown>;
    /** Custom loading placeholder */
    placeholder?: Snippet;
    /** Show skeleton while loading */
    showSkeleton?: boolean;
    /** Skeleton width */
    skeletonWidth?: string;
    /** Skeleton height */
    skeletonHeight?: string;
  }

  let {
    loader,
    shouldLoad = false,
    componentProps = {},
    placeholder,
    showSkeleton = true,
    skeletonWidth = '100%',
    skeletonHeight = '200px'
  }: Props = $props();

  let LoadedComponent = $state<T | null>(null);
  let loading = $state(false);
  let error = $state<Error | null>(null);
  let hasTriggeredLoad = $state(false);

  // Load component when shouldLoad becomes true
  $effect(() => {
    if (shouldLoad && !hasTriggeredLoad && !loader.isLoaded()) {
      hasTriggeredLoad = true;
      loading = true;
      
      loader.load()
        .then((component) => {
          LoadedComponent = component;
          loading = false;
        })
        .catch((err) => {
          error = err;
          loading = false;
          console.error('Failed to load lazy component:', err);
        });
    } else if (shouldLoad && loader.isLoaded()) {
      LoadedComponent = loader.getCached();
    }
  });
</script>

{#if LoadedComponent}
  <LoadedComponent {...componentProps} />
{:else if loading}
  {#if placeholder}
    {@render placeholder()}
  {:else if showSkeleton}
    <div style="width: {skeletonWidth}; height: {skeletonHeight};">
      <Skeleton width="100%" height="100%" />
    </div>
  {/if}
{:else if error}
  <div class="text-red-400 text-sm p-4">
    Failed to load component
  </div>
{/if}
