<script lang="ts">
  import type { ConflictInfo } from '$lib/api/types';
  
  type ConnectionMode = 'auto' | 'proxy';
  
  interface SetupTask {
    id: string;
    label: string;
    status: 'pending' | 'running' | 'done' | 'error';
    error?: string;
  }
  
  interface DownloadProgress {
    name: string;
    percent: number;
  }
  
  interface Props {
    setupTasks: SetupTask[];
    setupProgress: number;
    setupComplete: boolean;
    downloadProgress: DownloadProgress | null;
    selectedServicesCount: number;
    connectionMode: ConnectionMode;
    conflicts?: ConflictInfo[];
    onComplete: () => void;
  }
  
  let { 
    setupTasks, 
    setupProgress, 
    setupComplete, 
    downloadProgress, 
    selectedServicesCount, 
    connectionMode,
    conflicts = [],
    onComplete 
  }: Props = $props();
  
  let hasBlockingConflicts = $derived(
    conflicts.some(c => c.severity === 'critical' || c.severity === 'high')
  );
  
  function getSeverityColor(severity: string): string {
    switch (severity) {
      case 'critical': return 'text-red-400';
      case 'high': return 'text-orange-400';
      case 'medium': return 'text-yellow-400';
      default: return 'text-zinc-400';
    }
  }
  
  function getCategoryIcon(category: string): string {
    switch (category) {
      case 'network_filter': return '🛡️';
      case 'vpn': return '🔐';
      case 'network_optimization': return '⚡';
      case 'security': return '🔒';
      case 'windivert': return '⚠️';
      default: return '❓';
    }
  }
</script>

<div class="flex-1 flex flex-col items-center justify-center animate-fade-in">
  {#if !setupComplete}
    <!-- Setup in progress -->
    <div class="text-center mb-8">
      <div class="w-20 h-20 rounded-2xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 
                  border border-indigo-500/20 flex items-center justify-center mb-4 mx-auto">
        <span class="text-4xl animate-bounce">🚀</span>
      </div>
      <h2 class="text-2xl font-bold text-white mb-2">Setting Up</h2>
      <p class="text-zinc-400">Please wait, this will take a few seconds...</p>
    </div>
    
    <!-- Progress Bar -->
    <div class="w-full max-w-sm mb-8">
      <div class="h-2 bg-zinc-800 rounded-full overflow-hidden">
        <div 
          class="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full transition-all duration-300"
          style="width: {setupProgress}%"
        ></div>
      </div>
      <p class="text-center text-zinc-500 text-sm font-mono mt-2">{Math.round(setupProgress)}%</p>
    </div>
    
    <!-- Tasks List -->
    <div class="w-full max-w-sm space-y-3">
      {#each setupTasks as task}
        <div class="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/30 border border-white/5
                    {task.status === 'error' ? 'border-red-500/30' : ''}">
          {#if task.status === 'done'}
            <div class="w-8 h-8 rounded-lg bg-emerald-500/10 flex items-center justify-center">
              <svg class="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
          {:else if task.status === 'error'}
            <div class="w-8 h-8 rounded-lg bg-red-500/10 flex items-center justify-center">
              <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </div>
          {:else if task.status === 'running'}
            <div class="w-8 h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center">
              <div class="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
            </div>
          {:else}
            <div class="w-8 h-8 rounded-lg bg-zinc-800 flex items-center justify-center">
              <div class="w-2 h-2 rounded-full bg-zinc-600"></div>
            </div>
          {/if}
          <div class="flex-1 min-w-0">
            <span class="text-sm font-medium block
                         {task.status === 'done' ? 'text-emerald-400' : 
                          task.status === 'error' ? 'text-red-400' :
                          task.status === 'running' ? 'text-white' : 'text-zinc-500'}">
              {task.label}
            </span>
            {#if task.id === 'download' && task.status === 'running' && downloadProgress}
              <div class="mt-1">
                <div class="flex items-center justify-between text-xs text-zinc-500 mb-1">
                  <span>{downloadProgress.name}</span>
                  <span>{downloadProgress.percent}%</span>
                </div>
                <div class="h-1 bg-zinc-700 rounded-full overflow-hidden">
                  <div 
                    class="h-full bg-indigo-500 rounded-full transition-all duration-200"
                    style="width: {downloadProgress.percent}%"
                  ></div>
                </div>
              </div>
            {/if}
            {#if task.error}
              <span class="text-xs text-red-400/70 block mt-0.5 truncate">{task.error}</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <!-- Setup complete -->
    <div class="text-center">
      <div class="w-24 h-24 rounded-3xl bg-gradient-to-br from-emerald-500/20 to-cyan-500/20 
                  border border-emerald-500/20 flex items-center justify-center mb-6 mx-auto
                  animate-success-pop">
        <span class="text-5xl">✅</span>
      </div>
      <h2 class="text-3xl font-bold text-white mb-3">All Done!</h2>
      <p class="text-zinc-400 text-lg mb-8 max-w-sm">
        Isolate is configured and ready to go. Click the button below to start.
      </p>
      
      <!-- Summary -->
      <div class="p-4 rounded-xl bg-zinc-800/30 border border-white/5 text-left max-w-sm mx-auto mb-6">
        <div class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Your Settings</div>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span class="text-zinc-400">Services:</span>
            <span class="text-white font-medium">{selectedServicesCount} selected</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">Method:</span>
            <span class="text-white font-medium">{connectionMode === 'auto' ? 'Automatic' : 'Custom Proxy'}</span>
          </div>
        </div>
      </div>
      
      <!-- Conflicts Warning -->
      {#if conflicts.length > 0}
        <div class="p-4 rounded-xl bg-{hasBlockingConflicts ? 'red' : 'yellow'}-500/10 border border-{hasBlockingConflicts ? 'red' : 'yellow'}-500/20 text-left max-w-sm mx-auto mb-6">
          <div class="flex items-start gap-3">
            <span class="text-xl">{hasBlockingConflicts ? '⚠️' : '💡'}</span>
            <div>
              <h4 class="text-white font-medium text-sm mb-1">
                {hasBlockingConflicts ? 'Software Conflicts Detected' : 'Potential Conflicts'}
              </h4>
              <p class="text-zinc-400 text-xs mb-2">
                {hasBlockingConflicts 
                  ? 'These programs may prevent Isolate from working correctly:'
                  : 'These programs might affect performance:'}
              </p>
              <ul class="space-y-1">
                {#each conflicts as conflict}
                  <li class="flex items-center gap-2 text-xs">
                    <span>{getCategoryIcon(conflict.category)}</span>
                    <span class="{getSeverityColor(conflict.severity)}">{conflict.name}</span>
                  </li>
                {/each}
              </ul>
              <p class="text-zinc-500 text-xs mt-2">
                Check Diagnostics page for details
              </p>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  @keyframes success-pop {
    0% {
      transform: scale(0.8);
      opacity: 0;
    }
    50% {
      transform: scale(1.1);
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }
  
  .animate-fade-in {
    animation: fade-in 0.4s ease-out forwards;
  }
  
  .animate-success-pop {
    animation: success-pop 0.5s ease-out forwards;
  }
</style>
