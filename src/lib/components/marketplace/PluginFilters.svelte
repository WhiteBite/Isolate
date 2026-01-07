<script lang="ts">
  import { typeTabs, levelFilters, sortOptions, getTabColorClasses, getLevelBadgeClasses } from './types';
  import type { TypeFilter, LevelFilter, SortOption, MarketplacePlugin } from './types';

  interface Props {
    plugins: MarketplacePlugin[];
    searchQuery: string;
    selectedType: TypeFilter;
    selectedLevel: LevelFilter;
    sortBy: SortOption;
    showInstalledOnly: boolean;
    filteredCount: number;
    hasActiveFilters: boolean;
    onSearchChange: (value: string) => void;
    onTypeChange: (value: TypeFilter) => void;
    onLevelChange: (value: LevelFilter) => void;
    onSortChange: (value: SortOption) => void;
    onInstalledOnlyChange: (value: boolean) => void;
    onResetFilters: () => void;
  }

  let { 
    plugins, searchQuery, selectedType, selectedLevel, sortBy, showInstalledOnly,
    filteredCount, hasActiveFilters,
    onSearchChange, onTypeChange, onLevelChange, onSortChange, onInstalledOnlyChange, onResetFilters
  }: Props = $props();

  function getTypeCount(type: TypeFilter): number {
    if (type === 'all') return plugins.length;
    return plugins.filter(p => p.type === type).length;
  }
</script>

<!-- Type Tabs -->
<div class="mb-6">
  <div class="flex items-center gap-2 overflow-x-auto pb-2 scrollbar-thin scrollbar-thumb-void-200">
    {#each typeTabs as tab (tab.id)}
      <button
        onclick={() => onTypeChange(tab.id)}
        class="flex items-center gap-2.5 px-4 py-2.5 rounded-xl text-sm font-medium transition-all whitespace-nowrap
               {getTabColorClasses(tab.color, selectedType === tab.id)}"
        title={tab.description}
      >
        <span class="text-lg">{tab.icon}</span>
        <span>{tab.label}</span>
        <span class="px-2 py-0.5 text-xs rounded-md
                     {selectedType === tab.id ? 'bg-white/20' : 'bg-void-100'}">
          {getTypeCount(tab.id)}
        </span>
      </button>
    {/each}
  </div>
</div>

<!-- Secondary Filters Row -->
<div class="flex flex-wrap items-center gap-4 mb-6 p-4 bg-void-50 rounded-xl border border-glass-border">
  <!-- Search -->
  <div class="flex-1 min-w-[250px] relative">
    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"/>
      <path d="m21 21-4.35-4.35"/>
    </svg>
    <input
      type="text"
      value={searchQuery}
      oninput={(e) => onSearchChange(e.currentTarget.value)}
      placeholder="Search by name, description or author..."
      class="w-full pl-10 pr-10 py-2.5 bg-void-100 border border-glass-border rounded-xl
             text-text-primary placeholder-text-secondary
             focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20
             transition-all"
    />
    {#if searchQuery}
      <button
        onclick={() => onSearchChange('')}
        aria-label="Clear search"
        class="absolute right-3 top-1/2 -translate-y-1/2 text-text-secondary hover:text-text-primary transition-colors"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
      </button>
    {/if}
  </div>

  <!-- Level Filter -->
  <div class="flex items-center gap-2">
    <span class="text-xs text-text-secondary font-medium">Level:</span>
    <div class="flex items-center gap-1">
      {#each levelFilters as level (level.id)}
        <button
          onclick={() => onLevelChange(level.id)}
          class="px-3 py-1.5 rounded-lg text-xs font-semibold transition-all border
                 {selectedLevel === level.id 
                   ? getLevelBadgeClasses(level.id, true)
                   : getLevelBadgeClasses(level.id, false) + ' hover:bg-void-200'}"
          title={level.description}
        >
          {level.shortLabel}
        </button>
      {/each}
    </div>
  </div>

  <div class="w-px h-8 bg-glass-border"></div>

  <!-- Sort -->
  <div class="flex items-center gap-2">
    <span class="text-xs text-text-secondary font-medium">Sort:</span>
    <select
      value={sortBy}
      onchange={(e) => onSortChange(e.currentTarget.value as SortOption)}
      class="px-3 py-1.5 bg-void-100 border border-glass-border rounded-lg
             text-sm text-text-primary focus:outline-none focus:border-indigo-500/50 cursor-pointer
             transition-all"
    >
      {#each sortOptions as option (option.id)}
        <option value={option.id}>{option.icon} {option.label}</option>
      {/each}
    </select>
  </div>

  <!-- Installed Only Toggle -->
  <button
    onclick={() => onInstalledOnlyChange(!showInstalledOnly)}
    class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all border
           {showInstalledOnly 
             ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30' 
             : 'bg-void-100 text-text-secondary border-glass-border hover:bg-void-200'}"
  >
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="20 6 9 17 4 12"/>
    </svg>
    Installed
  </button>
</div>

<!-- Active Filters Summary -->
{#if hasActiveFilters}
  <div class="mb-4 flex items-center justify-between p-3 bg-indigo-500/10 border border-indigo-500/20 rounded-xl">
    <div class="flex items-center gap-3">
      <svg class="w-5 h-5 text-indigo-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
      </svg>
      <span class="text-sm text-text-primary">
        Found: <span class="font-semibold text-indigo-400">{filteredCount}</span> plugins
        {#if searchQuery}
          <span class="text-text-secondary">for query</span>
          <span class="px-2 py-0.5 bg-void-100 rounded text-text-primary">"{searchQuery}"</span>
        {/if}
      </span>
    </div>
    <button
      onclick={onResetFilters}
      class="text-sm text-indigo-400 hover:text-indigo-300 transition-colors flex items-center gap-1.5 px-3 py-1.5 rounded-lg hover:bg-indigo-500/10"
    >
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
      Reset filters
    </button>
  </div>
{/if}
