<script lang="ts">
  // StrategiesSkeleton - Skeleton loader for Strategies page
  // Mimics the strategy cards grid layout with category tabs
</script>

<div class="p-8 space-y-6 min-h-screen bg-void">
  <!-- Header Skeleton -->
  <div class="flex items-center justify-between">
    <div>
      <div class="h-9 w-36 rounded-lg bg-zinc-800/50 animate-pulse"></div>
      <div class="h-4 w-48 rounded bg-zinc-800/30 animate-pulse mt-2"></div>
    </div>
  </div>

  <!-- Category Tabs Skeleton -->
  <div class="flex flex-wrap gap-2">
    {#each Array(7) as _, i}
      <div 
        class="h-10 rounded-xl bg-zinc-800/50 animate-pulse"
        style="width: {60 + (i % 3) * 20}px; animation-delay: {i * 30}ms"
      ></div>
    {/each}
  </div>

  <!-- Family Filter Skeleton -->
  <div class="flex items-center gap-4">
    <div class="h-4 w-10 rounded bg-zinc-800/30 animate-pulse"></div>
    <div class="flex gap-2 p-1 bg-void rounded-xl border border-glass-border">
      <div class="h-9 w-12 rounded-lg bg-zinc-800/50 animate-pulse"></div>
      <div class="h-9 w-24 rounded-lg bg-zinc-800/30 animate-pulse"></div>
      <div class="h-9 w-20 rounded-lg bg-zinc-800/30 animate-pulse"></div>
    </div>
  </div>

  <!-- Search Skeleton -->
  <div class="relative">
    <div class="w-full h-12 rounded-xl bg-zinc-800/50 border border-glass-border animate-pulse"></div>
  </div>

  <!-- Strategies Grid Skeleton -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
    {#each Array(6) as _, i}
      <div 
        class="bg-zinc-900/40 rounded-xl p-5 border border-white/5"
        style="animation-delay: {i * 80}ms"
      >
        <!-- Header with Badges -->
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-2 flex-wrap">
            <!-- Family Badge -->
            <div class="h-6 w-16 rounded-lg bg-cyan-500/20 animate-pulse"></div>
            <!-- Category Badge -->
            <div class="h-6 w-8 rounded-lg bg-zinc-800/50 animate-pulse"></div>
            <!-- Label Badge (optional) -->
            {#if i % 2 === 0}
              <div class="h-6 w-24 rounded-lg bg-emerald-500/20 animate-pulse"></div>
            {/if}
          </div>
        </div>

        <!-- Title -->
        <div class="h-6 w-40 rounded bg-zinc-800/50 animate-pulse mb-2"></div>

        <!-- Description -->
        <div class="space-y-2 mb-4">
          <div class="h-4 w-full rounded bg-zinc-800/30 animate-pulse"></div>
          <div class="h-4 w-3/4 rounded bg-zinc-800/30 animate-pulse"></div>
        </div>

        <!-- Meta info row -->
        <div class="flex items-center justify-between mb-4 py-3 px-3 bg-void/50 rounded-xl border border-glass-border">
          <div class="flex items-center gap-2">
            <div class="w-4 h-4 rounded bg-zinc-800/30 animate-pulse"></div>
            <div class="h-4 w-20 rounded bg-zinc-800/30 animate-pulse"></div>
          </div>
          <div class="flex items-center gap-2">
            <div class="h-4 w-12 rounded bg-zinc-800/30 animate-pulse"></div>
            <div class="h-5 w-10 rounded bg-zinc-800/50 animate-pulse"></div>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex gap-2">
          <div class="flex-1 h-10 rounded-xl bg-indigo-500/20 animate-pulse"></div>
          <div class="w-10 h-10 rounded-xl bg-zinc-800/50 animate-pulse"></div>
          <div class="w-10 h-10 rounded-xl bg-zinc-800/50 animate-pulse"></div>
        </div>
      </div>
    {/each}
  </div>

  <!-- Stats Footer Skeleton -->
  <div class="bg-zinc-900/40 rounded-xl p-4 border border-glass-border">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-6">
        <div class="h-4 w-20 rounded bg-zinc-800/30 animate-pulse"></div>
        <div class="h-4 w-20 rounded bg-zinc-800/30 animate-pulse"></div>
        <div class="h-4 w-24 rounded bg-zinc-800/30 animate-pulse"></div>
        <div class="h-4 w-20 rounded bg-zinc-800/30 animate-pulse"></div>
      </div>
      <div class="h-4 w-16 rounded bg-zinc-800/30 animate-pulse"></div>
    </div>
  </div>
</div>
