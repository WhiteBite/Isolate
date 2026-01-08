<script lang="ts">
  import { libraryStore } from '$lib/stores/library.svelte';
  import LibraryFilters from '$lib/components/library/LibraryFilters.svelte';
  import ServiceRuleCard from '$lib/components/library/ServiceRuleCard.svelte';
  import AddRuleModal from '$lib/components/library/AddRuleModal.svelte';

  let showAddModal = $state(false);
  let selectedRuleId = $state<string | null>(null);

  // Load rules on mount
  $effect(() => {
    libraryStore.load();
  });

  // Stats
  let stats = $derived({
    total: libraryStore.rules.length,
    enabled: libraryStore.enabledCount,
    accessible: libraryStore.accessibleCount,
    blocked: libraryStore.blockedCount,
    filtered: libraryStore.filteredRules.length
  });
</script>

<div class="h-full flex flex-col">
  <!-- Header -->
  <div class="flex-shrink-0 px-6 py-4 border-b border-zinc-800">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-white">Library</h1>
        <p class="text-sm text-zinc-400 mt-1">
          Управление правилами доступа к сервисам
        </p>
      </div>
      
      <!-- Stats -->
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2 px-3 py-2 bg-zinc-900/50 border border-zinc-800 rounded-lg">
          <span class="text-xs text-zinc-400">Всего:</span>
          <span class="text-sm font-semibold text-white">{stats.total}</span>
        </div>
        <div class="flex items-center gap-2 px-3 py-2 bg-emerald-500/10 border border-emerald-500/20 rounded-lg">
          <span class="text-xs text-emerald-400">Доступно:</span>
          <span class="text-sm font-semibold text-emerald-400">{stats.accessible}</span>
        </div>
        <div class="flex items-center gap-2 px-3 py-2 bg-red-500/10 border border-red-500/20 rounded-lg">
          <span class="text-xs text-red-400">Заблокировано:</span>
          <span class="text-sm font-semibold text-red-400">{stats.blocked}</span>
        </div>
        
        <!-- Add Rule Button -->
        <button
          type="button"
          class="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 
                 text-white text-sm font-medium rounded-lg
                 transition-colors duration-150"
          onclick={() => showAddModal = true}
          aria-label="Добавить правило"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Добавить правило
        </button>
      </div>
    </div>
  </div>

  <!-- Filters -->
  <div class="flex-shrink-0 px-6 py-4">
    <LibraryFilters />
  </div>

  <!-- Content -->
  <div class="flex-1 overflow-y-auto px-6 pb-6">
    {#if libraryStore.loading}
      <!-- Loading State -->
      <div class="flex items-center justify-center h-64">
        <div class="flex flex-col items-center gap-3">
          <svg class="w-8 h-8 text-emerald-400 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <p class="text-sm text-zinc-400">Загрузка правил...</p>
        </div>
      </div>
    {:else if libraryStore.error}
      <!-- Error State -->
      <div class="flex items-center justify-center h-64">
        <div class="max-w-md w-full bg-red-500/10 border border-red-500/30 rounded-2xl p-6 text-center">
          <div class="w-14 h-14 mx-auto mb-4 rounded-full bg-red-500/20 border border-red-500/30 flex items-center justify-center">
            <svg class="w-7 h-7 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-zinc-100 mb-2">Ошибка загрузки</h3>
          <p class="text-sm text-red-300 mb-6">{libraryStore.error}</p>
          <button
            type="button"
            class="inline-flex items-center gap-2 px-5 py-2.5 bg-red-500/20 border border-red-500/30 rounded-xl
                   text-red-400 text-sm font-medium
                   hover:bg-red-500/30 hover:border-red-500/40
                   transition-all duration-200"
            onclick={() => libraryStore.load()}
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            Повторить
          </button>
        </div>
      </div>
    {:else if libraryStore.filteredRules.length === 0}
      <!-- Empty State -->
      <div class="flex items-center justify-center h-64">
        <div class="text-center max-w-md">
          <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-zinc-800 flex items-center justify-center">
            <svg class="w-8 h-8 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
          </div>
          
          {#if stats.total === 0}
            <h3 class="text-lg font-semibold text-zinc-100 mb-2">Нет правил</h3>
            <p class="text-sm text-zinc-400 mb-6">
              Добавьте первое правило для управления доступом к сервисам
            </p>
            <button
              type="button"
              class="inline-flex items-center gap-2 px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 
                     text-white text-sm font-medium rounded-lg
                     transition-colors duration-150"
              onclick={() => showAddModal = true}
            >
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Добавить правило
            </button>
          {:else}
            <h3 class="text-lg font-semibold text-zinc-100 mb-2">Ничего не найдено</h3>
            <p class="text-sm text-zinc-400 mb-6">
              Попробуйте изменить фильтры или поисковый запрос
            </p>
            <button
              type="button"
              class="inline-flex items-center gap-2 px-5 py-2.5 bg-zinc-800 hover:bg-zinc-700 
                     text-zinc-300 text-sm font-medium rounded-lg border border-zinc-700
                     transition-colors duration-150"
              onclick={() => libraryStore.clearFilters()}
            >
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
              Сбросить фильтры
            </button>
          {/if}
        </div>
      </div>
    {:else}
      <!-- Rules List -->
      <div class="space-y-3" role="list" aria-label="Список правил библиотеки">
        {#each libraryStore.filteredRules as rule, index (rule.id)}
          <ServiceRuleCard 
            {rule}
            isSelected={selectedRuleId === rule.id}
            onclick={() => selectedRuleId = rule.id}
            data-rule-id={rule.id}
            data-rule-index={index}
          />
        {/each}
      </div>
      
      <!-- Results count -->
      {#if stats.filtered < stats.total}
        <div class="mt-4 text-center">
          <p class="text-sm text-zinc-400">
            Показано {stats.filtered} из {stats.total} правил
          </p>
        </div>
      {/if}
    {/if}
  </div>
</div>

<!-- Add Rule Modal -->
<AddRuleModal 
  bind:isOpen={showAddModal}
  onClose={() => showAddModal = false}
/>
