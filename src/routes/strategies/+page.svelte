<script lang="ts">
  import { browser } from '$app/environment';
  import { appStatus } from '$lib/stores';

  // Types
  type Category = 'all' | 'youtube' | 'discord' | 'telegram' | 'general' | 'games' | 'custom';
  type Label = 'recommended' | 'experimental' | 'stable' | null;

  interface Strategy {
    id: string;
    name: string;
    family: 'zapret' | 'vless' | 'custom';
    category: Category;
    description: string;
    services: string[];
    score: number | null;
    lastTested: Date | null;
    isActive: boolean;
    label: Label;
    author: string;
  }

  interface CategoryInfo {
    id: Category;
    name: string;
    icon: string;
  }

  // Categories configuration
  const categories: CategoryInfo[] = [
    { id: 'all', name: 'Все', icon: '📋' },
    { id: 'youtube', name: 'YouTube', icon: '📺' },
    { id: 'discord', name: 'Discord', icon: '💬' },
    { id: 'telegram', name: 'Telegram', icon: '✈️' },
    { id: 'general', name: 'Общие', icon: '🌐' },
    { id: 'games', name: 'Игры', icon: '🎮' },
    { id: 'custom', name: 'Свои', icon: '⚙️' }
  ];

  // Family filter type
  type FamilyFilter = 'all' | 'zapret' | 'vless';

  // State using Svelte 5 runes
  let strategies = $state<Strategy[]>([]);
  let loading = $state(false);  // Start false, set true when loading
  let error = $state<string | null>(null);
  let searchQuery = $state('');
  let selectedCategory = $state<Category>('all');
  let selectedFamily = $state<FamilyFilter>('all');
  let applyingStrategy = $state<string | null>(null);
  let selectedStrategyDetails = $state<Strategy | null>(null);

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
      const matchesCategory = selectedCategory === 'all' || s.category === selectedCategory;
      const matchesFamily = selectedFamily === 'all' || s.family === selectedFamily;
      return matchesSearch && matchesCategory && matchesFamily;
    })
  );

  // Count strategies by family
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

  function getDemoStrategies(): Strategy[] {
    return [
      {
        id: 'discord-zapret-1',
        name: 'Discord Zapret Basic',
        family: 'zapret',
        category: 'discord',
        description: 'Базовая стратегия для Discord через Zapret. Работает с голосовыми каналами и текстовыми чатами.',
        services: ['discord'],
        score: 85,
        lastTested: new Date(),
        isActive: false,
        label: 'recommended',
        author: 'Isolate Team'
      },
      {
        id: 'youtube-zapret-1',
        name: 'YouTube Zapret',
        family: 'zapret',
        category: 'youtube',
        description: 'Обход блокировки YouTube. Поддерживает просмотр видео и стримов.',
        services: ['youtube'],
        score: 78,
        lastTested: new Date(),
        isActive: false,
        label: 'stable',
        author: 'Community'
      },
      {
        id: 'universal-vless-1',
        name: 'Universal VLESS',
        family: 'vless',
        category: 'general',
        description: 'Универсальная VLESS стратегия для всех сервисов. Высокая стабильность и скорость.',
        services: ['discord', 'youtube', 'telegram'],
        score: 95,
        lastTested: new Date(),
        isActive: false,
        label: 'recommended',
        author: 'Isolate Team'
      },
      {
        id: 'telegram-zapret-1',
        name: 'Telegram Zapret',
        family: 'zapret',
        category: 'telegram',
        description: 'Стратегия для Telegram. Поддерживает все функции мессенджера.',
        services: ['telegram'],
        score: 82,
        lastTested: new Date(),
        isActive: false,
        label: 'stable',
        author: 'Community'
      },
      {
        id: 'games-vless-1',
        name: 'Gaming VLESS',
        family: 'vless',
        category: 'games',
        description: 'Оптимизированная стратегия для онлайн-игр с низким пингом.',
        services: ['steam', 'epic', 'riot'],
        score: 88,
        lastTested: new Date(),
        isActive: false,
        label: 'experimental',
        author: 'GameDev'
      },
      {
        id: 'custom-user-1',
        name: 'Custom Strategy',
        family: 'custom',
        category: 'custom',
        description: 'Пользовательская стратегия с настраиваемыми параметрами.',
        services: ['custom'],
        score: null,
        lastTested: null,
        isActive: false,
        label: null,
        author: 'User'
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
          category: mapServiceToCategory(s.services),
          description: s.description || '',
          services: s.services || [],
          score: s.score,
          lastTested: s.last_tested ? new Date(s.last_tested) : null,
          isActive: appStatusValue.currentStrategy === s.id,
          label: s.label || null,
          author: s.author || 'Unknown'
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

  function mapServiceToCategory(services: string[]): Category {
    if (!services || services.length === 0) return 'general';
    const service = services[0].toLowerCase();
    if (service.includes('youtube')) return 'youtube';
    if (service.includes('discord')) return 'discord';
    if (service.includes('telegram')) return 'telegram';
    if (service.includes('steam') || service.includes('epic') || service.includes('riot') || service.includes('game')) return 'games';
    if (service.includes('custom')) return 'custom';
    return 'general';
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

  function showDetails(strategy: Strategy) {
    selectedStrategyDetails = strategy;
  }

  function closeDetails() {
    selectedStrategyDetails = null;
  }

  function getFamilyColor(family: string): string {
    switch (family) {
      case 'zapret': return '#00d4ff';
      case 'vless': return '#00ff88';
      case 'custom': return '#ffaa00';
      default: return '#a0a0a0';
    }
  }

  function getCategoryColor(category: Category): string {
    switch (category) {
      case 'youtube': return '#ff0000';
      case 'discord': return '#5865f2';
      case 'telegram': return '#0088cc';
      case 'general': return '#00d4ff';
      case 'games': return '#9b59b6';
      case 'custom': return '#ffaa00';
      default: return '#a0a0a0';
    }
  }

  function getLabelInfo(label: Label): { text: string; color: string; bg: string } {
    switch (label) {
      case 'recommended': return { text: 'Рекомендуется', color: '#00ff88', bg: '#00ff88' };
      case 'experimental': return { text: 'Экспериментальная', color: '#ffaa00', bg: '#ffaa00' };
      case 'stable': return { text: 'Стабильная', color: '#00d4ff', bg: '#00d4ff' };
      default: return { text: '', color: '', bg: '' };
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

  <!-- Category Tabs -->
  <div class="flex flex-wrap gap-2">
    {#each categories as cat}
      <button
        onclick={() => selectedCategory = cat.id}
        class="px-4 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 flex items-center gap-2
          {selectedCategory === cat.id 
            ? 'bg-[#00d4ff] text-white shadow-lg shadow-[#00d4ff]/20' 
            : 'bg-[#1a1f3a] text-[#a0a0a0] hover:bg-[#2a2f4a] hover:text-white border border-[#2a2f4a]'}"
      >
        <span class="text-base">{cat.icon}</span>
        <span>{cat.name}</span>
        {#if cat.id !== 'all'}
          <span class="px-1.5 py-0.5 text-xs rounded-md {selectedCategory === cat.id ? 'bg-white/20' : 'bg-[#2a2f4a]'}">
            {strategiesByCategory[cat.id] || 0}
          </span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Family Filter (Zapret/VLESS) -->
  <div class="flex items-center gap-4">
    <span class="text-[#a0a0a0] text-sm">Тип:</span>
    <div class="flex gap-2 p-1 bg-[#0d1229] rounded-xl border border-[#2a2f4a]">
      <button
        onclick={() => selectedFamily = 'all'}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200
          {selectedFamily === 'all' 
            ? 'bg-[#2a2f4a] text-white' 
            : 'text-[#a0a0a0] hover:text-white'}"
      >
        Все
      </button>
      <button
        onclick={() => selectedFamily = 'zapret'}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2
          {selectedFamily === 'zapret' 
            ? 'bg-[#00d4ff]/20 text-[#00d4ff]' 
            : 'text-[#a0a0a0] hover:text-[#00d4ff]'}"
      >
        <span class="w-2 h-2 rounded-full bg-[#00d4ff]"></span>
        Zapret
        <span class="text-xs opacity-70">({strategiesByFamily.zapret})</span>
      </button>
      <button
        onclick={() => selectedFamily = 'vless'}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2
          {selectedFamily === 'vless' 
            ? 'bg-[#00ff88]/20 text-[#00ff88]' 
            : 'text-[#a0a0a0] hover:text-[#00ff88]'}"
      >
        <span class="w-2 h-2 rounded-full bg-[#00ff88]"></span>
        VLESS
        <span class="text-xs opacity-70">({strategiesByFamily.vless})</span>
      </button>
    </div>
  </div>

  <!-- Search -->
  <div class="relative group">
    <svg class="w-5 h-5 text-[#a0a0a0] absolute left-4 top-1/2 -translate-y-1/2 transition-colors group-focus-within:text-[#00d4ff]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
    </svg>
    <input
      type="text"
      bind:value={searchQuery}
      placeholder="Поиск по названию или описанию..."
      class="w-full bg-[#0d1229] text-white rounded-xl pl-12 pr-12 py-3.5 border border-[#2a2f4a] focus:border-[#00d4ff] focus:outline-none focus:ring-1 focus:ring-[#00d4ff]/30 placeholder-[#a0a0a0]/70 transition-all"
    />
    {#if searchQuery}
      <button
        onclick={() => searchQuery = ''}
        class="absolute right-4 top-1/2 -translate-y-1/2 p-1 text-[#a0a0a0] hover:text-white transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>
    {/if}
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
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
      {#each filteredStrategies as strategy}
        {@const familyColor = getFamilyColor(strategy.family)}
        <div 
          class="group relative bg-gradient-to-br from-[#1a1f3a] to-[#0d1229] rounded-2xl p-5 border transition-all duration-300 hover:-translate-y-1
            {strategy.isActive 
              ? 'border-[#00ff88]/50 shadow-[0_0_30px_rgba(0,255,136,0.15)]' 
              : 'border-[#2a2f4a] hover:border-[#00d4ff]/30 hover:shadow-[0_0_20px_rgba(0,212,255,0.1)]'}"
          style="{strategy.isActive ? 'box-shadow: 0 0 40px rgba(0,255,136,0.2), inset 0 1px 0 rgba(0,255,136,0.1);' : ''}"
        >
          <!-- Glow effect for active strategy -->
          {#if strategy.isActive}
            <div class="absolute inset-0 rounded-2xl bg-gradient-to-br from-[#00ff88]/5 to-transparent pointer-events-none"></div>
          {/if}
          
          <!-- Glass overlay on hover -->
          <div class="absolute inset-0 rounded-2xl bg-gradient-to-br from-white/[0.02] to-transparent opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"></div>
          
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
                style="background: {getCategoryColor(strategy.category)}15; color: {getCategoryColor(strategy.category)}"
              >
                {categories.find(c => c.id === strategy.category)?.icon || '🌐'}
              </span>
              
              <!-- Label Badge -->
              {#if strategy.label}
                {@const labelInfo = getLabelInfo(strategy.label)}
                <span 
                  class="px-2 py-1 rounded-lg text-xs font-medium"
                  style="background: {labelInfo.bg}12; color: {labelInfo.color}"
                >
                  {labelInfo.text}
                </span>
              {/if}
            </div>
            
            {#if strategy.isActive}
              <div class="flex items-center gap-1.5 px-2.5 py-1 bg-[#00ff88]/15 rounded-full border border-[#00ff88]/30">
                <div class="w-2 h-2 rounded-full bg-[#00ff88] animate-pulse shadow-[0_0_8px_#00ff88]"></div>
                <span class="text-[#00ff88] text-xs font-semibold">Активна</span>
              </div>
            {/if}
          </div>

          <!-- Title -->
          <h3 class="relative text-white font-semibold text-lg mb-2 group-hover:text-[#00d4ff] transition-colors">{strategy.name}</h3>

          <!-- Description -->
          <p class="relative text-[#a0a0a0] text-sm mb-4 line-clamp-2 leading-relaxed">{strategy.description}</p>

          <!-- Meta info row -->
          <div class="relative flex items-center justify-between mb-4 py-3 px-3 bg-[#0a0e27]/50 rounded-xl border border-[#2a2f4a]/50">
            <div class="flex items-center gap-2 text-sm">
              <svg class="w-4 h-4 text-[#a0a0a0]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
              </svg>
              <span class="text-[#a0a0a0]">{strategy.author}</span>
            </div>
            
            <div class="flex items-center gap-2">
              <span class="text-[#a0a0a0] text-sm">Оценка:</span>
              {#if strategy.score !== null}
                <span 
                  class="font-mono font-bold text-sm px-2 py-0.5 rounded"
                  style="color: {getScoreColor(strategy.score)}; background: {getScoreColor(strategy.score)}15;"
                >
                  {strategy.score}%
                </span>
              {:else}
                <span class="text-[#a0a0a0] text-sm">—</span>
              {/if}
            </div>
          </div>

          <!-- Actions -->
          <div class="relative flex gap-2">
            {#if strategy.isActive}
              <button
                onclick={stopStrategy}
                class="flex-1 px-4 py-2.5 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-xl text-sm font-medium transition-all duration-200 border border-[#3a3f5a]"
              >
                Отключить
              </button>
            {:else}
              <button
                onclick={() => applyStrategy(strategy.id)}
                disabled={applyingStrategy === strategy.id}
                class="flex-1 px-4 py-2.5 bg-gradient-to-r from-[#00d4ff] to-[#00b8e0] hover:from-[#00b8e0] hover:to-[#0099cc] text-white rounded-xl text-sm font-semibold transition-all duration-200 disabled:opacity-50 shadow-lg shadow-[#00d4ff]/20 hover:shadow-[#00d4ff]/30"
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
              class="px-3 py-2.5 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-[#a0a0a0] hover:text-[#00d4ff] rounded-xl text-sm transition-all duration-200 border border-[#3a3f5a] hover:border-[#00d4ff]/30"
              title="Тестировать"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
            </button>
            
            <button
              onclick={() => showDetails(strategy)}
              class="px-3 py-2.5 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-[#a0a0a0] hover:text-white rounded-xl text-sm transition-all duration-200 border border-[#3a3f5a]"
              title="Детали"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- Stats Footer -->
  {#if !loading && !error}
    <div class="bg-gradient-to-r from-[#1a1f3a] to-[#0d1229] rounded-xl p-4 border border-[#2a2f4a]">
      <div class="flex items-center justify-between text-sm">
        <div class="flex items-center gap-6">
          <span class="text-[#a0a0a0]">
            Всего: <span class="text-white font-medium">{strategies.length}</span>
          </span>
          <span class="text-[#a0a0a0]">
            Показано: <span class="text-white font-medium">{filteredStrategies.length}</span>
          </span>
          <span class="text-[#a0a0a0] flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-[#00d4ff]"></span>
            Zapret: <span class="text-[#00d4ff] font-medium">{strategiesByFamily.zapret}</span>
          </span>
          <span class="text-[#a0a0a0] flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-[#00ff88]"></span>
            VLESS: <span class="text-[#00ff88] font-medium">{strategiesByFamily.vless}</span>
          </span>
        </div>
        <a 
          href="/testing"
          class="text-[#00d4ff] hover:text-[#00b8e0] flex items-center gap-1 transition-colors"
        >
          Тестирование
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
        </a>
      </div>
    </div>
  {/if}

  <!-- Details Modal -->
  {#if selectedStrategyDetails}
    <div 
      class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4"
      onclick={closeDetails}
    >
      <div 
        role="dialog"
        aria-modal="true"
        aria-labelledby="strategy-details-title"
        class="bg-[#1a1f3a] rounded-2xl p-6 max-w-lg w-full border border-[#2a2f4a] shadow-2xl"
        onclick={(e) => e.stopPropagation()}
      >
        <!-- Modal Header -->
        <div class="flex items-start justify-between mb-4">
          <div>
            <h2 id="strategy-details-title" class="text-xl font-bold text-white">{selectedStrategyDetails.name}</h2>
            <div class="flex items-center gap-2 mt-2">
              <span 
                class="px-2.5 py-1 rounded-lg text-xs font-medium"
                style="background: {getCategoryColor(selectedStrategyDetails.category)}20; color: {getCategoryColor(selectedStrategyDetails.category)}"
              >
                {categories.find(c => c.id === selectedStrategyDetails?.category)?.icon} {categories.find(c => c.id === selectedStrategyDetails?.category)?.name}
              </span>
              <span 
                class="px-2 py-0.5 rounded text-xs font-medium"
                style="background: {getFamilyColor(selectedStrategyDetails.family)}20; color: {getFamilyColor(selectedStrategyDetails.family)}"
              >
                {selectedStrategyDetails.family.toUpperCase()}
              </span>
            </div>
          </div>
          <button
            onclick={closeDetails}
            aria-label="Закрыть"
            class="p-2 hover:bg-[#2a2f4a] rounded-lg transition-colors"
          >
            <svg class="w-5 h-5 text-[#a0a0a0]" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <!-- Modal Content -->
        <div class="space-y-4">
          <div>
            <h4 class="text-[#a0a0a0] text-sm mb-1">Описание</h4>
            <p class="text-white">{selectedStrategyDetails.description}</p>
          </div>
          
          <div class="grid grid-cols-2 gap-4">
            <div>
              <h4 class="text-[#a0a0a0] text-sm mb-1">Автор</h4>
              <p class="text-white">{selectedStrategyDetails.author}</p>
            </div>
            <div>
              <h4 class="text-[#a0a0a0] text-sm mb-1">Оценка</h4>
              {#if selectedStrategyDetails.score !== null}
                <p class="font-mono font-bold" style="color: {getScoreColor(selectedStrategyDetails.score)}">{selectedStrategyDetails.score}%</p>
              {:else}
                <p class="text-[#a0a0a0]">Не тестировалась</p>
              {/if}
            </div>
          </div>
          
          <div>
            <h4 class="text-[#a0a0a0] text-sm mb-2">Сервисы</h4>
            <div class="flex flex-wrap gap-2">
              {#each selectedStrategyDetails.services as service}
                <span class="px-3 py-1.5 bg-[#2a2f4a] text-white text-sm rounded-lg">
                  {service}
                </span>
              {/each}
            </div>
          </div>
          
          <div>
            <h4 class="text-[#a0a0a0] text-sm mb-1">Последнее тестирование</h4>
            <p class="text-white">{formatDate(selectedStrategyDetails.lastTested)}</p>
          </div>
          
          {#if selectedStrategyDetails.label}
            {@const labelInfo = getLabelInfo(selectedStrategyDetails.label)}
            <div class="p-3 rounded-lg" style="background: {labelInfo.bg}10; border: 1px solid {labelInfo.bg}30">
              <span class="font-medium" style="color: {labelInfo.color}">{labelInfo.text}</span>
            </div>
          {/if}
        </div>

        <!-- Modal Actions -->
        <div class="flex gap-3 mt-6">
          {#if selectedStrategyDetails.isActive}
            <button
              onclick={() => { stopStrategy(); closeDetails(); }}
              class="flex-1 px-4 py-3 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-xl font-medium transition-all duration-200"
            >
              Отключить
            </button>
          {:else}
            <button
              onclick={() => { if (selectedStrategyDetails) { applyStrategy(selectedStrategyDetails.id); closeDetails(); } }}
              class="flex-1 px-4 py-3 bg-[#00d4ff] hover:bg-[#00b8e0] text-white rounded-xl font-medium transition-all duration-200"
            >
              Применить
            </button>
          {/if}
          <button
            onclick={() => { if (selectedStrategyDetails) { testStrategy(selectedStrategyDetails.id); closeDetails(); } }}
            class="px-4 py-3 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-xl font-medium transition-all duration-200"
          >
            Тестировать
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
