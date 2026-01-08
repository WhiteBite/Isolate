<script lang="ts">
  import type { TrafficPoint } from '$lib/stores/dashboard.svelte';

  interface Props {
    data: TrafficPoint[];
    height?: number;
    maxPoints?: number;
  }

  let { data, height = 120, maxPoints = 60 }: Props = $props();

  // Keep last N points
  const displayData = $derived(data.slice(-maxPoints));

  // Calculate max values for scaling
  const maxDownload = $derived(
    displayData.length > 0 
      ? Math.max(...displayData.map(d => d.download), 1) 
      : 1
  );
  const maxUpload = $derived(
    displayData.length > 0 
      ? Math.max(...displayData.map(d => d.upload), 1) 
      : 1
  );
  const maxValue = $derived(Math.max(maxDownload, maxUpload, 1));

  // Current values (last point)
  const currentDownload = $derived(
    displayData.length > 0 ? displayData[displayData.length - 1].download : 0
  );
  const currentUpload = $derived(
    displayData.length > 0 ? displayData[displayData.length - 1].upload : 0
  );

  // SVG dimensions
  const width = 400;
  const padding = { top: 10, right: 10, bottom: 10, left: 10 };
  const chartWidth = $derived(width - padding.left - padding.right);
  const chartHeight = $derived(height - padding.top - padding.bottom);

  // Generate SVG path for area chart
  function generatePath(points: TrafficPoint[], getValue: (p: TrafficPoint) => number): string {
    if (points.length === 0) return '';

    const xStep = chartWidth / Math.max(maxPoints - 1, 1);
    const startOffset = Math.max(0, maxPoints - points.length);

    let path = `M ${padding.left + startOffset * xStep} ${padding.top + chartHeight}`;

    points.forEach((point, i) => {
      const x = padding.left + (startOffset + i) * xStep;
      const value = getValue(point);
      const normalizedValue = Math.max(0, Math.min(value, maxValue)); // Clamp value
      const y = padding.top + chartHeight - (normalizedValue / maxValue) * chartHeight;
      path += ` L ${x} ${y}`;
    });

    const lastX = padding.left + (startOffset + points.length - 1) * xStep;
    path += ` L ${lastX} ${padding.top + chartHeight}`;
    path += ' Z';

    return path;
  }
  
  // Generate line path (without area fill)
  function generateLinePath(points: TrafficPoint[], getValue: (p: TrafficPoint) => number): string {
    if (points.length === 0) return '';

    const xStep = chartWidth / Math.max(maxPoints - 1, 1);
    const startOffset = Math.max(0, maxPoints - points.length);

    let path = '';

    points.forEach((point, i) => {
      const x = padding.left + (startOffset + i) * xStep;
      const value = getValue(point);
      const normalizedValue = Math.max(0, Math.min(value, maxValue)); // Clamp value
      const y = padding.top + chartHeight - (normalizedValue / maxValue) * chartHeight;
      
      if (i === 0) {
        path = `M ${x} ${y}`;
      } else {
        path += ` L ${x} ${y}`;
      }
    });

    return path;
  }

  const downloadPath = $derived(generatePath(displayData, p => p.download));
  const uploadPath = $derived(generatePath(displayData, p => p.upload));
  const downloadLinePath = $derived(generateLinePath(displayData, p => p.download));
  const uploadLinePath = $derived(generateLinePath(displayData, p => p.upload));

  // Format bytes to human-readable
  function formatSpeed(bytesPerSec: number): string {
    if (bytesPerSec < 1024) return `${bytesPerSec.toFixed(0)} B/s`;
    if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
    return `${(bytesPerSec / 1024 / 1024).toFixed(2)} MB/s`;
  }
</script>

<div class="flex flex-col gap-3">
  <!-- Legend with current values -->
  <div class="flex items-center justify-between gap-4 px-2">
    <div class="flex items-center gap-2">
      <div class="w-3 h-3 rounded-full bg-emerald-500 shadow-lg shadow-emerald-500/50"></div>
      <span class="text-xs text-slate-400">Download</span>
      <span class="text-xs font-mono font-semibold text-emerald-400">
        {formatSpeed(currentDownload)}
      </span>
    </div>
    
    <div class="flex items-center gap-2">
      <div class="w-3 h-3 rounded-full bg-blue-500 shadow-lg shadow-blue-500/50"></div>
      <span class="text-xs text-slate-400">Upload</span>
      <span class="text-xs font-mono font-semibold text-blue-400">
        {formatSpeed(currentUpload)}
      </span>
    </div>
  </div>

  <!-- Chart SVG -->
  <div class="relative w-full rounded-lg bg-slate-900/50 border border-slate-700/30 overflow-hidden">
    <svg 
      viewBox="0 0 {width} {height}" 
      class="w-full h-auto"
      preserveAspectRatio="none"
    >
      <!-- Gradient definitions -->
      <defs>
        <linearGradient id="downloadGradient" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" style="stop-color: rgb(16, 185, 129); stop-opacity: 0.6" />
          <stop offset="100%" style="stop-color: rgb(16, 185, 129); stop-opacity: 0.05" />
        </linearGradient>
        
        <linearGradient id="uploadGradient" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" style="stop-color: rgb(59, 130, 246); stop-opacity: 0.5" />
          <stop offset="100%" style="stop-color: rgb(59, 130, 246); stop-opacity: 0.05" />
        </linearGradient>
      </defs>

      <!-- Grid lines (optional, subtle) -->
      <line 
        x1={padding.left} 
        y1={padding.top + chartHeight / 2} 
        x2={width - padding.right} 
        y2={padding.top + chartHeight / 2} 
        stroke="rgb(71, 85, 105)" 
        stroke-width="0.5" 
        stroke-dasharray="2,2"
        opacity="0.3"
      />

      <!-- Upload area (behind) -->
      {#if uploadPath}
        <path
          d={uploadPath}
          fill="url(#uploadGradient)"
          class="transition-all duration-300"
        />
      {/if}

      <!-- Download area (front) -->
      {#if downloadPath}
        <path
          d={downloadPath}
          fill="url(#downloadGradient)"
          class="transition-all duration-300"
        />
      {/if}

      <!-- Upload line -->
      {#if uploadLinePath}
        <path
          d={uploadLinePath}
          fill="none"
          stroke="rgb(59, 130, 246)"
          stroke-width="1.5"
          class="transition-all duration-300"
        />
      {/if}

      <!-- Download line -->
      {#if downloadLinePath}
        <path
          d={downloadLinePath}
          fill="none"
          stroke="rgb(16, 185, 129)"
          stroke-width="1.5"
          class="transition-all duration-300"
        />
      {/if}
    </svg>

    <!-- Empty state -->
    {#if displayData.length === 0}
      <div class="absolute inset-0 flex items-center justify-center">
        <p class="text-xs text-slate-600">Ожидание данных...</p>
      </div>
    {/if}
  </div>
</div>

<style>
  svg {
    display: block;
  }
</style>
