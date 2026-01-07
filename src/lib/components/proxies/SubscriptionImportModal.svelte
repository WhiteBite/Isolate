<script lang="ts">
  import { Modal, Button } from '$lib/components';

  interface Props {
    open: boolean;
    onImport: (url: string) => Promise<void>;
    onClose: () => void;
  }

  let { open = $bindable(), onImport, onClose }: Props = $props();

  let subscriptionUrl = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function handleImport() {
    if (!subscriptionUrl.trim()) return;
    
    loading = true;
    error = null;
    
    try {
      await onImport(subscriptionUrl.trim());
      subscriptionUrl = '';
      open = false;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    subscriptionUrl = '';
    error = null;
    onClose();
  }

  // Export error setter for parent to use
  export function setError(msg: string | null) {
    error = msg;
  }
</script>

<Modal bind:open title="Import Subscription" onclose={handleClose}>
  <div class="space-y-4">
    <div class="p-4 bg-indigo-500/10 border border-indigo-500/20 rounded-xl">
      <div class="flex items-start gap-3">
        <svg class="w-5 h-5 text-indigo-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <div class="text-sm text-indigo-300/90">
          <p class="font-medium mb-1">What is a subscription?</p>
          <p class="text-indigo-300/70">A subscription is a URL that returns a list of proxy servers. Usually provided by a VPN provider and automatically updated.</p>
        </div>
      </div>
    </div>
    
    <div>
      <label for="subscription-url" class="block text-sm font-medium text-text-secondary mb-2">Subscription URL</label>
      <input
        id="subscription-url"
        type="url"
        bind:value={subscriptionUrl}
        placeholder="https://example.com/subscription/..."
        class="w-full px-4 py-3 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-indigo-500/30 focus:ring-1 focus:ring-indigo-500/20 font-mono text-sm transition-all duration-200"
      />
      <p class="mt-2 text-xs text-text-muted">Supported: base64-encoded lists, plain text with links</p>
    </div>
    
    {#if error}
      <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-xl">
        <p class="text-sm text-red-400">{error}</p>
      </div>
    {/if}
    
    <div class="flex justify-end gap-3 pt-2">
      <Button variant="secondary" onclick={handleClose}>Cancel</Button>
      <button 
        onclick={handleImport}
        disabled={loading || !subscriptionUrl.trim()}
        class="flex items-center gap-2 px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium text-sm rounded-xl transition-all duration-200"
      >
        {#if loading}
          <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24" aria-hidden="true">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Importing...
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
          </svg>
          Import
        {/if}
      </button>
    </div>
  </div>
</Modal>
