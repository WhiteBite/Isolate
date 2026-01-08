<script lang="ts">
  import { BentoWidget } from '$lib/components';
  import type { QueueItem } from './types';
  import { t } from '$lib/i18n';

  interface Props {
    queue: QueueItem[];
  }

  let { queue }: Props = $props();

  let passedCount = $derived(queue.filter(q => q.status === 'success').length);
  let failedCount = $derived(queue.filter(q => q.status === 'failed').length);
  let testingCount = $derived(queue.filter(q => q.status === 'testing').length);
  let pendingCount = $derived(queue.filter(q => q.status === 'pending').length);
</script>

<BentoWidget title={t('orchestra.widgets.stats')} icon="📈">
  <div class="grid grid-cols-2 gap-3">
    <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
      <div class="text-2xl font-bold text-emerald-400">{passedCount}</div>
      <div class="text-xs text-zinc-400 mt-1">{t('orchestra.stats.success')}</div>
    </div>
    <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
      <div class="text-2xl font-bold text-red-400">{failedCount}</div>
      <div class="text-xs text-zinc-400 mt-1">{t('orchestra.stats.failed')}</div>
    </div>
    <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
      <div class="text-2xl font-bold text-amber-400">{testingCount}</div>
      <div class="text-xs text-zinc-400 mt-1">{t('orchestra.stats.testing')}</div>
    </div>
    <div class="p-3 bg-zinc-800/30 rounded-lg text-center">
      <div class="text-2xl font-bold text-zinc-400">{pendingCount}</div>
      <div class="text-xs text-zinc-400 mt-1">{t('orchestra.stats.pending')}</div>
    </div>
  </div>
</BentoWidget>
