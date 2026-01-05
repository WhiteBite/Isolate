<script lang="ts">
  /**
   * Dashboard skeleton loader - shows while data is loading
   * Mimics the structure of the actual dashboard widgets
   */
  import Skeleton from '../Skeleton.svelte';
  import BentoGrid from '../BentoGrid.svelte';
  import BentoWidget from '../BentoWidget.svelte';
</script>

<!-- Bento Grid Skeleton -->
<BentoGrid columns={4} gap={4}>
  <!-- Status Widget Skeleton (2x2) -->
  <BentoWidget colspan={2} rowspan={2}>
    <div class="flex flex-col items-center justify-center h-full gap-6">
      <!-- Big toggle button skeleton -->
      <Skeleton variant="circle" width="120px" height="120px" />
      
      <!-- Status text -->
      <div class="flex flex-col items-center gap-2">
        <Skeleton variant="text" width="140px" height="18px" />
        <Skeleton variant="text" width="100px" height="14px" />
      </div>
    </div>
  </BentoWidget>

  <!-- Health Monitor Skeleton (2x1) -->
  <BentoWidget colspan={2} title="Health Monitor" icon="💚">
    <div class="flex flex-col gap-2 h-full">
      {#each Array(4) as _}
        <div class="flex items-center justify-between py-2.5 px-3 rounded-lg bg-zinc-900/30 border border-white/5">
          <div class="flex items-center gap-3">
            <Skeleton variant="circle" width="8px" height="8px" />
            <Skeleton variant="text" width="80px" height="14px" />
          </div>
          <div class="flex items-center gap-3">
            <Skeleton variant="text" width="30px" height="10px" />
            <Skeleton variant="text" width="45px" height="12px" />
          </div>
        </div>
      {/each}
    </div>
  </BentoWidget>

  <!-- Method Widget Skeleton (1x1) -->
  <BentoWidget title="Active Method" icon="⚡">
    <div class="flex flex-col h-full justify-between">
      <div class="flex items-start gap-3">
        <Skeleton variant="custom" width="40px" height="40px" rounded="lg" />
        <div class="flex-1 space-y-2">
          <Skeleton variant="text" width="100px" height="16px" />
          <Skeleton variant="text" width="70px" height="14px" />
        </div>
      </div>
      <div class="mt-3 pt-3 border-t border-white/5">
        <div class="flex items-center gap-2">
          <Skeleton variant="text" width="20px" height="12px" />
          <Skeleton variant="custom" width="100px" height="20px" rounded="md" />
        </div>
      </div>
    </div>
  </BentoWidget>

  <!-- Quick Actions Skeleton (1x1) -->
  <BentoWidget title="Quick Actions" icon="🚀">
    <div class="grid grid-cols-2 gap-2 h-full">
      {#each Array(4) as _}
        <div class="flex flex-col items-center justify-center gap-2 p-3 rounded-lg bg-zinc-900/40 border border-white/5">
          <Skeleton variant="custom" width="32px" height="32px" rounded="lg" />
          <Skeleton variant="text" width="50px" height="12px" />
        </div>
      {/each}
    </div>
  </BentoWidget>
</BentoGrid>
