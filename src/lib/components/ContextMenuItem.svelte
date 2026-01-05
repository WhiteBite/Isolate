<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    icon?: string;
    shortcut?: string;
    disabled?: boolean;
    variant?: 'default' | 'danger';
    onclick?: () => void;
    children?: Snippet;
  }

  let {
    icon,
    shortcut,
    disabled = false,
    variant = 'default',
    onclick,
    children
  }: Props = $props();

  function handleClick() {
    if (!disabled && onclick) {
      onclick();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleClick();
    }
  }
</script>

<button
  type="button"
  class="w-full flex items-center gap-3 px-3 py-2 text-sm text-left rounded-md transition-colors duration-100
         {variant === 'danger' 
           ? 'text-neon-red hover:bg-neon-red/10' 
           : 'text-white/90 hover:bg-void-200'}
         {disabled ? 'opacity-40 cursor-not-allowed' : 'cursor-pointer'}"
  {disabled}
  onclick={handleClick}
  onkeydown={handleKeydown}
  role="menuitem"
  tabindex={disabled ? -1 : 0}
>
  {#if icon}
    <span class="w-5 text-center flex-shrink-0">{icon}</span>
  {/if}
  
  <span class="flex-1">
    {#if children}
      {@render children()}
    {/if}
  </span>
  
  {#if shortcut}
    <span class="text-xs text-white/40 ml-auto pl-4">{shortcut}</span>
  {/if}
</button>
