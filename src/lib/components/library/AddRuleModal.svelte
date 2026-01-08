<script lang="ts">
  import { libraryStore, type AccessMethod, type AccessMethodType } from '$lib/stores/library.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  let domain = $state('');
  let name = $state('');
  let category = $state('custom');
  let methodType = $state<AccessMethodType>('auto');

  let isValid = $derived(domain.trim() !== '' && name.trim() !== '');

  function handleSubmit(event: Event) {
    event.preventDefault();
    if (!isValid) return;

    const method: AccessMethod = { type: methodType };
    libraryStore.addRule(domain.trim(), name.trim(), category, method);
    
    // Reset form
    domain = '';
    name = '';
    category = 'custom';
    methodType = 'auto';
    
    onClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    }
  }
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div 
    class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50"
    onclick={onClose}
    onkeydown={handleKeydown}
    role="presentation"
  ></div>

  <!-- Modal -->
  <div 
    class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50
           w-full max-w-md p-6 bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
  >
    <h2 id="modal-title" class="text-xl font-semibold text-white mb-6">
      Добавить правило
    </h2>

    <form onsubmit={handleSubmit} class="space-y-4">
      <!-- Domain -->
      <div>
        <label for="domain" class="block text-sm font-medium text-zinc-400 mb-1.5">
          Домен
        </label>
        <input
          id="domain"
          type="text"
          placeholder="example.com"
          class="w-full px-4 py-2.5 bg-zinc-800 border border-zinc-700 rounded-lg
                 text-white placeholder-zinc-500
                 focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                 transition-colors duration-150"
          bind:value={domain}
          required
        />
      </div>

      <!-- Name -->
      <div>
        <label for="name" class="block text-sm font-medium text-zinc-400 mb-1.5">
          Название
        </label>
        <input
          id="name"
          type="text"
          placeholder="My Service"
          class="w-full px-4 py-2.5 bg-zinc-800 border border-zinc-700 rounded-lg
                 text-white placeholder-zinc-500
                 focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                 transition-colors duration-150"
          bind:value={name}
          required
        />
      </div>

      <!-- Category -->
      <div>
        <label for="category" class="block text-sm font-medium text-zinc-400 mb-1.5">
          Категория
        </label>
        <select
          id="category"
          class="w-full px-4 py-2.5 bg-zinc-800 border border-zinc-700 rounded-lg
                 text-white cursor-pointer
                 focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                 transition-colors duration-150"
          bind:value={category}
        >
          <option value="custom">Custom</option>
          <option value="video">Video</option>
          <option value="social">Social</option>
          <option value="music">Music</option>
          <option value="gaming">Gaming</option>
          <option value="work">Work</option>
        </select>
      </div>

      <!-- Method -->
      <div>
        <label for="method" class="block text-sm font-medium text-zinc-400 mb-1.5">
          Метод доступа
        </label>
        <select
          id="method"
          class="w-full px-4 py-2.5 bg-zinc-800 border border-zinc-700 rounded-lg
                 text-white cursor-pointer
                 focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
                 transition-colors duration-150"
          bind:value={methodType}
        >
          <option value="direct">Напрямую</option>
          <option value="auto">Авто</option>
          <option value="strategy">Стратегия</option>
          <option value="vless">VLESS</option>
          <option value="proxy">Прокси</option>
        </select>
      </div>

      <!-- Actions -->
      <div class="flex justify-end gap-3 pt-4">
        <button
          type="button"
          class="px-4 py-2 text-sm font-medium text-zinc-400 
                 hover:text-white hover:bg-zinc-800 rounded-lg
                 transition-colors duration-150"
          onclick={onClose}
        >
          Отмена
        </button>
        <button
          type="submit"
          class="px-4 py-2 text-sm font-medium text-white 
                 bg-emerald-600 hover:bg-emerald-500 rounded-lg
                 disabled:opacity-50 disabled:cursor-not-allowed
                 transition-colors duration-150"
          disabled={!isValid}
        >
          Добавить
        </button>
      </div>
    </form>
  </div>
{/if}
