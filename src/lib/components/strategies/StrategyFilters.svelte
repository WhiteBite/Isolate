<script lang="ts">
  import type { Category, FamilyFilter, CategoryInfo } from './types';
  import { categories } from './types';

  interface Props {
    selectedCategory: Category;
    selectedFamily: FamilyFilter;
    searchQuery: string;
    strategiesByCategory: Record<string, number>;
    strategiesByFamily: { zapret: number; vless: number };
    onCategoryChange: (category: Category) => void;
    onFamilyChange: (family: FamilyFilter) => void;
    onSearchChange: (query: string) => void;
  }

  let { 
    selectedCategory, 
    selectedFamily, 
    searchQuery,
    strategiesByCategory,
    strategiesByFamily,
    onCategoryChange, 
    onFamilyChange,
    onSearchChange
  }: Props = $props();
</script>

<!-- Category Tabs -->
<div class="flex flex-wrap gap-2">
  {#each categories as cat}
    <button
      onclick={() => onCategoryChange(cat.id)}
      class="px-4 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 flex items-center gap-2
        {selectedCategory === cat.id 
          ? 'bg-indigo-500 text-white shadow-lg shadow-glow' 
          : 'bg-void-50 text-text-secondary hover:bg-void-100 hover:text-text-primary border border-glass-border'}"
    >
      <span class="text-base">{cat.icon}</span>
      <span>{cat.name}</span>
      {#if cat.id !== 'all'}
        <span class="px-1.5 py-0.5 text-xs rounded-md {selectedCategory === cat.id ? 'bg-white/20' : 'bg-void-100'}">
          {strategiesByCategory[cat.id] || 0}
        </span>
      {/if}
    </button>
  {/each}
</div>

<!-- Family Filter (Zapret/VLESS) -->
<div class="flex items-center gap-4">
  <span class="text-text-muted text-sm">Type:</span>
  <div class="flex gap-2 p-1 bg-void rounded-xl border border-glass-border">
    <button
      onclick={() => onFamilyChange('all')}
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200
        {selectedFamily === 'all' 
          ? 'bg-void-100 text-text-primary' 
          : 'text-text-muted hover:text-text-primary'}"
    >
      All
    </button>
    <button
      onclick={() => onFamilyChange('zapret')}
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2
        {selectedFamily === 'zapret' 
          ? 'bg-neon-cyan/20 text-neon-cyan' 
          : 'text-text-muted hover:text-neon-cyan'}"
    >
      <span class="w-2 h-2 rounded-full bg-neon-cyan"></span>
      Zapret
      <span class="text-xs opacity-70">({strategiesByFamily.zapret})</span>
    </button>
    <button
      onclick={() => onFamilyChange('vless')}
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 flex items-center gap-2
        {selectedFamily === 'vless' 
          ? 'bg-neon-green/20 text-neon-green' 
          : 'text-text-muted hover:text-neon-green'}"
    >
      <span class="w-2 h-2 rounded-full bg-neon-green"></span>
      VLESS
      <span class="text-xs opacity-70">({strategiesByFamily.vless})</span>
    </button>
  </div>
</div>

<!-- Search -->
<div class="relative group">
  <svg class="w-5 h-5 text-text-muted absolute left-4 top-1/2 -translate-y-1/2 transition-colors group-focus-within:text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
  </svg>
  <input
    type="text"
    value={searchQuery}
    oninput={(e) => onSearchChange(e.currentTarget.value)}
    placeholder="Search by name or description..."
    class="w-full bg-void text-text-primary rounded-xl pl-12 pr-12 py-3.5 border border-glass-border focus:border-indigo-500/50 focus:outline-none focus:ring-1 focus:ring-indigo-500/30 placeholder-text-muted transition-all"
  />
  {#if searchQuery}
    <button
      aria-label="Clear search"
      onclick={() => onSearchChange('')}
      class="absolute right-4 top-1/2 -translate-y-1/2 p-1 text-text-muted hover:text-text-primary transition-colors"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
      </svg>
    </button>
  {/if}
</div>
