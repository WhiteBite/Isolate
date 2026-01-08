<script lang="ts">
  import type { Snippet } from 'svelte';

  type CardStatus = 'idle' | 'loading' | 'success' | 'error';

  interface Props {
    /** Выделена ли карточка */
    selected?: boolean;
    /** Заблокирована ли карточка */
    disabled?: boolean;
    /** Статус карточки */
    status?: CardStatus;
    /** Обработчик клика */
    onclick?: () => void;
    /** Snippet для иконки слева */
    icon?: Snippet;
    /** Snippet для основного контента */
    content?: Snippet;
    /** Snippet для действий справа */
    actions?: Snippet;
    /** Snippet для опционального footer */
    footer?: Snippet;
  }

  let {
    selected = false,
    disabled = false,
    status = 'idle',
    onclick,
    icon,
    content,
    actions,
    footer
  }: Props = $props();

  // Computed classes based on state
  let cardClasses = $derived.by(() => {
    const base = [
      'group relative flex flex-col',
      'bg-zinc-900/50 border rounded-xl',
      'transition-all duration-200'
    ];

    // Border color based on status and selection
    if (selected) {
      base.push('border-blue-500/50 ring-1 ring-blue-500/20');
    } else if (status === 'success') {
      base.push('border-emerald-500/30');
    } else if (status === 'error') {
      base.push('border-red-500/30');
    } else {
      base.push('border-zinc-800');
    }

    // Hover states
    if (!disabled) {
      base.push('hover:bg-zinc-800/50 hover:border-zinc-700');
      if (onclick) {
        base.push('cursor-pointer');
      }
    } else {
      base.push('opacity-50 cursor-not-allowed');
    }

    return base.join(' ');
  });

  function handleClick() {
    if (!disabled && onclick) {
      onclick();
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!disabled && onclick && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      onclick();
    }
  }
</script>

<div
  class={cardClasses}
  onclick={handleClick}
  onkeydown={handleKeyDown}
  role={onclick ? 'button' : 'article'}
  tabindex={onclick && !disabled ? 0 : -1}
  aria-disabled={disabled}
  aria-selected={selected}
>
  <!-- Main content area -->
  <div class="flex items-center gap-4 p-4">
    <!-- Icon slot -->
    {#if icon}
      <div class="flex-shrink-0 w-12 h-12 flex items-center justify-center 
                  bg-zinc-800 rounded-xl relative overflow-hidden">
        {@render icon()}
        
        <!-- Loading overlay on icon -->
        {#if status === 'loading'}
          <div class="absolute inset-0 bg-zinc-800/80 flex items-center justify-center">
            <svg class="w-5 h-5 text-blue-400 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Content slot -->
    {#if content}
      <div class="flex-1 min-w-0">
        {@render content()}
      </div>
    {/if}

    <!-- Actions slot -->
    {#if actions}
      <div class="flex items-center gap-2 flex-shrink-0">
        {@render actions()}
      </div>
    {/if}
  </div>

  <!-- Footer slot (optional) -->
  {#if footer}
    <div class="px-4 pb-4 pt-0 border-t border-zinc-800/50 mt-0">
      <div class="pt-3">
        {@render footer()}
      </div>
    </div>
  {/if}

  <!-- Status indicator line at bottom -->
  {#if status !== 'idle'}
    <div class="absolute bottom-0 left-0 right-0 h-0.5 rounded-b-xl overflow-hidden">
      {#if status === 'loading'}
        <div class="h-full bg-blue-500/50 animate-pulse"></div>
      {:else if status === 'success'}
        <div class="h-full bg-emerald-500/50"></div>
      {:else if status === 'error'}
        <div class="h-full bg-red-500/50"></div>
      {/if}
    </div>
  {/if}

  <!-- Selection indicator -->
  {#if selected}
    <div class="absolute top-2 right-2">
      <div class="w-2 h-2 rounded-full bg-blue-500 animate-pulse"></div>
    </div>
  {/if}
</div>
