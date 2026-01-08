<script lang="ts">
  import { BentoWidget } from '$lib/components';
  import type { QueueItem } from './types';

  interface Props {
    queue: QueueItem[];
  }

  let { queue }: Props = $props();

  let passedCount = $derived(queue.filter(q => q.status === 'success').length);
  let failedCount = $derived(queue.filter(q => q.status === 'failed').length);
  let testingCount = $derived(queue.filter(q => q.status === 'testing').length);
  let pendingCount = $derived(queue.filter(q => q.status === 'pending').length);
</script>

<BentoWidget title="Статистика" icon="📈">
  <div class="grid grid-cols-2 gap-3">
    <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
      <div class="text-2xl font-bold text-emerald-400">{passedCount}</div>
      <div class="text-xs text-zinc-500 mt-1">Успешно</div>
    </div>
    <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
      <div class="text-2xl font-bold text-red-400">{failedCount}</div>
      <div class="text-xs text-zinc-500 mt-1">Неудачно</div>
    </div>
    <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
      <div class="text-2xl font-bold text-amber-400">{testingCount}</div>
      <div class="text-xs text-zinc-500 mt-1">Тестируется</div>
    </div>
    <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
      <div class="text-2xl font-bold text-zinc-400">{pendingCount}</div>
      <div class="text-xs text-zinc-500 mt-1">В очереди</div>
    </div>
  </div>
</BentoWidget>
