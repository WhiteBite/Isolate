<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { browser } from '$app/environment';

  // Redirect to main plugins page with selected plugin
  // The main /plugins page has master-detail layout that shows plugin details
  $effect(() => {
    if (browser) {
      const pluginId = $page.params.id;
      if (pluginId) {
        // Redirect to main plugins page - it will show the plugin in detail panel
        goto(`/plugins?selected=${encodeURIComponent(pluginId)}`, { replaceState: true });
      } else {
        goto('/plugins', { replaceState: true });
      }
    }
  });
</script>

<!-- Fallback content while redirecting -->
<div class="h-full flex items-center justify-center bg-void">
  <div class="text-center">
    <div class="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto mb-3"></div>
    <p class="text-sm text-zinc-400">Перенаправление...</p>
  </div>
</div>
