<script lang="ts">
  import { BentoWidget } from '$lib/components';
  import type { ServiceInfo, OrchestraState } from './types';
  import { t } from '$lib/i18n';

  interface Props {
    services: ServiceInfo[];
    selectedServices: Set<string>;
    isLearning: boolean;
    onToggle: (id: string) => void;
  }

  let { services, selectedServices, isLearning, onToggle }: Props = $props();
</script>

<BentoWidget title={t('orchestra.widgets.services')} icon="🎯">
  <div class="space-y-2 max-h-[180px] overflow-y-auto pr-1">
    {#each services as service}
      <button
        onclick={() => onToggle(service.id)}
        disabled={isLearning}
        class="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-all
          {selectedServices.has(service.id) 
            ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30' 
            : 'bg-zinc-800/30 text-zinc-400 border border-white/5 hover:bg-zinc-800/50'}
          {isLearning ? 'opacity-50 cursor-not-allowed' : ''}"
      >
        <span>{service.icon}</span>
        <span class="flex-1 text-left">{service.name}</span>
        {#if selectedServices.has(service.id)}
          <svg class="w-4 h-4 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        {/if}
      </button>
    {/each}
  </div>
  <div class="mt-2 pt-2 border-t border-white/5 text-xs text-zinc-500 text-center">
    {t('orchestra.services.selected')}: {selectedServices.size}
  </div>
</BentoWidget>
