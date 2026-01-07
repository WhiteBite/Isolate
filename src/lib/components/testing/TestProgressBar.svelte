<script lang="ts">
  import Spinner from '$lib/components/Spinner.svelte';

  // Types
  type StageStatus = 'pending' | 'running' | 'done' | 'failed';
  
  interface TestStage {
    id: string;
    name: string;
    description: string;
    status: StageStatus;
    duration?: number;
    error?: string;
  }

  interface TestProgress {
    current_item: string;
    current_type: string;
    tested_count: number;
    total_count: number;
    percent: number;
  }

  // Props
  let {
    progress = null,
    stages = [],
    elapsedTime = 0,
    isTesting = false
  }: {
    progress: TestProgress | null;
    stages: TestStage[];
    elapsedTime: number;
    isTesting: boolean;
  } = $props();

  // Format elapsed time
  function formatElapsedTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return mins > 0 ? `${mins}м ${secs}с` : `${secs}с`;
  }

  // Stage icon and color helpers
  function getStageIcon(status: StageStatus): string {
    switch (status) {
      case 'done': return '✓';
      case 'running': return '●';
      case 'failed': return '✕';
      default: return '○';
    }
  }

  function getStageColor(status: StageStatus): string {
    switch (status) {
      case 'done': return 'text-[#00ff88]';
      case 'running': return 'text-[#00d4ff]';
      case 'failed': return 'text-[#ff3333]';
      default: return 'text-[#4a4f6a]';
    }
  }

  function getStageBgColor(status: StageStatus): string {
    switch (status) {
      case 'done': return 'bg-[#00ff88]/10 border-[#00ff88]/30';
      case 'running': return 'bg-[#00d4ff]/10 border-[#00d4ff]/30';
      case 'failed': return 'bg-[#ff3333]/10 border-[#ff3333]/30';
      default: return 'bg-[#1a1f3a] border-[#2a2f4a]';
    }
  }

  function getStageGlow(status: StageStatus): string {
    switch (status) {
      case 'done': return 'shadow-[0_0_15px_rgba(0,255,136,0.3)]';
      case 'running': return 'shadow-[0_0_20px_rgba(0,212,255,0.4)] animate-pulse';
      case 'failed': return 'shadow-[0_0_15px_rgba(255,51,51,0.3)]';
      default: return '';
    }
  }
</script>

{#if isTesting || stages.length > 0}
  <div class="bg-[#1a1f3a]/80 backdrop-blur-sm rounded-xl p-5 border border-[#2a2f4a] space-y-4">
    <!-- Header with timer -->
    <div class="flex items-center justify-between">
      <h3 class="text-white font-semibold flex items-center gap-2">
        {#if isTesting}
          <span class="relative flex h-3 w-3">
            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#00d4ff] opacity-75"></span>
            <span class="relative inline-flex rounded-full h-3 w-3 bg-[#00d4ff]"></span>
          </span>
        {/if}
        Testing Progress
      </h3>
      {#if elapsedTime > 0}
        <span class="text-[#a0a0a0] text-sm font-mono">
          ⏱ {formatElapsedTime(elapsedTime)}
        </span>
      {/if}
    </div>

    <!-- Enhanced Progress Bar -->
    {#if progress}
      <div class="space-y-2">
        <div class="flex items-center justify-between text-sm">
          <span class="text-[#a0a0a0]">{progress.tested_count} of {progress.total_count}</span>
          <span class="text-[#00d4ff] font-bold text-lg">{progress.percent}%</span>
        </div>
        
        <!-- Glowing Progress Bar -->
        <div class="relative h-4 bg-[#0a0e27] rounded-full overflow-hidden">
          <!-- Background glow -->
          <div 
            class="absolute inset-0 bg-gradient-to-r from-[#00d4ff]/20 to-[#00ff88]/20 blur-sm transition-all duration-500"
            style="width: {progress.percent}%"
          ></div>
          <!-- Main bar -->
          <div 
            class="relative h-full bg-gradient-to-r from-[#00d4ff] via-[#00a8cc] to-[#00ff88] rounded-full transition-all duration-300 ease-out"
            style="width: {progress.percent}%"
          >
            <!-- Shimmer effect -->
            <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer"></div>
          </div>
          <!-- Glow on edge -->
          {#if progress.percent > 0 && progress.percent < 100}
            <div 
              class="absolute top-0 h-full w-2 bg-white/50 blur-sm transition-all duration-300"
              style="left: calc({progress.percent}% - 4px)"
            ></div>
          {/if}
        </div>
        
        <!-- Current item -->
        <div class="flex items-center gap-2 mt-2">
          <span class="px-2 py-0.5 text-xs rounded-full {progress.current_type === 'proxy' ? 'bg-purple-500/20 text-purple-400' : 'bg-blue-500/20 text-blue-400'}">
            {progress.current_type === 'proxy' ? 'Proxy' : 'Strategy'}
          </span>
          <span class="text-white text-sm truncate">{progress.current_item}</span>
        </div>
      </div>
    {/if}

    <!-- Test Stages Visualization -->
    {#if stages.length > 0}
      <div class="space-y-2 pt-2 border-t border-[#2a2f4a]">
        <span class="text-[#a0a0a0] text-xs uppercase tracking-wider">Stages</span>
        <div class="space-y-2">
          {#each stages as stage, idx}
            <div 
              class="flex items-center gap-3 p-3 rounded-lg border transition-all duration-300 {getStageBgColor(stage.status)} {getStageGlow(stage.status)}"
            >
              <!-- Stage indicator -->
              <div class="flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center {stage.status === 'running' ? 'bg-[#00d4ff]/20' : stage.status === 'done' ? 'bg-[#00ff88]/20' : stage.status === 'failed' ? 'bg-[#ff3333]/20' : 'bg-[#2a2f4a]'}">
                {#if stage.status === 'running'}
                  <Spinner size="sm" color="cyan" />
                {:else}
                  <span class="{getStageColor(stage.status)} text-sm font-bold">{getStageIcon(stage.status)}</span>
                {/if}
              </div>
              
              <!-- Stage info -->
              <div class="flex-1 min-w-0">
                <p class="text-white text-sm font-medium">{stage.name}</p>
                <p class="text-[#a0a0a0] text-xs truncate">
                  {#if stage.error}
                    <span class="text-[#ff3333]">{stage.error}</span>
                  {:else}
                    {stage.description}
                  {/if}
                </p>
              </div>
              
              <!-- Connection line to next stage -->
              {#if idx < stages.length - 1}
                <div class="absolute left-[2.25rem] mt-12 w-0.5 h-4 {stage.status === 'done' ? 'bg-[#00ff88]/50' : 'bg-[#2a2f4a]'}"></div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{:else if progress}
  <div class="bg-[#1a1f3a] rounded-xl p-5 border border-[#2a2f4a]">
    <div class="flex items-center justify-between mb-2">
      <span class="text-[#a0a0a0] text-sm">Progress</span>
      <span class="text-white font-medium">{progress.tested_count} of {progress.total_count}</span>
    </div>
    
    <!-- Progress Bar -->
    <div class="h-3 bg-[#0a0e27] rounded-full overflow-hidden mb-3">
      <div 
        class="h-full bg-gradient-to-r from-[#00d4ff] to-[#00a8cc] rounded-full transition-all duration-300"
        style="width: {progress.percent}%"
      ></div>
    </div>
    
    <p class="text-white text-sm truncate">{progress.current_item}</p>
    <p class="text-[#a0a0a0] text-xs mt-1">{progress.current_type === 'proxy' ? 'Proxy' : 'Strategy'}</p>
    <p class="text-[#00d4ff] text-lg font-bold mt-1">{progress.percent}%</p>
  </div>
{/if}

<style>
  @keyframes shimmer {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(100%);
    }
  }
  
  .animate-shimmer {
    animation: shimmer 2s infinite;
  }
</style>
