<script lang="ts">
  import { tick } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  interface Props {
    open: boolean;
    onclose: () => void;
    class?: string;
    style?: string;
    ariaLabel?: string;
    ariaDescribedBy?: string;
    /** Prevent closing with Esc or backdrop click (for critical actions) */
    preventClose?: boolean;
    children: import('svelte').Snippet;
  }

  let { 
    open = $bindable(), 
    onclose, 
    class: className = '', 
    style = '', 
    ariaLabel, 
    ariaDescribedBy, 
    preventClose = false,
    children 
  }: Props = $props();

  let modalRef: HTMLDivElement | undefined = $state();
  let previousActiveElement: HTMLElement | null = null;

  // Get all focusable elements within the modal
  function getFocusableElements(): HTMLElement[] {
    if (!modalRef) return [];
    return Array.from(
      modalRef.querySelectorAll<HTMLElement>(
        'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"]):not([disabled])'
      )
    ).filter(el => el.offsetParent !== null); // Only visible elements
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !preventClose) {
      onclose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && !preventClose) {
      e.preventDefault();
      onclose();
      return;
    }

    // Focus trap: Tab and Shift+Tab
    if (e.key === 'Tab') {
      const focusableElements = getFocusableElements();
      if (focusableElements.length === 0) {
        e.preventDefault();
        return;
      }

      const firstElement = focusableElements[0];
      const lastElement = focusableElements[focusableElements.length - 1];

      if (e.shiftKey) {
        // Shift+Tab: if on first element, go to last
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement.focus();
        }
      } else {
        // Tab: if on last element, go to first
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement.focus();
        }
      }
    }
  }

  // Focus trap and initial focus
  $effect(() => {
    if (open && modalRef) {
      // Store the previously focused element
      previousActiveElement = document.activeElement as HTMLElement;
      
      // Lock body scroll
      document.body.classList.add('overflow-hidden');
      
      tick().then(() => {
        const focusable = modalRef?.querySelector<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        focusable?.focus();
      });
    }
  });

  // Restore focus and unlock scroll when modal closes
  $effect(() => {
    if (!open && previousActiveElement) {
      // Unlock body scroll
      document.body.classList.remove('overflow-hidden');
      
      tick().then(() => {
        previousActiveElement?.focus();
        previousActiveElement = null;
      });
    }
  });

  // Cleanup on unmount
  $effect(() => {
    return () => {
      // Ensure scroll is unlocked when component unmounts
      document.body.classList.remove('overflow-hidden');
    };
  });
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="dialog"
    aria-modal="true"
    aria-label={ariaLabel}
    aria-describedby={ariaDescribedBy}
    tabindex="-1"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    transition:fade={{ duration: 200 }}
  >
    <div
      bind:this={modalRef}
      class="bg-void-100 border border-white/5 rounded-2xl shadow-2xl {className}"
      style={style}
      transition:scale={{ duration: 200, start: 0.95, opacity: 0, easing: cubicOut }}
    >
      {@render children()}
    </div>
  </div>
{/if}
