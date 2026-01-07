<script lang="ts">
  import { toasts } from '$lib/stores/toast';
  import Toast from './Toast.svelte';
</script>

<div 
  class="fixed bottom-4 right-4 z-50 flex flex-col gap-2"
  role="region"
  aria-label="Notifications"
  aria-live="polite"
>
  {#each $toasts as toast (toast.id)}
    <div role="alert" aria-atomic="true">
      <Toast {toast} />
    </div>
  {/each}
</div>
