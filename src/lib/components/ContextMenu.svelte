<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();

  let visible = $state(false);
  let x = $state(0);
  let y = $state(0);
  let menuElement: HTMLDivElement | null = $state(null);

  // Экспортируемые методы для управления меню
  export function show(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    
    // Начальная позиция у курсора
    x = event.clientX;
    y = event.clientY;
    visible = true;

    // После рендера корректируем позицию с учётом границ экрана
    requestAnimationFrame(() => {
      adjustPosition();
    });
  }

  export function hide() {
    visible = false;
  }

  function adjustPosition() {
    if (!menuElement) return;

    const rect = menuElement.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const padding = 8; // Отступ от края экрана

    // Корректировка по горизонтали
    if (x + rect.width > viewportWidth - padding) {
      x = viewportWidth - rect.width - padding;
    }
    if (x < padding) {
      x = padding;
    }

    // Корректировка по вертикали
    if (y + rect.height > viewportHeight - padding) {
      y = viewportHeight - rect.height - padding;
    }
    if (y < padding) {
      y = padding;
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (menuElement && !menuElement.contains(event.target as Node)) {
      hide();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      hide();
    }
  }

  function handleItemClick() {
    // Закрываем меню после клика на элемент
    hide();
  }

  // Cleanup при размонтировании компонента
  $effect(() => {
    return () => {
      visible = false;
    };
  });
</script>

<svelte:window 
  onclick={visible ? handleClickOutside : undefined}
  onkeydown={visible ? handleKeydown : undefined}
/>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={menuElement}
    class="fixed z-[100] min-w-[180px] py-1 
           bg-void-100/95 backdrop-blur-md
           border border-glass-border-active rounded-lg shadow-lg
           animate-context-menu"
    style="left: {x}px; top: {y}px;"
    role="menu"
    tabindex="-1"
    onclick={handleItemClick}
    onkeydown={(e) => e.key === 'Enter' && handleItemClick()}
  >
    {#if children}
      {@render children()}
    {/if}
  </div>
{/if}

<style>
  @keyframes context-menu-appear {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .animate-context-menu {
    animation: context-menu-appear 100ms ease-out forwards;
    transform-origin: top left;
  }
</style>
