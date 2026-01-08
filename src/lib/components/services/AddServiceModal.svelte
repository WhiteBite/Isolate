<script lang="ts">
  import BaseModal from '$lib/components/BaseModal.svelte';

  interface Props {
    open: boolean;
    onclose: () => void;
    onadd: (service: { name: string; url: string; category: string }) => Promise<void>;
  }

  let { open, onclose, onadd }: Props = $props();

  let name = $state('');
  let url = $state('');
  let category = $state('other');
  let adding = $state(false);

  async function handleAdd() {
    if (!name || !url) return;
    
    adding = true;
    try {
      await onadd({ name, url, category });
      // Reset form on success
      name = '';
      url = '';
      category = 'other';
    } finally {
      adding = false;
    }
  }

  function handleClose() {
    if (!adding) {
      onclose();
    }
  }
</script>

<BaseModal {open} onclose={handleClose} preventClose={adding} class="w-full max-w-md">
  <div class="p-6">
    <h3 class="text-lg font-semibold text-zinc-100 mb-4">Add Custom Service</h3>
    
    <div class="space-y-4">
      <div>
        <label for="service-name" class="block text-sm text-zinc-400 mb-1.5">Service Name</label>
        <input
          id="service-name"
          type="text"
          bind:value={name}
          placeholder="e.g. My Service"
          class="w-full px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                 text-zinc-100 placeholder-zinc-400
                 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20"
        />
      </div>
      
      <div>
        <label for="service-url" class="block text-sm text-zinc-400 mb-1.5">Check URL</label>
        <input
          id="service-url"
          type="url"
          bind:value={url}
          placeholder="https://example.com"
          class="w-full px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                 text-zinc-100 placeholder-zinc-400
                 focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20"
        />
      </div>
      
      <div>
        <label for="service-category" class="block text-sm text-zinc-400 mb-1.5">Category</label>
        <select
          id="service-category"
          bind:value={category}
          class="w-full px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                 text-zinc-100 focus:outline-none focus:border-indigo-500/50"
        >
          <option value="social">Social</option>
          <option value="video">Video</option>
          <option value="gaming">Gaming</option>
          <option value="messaging">Messaging</option>
          <option value="streaming">Streaming</option>
          <option value="other">Other</option>
        </select>
      </div>
    </div>
    
    <div class="flex gap-3 mt-6">
      <button
        onclick={handleClose}
        disabled={adding}
        class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
               text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 
               disabled:opacity-50 transition-colors"
      >
        Cancel
      </button>
      <button
        onclick={handleAdd}
        disabled={adding || !name || !url}
        class="flex-1 px-4 py-2.5 bg-indigo-500 rounded-xl
               text-white font-medium text-sm hover:bg-indigo-600
               disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {adding ? 'Adding...' : 'Add Service'}
      </button>
    </div>
  </div>
</BaseModal>
