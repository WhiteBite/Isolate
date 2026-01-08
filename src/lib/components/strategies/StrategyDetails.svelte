<script lang="ts">
  import BaseModal from '../BaseModal.svelte';
  import type { Strategy } from './types';
  import { categories, getFamilyColor, getCategoryColor, getLabelInfo, getScoreColor, formatDate } from './types';

  interface Props {
    open: boolean;
    strategy: Strategy;
    onClose: () => void;
    onApply: (id: string) => void;
    onStop: () => void;
    onTest: (id: string) => void;
  }

  let { open = $bindable(), strategy, onClose, onApply, onStop, onTest }: Props = $props();

  let familyColor = $derived(getFamilyColor(strategy.family));
  let categoryColor = $derived(getCategoryColor(strategy.category));
  let categoryInfo = $derived(categories.find(c => c.id === strategy.category));
  let labelInfo = $derived(getLabelInfo(strategy.label));
  let scoreColor = $derived(getScoreColor(strategy.score));

  function handleApplyAndClose() {
    onApply(strategy.id);
    onClose();
  }

  function handleStopAndClose() {
    onStop();
    onClose();
  }

  function handleTestAndClose() {
    onTest(strategy.id);
    onClose();
  }
</script>

<BaseModal bind:open onclose={onClose} class="max-w-lg w-full" ariaLabel="Strategy Details">
  <div class="p-6">
    <!-- Modal Header -->
    <div class="flex items-start justify-between mb-4">
      <div>
        <h2 id="strategy-details-title" class="text-xl font-bold text-text-primary">{strategy.name}</h2>
        <div class="flex items-center gap-2 mt-2">
          <span 
            class="px-2.5 py-1 rounded-lg text-xs font-medium"
            style="background: {categoryColor}20; color: {categoryColor}"
          >
            {categoryInfo?.icon} {categoryInfo?.name}
          </span>
          <span 
            class="px-2 py-0.5 rounded text-xs font-medium"
            style="background: {familyColor}20; color: {familyColor}"
          >
            {strategy.family.toUpperCase()}
          </span>
        </div>
      </div>
      <button
        onclick={onClose}
        aria-label="Close"
        class="p-2 hover:bg-void-100 rounded-lg transition-colors"
      >
        <svg class="w-5 h-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>
    </div>

    <!-- Modal Content -->
    <div class="space-y-4">
      <div>
        <h4 class="text-text-muted text-sm mb-1">Description</h4>
        <p class="text-text-primary">{strategy.description}</p>
      </div>
      
      <div class="grid grid-cols-2 gap-4">
        <div>
          <h4 class="text-text-muted text-sm mb-1">Author</h4>
          <p class="text-text-primary">{strategy.author}</p>
        </div>
        <div>
          <h4 class="text-text-muted text-sm mb-1">Score</h4>
          {#if strategy.score !== null}
            <p class="font-mono font-bold" style="color: {scoreColor}">{strategy.score}%</p>
          {:else}
            <p class="text-text-muted">Not tested</p>
          {/if}
        </div>
      </div>
      
      <div>
        <h4 class="text-text-muted text-sm mb-2">Services</h4>
        <div class="flex flex-wrap gap-2">
          {#each strategy.services as service}
            <span class="px-3 py-1.5 bg-void-100 text-text-primary text-sm rounded-lg border border-glass-border">
              {service}
            </span>
          {/each}
        </div>
      </div>
      
      <div>
        <h4 class="text-text-muted text-sm mb-1">Last Tested</h4>
        <p class="text-text-primary">{formatDate(strategy.lastTested)}</p>
      </div>
      
      {#if strategy.label}
        <div class="p-3 rounded-lg" style="background: {labelInfo.bg}10; border: 1px solid {labelInfo.bg}30">
          <span class="font-medium" style="color: {labelInfo.color}">{labelInfo.text}</span>
        </div>
      {/if}
    </div>

    <!-- Modal Actions -->
    <div class="flex gap-3 mt-6">
      {#if strategy.isActive}
        <button
          onclick={handleStopAndClose}
          class="flex-1 px-4 py-3 bg-void-100 hover:bg-void-200 text-text-primary rounded-xl font-medium transition-all duration-200 border border-glass-border"
        >
          Disable
        </button>
      {:else}
        <button
          onclick={handleApplyAndClose}
          class="flex-1 px-4 py-3 bg-indigo-500 hover:bg-indigo-600 text-white rounded-xl font-medium transition-all duration-200 shadow-glow"
        >
          Apply
        </button>
      {/if}
      <button
        onclick={handleTestAndClose}
        class="px-4 py-3 bg-void-100 hover:bg-void-200 text-text-primary rounded-xl font-medium transition-all duration-200 border border-glass-border"
      >
        Test
      </button>
    </div>
  </div>
</BaseModal>
