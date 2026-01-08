<script lang="ts">
  import type { TrafficPoint } from '$lib/stores/dashboard.svelte';

  interface Props {
    data: TrafficPoint[];
    height?: number;
  }

  let { data, height = 120 }: Props = $props();

  const width = 300;
  const padding = { top: 10, right: 10, bottom: 20, left: 10 };
  const chartWidth = width - padding.left - padding.right;
  const chartHeight = height - padding.top - padding.bottom;

  // Calculate max value for scaling
  const maxValue = $derived(
    Math.max(
      1024, // Minimum 1KB for scale
      ...data.map(p => Math.max(p.download, p.upload))
    )
  );

  // Generate path for area chart
  function generatePath(points: TrafficPoint[], key: 'download' | 'upload'): string {
    if (points.length === 0) return '';
    
    const xStep = chartWidth / 59; // 60 points
    
    const pathPoints = points.map((p, i) => {
      const x = padding.left + i * xStep;
      const y = padding.top + chartHeight - (p[key] / maxValue) * chartHeight;
      return `${x},${y}`;
    });

    // Create area path
    const linePath = pathPoints.join(' L ');
    const startX = padding.left;
    const endX = padding.left + (points.length - 1) * xStep;
    const bottomY = padding.top + chartHeight;

    return `M ${startX},${bottomY} L ${linePath} L ${endX},${bottomY} Z`;
  }

  function generateLinePath(points: TrafficPoint[], key: 'download' | 'upload'): string {
    if (points.length === 0) return '';
    
    const xStep = chartWidth / 59;
    
    const pathPoints = points.map((p, i) => {
      const x = padding.left + i * xStep;
      const y = padding.top + chartHeight - (p[key] / maxValue) * chartHeight;
      return `${x},${y}`;
    });

    return `M ${pathPoints.join(' L ')}`;
  }

  function formatSpeed(bytes: number): string {
    if (bytes < 1024) return `${bytes} B/s`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB/s`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB/s`;
  }

  const currentDownload = $derived(data.length > 0 ? data[data.length - 1].download : 0);
  const currentUpload = $derived(data.length > 0 ? data[data.length - 1].upload : 0);
</script>

<div class="relative">
  <svg {width} {height} class="w-full" viewBox="0 0 {width} {height}" preserveAspectRatio="xMidYMid meet">
    <!-- Gradient definitions -->
    <defs>
      <linearGradient id="downloadGradient" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="rgb(59, 130, 246)" stop-opacity="0.5" />
        <stop offset="100%" stop-color="rgb(59, 130, 246)" stop-opacity="0.05" />
      </linearGradient>
      <linearGradient id="uploadGradient" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="rgb(34, 197, 94)" stop-opacity="0.5" />
        <stop offset="100%" stop-color="rgb(34, 197, 94)" stop-opacity="0.05" />
      </linearGradient>
    </defs>

    <!-- Grid lines -->
    {#each [0.25, 0.5, 0.75] as ratio}
      <line 
        x1={padding.left} 
        y1={padding.top + chartHeight * ratio} 
        x2={width - padding.right} 
        y2={padding.top + chartHeight * ratio}
        stroke="rgb(51, 65, 85)"
        stroke-width="1"
        stroke-dasharray="4,4"
        opacity="0.5"
      />
    {/each}

    <!-- Download area -->
    <path 
      d={generatePath(data, 'download')} 
      fill="url(#downloadGradient)"
      class="transition-all duration-300"
    />
    
    <!-- Upload area -->
    <path 
      d={generatePath(data, 'upload')} 
      fill="url(#uploadGradient)"
      class="transition-all duration-300"
    />

    <!-- Download line -->
    <path 
      d={generateLinePath(data, 'download')} 
      fill="none"
      stroke="rgb(59, 130, 246)"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      class="transition-all duration-300"
    />

    <!-- Upload line -->
    <path 
      d={generateLinePath(data, 'upload')} 
      fill="none"
      stroke="rgb(34, 197, 94)"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      class="transition-all duration-300"
    />

    <!-- Bottom axis -->
    <line 
      x1={padding.left} 
      y1={padding.top + chartHeight} 
      x2={width - padding.right} 
      y2={padding.top + chartHeight}
      stroke="rgb(71, 85, 105)"
      stroke-width="1"
    />
  </svg>

  <!-- Legend -->
  <div class="flex justify-center gap-6 mt-2">
    <div class="flex items-center gap-2">
      <div class="w-3 h-3 rounded-full bg-blue-500"></div>
      <span class="text-xs text-slate-400">↓ {formatSpeed(currentDownload)}</span>
    </div>
    <div class="flex items-center gap-2">
      <div class="w-3 h-3 rounded-full bg-green-500"></div>
      <span class="text-xs text-slate-400">↑ {formatSpeed(currentUpload)}</span>
    </div>
  </div>
</div>
