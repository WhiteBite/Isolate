<script lang="ts">
  interface Props {
    totalPlugins: number;
    installedCount: number;
    reloading: boolean;
    viewMode: 'grid' | 'list';
    onReloadAll: () => void;
    onViewModeChange: (mode: 'grid' | 'list') => void;
  }

  let { totalPlugins, installedCount, reloading, viewMode, onReloadAll, onViewModeChange }: Props = $props();
</script>

<div class="flex items-center justify-between mb-6">
  <div>
    <h1 class="text-2xl font-bold text-text-primary">Marketplace</h1>
    <p class="text-text-secondary mt-1">Extensions and plugins for Isolate</p>
  </div>
  <div class="flex items-center gap-3">
    <!-- Stats badges -->
    <div class="flex items-center gap-2 text-sm">
      <span class="px-3 py-1.5 bg-void-100 border border-glass-border rounded-lg text-text-secondary flex items-center gap-2">
        <span class="text-text-secondary">📦</span>
        <span class="font-medium text-text-primary">{totalPlugins}</span>
        <span class="text-text-secondary">plugins</span>
      </span>
      <span class="px-3 py-1.5 bg-emerald-500/10 rounded-lg text-emerald-400 flex items-center gap-2 border border-emerald-500/20">
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
        <span class="font-medium">{installedCount}</span>
        <span class="text-emerald-400">installed</span>
      </span>
    </div>
    
    <!-- Reload All button -->
    <button
      onclick={onReloadAll}
      disabled={reloading}
      class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-all
             {reloading 
               ? 'bg-void-100 text-text-secondary cursor-wait' 
               : 'bg-indigo-500 text-white hover:bg-indigo-600'}"
      title="Reload all plugins"
    >
      {#if reloading}
        <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" stroke-opacity="0.25"/>
          <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
        </svg>
        <span>Updating...</span>
      {:else}
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M1 4v6h6M23 20v-6h-6"/>
          <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15"/>
        </svg>
        <span>Reload All</span>
      {/if}
    </button>
    
    <!-- View mode toggle -->
    <div class="flex items-center bg-void-50 border border-glass-border rounded-lg p-1">
      <button
        onclick={() => onViewModeChange('grid')}
        class="p-2 rounded-md transition-all {viewMode === 'grid' ? 'bg-void-200 text-text-primary' : 'text-text-secondary hover:text-text-primary'}"
        title="Grid"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="7" rx="1"/>
          <rect x="14" y="3" width="7" height="7" rx="1"/>
          <rect x="3" y="14" width="7" height="7" rx="1"/>
          <rect x="14" y="14" width="7" height="7" rx="1"/>
        </svg>
      </button>
      <button
        onclick={() => onViewModeChange('list')}
        class="p-2 rounded-md transition-all {viewMode === 'list' ? 'bg-void-200 text-text-primary' : 'text-text-secondary hover:text-text-primary'}"
        title="List"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="8" y1="6" x2="21" y2="6"/>
          <line x1="8" y1="12" x2="21" y2="12"/>
          <line x1="8" y1="18" x2="21" y2="18"/>
          <line x1="3" y1="6" x2="3.01" y2="6"/>
          <line x1="3" y1="12" x2="3.01" y2="12"/>
          <line x1="3" y1="18" x2="3.01" y2="18"/>
        </svg>
      </button>
    </div>
  </div>
</div>
