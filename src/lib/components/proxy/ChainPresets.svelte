<script lang="ts">
  import { proxyChainStore, chainPresets, type ChainPreset } from '$lib/stores/proxyChain.svelte';
  
  interface Props {
    onApply?: (preset: ChainPreset) => void;
  }
  
  let { onApply }: Props = $props();
  
  let selectedPresetId = $state<string | null>(null);
  
  // Иконки для пресетов
  const presetIcons: Record<string, string> = {
    'basic-dpi': `<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
    </svg>`,
    'single-proxy': `<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M13 10V3L4 14h7v7l9-11h-7z" />
    </svg>`,
    'double-proxy': `<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
    </svg>`
  };
  
  function handleApply(preset: ChainPreset) {
    proxyChainStore.applyPreset(preset);
    selectedPresetId = preset.id;
    onApply?.(preset);
  }
</script>

<div class="space-y-3">
  <h3 class="text-sm font-medium text-gray-300 mb-3">Готовые цепочки</h3>
  
  {#each chainPresets as preset}
    {@const isSelected = selectedPresetId === preset.id}
    <button
      onclick={() => handleApply(preset)}
      class="w-full text-left p-3 rounded-xl border transition-all duration-200
             {isSelected 
               ? 'bg-indigo-500/20 border-indigo-500/50 shadow-lg shadow-indigo-500/10' 
               : 'bg-white/5 border-white/10 hover:bg-white/10 hover:border-white/20'}"
    >
      <div class="flex items-start gap-3">
        <!-- Icon -->
        <div class="flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center
                    {isSelected ? 'bg-indigo-500/30 text-indigo-400' : 'bg-white/10 text-gray-400'}">
          {@html presetIcons[preset.id] || presetIcons['basic-dpi']}
        </div>
        
        <!-- Content -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <h4 class="text-white font-medium text-sm">{preset.name}</h4>
            {#if isSelected}
              <span class="px-1.5 py-0.5 text-xs rounded bg-indigo-500/30 text-indigo-300">
                Активно
              </span>
            {/if}
          </div>
          <p class="text-gray-400 text-xs mt-0.5">{preset.description}</p>
          
          <!-- Visual chain preview -->
          <div class="flex items-center gap-1 mt-2">
            {#each preset.blocks as block, i}
              <div class="flex items-center">
                <span class="px-2 py-0.5 text-xs rounded-full
                            {block.type === 'dpi' ? 'bg-amber-500/20 text-amber-400' : ''}
                            {block.type === 'proxy' ? 'bg-blue-500/20 text-blue-400' : ''}
                            {block.type === 'internet' ? 'bg-green-500/20 text-green-400' : ''}">
                  {block.type === 'dpi' ? 'DPI' : ''}
                  {block.type === 'proxy' ? block.data.country || 'Proxy' : ''}
                  {block.type === 'internet' ? '🌐' : ''}
                </span>
                {#if i < preset.blocks.length - 1}
                  <svg class="w-4 h-4 text-gray-500 mx-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                {/if}
              </div>
            {/each}
          </div>
        </div>
        
        <!-- Apply indicator -->
        <div class="flex-shrink-0">
          {#if isSelected}
            <svg class="w-5 h-5 text-indigo-400" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
            </svg>
          {:else}
            <svg class="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          {/if}
        </div>
      </div>
    </button>
  {/each}
</div>
