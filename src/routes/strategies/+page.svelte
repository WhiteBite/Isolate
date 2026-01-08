<script lang="ts">
  import { browser } from '$app/environment';
  import { appStatus } from '$lib/stores';
  import { waitForBackend } from '$lib/utils/backend';
  import {
    StrategyHeader,
    StrategyFilters,
    StrategyList,
    StrategyDetails,
    StrategyStats,
    StrategyHistory,
    mapServiceToCategory,
    type Strategy,
    type Category,
    type FamilyFilter
  } from '$lib/components/strategies';

  // Tab type
  type Tab = 'strategies' | 'history';

  // State using Svelte 5 runes
  let strategies = $state<Strategy[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let searchQuery = $state('');
  let selectedCategory = $state<Category>('all');
  let selectedFamily = $state<FamilyFilter>('all');
  let applyingStrategy = $state<string | null>(null);
  let selectedStrategyDetails = $state<Strategy | null>(null);
  let activeTab = $state<Tab>('strategies');

  // Local store values
  let appStatusValue = $state<{
    isActive: boolean;
    currentStrategy: string | null;
    currentStrategyName: string | null;
  }>({
    isActive: false,
    currentStrategy: null,
    currentStrategyName: null
  });

  // Derived values
  let filteredStrategies = $derived(
    strategies.filter(s => {
      const matchesSearch = s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           s.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesCategory = selectedCategory === 'all' || s.category === selectedCategory;
      const matchesFamily = selectedFamily === 'all' || s.family === selectedFamily;
      return matchesSearch && matchesCategory && matchesFamily;
    })
  );

  let strategiesByFamily = $derived({
    zapret: strategies.filter(s => s.family === 'zapret').length,
    vless: strategies.filter(s => s.family === 'vless').length
  });

  let strategiesByCategory = $derived({
    youtube: strategies.filter(s => s.category === 'youtube').length,
    discord: strategies.filter(s => s.category === 'discord').length,
    telegram: strategies.filter(s => s.category === 'telegram').length,
    general: strategies.filter(s => s.category === 'general').length,
    games: strategies.filter(s => s.category === 'games').length,
    custom: strategies.filter(s => s.category === 'custom').length
  });

  // Initialize on mount using $effect
  $effect(() => {
    if (!browser) return;
    const unsub = appStatus.subscribe(v => { appStatusValue = v; });
    loadStrategies();
    return () => unsub();
  });

  async function loadStrategies() {
    loading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const ready = await waitForBackend(30, 300);
      if (!ready) return;
      
      const loadedStrategies = await invoke<any[]>('get_strategies');
      if (loadedStrategies && loadedStrategies.length > 0) {
        strategies = loadedStrategies.map((s: any) => ({
          id: s.id,
          name: s.name,
          family: (s.engine === 'sing_box' ? 'vless' : s.engine) as 'zapret' | 'vless' | 'custom',
          category: mapServiceToCategory(s.services),
          description: s.description || '',
          services: s.services || [],
          score: s.score,
          lastTested: s.last_tested ? new Date(s.last_tested) : null,
          isActive: appStatusValue.currentStrategy === s.id,
          label: s.label || null,
          author: s.author || 'Unknown'
        }));
      }
    } catch {
      // Leave empty array on error
    } finally {
      loading = false;
    }
  }

  async function applyStrategy(strategyId: string) {
    if (!browser) return;
    applyingStrategy = strategyId;
    try {
      const tauriCore = (window as any).__TAURI__?.core;
      if (!tauriCore) throw new Error('Tauri not available');
      await tauriCore.invoke('apply_strategy', { strategyId });
      const strategy = strategies.find(s => s.id === strategyId);
      appStatus.set({
        isActive: true,
        currentStrategy: strategyId,
        currentStrategyName: strategy?.name ?? null
      });
      strategies = strategies.map(s => ({ ...s, isActive: s.id === strategyId }));
    } catch (e) {
      error = `Strategy apply error: ${e}`;
    } finally {
      applyingStrategy = null;
    }
  }

  async function stopStrategy() {
    if (!browser) return;
    try {
      const tauriCore = (window as any).__TAURI__?.core;
      if (!tauriCore) throw new Error('Tauri not available');
      await tauriCore.invoke('stop_strategy');
      appStatus.set({ isActive: false, currentStrategy: null, currentStrategyName: null });
      strategies = strategies.map(s => ({ ...s, isActive: false }));
    } catch {
      // Silent fail
    }
  }

  async function testStrategy(strategyId: string) {
    if (!browser) return;
    try {
      const tauriCore = (window as any).__TAURI__?.core;
      if (!tauriCore) throw new Error('Tauri not available');
      const result = await tauriCore.invoke('test_strategy', { strategyId }) as { score: number };
      strategies = strategies.map(s => 
        s.id === strategyId ? { ...s, score: Math.round(result.score), lastTested: new Date() } : s
      );
    } catch (e) {
      error = `Testing error: ${e}`;
    }
  }

  function showDetails(strategy: Strategy) { selectedStrategyDetails = strategy; }
  function closeDetails() { selectedStrategyDetails = null; }
</script>

<div class="p-8 space-y-6 min-h-screen bg-void">
  <StrategyHeader 
    currentStrategyName={appStatusValue.currentStrategyName}
    isActive={appStatusValue.isActive}
    onStop={stopStrategy}
  />

  <!-- Tab Navigation -->
  <div class="flex items-center gap-2 border-b border-glass-border pb-4">
    <button
      onclick={() => activeTab = 'strategies'}
      class="px-4 py-2 rounded-lg text-sm font-medium transition-colors {activeTab === 'strategies' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-text-muted hover:text-text-primary hover:bg-void-50'}"
    >
      <span class="flex items-center gap-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16"/>
        </svg>
        Strategies
      </span>
    </button>
    <button
      onclick={() => activeTab = 'history'}
      class="px-4 py-2 rounded-lg text-sm font-medium transition-colors {activeTab === 'history' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-text-muted hover:text-text-primary hover:bg-void-50'}"
    >
      <span class="flex items-center gap-2">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
        </svg>
        History
      </span>
    </button>
  </div>

  {#if activeTab === 'strategies'}
    <StrategyFilters
      {selectedCategory}
      {selectedFamily}
      {searchQuery}
      {strategiesByCategory}
      {strategiesByFamily}
      onCategoryChange={(cat) => selectedCategory = cat}
      onFamilyChange={(fam) => selectedFamily = fam}
      onSearchChange={(q) => searchQuery = q}
    />

    <StrategyList
      strategies={filteredStrategies}
      {loading}
      {error}
      {applyingStrategy}
      onApply={applyStrategy}
      onStop={stopStrategy}
      onTest={testStrategy}
      onShowDetails={showDetails}
    />

    {#if !loading && !error}
      <StrategyStats
        totalStrategies={strategies.length}
        filteredCount={filteredStrategies.length}
        zapretCount={strategiesByFamily.zapret}
        vlessCount={strategiesByFamily.vless}
      />
    {/if}
  {:else if activeTab === 'history'}
    <StrategyHistory />
  {/if}

  {#if selectedStrategyDetails}
    <StrategyDetails
      open={selectedStrategyDetails !== null}
      strategy={selectedStrategyDetails}
      onClose={closeDetails}
      onApply={applyStrategy}
      onStop={stopStrategy}
      onTest={testStrategy}
    />
  {/if}
</div>
