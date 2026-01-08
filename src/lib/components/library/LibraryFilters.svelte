<script lang="ts">
  import { libraryStore, type AccessMethodType, type ServiceStatus, CRITICAL_SERVICES } from '$lib/stores/library.svelte';

  const statusOptions: { value: 'all' | ServiceStatus; label: string }[] = [
    { value: 'all', label: 'Все статусы' },
    { value: 'accessible', label: 'Доступные' },
    { value: 'blocked', label: 'Заблокированные' },
    { value: 'unknown', label: 'Неизвестно' }
  ];

  const methodOptions: { value: 'all' | AccessMethodType; label: string }[] = [
    { value: 'all', label: 'Все методы' },
    { value: 'direct', label: 'Напрямую' },
    { value: 'auto', label: 'Авто' },
    { value: 'strategy', label: 'Стратегия' },
    { value: 'vless', label: 'VLESS' },
    { value: 'proxy', label: 'Прокси' }
  ];

  let hasActiveFilters = $derived(
    libraryStore.filters.search !== '' ||
    libraryStore.filters.status !== 'all' ||
    libraryStore.filters.method !== 'all' ||
    libraryStore.filters.category !== 'all' ||
    libraryStore.filters.criticalOnly
  );
</script>

<div class="flex flex-wrap items-center gap-3 p-4 bg-zinc-900/50 border border-zinc-800 rounded-xl">
  <!-- Critical only preset button -->
  <button
    type="button"
    class="flex items-center gap-1.5 px-3 py-2 text-sm font-medium rounded-lg
           transition-colors duration-150
           {libraryStore.filters.criticalOnly 
             ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' 
             : 'text-zinc-400 hover:text-white hover:bg-zinc-800 border border-transparent'}"
    onclick={() => libraryStore.toggleCriticalOnly()}
    aria-label="Только критичные сервисы"
    title="YouTube, Discord, Telegram, Twitch, Steam"
  >
    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
    </svg>
    Критичные
  </button>

  <!-- Search -->
  <div class="relative flex-1 min-w-[200px]">
    <svg 
      class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-400" 
      fill="none" 
      viewBox="0 0 24 24" 
      stroke="currentColor"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
    <input
      type="text"
      placeholder="Поиск по названию или домену..."
      class="w-full pl-10 pr-4 py-2 bg-zinc-800 border border-zinc-700 rounded-lg
             text-sm text-white placeholder-zinc-400
             focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
             transition-colors duration-150"
      value={libraryStore.filters.search}
      oninput={(e) => libraryStore.setFilter('search', e.currentTarget.value)}
      aria-label="Поиск сервисов"
    />
  </div>

  <!-- Status filter -->
  <select
    class="px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg
           text-sm text-white cursor-pointer
           focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
           transition-colors duration-150"
    value={libraryStore.filters.status}
    onchange={(e) => libraryStore.setFilter('status', e.currentTarget.value as 'all' | ServiceStatus)}
    aria-label="Фильтр по статусу"
  >
    {#each statusOptions as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>

  <!-- Method filter -->
  <select
    class="px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg
           text-sm text-white cursor-pointer
           focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
           transition-colors duration-150"
    value={libraryStore.filters.method}
    onchange={(e) => libraryStore.setFilter('method', e.currentTarget.value as 'all' | AccessMethodType)}
    aria-label="Фильтр по методу"
  >
    {#each methodOptions as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>

  <!-- Category filter -->
  {#if libraryStore.categories.length > 0}
    <select
      class="px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg
             text-sm text-white cursor-pointer
             focus:outline-none focus:border-zinc-600 focus:ring-1 focus:ring-zinc-600
             transition-colors duration-150"
      value={libraryStore.filters.category}
      onchange={(e) => libraryStore.setFilter('category', e.currentTarget.value)}
      aria-label="Фильтр по категории"
    >
      <option value="all">Все категории</option>
      {#each libraryStore.categories as category}
        <option value={category}>{category}</option>
      {/each}
    </select>
  {/if}

  <!-- Clear filters -->
  {#if hasActiveFilters}
    <button
      type="button"
      class="flex items-center gap-1.5 px-3 py-2 text-sm text-zinc-400 
             hover:text-white hover:bg-zinc-800 rounded-lg
             transition-colors duration-150"
      onclick={() => libraryStore.clearFilters()}
      aria-label="Сбросить фильтры"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
      Сбросить
    </button>
  {/if}
</div>
