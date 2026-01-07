<script lang="ts">
  // ServicesSkeleton - Skeleton loader for Services page
  // Mimics the two-panel layout: service list on left, details on right
</script>

<div class="h-full flex">
  <!-- Left Panel - Service List Skeleton -->
  <div class="w-[320px] flex-shrink-0 border-r border-white/5 flex flex-col bg-zinc-900/30">
    <!-- Header -->
    <div class="p-4 border-b border-white/5">
      <div class="flex items-center justify-between mb-3">
        <div class="h-6 w-24 rounded-lg bg-zinc-800/50 animate-pulse"></div>
        <div class="w-9 h-9 rounded-lg bg-zinc-800/50 animate-pulse"></div>
      </div>
      <!-- Check All Button Skeleton -->
      <div class="w-full h-11 rounded-xl bg-indigo-500/10 border border-indigo-500/20 animate-pulse"></div>
    </div>

    <!-- Service List Skeleton -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      {#each Array(8) as _, i}
        <div 
          class="w-full flex items-center gap-3 p-3 rounded-xl bg-zinc-900/40 border border-white/5"
          style="animation-delay: {i * 50}ms"
        >
          <!-- Icon -->
          <div class="w-10 h-10 flex-shrink-0 rounded-lg bg-zinc-800/50 animate-pulse"></div>
          
          <!-- Info -->
          <div class="flex-1 min-w-0 space-y-2">
            <div class="h-4 w-24 rounded bg-zinc-800/50 animate-pulse"></div>
            <div class="h-3 w-16 rounded bg-zinc-800/30 animate-pulse"></div>
          </div>

          <!-- Status -->
          <div class="w-2.5 h-2.5 rounded-full bg-zinc-700/50 animate-pulse"></div>
        </div>
      {/each}
    </div>

    <!-- Footer -->
    <div class="p-3 border-t border-white/5">
      <div class="h-4 w-32 mx-auto rounded bg-zinc-800/30 animate-pulse"></div>
    </div>
  </div>

  <!-- Right Panel - Details Skeleton -->
  <div class="flex-1 overflow-y-auto bg-zinc-950 p-6">
    <div class="max-w-3xl">
      <!-- Header -->
      <div class="flex items-start gap-5 mb-6">
        <div class="relative">
          <div class="w-20 h-20 rounded-2xl bg-zinc-900/60 border border-white/5 animate-pulse"></div>
          <div class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full bg-zinc-700/50 border-2 border-zinc-950 animate-pulse"></div>
        </div>
        <div class="flex-1 space-y-3">
          <div class="h-8 w-48 rounded-lg bg-zinc-800/50 animate-pulse"></div>
          <div class="flex items-center gap-3">
            <div class="h-6 w-20 rounded-lg bg-zinc-800/50 animate-pulse"></div>
            <div class="h-4 w-16 rounded bg-zinc-800/30 animate-pulse"></div>
          </div>
        </div>
      </div>

      <!-- Tabs Skeleton -->
      <div class="flex gap-1 mb-6 p-1 bg-zinc-900/40 rounded-xl border border-white/5">
        <div class="flex-1 h-10 rounded-lg bg-zinc-800/50 animate-pulse"></div>
        <div class="flex-1 h-10 rounded-lg bg-zinc-800/30 animate-pulse"></div>
      </div>

      <!-- Stats Grid Skeleton -->
      <div class="grid grid-cols-3 gap-4 mb-6">
        {#each Array(3) as _}
          <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
            <div class="h-3 w-16 rounded bg-zinc-800/30 animate-pulse mb-2"></div>
            <div class="h-7 w-12 rounded bg-zinc-800/50 animate-pulse"></div>
          </div>
        {/each}
      </div>

      <!-- Ping History Chart Skeleton -->
      <div class="p-5 bg-zinc-900/30 border border-white/5 rounded-xl mb-6">
        <div class="flex items-center justify-between mb-3">
          <div class="h-4 w-28 rounded bg-zinc-800/50 animate-pulse"></div>
          <div class="h-3 w-24 rounded bg-zinc-800/30 animate-pulse"></div>
        </div>
        <div class="h-20 w-full rounded-lg bg-zinc-800/30 animate-pulse"></div>
      </div>

      <!-- Connection Details Skeleton -->
      <div class="p-5 bg-zinc-900/30 border border-white/5 rounded-xl mb-6">
        <div class="h-4 w-36 rounded bg-zinc-800/50 animate-pulse mb-4"></div>
        <div class="space-y-3">
          {#each Array(3) as _}
            <div class="flex items-center justify-between">
              <div class="h-4 w-28 rounded bg-zinc-800/30 animate-pulse"></div>
              <div class="h-4 w-16 rounded bg-zinc-800/50 animate-pulse"></div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Actions Skeleton -->
      <div class="flex gap-3">
        <div class="flex-1 h-12 rounded-xl bg-indigo-500/20 animate-pulse"></div>
        <div class="flex-1 h-12 rounded-xl bg-zinc-800/50 animate-pulse"></div>
      </div>
    </div>
  </div>
</div>
