<script lang="ts">
  import { page } from '$app/stores';
  import { fade, fly } from 'svelte/transition';
  import type { Snippet } from 'svelte';
  
  interface Props {
    children: Snippet;
  }
  
  let { children }: Props = $props();
  
  // Key для перерендера при смене страницы
  let key = $derived($page.url.pathname);
</script>

{#key key}
  <div 
    in:fly={{ y: 10, duration: 200, delay: 100 }}
    out:fade={{ duration: 100 }}
    class="h-full"
  >
    {@render children()}
  </div>
{/key}
