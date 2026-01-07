<script lang="ts">
  interface Props {
    data: number[];  // последние N значений ping
    maxPoints?: number;
    height?: number;
    color?: string;
  }
  
  let { data, maxPoints = 30, height = 60, color = '#22C55E' }: Props = $props();
  let canvas: HTMLCanvasElement;
  
  $effect(() => {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    // Get actual canvas dimensions
    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    
    // Set canvas size accounting for device pixel ratio
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    
    const width = rect.width;
    const chartHeight = rect.height;
    
    // Clear
    ctx.clearRect(0, 0, width, chartHeight);
    
    // Draw line
    const points = data.slice(-maxPoints);
    if (points.length < 2) {
      // Draw placeholder line if not enough data
      ctx.beginPath();
      ctx.strokeStyle = color + '30';
      ctx.lineWidth = 1;
      ctx.setLineDash([4, 4]);
      ctx.moveTo(0, chartHeight / 2);
      ctx.lineTo(width, chartHeight / 2);
      ctx.stroke();
      ctx.setLineDash([]);
      return;
    }
    
    const max = Math.max(...points, 100);
    const min = Math.min(...points);
    const range = max - min || 100;
    const padding = 4; // Padding from top/bottom
    const stepX = width / (maxPoints - 1);
    
    // Draw grid lines
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
    ctx.lineWidth = 1;
    for (let i = 0; i < 3; i++) {
      const y = padding + ((chartHeight - padding * 2) / 2) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
    
    // Calculate points positions
    const pointsCoords: { x: number; y: number }[] = [];
    const startX = (maxPoints - points.length) * stepX;
    
    points.forEach((val, i) => {
      const x = startX + i * stepX;
      const normalizedVal = (val - min) / range;
      const y = padding + (chartHeight - padding * 2) * (1 - normalizedVal);
      pointsCoords.push({ x, y });
    });
    
    // Draw gradient fill first
    ctx.beginPath();
    ctx.moveTo(pointsCoords[0].x, chartHeight);
    pointsCoords.forEach((p) => {
      ctx.lineTo(p.x, p.y);
    });
    ctx.lineTo(pointsCoords[pointsCoords.length - 1].x, chartHeight);
    ctx.closePath();
    
    const gradient = ctx.createLinearGradient(0, 0, 0, chartHeight);
    gradient.addColorStop(0, color + '40');
    gradient.addColorStop(1, color + '00');
    ctx.fillStyle = gradient;
    ctx.fill();
    
    // Draw smooth line using bezier curves
    ctx.beginPath();
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    
    ctx.moveTo(pointsCoords[0].x, pointsCoords[0].y);
    
    for (let i = 1; i < pointsCoords.length; i++) {
      const prev = pointsCoords[i - 1];
      const curr = pointsCoords[i];
      const next = pointsCoords[i + 1];
      
      if (next) {
        // Smooth curve
        const cpx = (prev.x + curr.x) / 2;
        const cpy = curr.y;
        ctx.quadraticCurveTo(prev.x, prev.y, cpx, cpy);
      } else {
        // Last point
        ctx.lineTo(curr.x, curr.y);
      }
    }
    
    ctx.stroke();
    
    // Draw current value dot
    const lastPoint = pointsCoords[pointsCoords.length - 1];
    ctx.beginPath();
    ctx.fillStyle = color;
    ctx.arc(lastPoint.x, lastPoint.y, 3, 0, Math.PI * 2);
    ctx.fill();
    
    // Glow effect for the dot
    ctx.beginPath();
    ctx.fillStyle = color + '40';
    ctx.arc(lastPoint.x, lastPoint.y, 6, 0, Math.PI * 2);
    ctx.fill();
  });
</script>

<canvas 
  bind:this={canvas} 
  style="height: {height}px" 
  class="w-full"
></canvas>
