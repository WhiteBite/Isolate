<script lang="ts">
  import type { MarketplacePlugin } from './types';
  import { renderStars, formatDownloads } from './types';

  interface Props {
    plugins: MarketplacePlugin[];
    onInstall: (id: string) => void;
    onViewDetails: (id: string) => void;
  }

  let { plugins, onInstall, onViewDetails }: Props = $props();
  let featuredIndex = $state(0);
</script>

<div class="mb-8">
  <!-- Featured Header -->
  <div class="flex items-center justify-between mb-5">
    <div class="flex items-center gap-3">
      <div class="p-2 bg-gradient-to-br from-indigo-500 via-purple-500 to-pink-500 rounded-xl shadow-lg shadow-indigo-500/20">
        <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
        </svg>
      </div>
      <div>
        <h2 class="text-xl font-bold text-zinc-100">Featured Plugins</h2>
        <p class="text-xs text-zinc-400">Hand-picked by the Isolate team</p>
      </div>
    </div>
    <!-- Featured navigation dots -->
    <div class="flex items-center gap-2">
      {#each plugins as _, i}
        <button
          onclick={() => featuredIndex = i}
          aria-label="Go to slide {i + 1}"
          class="w-2 h-2 rounded-full transition-all {featuredIndex === i ? 'bg-indigo-500 w-6' : 'bg-zinc-700 hover:bg-zinc-600'}"
        ></button>
      {/each}
    </div>
  </div>
  
  <!-- Featured Cards Grid -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-5">
    {#each plugins as plugin (plugin.id)}
      <div 
        class="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-zinc-900/80 to-zinc-900/40 
               border border-indigo-500/20 hover:border-indigo-500/40
               shadow-xl shadow-indigo-500/5 hover:shadow-indigo-500/15
               transform hover:scale-[1.02] hover:-translate-y-1
               transition-all duration-300 ease-out"
      >
        <div class="absolute inset-0 bg-gradient-to-br from-indigo-500/10 via-purple-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
        
        <div class="relative p-5">
          <!-- Featured badge -->
          <div class="absolute top-4 right-4">
            <span class="flex items-center gap-1.5 px-2.5 py-1 bg-gradient-to-r from-indigo-500/20 to-purple-500/20 rounded-full text-xs text-indigo-300 border border-indigo-500/30 backdrop-blur-sm">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
              </svg>
              Featured
            </span>
          </div>
          
          <!-- Icon -->
          <div class="w-16 h-16 rounded-2xl bg-zinc-800/80 flex items-center justify-center text-4xl mb-4 
                      group-hover:scale-110 group-hover:shadow-lg group-hover:shadow-indigo-500/20 transition-all duration-300">
            {plugin.icon}
          </div>
          
          <!-- Title & Version -->
          <div class="flex items-center gap-2 mb-2">
            <h3 class="text-lg font-bold text-zinc-100 group-hover:text-indigo-200 transition-colors">{plugin.name}</h3>
            <span class="text-xs text-zinc-400 font-mono">v{plugin.version}</span>
          </div>
          
          <!-- Description -->
          <p class="text-sm text-zinc-400 line-clamp-2 mb-4">{plugin.description}</p>
          
          <!-- Stats row -->
          <div class="flex items-center gap-4 mb-4">
            <!-- Rating -->
            <div class="flex items-center gap-1.5">
              <div class="flex items-center">
                {#each Array(renderStars(plugin.rating).full) as _}
                  <svg class="w-4 h-4 text-amber-400" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                {/each}
              </div>
              <span class="text-sm font-semibold text-amber-400">{plugin.rating.toFixed(1)}</span>
            </div>
            
            <!-- Downloads -->
            <div class="flex items-center gap-1.5 text-zinc-400">
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
              </svg>
              <span class="text-sm font-medium">{formatDownloads(plugin.downloads)}</span>
            </div>
          </div>
          
          <!-- Author & Actions -->
          <div class="flex items-center justify-between pt-4 border-t border-white/5">
            <span class="text-xs text-zinc-400 flex items-center gap-1.5">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
              </svg>
              {plugin.author}
            </span>
            
            <div class="flex items-center gap-2">
              <button
                onclick={() => onViewDetails(plugin.id)}
                class="p-2 rounded-lg text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800/50 transition-all"
                title="View Details"
              >
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>
                </svg>
              </button>
              
              <button
                onclick={() => onInstall(plugin.id)}
                disabled={plugin.installed}
                class="px-4 py-2 rounded-xl text-sm font-semibold transition-all
                       {plugin.installed 
                         ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30' 
                         : 'bg-indigo-500 hover:bg-indigo-400 text-white shadow-lg shadow-indigo-500/25 hover:shadow-indigo-500/40 hover:scale-105'}"
              >
                {#if plugin.installed}
                  <span class="flex items-center gap-1.5">
                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="20 6 9 17 4 12"/>
                    </svg>
                    Installed
                  </span>
                {:else}
                  Install
                {/if}
              </button>
            </div>
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>

<!-- Divider -->
<div class="flex items-center gap-4 mb-6">
  <div class="flex-1 h-px bg-gradient-to-r from-transparent via-zinc-700 to-transparent"></div>
  <span class="text-xs text-zinc-400 font-medium">ALL PLUGINS</span>
  <div class="flex-1 h-px bg-gradient-to-r from-transparent via-zinc-700 to-transparent"></div>
</div>
