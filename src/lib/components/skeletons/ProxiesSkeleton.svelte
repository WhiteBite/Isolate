<script lang="ts">
  // ProxiesSkeleton - Skeleton loader for Proxies page
  // Mimics the proxy cards list layout with drag handles
</script>

<div class="min-h-screen bg-void p-8 space-y-6">
  <!-- Header Skeleton -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-4">
      <div class="w-10 h-10 rounded-xl bg-zinc-800/50 animate-pulse"></div>
      <div class="h-9 w-32 rounded-lg bg-zinc-800/50 animate-pulse"></div>
    </div>
    <div class="flex items-center gap-3">
      <div class="h-10 w-40 rounded-xl bg-zinc-800/50 animate-pulse"></div>
      <div class="h-10 w-20 rounded-xl bg-indigo-500/30 animate-pulse"></div>
    </div>
  </div>

  <!-- Proxy Cards List Skeleton -->
  <div class="space-y-3">
    {#each Array(5) as _, i}
      <div 
        class="flex items-center gap-3"
        style="animation-delay: {i * 80}ms"
      >
        <!-- Drag Handle Skeleton -->
        <div class="flex-shrink-0 w-9 h-9 rounded-lg bg-zinc-800/30 animate-pulse"></div>
        
        <!-- Proxy Card Skeleton -->
        <div class="flex-1 p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
          <div class="flex items-center gap-4">
            <!-- Protocol Badge -->
            <div class="w-16 h-6 rounded-lg bg-indigo-500/20 animate-pulse"></div>
            
            <!-- Country Flag Placeholder -->
            <div class="w-8 h-6 rounded bg-zinc-800/50 animate-pulse"></div>
            
            <!-- Server Info -->
            <div class="flex-1 min-w-0 space-y-2">
              <div class="h-5 w-40 rounded bg-zinc-800/50 animate-pulse"></div>
              <div class="h-4 w-56 rounded bg-zinc-800/30 animate-pulse"></div>
            </div>
            
            <!-- Ping -->
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full bg-emerald-500/30 animate-pulse"></div>
              <div class="h-4 w-12 rounded bg-zinc-800/50 animate-pulse"></div>
            </div>
            
            <!-- Actions -->
            <div class="flex items-center gap-2">
              <div class="w-8 h-8 rounded-lg bg-zinc-800/50 animate-pulse"></div>
              <div class="w-8 h-8 rounded-lg bg-zinc-800/50 animate-pulse"></div>
              <div class="w-8 h-8 rounded-lg bg-zinc-800/50 animate-pulse"></div>
              <div class="w-12 h-6 rounded-full bg-zinc-800/50 animate-pulse"></div>
            </div>
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>
