<script lang="ts">
  import { toasts, MAX_VISIBLE_TOASTS } from '$lib/stores/toast';
  import Toast from './Toast.svelte';

  // Сортируем toasts: error > warning > progress > info > success
  // Внутри одного приоритета — новые выше
  const sortedToasts = $derived(
    [...$toasts].sort((a, b) => {
      if (a.priority !== b.priority) {
        return b.priority - a.priority;
      }
      return b.createdAt - a.createdAt;
    })
  );
</script>

<div 
  class="fixed bottom-4 right-4 z-50 flex flex-col-reverse gap-2 max-h-[calc(100vh-2rem)] overflow-hidden"
  role="region"
  aria-label="Notifications"
  aria-live="polite"
>
  {#each sortedToasts as toast (toast.id)}
    <div role="alert" aria-atomic="true">
      <Toast {toast} />
    </div>
  {/each}
</div>
