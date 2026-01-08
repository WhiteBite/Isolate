<script lang="ts">
  import { BentoWidget } from '$lib/components';

  interface Props {
    logLines: string[];
    onClear: () => void;
  }

  let { logLines, onClear }: Props = $props();

  function getLogLineClass(line: string): string {
    if (line.includes('[ERROR]')) return 'text-red-400';
    if (line.includes('[SUCCESS]') || line.includes('[COMPLETE]')) return 'text-emerald-400';
    if (line.includes('[PROGRESS]') || line.includes('[TEST]')) return 'text-amber-400';
    if (line.includes('[START]')) return 'text-cyan-400';
    return 'text-zinc-400';
  }
</script>

<BentoWidget colspan={2} rowspan={2} title="Журнал активности" icon="📜">
  <div class="h-full flex flex-col">
    <!-- Log header -->
    <div class="flex items-center justify-between mb-3">
      <span class="text-xs text-zinc-500">{logLines.length} записей</span>
      <button
        onclick={onClear}
        class="px-2 py-1 text-xs text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50 rounded transition-colors"
      >
        Очистить
      </button>
    </div>

    <!-- Log content -->
    <div class="flex-1 overflow-auto font-mono text-xs bg-black/30 rounded-lg border border-white/5 p-3">
      {#each logLines as line}
        <div class="py-0.5 {getLogLineClass(line)}">
          {line}
        </div>
      {/each}

      {#if logLines.length === 0}
        <div class="text-zinc-600 italic">Ожидание активности...</div>
      {/if}
    </div>
  </div>
</BentoWidget>
