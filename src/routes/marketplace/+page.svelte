<script lang="ts">
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  // Redirect to unified plugins page with marketplace tab
  $effect(() => {
    if (browser) {
      goto('/plugins?tab=marketplace', { replaceState: true });
    }
  });
</script>

<div class="min-h-screen bg-[#0a0e27] flex items-center justify-center">
  <p class="text-zinc-400">Redirecting to Plugins...</p>
</div>
