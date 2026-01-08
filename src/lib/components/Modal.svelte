<script lang="ts">
  import type { Snippet } from 'svelte';
  import BaseModal from './BaseModal.svelte';

  interface Props {
    open: boolean;
    title?: string;
    onclose?: () => void;
    /** Prevent closing with Esc or backdrop click (for critical actions) */
    preventClose?: boolean;
    children?: Snippet;
  }

  let { 
    open = $bindable(false), 
    title, 
    onclose,
    preventClose = false,
    children 
  }: Props = $props();

  function handleClose() {
    open = false;
    onclose?.();
  }
</script>

<BaseModal bind:open onclose={handleClose} {preventClose} class="w-full max-w-lg">
  {#if title}
    <div class="flex items-center justify-between px-6 py-4 border-b border-white/5">
      <h2 id="modal-title" class="text-lg font-semibold text-white">{title}</h2>
      {#if !preventClose}
        <button
          onclick={handleClose}
          class="p-1.5 text-zinc-400 hover:text-white rounded-lg hover:bg-white/5 transition-colors"
          aria-label="Close modal"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}
    </div>
  {:else if !preventClose}
    <button
      onclick={handleClose}
      class="absolute top-4 right-4 p-1.5 text-zinc-400 hover:text-white rounded-lg hover:bg-white/5 transition-colors z-10"
      aria-label="Close modal"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  {/if}
  <div class="p-6">
    {#if children}
      {@render children()}
    {/if}
  </div>
</BaseModal>
