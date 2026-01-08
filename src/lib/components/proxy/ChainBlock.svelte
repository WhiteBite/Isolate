<script lang="ts">
  import type { ChainBlock } from '$lib/stores/proxyChain.svelte';
  import CountryFlag from './CountryFlag.svelte';
  
  interface Props {
    block: ChainBlock;
    selected?: boolean;
    onSelect?: (id: string) => void;
    onDelete?: (id: string) => void;
    onDragStart?: (id: string, offset: { x: number; y: number }) => void;
    onDrag?: (id: string, position: { x: number; y: number }) => void;
    onDragEnd?: (id: string) => void;
  }
  
  let { 
    block, 
    selected = false, 
    onSelect, 
    onDelete,
    onDragStart,
    onDrag,
    onDragEnd
  }: Props = $props();
  
  let isDragging = $state(false);
  let dragStartPos = $state({ x: 0, y: 0 });
  
  // Иконки для разных типов блоков
  const icons = {
    dpi: `<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
    </svg>`,
    proxy: `<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
    </svg>`,
    internet: `<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>`
  };
  
  // Цвета для разных типов
  let typeColors = $derived.by(() => {
    switch (block.type) {
      case 'dpi': return {
        bg: 'bg-gradient-to-br from-amber-500/20 to-orange-500/20',
        border: 'border-amber-500/50',
        icon: 'text-amber-400',
        glow: 'shadow-amber-500/20'
      };
      case 'proxy': return {
        bg: 'bg-gradient-to-br from-blue-500/20 to-indigo-500/20',
        border: 'border-blue-500/50',
        icon: 'text-blue-400',
        glow: 'shadow-blue-500/20'
      };
      case 'internet': return {
        bg: 'bg-gradient-to-br from-green-500/20 to-emerald-500/20',
        border: 'border-green-500/50',
        icon: 'text-green-400',
        glow: 'shadow-green-500/20'
      };
    }
  });
  
  function handleMouseDown(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('button')) return;
    
    isDragging = true;
    dragStartPos = { x: e.clientX - block.position.x, y: e.clientY - block.position.y };
    onDragStart?.(block.id, dragStartPos);
    onSelect?.(block.id);
    
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }
  
  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    
    const newPos = {
      x: Math.max(0, e.clientX - dragStartPos.x),
      y: Math.max(0, e.clientY - dragStartPos.y)
    };
    onDrag?.(block.id, newPos);
  }
  
  function handleMouseUp() {
    isDragging = false;
    onDragEnd?.(block.id);
    
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  }
  
  function handleClick() {
    onSelect?.(block.id);
  }
  
  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    onDelete?.(block.id);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="absolute w-40 select-none cursor-move transition-shadow duration-200
         {typeColors.bg} {selected ? 'ring-2 ring-white/50' : ''} 
         border {typeColors.border} rounded-xl backdrop-blur-sm
         hover:shadow-lg {typeColors.glow}
         {isDragging ? 'z-50 scale-105' : 'z-10'}"
  style="left: {block.position.x}px; top: {block.position.y}px;"
  onmousedown={handleMouseDown}
  onclick={handleClick}
  role="button"
  tabindex="0"
>
  <!-- Drag handle -->
  <div class="absolute -top-1 left-1/2 -translate-x-1/2 w-8 h-1 rounded-full bg-white/20" />
  
  <!-- Delete button -->
  {#if block.type !== 'internet'}
    <button
      onclick={handleDelete}
      class="absolute -top-2 -right-2 w-5 h-5 rounded-full bg-red-500/80 
             text-white flex items-center justify-center opacity-0 
             group-hover:opacity-100 hover:bg-red-500 transition-all
             hover:scale-110 shadow-lg"
      title="Удалить"
    >
      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  {/if}
  
  <!-- Content -->
  <div class="p-3 group">
    <div class="flex items-center gap-2 mb-2">
      <!-- Icon -->
      <div class="{typeColors.icon}">
        {@html icons[block.type]}
      </div>
      
      <!-- Country flag for proxy -->
      {#if block.type === 'proxy' && block.data.country}
        <CountryFlag countryCode={block.data.country} size="sm" />
      {/if}
    </div>
    
    <!-- Name -->
    <h4 class="text-white font-medium text-sm truncate">{block.data.name}</h4>
    
    <!-- Description -->
    {#if block.data.description}
      <p class="text-gray-400 text-xs truncate mt-0.5">{block.data.description}</p>
    {/if}
    
    <!-- Protocol badge for proxy -->
    {#if block.type === 'proxy' && block.data.protocol}
      <span class="inline-block mt-2 px-2 py-0.5 text-xs rounded-full 
                   bg-white/10 text-gray-300 uppercase">
        {block.data.protocol}
      </span>
    {/if}
  </div>
  
  <!-- Connection points -->
  <div class="absolute top-1/2 -left-2 w-4 h-4 -translate-y-1/2 
              rounded-full bg-white/20 border-2 border-white/40
              hover:bg-white/40 transition-colors" />
  <div class="absolute top-1/2 -right-2 w-4 h-4 -translate-y-1/2 
              rounded-full bg-white/20 border-2 border-white/40
              hover:bg-white/40 transition-colors" />
</div>
