<script lang="ts">
  import { browser } from '$app/environment';
  import { appStatus } from '$lib/stores';

  // Types
  interface Strategy {
    id: string;
    name: string;
    family: 'zapret' | 'vless' | 'custom';
    description: string;
    services: string[];
    score: number | null;
    lastTested: Date | null;
    isActive: boolean;
  }

  // State using Svelte 5 runes
  let strategies = $state<Strategy[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let searchQuery = $state('');
  let filterFamily = $state<'all' | 'zapret' | 'vless' | 'custom'>('all');
  let applyingStrategy = $state<string | null>(null);

  // Local store values
  let appStatusValue = $state<{isActive: boolean; currentStrategy: string | null; currentStrategyName: string | null}>({
    isActive: false,
    currentStrategy: null,
    currentStrategyName: null
  });

  // Derived values
  let filteredStrategies = $derived(
    strategies.filter(s => {
      const matchesSearch = s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           s.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesFamily = filterFamily === 'all' || s.family === filterFamily;
      return matchesSearch && matchesFamily;
    })
  );

  let strategiesByFamily = $derived({
    zapret: filteredStrategies.filter(s => s.family === 'zapret'),
    vless: filteredStrategies.filter(s => s.family === 'vless'),
    custom: filteredStrategies.filter(s => s.family === 'custom')
  });

  function getDemoStrategies(): Strategy[] {
    return [
      {
        id: 'discord-zapret-1',
        name: 'Discord Zapret Basic',
        family: 'zapret',
        description: 'Базовая стратегия для Discord через Zapret',
        services: ['discord'],
        score: 85,
        lastTested: new Date(),
        isActive: false
      },
      {
        id: 'youtube-zapret-1',
        name: 'YouTube Zapret',
        family: 'zapret',
        description: 'Обход блокировки YouTube',
        services: ['youtube'],
        score: 78,
        lastTested: new Date(),
        isActive: false
      },
      {
        id: 'universal-vless-1',
        name: 'Universal VLESS',
        family: 'vless',
        description: 'Универсальная VLESS стратегия для всех сервисов',
        services: ['discord', 'youtube', 'telegram'],
        score: 95,
        lastTested: new Date(),
        isActive: false
      }
    ];
  }

  // Initialize on mount using $effect
  $effect(() => {
    if (!browser) return;
    
    console.log('[Strategies] $effect running, loading strategies...');
    
    // Subscribe to store
    const unsub = appStatus.subscribe(v => { appStatusValue = v; });
    
    // Load strategies
    loadStrategies();
    
    return () => unsub();
  });

  async function loadStrategies() {
    const tauriCore = (window as any).__TAURI__?.core;
    
    if (!tauriCore) {
      console.log('[Strategies] Not in Tauri, using demo data');
      strategies = getDemoStrategies();
      loading = false;
      return;
    }

    try {
      console.log('[Strategies] Calling get_strategies...');
      const loadedStrategies = await tauriCore.invoke('get_strategies');
      console.log('[Strategies] Loaded:', loadedStrategies?.length ?? 0, 'strategies');
      
      if (loadedStrategies && loadedStrategies.length > 0) {
        strategies = loadedStrategies.map((s: any) => ({
          id: s.id,
          name: s.name,
          family: (s.engine === 'sing_box' ? 'vless' : s.engine) as 'zapret' | 'vless' | 'custom',
          description: s.description || '',
          services: s.services || [],
          score: s.score,
          lastTested: s.last_tested ? new Date(s.last_tested) : null,
          isActive: appStatusValue.currentStrategy === s.id
        }));
      } else {
        strategies = getDemoStrategies();
      }
    } catch (e) {
      console.error('[Strategies] Failed to load:', e);
      error = 'Не удалось загрузить стратегии';
      strategies = getDemoStrategies();
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

      // Update local state
      strategies = strategies.map(s => ({
        ...s,
        isActive: s.id === strategyId
      }));
    } catch (e) {
      console.error('Failed to apply strategy:', e);
      error = `Ошибка применения стратегии: ${e}`;
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
      
      appStatus.set({
        isActive: false,
        currentStrategy: null,
        currentStrategyName: null
      });

      strategies = strategies.map(s => ({
        ...s,
        isActive: false
      }));
    } catch (e) {
      console.error('Failed to stop strategy:', e);
    }
  }

  async function testStrategy(strategyId: string) {
    if (!browser) return;
    
    try {
      const tauriCore = (window as any).__TAURI__?.core;
      if (!tauriCore) throw new Error('Tauri not available');
      
      const result = await tauriCore.invoke('test_strategy', { strategyId }) as {
        strategy_id: string;
        score: number;
        success_rate: number;
        avg_latency_ms: number;
        services_passed: string[];
        services_failed: string[];
      };
      
      strategies = strategies.map(s => 
        s.id === strategyId 
          ? { ...s, score: Math.round(result.score), lastTested: new Date() }
          : s
      );
    } catch (e) {
      console.error('Failed to test strategy:', e);
      error = `Ошибка тестирования: ${e}`;
    }
  }

  function getFamilyColor(family: string): string {
    switch (family) {
      case 'zapret': return '#00d4ff';
      case 'vless': return '#00ff88';
      case 'custom': return '#ffaa00';
      default: return '#a0a0a0';
    }
  }

  function getFamilyIcon(family: string): string {
    switch (family) {
      case 'zapret': return 'M13 10V3L4 14h7v7l9-11h-7z';
      case 'vless': return 'M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0';
      case 'custom': return 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z';
      default: return '';
    }
  }

  function getScoreColor(score: number | null): string {
    if (score === null) return '#a0a0a0';
    if (score >= 80) return '#00ff88';
    if (score >= 50) return '#ffaa00';
    return '#ff3333';
  }

  function formatDate(date: Date | null): string {
    if (!date) return 'Не тестировалась';
    return date.toLocaleDateString('ru-RU', { 
      day: 'numeric', 
      month: 'short',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
</script>

<div class="p-8 space-y-6 min-h-screen bg-[#0a0e27]">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-white">Стратегии</h1>
      <p class="text-[#a0a0a0] mt-1">Управление стратегиями обхода блокировок</p>
    </div>
    
    {#if appStatusValue.currentStrategy}
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2 px-4 py-2 bg-[#00ff88]/10 rounded-xl">
          <div class="w-2 h-2 rounded-full bg-[#00ff88] animate-pulse"></div>
          <span class="text-[#00ff88] font-medium">{appStatusValue.currentStrategyName}</span>
        </div>
        <button
          onclick={stopStrategy}
          class="px-4 py-2 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-xl transition-all duration-200"
        >
          Отключить
        </button>
      </div>
    {/if}
  </div>

  <!-- Search and Filters -->
  <div class="flex items-center gap-4">
    <!-- Search -->
    <div class="flex-1 relative">
      <svg class="w-5 h-5 text-[#a0a0a0] absolute left-4 top-1/2 -translate-y-1/2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
      </svg>
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Поиск стратегий..."
        class="w-full bg-[#1a1f3a] text-white rounded-xl pl-12 pr-4 py-3 border border-[#2a2f4a] focus:border-[#00d4ff] focus:outline-none placeholder-[#a0a0a0]"
      />
    </div>

    <!-- Family Filter -->
    <div class="flex items-center gap-2 bg-[#1a1f3a] rounded-xl p-1 border border-[#2a2f4a]">
      <button
        onclick={() => filterFamily = 'all'}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 {filterFamily === 'all' ? 'bg-[#2a2f4a] text-white' : 'text-[#a0a0a0] hover:text-white'}"
      >
        Все
      </button>
      <button
        onclick={() => filterFamily = 'zapret'}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 {filterFamily === 'zapret' ? 'bg-[#00d4ff]/20 text-[#00d4ff]' : 'text-[#a0a0a0] hover:text-white'}"
      >
        Zapret
      </button>
      <button
        onclick={() => filterFamily = 'vless'}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 {filterFamily === 'vless' ? 'bg-[#00ff88]/20 text-[#00ff88]' : 'text-[#a0a0a0] hover:text-white'}"
      >
        VLESS
      </button>
      <button
        onclick={() => filterFamily = 'custom'}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 {filterFamily === 'custom' ? 'bg-[#ffaa00]/20 text-[#ffaa00]' : 'text-[#a0a0a0] hover:text-white'}"
      >
        Custom
      </button>
    </div>
  </div>

  <!-- Loading State -->
  {#if loading}
    <div class="flex items-center justify-center py-20">
      <div class="flex flex-col items-center gap-4">
        <svg class="w-12 h-12 text-[#00d4ff] animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <p class="text-[#a0a0a0]">Загрузка стратегий...</p>
      </div>
    </div>
  {:else if error}
    <div class="bg-[#ff3333]/10 border border-[#ff3333]/20 rounded-xl p-6 text-center">
      <svg class="w-12 h-12 text-[#ff3333] mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
      </svg>
      <p class="text-[#ff3333]">{error}</p>
    </div>
  {:else if filteredStrategies.length === 0}
    <div class="bg-[#1a1f3a] rounded-2xl p-12 border border-[#2a2f4a] text-center">
      <svg class="w-16 h-16 text-[#a0a0a0]/50 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
      </svg>
      <p class="text-[#a0a0a0] text-lg">Стратегии не найдены</p>
      <p class="text-[#a0a0a0]/70 text-sm mt-2">Попробуйте изменить параметры поиска</p>
    </div>
  {:else}
    <!-- Strategies Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each filteredStrategies as strategy}
        <div 
          class="bg-[#1a1f3a] rounded-xl p-5 border transition-all duration-200 hover:border-[{getFamilyColor(strategy.family)}]/50 {strategy.isActive ? 'border-[#00ff88] ring-1 ring-[#00ff88]/20' : 'border-[#2a2f4a]'}"
        >
          <!-- Header -->
          <div class="flex items-start justify-between mb-4">
            <div class="flex items-center gap-3">
              <div 
                class="w-10 h-10 rounded-lg flex items-center justify-center"
                style="background: {getFamilyColor(strategy.family)}20"
              >
                <svg class="w-5 h-5" style="color: {getFamilyColor(strategy.family)}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getFamilyIcon(strategy.family)}/>
                </svg>
              </div>
              <div>
                <h3 class="text-white font-semibold">{strategy.name}</h3>
                <span 
                  class="text-xs font-medium px-2 py-0.5 rounded"
                  style="background: {getFamilyColor(strategy.family)}20; color: {getFamilyColor(strategy.family)}"
                >
                  {strategy.family.toUpperCase()}
                </span>
              </div>
            </div>
            
            {#if strategy.isActive}
              <div class="flex items-center gap-1 px-2 py-1 bg-[#00ff88]/10 rounded-full">
                <div class="w-2 h-2 rounded-full bg-[#00ff88] animate-pulse"></div>
                <span class="text-[#00ff88] text-xs font-medium">Активна</span>
              </div>
            {/if}
          </div>

          <!-- Description -->
          <p class="text-[#a0a0a0] text-sm mb-4 line-clamp-2">{strategy.description}</p>

          <!-- Services -->
          <div class="flex flex-wrap gap-1 mb-4">
            {#each strategy.services as service}
              <span class="px-2 py-1 bg-[#2a2f4a] text-[#a0a0a0] text-xs rounded">
                {service}
              </span>
            {/each}
          </div>

          <!-- Score & Last Tested -->
          <div class="flex items-center justify-between mb-4 text-sm">
            <div class="flex items-center gap-2">
              <span class="text-[#a0a0a0]">Оценка:</span>
              {#if strategy.score !== null}
                <span class="font-mono font-bold" style="color: {getScoreColor(strategy.score)}">{strategy.score}%</span>
              {:else}
                <span class="text-[#a0a0a0]">—</span>
              {/if}
            </div>
            <span class="text-[#a0a0a0] text-xs">{formatDate(strategy.lastTested)}</span>
          </div>

          <!-- Actions -->
          <div class="flex gap-2">
            {#if strategy.isActive}
              <button
                onclick={stopStrategy}
                class="flex-1 px-4 py-2 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-lg text-sm font-medium transition-all duration-200"
              >
                Отключить
              </button>
            {:else}
              <button
                onclick={() => applyStrategy(strategy.id)}
                disabled={applyingStrategy === strategy.id}
                class="flex-1 px-4 py-2 text-white rounded-lg text-sm font-medium transition-all duration-200 disabled:opacity-50"
                style="background: {getFamilyColor(strategy.family)}"
              >
                {#if applyingStrategy === strategy.id}
                  <svg class="w-4 h-4 animate-spin mx-auto" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                {:else}
                  Применить
                {/if}
              </button>
            {/if}
            <button
              onclick={() => testStrategy(strategy.id)}
              class="px-4 py-2 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-[#a0a0a0] hover:text-white rounded-lg text-sm transition-all duration-200"
              title="Тестировать"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- Stats Footer -->
  {#if !loading && !error}
    <div class="bg-[#1a1f3a] rounded-xl p-4 border border-[#2a2f4a]">
      <div class="flex items-center justify-between text-sm">
        <div class="flex items-center gap-6">
          <span class="text-[#a0a0a0]">
            Всего: <span class="text-white font-medium">{strategies.length}</span>
          </span>
          <span class="text-[#a0a0a0]">
            Zapret: <span class="text-[#00d4ff] font-medium">{strategiesByFamily.zapret.length}</span>
          </span>
          <span class="text-[#a0a0a0]">
            VLESS: <span class="text-[#00ff88] font-medium">{strategiesByFamily.vless.length}</span>
          </span>
          <span class="text-[#a0a0a0]">
            Custom: <span class="text-[#ffaa00] font-medium">{strategiesByFamily.custom.length}</span>
          </span>
        </div>
        <a 
          href="/testing"
          class="text-[#00d4ff] hover:underline flex items-center gap-1"
        >
          Тестирование
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
        </a>
      </div>
    </div>
  {/if}
</div>
