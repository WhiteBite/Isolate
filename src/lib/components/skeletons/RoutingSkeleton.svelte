<script lang="ts">
  // RoutingSkeleton - Skeleton loader for Routing Rules page
  // Mimics the flow-based routing rules layout
</script>

<div class="h-full overflow-auto bg-gradient-to-br from-zinc-950 to-black">
  <div class="p-8 max-w-5xl mx-auto">
    <!-- Header Skeleton -->
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div>
          <div class="h-9 w-48 rounded-lg bg-zinc-800/50 animate-pulse"></div>
          <div class="h-4 w-64 rounded bg-zinc-800/30 animate-pulse mt-2"></div>
        </div>
        <div class="h-10 w-28 rounded-xl bg-indigo-500/30 animate-pulse"></div>
      </div>
    </div>

    <!-- Stats Bar Skeleton -->
    <div class="grid grid-cols-4 gap-4 mb-8">
      {#each Array(4) as _, i}
        <div 
          class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl"
          style="animation-delay: {i * 50}ms"
        >
          <div class="h-3 w-20 rounded bg-zinc-800/30 animate-pulse mb-2"></div>
          <div class="h-7 w-10 rounded bg-zinc-800/50 animate-pulse"></div>
        </div>
      {/each}
    </div>

    <!-- Rule Cards Skeleton -->
    <div class="space-y-4">
      {#each Array(4) as _, i}
        <div 
          class="bg-zinc-900/40 border border-white/5 rounded-2xl overflow-hidden"
          style="animation-delay: {i * 100}ms"
        >
          <div class="p-5">
            <!-- Header Row -->
            <div class="flex items-center gap-4">
              <!-- Drag Handle Skeleton -->
              <div class="w-7 h-7 rounded-lg bg-zinc-800/30 animate-pulse"></div>
              
              <!-- Toggle Skeleton -->
              <div class="w-12 h-6 rounded-full bg-zinc-800/50 animate-pulse"></div>
              
              <!-- Rule Name Skeleton -->
              <div class="flex-1">
                <div class="h-6 w-44 rounded bg-zinc-800/50 animate-pulse"></div>
              </div>
              
              <!-- Actions Skeleton (hidden by default, shown on hover) -->
              <div class="flex items-center gap-2 opacity-30">
                <div class="w-8 h-8 rounded-lg bg-zinc-800/50 animate-pulse"></div>
                <div class="w-8 h-8 rounded-lg bg-zinc-800/50 animate-pulse"></div>
              </div>
            </div>

            <!-- Flow Cards Skeleton -->
            <div class="mt-4 flex items-center gap-3">
              <!-- Source Card Skeleton -->
              <div class="flex-1 p-4 bg-black/40 border border-white/5 rounded-xl">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-lg bg-zinc-700/50 animate-pulse"></div>
                  <div class="flex-1 space-y-2">
                    <div class="h-3 w-12 rounded bg-zinc-800/30 animate-pulse"></div>
                    <div class="h-4 w-28 rounded bg-zinc-800/50 animate-pulse"></div>
                  </div>
                </div>
              </div>

              <!-- Arrow Skeleton -->
              <div class="flex-shrink-0 w-10 h-10 rounded-lg bg-zinc-800/20 animate-pulse"></div>

              <!-- Condition Card Skeleton -->
              <div class="flex-shrink-0 w-12 h-12 rounded-xl bg-black/40 border border-white/5 animate-pulse"></div>

              <!-- Arrow Skeleton -->
              <div class="flex-shrink-0 w-10 h-10 rounded-lg bg-zinc-800/20 animate-pulse"></div>

              <!-- Action Card Skeleton -->
              <div class="flex-1 p-4 bg-zinc-800/30 border border-white/5 rounded-xl">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-lg bg-zinc-700/50 animate-pulse"></div>
                  <div class="flex-1 space-y-2">
                    <div class="h-3 w-12 rounded bg-zinc-800/30 animate-pulse"></div>
                    <div class="h-4 w-32 rounded bg-zinc-800/50 animate-pulse"></div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Bottom accent line skeleton -->
          <div class="h-0.5 bg-gradient-to-r from-zinc-700/50 to-transparent"></div>
        </div>
      {/each}
    </div>
  </div>
</div>
