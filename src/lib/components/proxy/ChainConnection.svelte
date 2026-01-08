<script lang="ts">
  interface Props {
    from: { x: number; y: number };
    to: { x: number; y: number };
    animated?: boolean;
    selected?: boolean;
  }
  
  let { from, to, animated = true, selected = false }: Props = $props();
  
  // Вычисляем контрольные точки для кривой Безье
  let controlPoint1 = $derived({
    x: from.x + (to.x - from.x) * 0.5,
    y: from.y
  });
  
  let controlPoint2 = $derived({
    x: from.x + (to.x - from.x) * 0.5,
    y: to.y
  });
  
  // Путь для кривой
  let path = $derived(
    `M ${from.x} ${from.y} C ${controlPoint1.x} ${controlPoint1.y}, ${controlPoint2.x} ${controlPoint2.y}, ${to.x} ${to.y}`
  );
  
  // Вычисляем угол для стрелки
  let arrowAngle = $derived(() => {
    const dx = to.x - controlPoint2.x;
    const dy = to.y - controlPoint2.y;
    return Math.atan2(dy, dx) * (180 / Math.PI);
  });
  
  // Уникальный ID для градиента
  let gradientId = $derived(`gradient-${from.x}-${from.y}-${to.x}-${to.y}`.replace(/\./g, '-'));
</script>

<svg class="absolute inset-0 w-full h-full pointer-events-none overflow-visible" style="z-index: 0;">
  <defs>
    <!-- Градиент для линии -->
    <linearGradient id={gradientId} x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color={selected ? '#3b82f6' : '#6366f1'} stop-opacity="0.8" />
      <stop offset="50%" stop-color={selected ? '#60a5fa' : '#8b5cf6'} stop-opacity="1" />
      <stop offset="100%" stop-color={selected ? '#3b82f6' : '#6366f1'} stop-opacity="0.8" />
    </linearGradient>
    
    <!-- Маркер стрелки -->
    <marker
      id="arrowhead-{gradientId}"
      markerWidth="10"
      markerHeight="7"
      refX="9"
      refY="3.5"
      orient="auto"
    >
      <polygon
        points="0 0, 10 3.5, 0 7"
        fill={selected ? '#60a5fa' : '#8b5cf6'}
      />
    </marker>
    
    <!-- Фильтр свечения -->
    <filter id="glow-{gradientId}" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="2" result="coloredBlur"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>
  
  <!-- Фоновая линия (тень) -->
  <path
    d={path}
    fill="none"
    stroke="rgba(0,0,0,0.3)"
    stroke-width="4"
    stroke-linecap="round"
  />
  
  <!-- Основная линия -->
  <path
    d={path}
    fill="none"
    stroke="url(#{gradientId})"
    stroke-width="2"
    stroke-linecap="round"
    marker-end="url(#arrowhead-{gradientId})"
    filter={selected ? `url(#glow-${gradientId})` : undefined}
    class={animated ? 'animate-dash' : ''}
  />
  
  <!-- Анимированные точки вдоль линии -->
  {#if animated}
    <circle r="3" fill="#a78bfa" class="animate-flow">
      <animateMotion dur="2s" repeatCount="indefinite">
        <mpath href="#flow-path-{gradientId}" />
      </animateMotion>
    </circle>
    <path id="flow-path-{gradientId}" d={path} fill="none" stroke="none" />
  {/if}
</svg>

<style>
  @keyframes dash {
    to {
      stroke-dashoffset: -20;
    }
  }
  
  .animate-dash {
    stroke-dasharray: 5, 5;
    animation: dash 1s linear infinite;
  }
  
  @keyframes flow {
    0% { opacity: 0; }
    50% { opacity: 1; }
    100% { opacity: 0; }
  }
  
  .animate-flow {
    animation: flow 2s ease-in-out infinite;
  }
</style>
