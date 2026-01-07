<script lang="ts">
  import { browser } from '$app/environment';

  interface NetworkStats {
    downloadSpeed: number; // bytes per second
    uploadSpeed: number;   // bytes per second
    totalDownload: number; // total bytes
    totalUpload: number;   // total bytes
    activeConnections: number;
    isSimulated?: boolean; // Flag to indicate if data is simulated
  }

  interface Props {
    stats?: NetworkStats;
    compact?: boolean;
  }

  let { 
    stats = {
      downloadSpeed: 0,
      uploadSpeed: 0,
      totalDownload: 0,
      totalUpload: 0,
      activeConnections: 0,
      isSimulated: false
    },
    compact = false
  }: Props = $props();

  // Format bytes to human readable
  function formatSpeed(bytesPerSec: number): { value: string; unit: string } {
    if (bytesPerSec === 0) return { value: '0', unit: 'B/s' };
    
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    let unitIndex = 0;
    let value = bytesPerSec;
    
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }
    
    return { 
      value: value < 10 ? value.toFixed(1) : Math.round(value).toString(),
      unit: units[unitIndex]
    };
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let unitIndex = 0;
    let value = bytes;
    
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }
    
    return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[unitIndex]}`;
  }

  // Derived formatted values
  let download = $derived(formatSpeed(stats.downloadSpeed));
  let upload = $derived(formatSpeed(stats.uploadSpeed));

  // Sparkline data (last 20 points)
  let downloadHistory = $state<number[]>(Array(20).fill(0));
  let uploadHistory = $state<number[]>(Array(20).fill(0));

  // Update history only when stats.downloadSpeed or stats.uploadSpeed changes
  // Use a ref to track previous values without triggering reactivity
  let prevStats = { download: 0, upload: 0 };

  $effect(() => {
    // Only depend on stats prop, not on history arrays
    const newDownload = stats.downloadSpeed;
    const newUpload = stats.uploadSpeed;
    
    if (browser && (newDownload !== prevStats.download || newUpload !== prevStats.upload)) {
      prevStats = { download: newDownload, upload: newUpload };
      // Mutate arrays in place to avoid triggering effect again
      downloadHistory.shift();
      downloadHistory.push(newDownload);
      downloadHistory = downloadHistory; // trigger reactivity for derived
      
      uploadHistory.shift();
      uploadHistory.push(newUpload);
      uploadHistory = uploadHistory; // trigger reactivity for derived
    }
  });

  // Generate sparkline path
  function generateSparkline(data: number[]): string {
    const max = Math.max(...data, 1);
    const width = 100;
    const height = 24;
    
    const points = data.map((value, i) => {
      const x = (i / (data.length - 1)) * width;
      const y = height - (value / max) * height;
      return `${x},${y}`;
    });
    
    return `M ${points.join(' L ')}`;
  }

  let downloadPath = $derived(generateSparkline(downloadHistory));
  let uploadPath = $derived(generateSparkline(uploadHistory));
</script>

<div class="flex flex-col h-full {compact ? 'gap-2' : 'gap-3'}">
  <!-- Download Speed -->
  <div class="flex-1 flex flex-col justify-center p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
    <div class="flex items-center justify-between mb-2">
      <div class="flex items-center gap-2">
        <svg class="w-4 h-4 text-neon-cyan" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 3v18M5 12l7 7 7-7"/>
        </svg>
        <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Download</span>
      </div>
      <div class="flex items-baseline gap-1">
        <span class="text-lg font-bold text-white font-mono">{download.value}</span>
        <span class="text-[10px] text-zinc-400">{download.unit}</span>
      </div>
    </div>
    
    <!-- Mini sparkline -->
    <div class="h-6 w-full overflow-hidden">
      <svg class="w-full h-full" viewBox="0 0 100 24" preserveAspectRatio="none">
        <defs>
          <linearGradient id="downloadGradient" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="rgb(34, 211, 238)" stop-opacity="0.3"/>
            <stop offset="100%" stop-color="rgb(34, 211, 238)" stop-opacity="0"/>
          </linearGradient>
        </defs>
        <path 
          d="{downloadPath} L 100,24 L 0,24 Z" 
          fill="url(#downloadGradient)"
        />
        <path 
          d={downloadPath} 
          fill="none" 
          stroke="rgb(34, 211, 238)" 
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </div>
  </div>

  <!-- Upload Speed -->
  <div class="flex-1 flex flex-col justify-center p-3 rounded-lg bg-zinc-900/30 border border-white/5 hover:border-white/10 transition-colors">
    <div class="flex items-center justify-between mb-2">
      <div class="flex items-center gap-2">
        <svg class="w-4 h-4 text-indigo-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 21V3M5 10l7-7 7 7"/>
        </svg>
        <span class="text-[10px] uppercase tracking-wider text-zinc-400 font-medium">Upload</span>
      </div>
      <div class="flex items-baseline gap-1">
        <span class="text-lg font-bold text-white font-mono">{upload.value}</span>
        <span class="text-[10px] text-zinc-400">{upload.unit}</span>
      </div>
    </div>
    
    <!-- Mini sparkline -->
    <div class="h-6 w-full overflow-hidden">
      <svg class="w-full h-full" viewBox="0 0 100 24" preserveAspectRatio="none">
        <defs>
          <linearGradient id="uploadGradient" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="rgb(129, 140, 248)" stop-opacity="0.3"/>
            <stop offset="100%" stop-color="rgb(129, 140, 248)" stop-opacity="0"/>
          </linearGradient>
        </defs>
        <path 
          d="{uploadPath} L 100,24 L 0,24 Z" 
          fill="url(#uploadGradient)"
        />
        <path 
          d={uploadPath} 
          fill="none" 
          stroke="rgb(129, 140, 248)" 
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </div>
  </div>

  <!-- Stats footer -->
  {#if !compact}
    <div class="flex items-center justify-between px-1 pt-1">
      <div class="flex items-center gap-1.5">
        {#if stats.isSimulated}
          <span class="px-1.5 py-0.5 text-[9px] uppercase tracking-wider bg-amber-500/20 text-amber-400 rounded font-medium">Demo</span>
        {:else}
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
        {/if}
        <span class="text-[10px] text-zinc-400">{stats.activeConnections} active</span>
      </div>
      <div class="text-[10px] text-zinc-600">
        ↓ {formatBytes(stats.totalDownload)} / ↑ {formatBytes(stats.totalUpload)}
      </div>
    </div>
  {/if}
</div>
