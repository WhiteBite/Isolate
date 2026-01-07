<script lang="ts">
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  // Redirect to /network (proxies merged into network page)
  $effect(() => {
    if (browser) {
      goto('/network', { replaceState: true });
    }
  });
</script>

<div class="flex items-center justify-center h-full">
  <p class="text-zinc-400">Redirecting to Network...</p>
</div>
