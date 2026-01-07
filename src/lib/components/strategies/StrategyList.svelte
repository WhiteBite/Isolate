<script lang="ts">
  import type { Strategy } from './types';
  import StrategyCard from './StrategyCard.svelte';
  import { StrategiesSkeleton } from '$lib/components/skeletons';

  interface Props {
    strategies: Strategy[];
    loading: boolean;
    error: string | null;
    applyingStrategy: string | null;
    onApply: (id: string) => void;
    onStop: () => void;
    onTest: (id: string) => void;
    onShowDetails: (strategy: Strategy) => void;
  }

  let { 
    strategies, 
    loading, 
    error, 
    applyingStrategy,
    onApply, 
    onStop, 
    onTest, 
    onShowDetails 
  }: Props = $props();
</script>

{#if loading}
  <StrategiesSkeleton />
{:else if error}
  <div class="bg-neon-red/10 border border-neon-red/20 rounded-xl p-6 text-center">
    <svg class="w-12 h-12 text-neon-red mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
    </svg>
    <p class="text-neon-red">{error}</p>
  </div>
{:else if strategies.length === 0}
  <div class="bg-void-50 rounded-2xl p-12 border border-glass-border text-center">
    <svg class="w-16 h-16 text-text-muted/50 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
    </svg>
    <p class="text-text-secondary text-lg">No strategies found</p>
    <p class="text-text-muted text-sm mt-2">Try changing search parameters</p>
  </div>
{:else}
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
    {#each strategies as strategy (strategy.id)}
      <StrategyCard 
        {strategy}
        isApplying={applyingStrategy === strategy.id}
        {onApply}
        {onStop}
        {onTest}
        {onShowDetails}
      />
    {/each}
  </div>
{/if}
