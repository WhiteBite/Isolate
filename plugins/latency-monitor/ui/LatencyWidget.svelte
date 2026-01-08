<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  interface Props {
    context?: PluginContext;
  }
  
  let { context }: Props = $props();
  
  // State
  let pingData = $state<number[]>([]);
  let isMonitoring = $state(false);
  let selectedService = $state('discord');
  let canvas: HTMLCanvasElement;
  let intervalId: ReturnType<typeof setInterval> | null = null;
  
  // Services to monitor
  const services = [
    { id: 'discord', name: 'Discord', url: 'discord.com', color: '#5865F2' },
    { id: 'youtube', name: 'YouTube', url: 'youtube.com', color: '#FF0000' },
    { id: 'telegram', name: 'Telegram', url: 'telegram.org', color: '#0088CC' },
    { id: 'google', name: 'Google', url: 'google.com', color: '#4285F4' }
  ];
  
  let currentService = $derived(services.find(s => s.id === selectedService) || services[0]);
  
  // Stats
  let avgPing = $derived(
    pingData.length > 0 
      ? Math.round(pingData.reduce((a, b) => a + b, 0) / pingData.length)
      : 0
  );
  
  let minPing = $derived(
    pingData.length > 0 ? Math.min(...pingData) : 0
  );
  
  let maxPing = $derived(
    pingData.length > 0 ? Math.max(...pingData) : 0
  );
  
  let currentPing = $derived(
    pingData.length > 0 ? pingData[pingData.length - 1] : 0
  );
  
  // Draw chart when data changes
  $effect(() => {
    if (canvas && pingData.length > 0) {
      drawChart();
    }
  });
  
  // Cleanup on unmount
  $effect(() => {
    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  });
  
  function toggleMonitoring() {
    if (isMonitoring) {
      stopMonitoring();
    } else {
      startMonitoring();
    }
  }
  
  function startMonitoring() {
    isMonitoring = true;
    pingData = [];
    
    // Simulate ping (in real implementation, would use actual ping)
    measurePing();
    intervalId = setInterval(measurePing, 1000);
  }
  
  function stopMonitoring() {
    isMonitoring = false;
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
  }
  
  function measurePing() {
    // Simulate ping with some variance
    const basePing = 30 + Math.random() * 20;
    const spike = Math.random() > 0.9 ? Math.random() * 50 : 0;
    const ping = Math.round(basePing + spike);
    
    pingData = [...pingData.slice(-59), ping];
  }
  
  function drawChart() {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    
    const width = rect.width;
    const height = rect.height;
    const padding = 4;
    const maxPoints = 60;
    
    // Clear
    ctx.clearRect(0, 0, width, height);
    
    // Draw grid
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
    ctx.lineWidth = 1;
    for (let i = 0; i < 4; i++) {
      const y = padding + ((height - padding * 2) / 3) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
    
    if (pingData.length < 2) return;
    
    const max = Math.max(...pingData, 100);
    const min = Math.min(...pingData);
    const range = max - min || 100;
    const stepX = width / (maxPoints - 1);
    
    // Calculate points
    const points: { x: number; y: number }[] = [];
    const startX = (maxPoints - pingData.length) * stepX;
    
    pingData.forEach((val, i) => {
      const x = startX + i * stepX;
      const normalizedVal = (val - min) / range;
      const y = padding + (height - padding * 2) * (1 - normalizedVal);
      points.push({ x, y });
    });
    
    // Draw gradient fill
    ctx.beginPath();
    ctx.moveTo(points[0].x, height);
    points.forEach(p => ctx.lineTo(p.x, p.y));
    ctx.lineTo(points[points.length - 1].x, height);
    ctx.closePath();
    
    const gradient = ctx.createLinearGradient(0, 0, 0, height);
    gradient.addColorStop(0, currentService.color + '40');
    gradient.addColorStop(1, currentService.color + '00');
    ctx.fillStyle = gradient;
    ctx.fill();
    
    // Draw line
    ctx.beginPath();
    ctx.strokeStyle = currentService.color;
    ctx.lineWidth = 2;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    
    ctx.moveTo(points[0].x, points[0].y);
    for (let i = 1; i < points.length; i++) {
      const prev = points[i - 1];
      const curr = points[i];
      const cpx = (prev.x + curr.x) / 2;
      ctx.quadraticCurveTo(prev.x, prev.y, cpx, (prev.y + curr.y) / 2);
    }
    ctx.lineTo(points[points.length - 1].x, points[points.length - 1].y);
    ctx.stroke();
    
    // Draw current point
    const lastPoint = points[points.length - 1];
    ctx.beginPath();
    ctx.fillStyle = currentService.color;
    ctx.arc(lastPoint.x, lastPoint.y, 4, 0, Math.PI * 2);
    ctx.fill();
    
    // Glow
    ctx.beginPath();
    ctx.fillStyle = currentService.color + '40';
    ctx.arc(lastPoint.x, lastPoint.y, 8, 0, Math.PI * 2);
    ctx.fill();
  }
  
  function getPingColor(ping: number): string {
    if (ping < 50) return 'text-green-400';
    if (ping < 100) return 'text-yellow-400';
    return 'text-red-400';
  }
</script>

<div class="flex flex-col h-full gap-3">
  <!-- Header with service selector -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <select
        bind:value={selectedService}
        onchange={() => { pingData = []; }}
        class="
          bg-white/5 border border-white/10 rounded-lg
          px-2 py-1 text-xs text-zinc-300
          focus:outline-none focus:border-cyan-500/50
          cursor-pointer
        "
      >
        {#each services as service}
          <option value={service.id}>{service.name}</option>
        {/each}
      </select>
    </div>
    
    <button
      onclick={toggleMonitoring}
      class="
        px-3 py-1 rounded-lg text-xs font-medium
        transition-all duration-200
        {isMonitoring 
          ? 'bg-red-500/20 text-red-400 border border-red-500/30 hover:bg-red-500/30' 
          : 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 hover:bg-cyan-500/30'}
      "
    >
      {isMonitoring ? 'Stop' : 'Start'}
    </button>
  </div>
  
  <!-- Current Ping Display -->
  <div class="flex items-center justify-center py-2">
    <div class="text-center">
      <div class="text-4xl font-bold tabular-nums {getPingColor(currentPing)} transition-colors">
        {currentPing}
        <span class="text-lg font-normal text-zinc-400">ms</span>
      </div>
      {#if isMonitoring}
        <div class="flex items-center justify-center gap-1 mt-1">
          <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
          <span class="text-[10px] text-zinc-400">Monitoring</span>
        </div>
      {/if}
    </div>
  </div>
  
  <!-- Chart -->
  <div class="flex-1 min-h-[80px] relative">
    <canvas 
      bind:this={canvas}
      class="w-full h-full"
    />
    {#if pingData.length === 0}
      <div class="absolute inset-0 flex items-center justify-center text-zinc-400 text-xs">
        Click "Start" to begin monitoring
      </div>
    {/if}
  </div>
  
  <!-- Stats -->
  <div class="grid grid-cols-3 gap-2 pt-2 border-t border-white/5">
    <div class="text-center">
      <div class="text-[10px] uppercase tracking-wider text-zinc-400">Min</div>
      <div class="text-sm font-medium text-green-400 tabular-nums">{minPing}ms</div>
    </div>
    <div class="text-center">
      <div class="text-[10px] uppercase tracking-wider text-zinc-400">Avg</div>
      <div class="text-sm font-medium text-yellow-400 tabular-nums">{avgPing}ms</div>
    </div>
    <div class="text-center">
      <div class="text-[10px] uppercase tracking-wider text-zinc-400">Max</div>
      <div class="text-sm font-medium text-red-400 tabular-nums">{maxPing}ms</div>
    </div>
  </div>
</div>

<style>
  select option {
    background: #18181b;
    color: #d4d4d8;
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
  
  .animate-pulse {
    animation: pulse 1.5s ease-in-out infinite;
  }
</style>
