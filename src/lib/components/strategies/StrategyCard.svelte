<script lang="ts">
  import type { Strategy, Category } from './types';
  import { categories, getFamilyColor, getCategoryColor, getLabelInfo, getScoreColor } from './types';
  import { providerStore } from '$lib/stores/providers';

  interface Props {
    strategy: Strategy;
    isApplying?: boolean;
    onApply: (id: string) => void;
    onStop: () => void;
    onTest: (id: string) => void;
    onShowDetails: (strategy: Strategy) => void;
  }

  let { strategy, isApplying = false, onApply, onStop, onTest, onShowDetails }: Props = $props();

  let familyColor = $derived(getFamilyColor(strategy.family));
  let categoryColor = $derived(getCategoryColor(strategy.category));
  let labelInfo = $derived(getLabelInfo(strategy.label));
  let scoreColor = $derived(getScoreColor(strategy.score));
  let categoryInfo = $derived(categories.find(c => c.id === strategy.category));

  // Provider recommendation state
  let providerState = $state(providerStore.getState());
  
  $effect(() => {
    const unsubscribe = providerStore.subscribe(() => {
      providerState = providerStore.getState();
    });
    return unsubscribe;
  });

  let isRecommended = $derived(
    providerState.selectedProviderId && 
    providerStore.isStrategyRecommended(strategy.id)
  );
  
  let recommendationPriority = $derived(
    providerStore.getStrategyPriority(strategy.id)
  );
</script>

<div 
  class="group relative bg-void-50 rounded-xl p-5 border transition-all duration-300 hover:-translate-y-1
    {strategy.isActive 
      ? 'border-indigo-500/50 shadow-glow' 
      : 'border-glass-border hover:border-glass-border-active hover:shadow-card'}"
>
  <!-- Glow effect for active strategy -->
  {#if strategy.isActive}
    <div class="absolute inset-0 rounded-xl bg-gradient-to-br from-indigo-500/5 to-transparent pointer-events-none"></div>
  {/if}
  
  <!-- Glass overlay on hover -->
  <div class="absolute inset-0 rounded-xl bg-gradient-to-br from-white/[0.02] to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
  
  <!-- Header with Category Badge -->
  <div class="relative flex items-start justify-between mb-3">
    <div class="flex items-center gap-2 flex-wrap">
      <!-- Family Badge with glow -->
      <span 
        class="px-2.5 py-1 rounded-lg text-xs font-bold uppercase tracking-wide flex items-center gap-1.5"
        style="background: {familyColor}15; color: {familyColor}; box-shadow: 0 0 10px {familyColor}20;"
      >
        <span class="w-1.5 h-1.5 rounded-full" style="background: {familyColor}; box-shadow: 0 0 6px {familyColor};"></span>
        {strategy.family}
      </span>
      
      <!-- Category Badge -->
      <span 
        class="px-2 py-1 rounded-lg text-xs font-medium flex items-center gap-1"
        style="background: {categoryColor}15; color: {categoryColor}"
      >
        {categoryInfo?.icon || '🌐'}
      </span>
      
      <!-- Label Badge -->
      {#if strategy.label}
        <span 
          class="px-2 py-1 rounded-lg text-xs font-medium"
          style="background: {labelInfo.bg}12; color: {labelInfo.color}"
        >
          {labelInfo.text}
        </span>
      {/if}

      <!-- Provider Recommendation Badge -->
      {#if isRecommended}
        <span 
          class="px-2 py-1 rounded-lg text-xs font-medium flex items-center gap-1"
          style="background: rgba(16, 185, 129, 0.12); color: rgb(16, 185, 129)"
          title="Recommended for {providerState.selectedProvider?.name || 'your provider'}"
        >
          {#if recommendationPriority === 0}
            ⭐ Best for ISP
          {:else}
            ✓ ISP Recommended
          {/if}
        </span>
      {/if}
    </div>
    
    {#if strategy.isActive}
      <div class="flex items-center gap-1.5 px-2.5 py-1 bg-neon-green/10 rounded-full border border-neon-green/30">
        <div class="w-2 h-2 rounded-full bg-neon-green animate-pulse shadow-glow-green"></div>
        <span class="text-neon-green text-xs font-semibold">Active</span>
      </div>
    {/if}
  </div>

  <!-- Title -->
  <h3 class="relative text-text-primary font-semibold text-lg mb-2 group-hover:text-indigo-400 transition-colors">{strategy.name}</h3>

  <!-- Description -->
  <p class="relative text-text-secondary text-sm mb-4 line-clamp-2 leading-relaxed">{strategy.description}</p>

  <!-- Meta info row -->
  <div class="relative flex items-center justify-between mb-4 py-3 px-3 bg-void/50 rounded-xl border border-glass-border">
    <div class="flex items-center gap-2 text-sm">
      <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
      </svg>
      <span class="text-text-muted">{strategy.author}</span>
    </div>
    
    <div class="flex items-center gap-2">
      <span class="text-text-muted text-sm">Score:</span>
      {#if strategy.score !== null}
        <span 
          class="font-mono font-bold text-sm px-2 py-0.5 rounded"
          style="color: {scoreColor}; background: {scoreColor}15;"
        >
          {strategy.score}%
        </span>
      {:else}
        <span class="text-text-muted text-sm">—</span>
      {/if}
    </div>
  </div>

  <!-- Actions -->
  <div class="relative flex gap-2">
    {#if strategy.isActive}
      <button
        onclick={onStop}
        class="flex-1 px-4 py-2.5 bg-void-100 hover:bg-void-200 text-text-primary rounded-xl text-sm font-medium transition-all duration-200 border border-glass-border"
      >
        Disable
      </button>
    {:else}
      <button
        onclick={() => onApply(strategy.id)}
        disabled={isApplying}
        class="flex-1 px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 text-white rounded-xl text-sm font-semibold transition-all duration-200 disabled:opacity-50 shadow-lg shadow-glow hover:shadow-glow-lg"
      >
        {#if isApplying}
          <svg class="w-4 h-4 animate-spin mx-auto" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        {:else}
          Apply
        {/if}
      </button>
    {/if}
    
    <button
      onclick={() => onTest(strategy.id)}
      class="px-3 py-2.5 bg-void-100 hover:bg-void-200 text-text-muted hover:text-indigo-400 rounded-xl text-sm transition-all duration-200 border border-glass-border hover:border-indigo-500/30"
      title="Test"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
      </svg>
    </button>
    
    <button
      onclick={() => onShowDetails(strategy)}
      class="px-3 py-2.5 bg-void-100 hover:bg-void-200 text-text-muted hover:text-text-primary rounded-xl text-sm transition-all duration-200 border border-glass-border"
      title="Details"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
      </svg>
    </button>
  </div>
</div>
