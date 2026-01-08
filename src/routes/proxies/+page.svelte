<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  // Redirect to /network (proxies merged into network page)
  onMount(() => {
    goto('/network', { replaceState: true });
  });
</script>

<div class="flex items-center justify-center h-full">
  <p class="text-zinc-400">Redirecting to Network...</p>
</div>
