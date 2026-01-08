<script lang="ts">
  import { libraryStore } from '$lib/stores/library.svelte';
  import { 
    ServiceRuleCard, 
    LibraryFilters, 
    AddRuleModal 
  } from '$lib/components/library';

  let showAddModal = $state(false);
  let selectedIndex = $state(0);

  // Получаем текущий выбранный сервис
  let selectedRule = $derived(libraryStore.filteredRules[selectedIndex]);

  $effect(() => {
    libraryStore.load();
  });

  // Сбрасываем индекс при изменении фильтров
  $effect(() => {
    // Подписываемся на изменения filteredRules
    const _ = libraryStore.filteredRules.length;
    // Корректируем индекс если он выходит за границы
    if (selectedIndex >= libraryStore.filteredRules.length) {
      selectedIndex = Math.max(0, libraryStore.filteredRules.length - 1);
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    // Игнорируем если открыт модал или фокус в input
    if (showAddModal) return;
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      return;
    }

    const rulesCount = libraryStore.filteredRules.length;
    if (rulesCount === 0) return;

    switch (event.key) {
      case 'ArrowUp':
        event.preventDefault();
        selectedIndex = selectedIndex > 0 ? selectedIndex - 1 : rulesCount - 1;
        scrollToSelected();
        break;
      case 'ArrowDown':
        event.preventDefault();
        selectedIndex = selectedIndex < rulesCount - 1 ? selectedIndex + 1 : 0;
        scrollToSelected();
        break;
      case 'Enter':
        event.preventDefault();
        if (selectedRule) {
          // Открываем детали/редактирование — пока просто фокусируем карточку
          const card = document.querySelector(`[data-rule-id="${selectedRule.id}"]`);
          if (card) {
            (card as HTMLElement).click();
          }
        }
        break;
      case 'c':
      case 'C':
      case 'с': // Русская 'с'
      case 'С':
        // Проверяем что нет модификаторов (Ctrl+C — копирование)
        if (!event.ctrlKey && !event.metaKey && !event.altKey) {
          event.preventDefault();
          if (selectedRule && selectedRule.status !== 'checking') {
            libraryStore.checkRule(selectedRule.id);
          }
        }
        break;
    }
  }

  function scrollToSelected() {
    // Прокручиваем к выбранному элементу
    requestAnimationFrame(() => {
      const selected = document.querySelector(`[data-rule-index="${selectedIndex}"]`);
      if (selected) {
        selected.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      }
    });
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<svelte:head>
  <title>Library | Isolate</title>
</svelte:head>

<div class="flex flex-col h-full p-6 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold text-white">Library</h1>
      <p class="text-sm text-zinc-400 mt-1">
        Управление сервисами и методами доступа
      </p>
    </div>
    
    <div class="flex items-center gap-4">
      <!-- Stats -->
      <div class="flex items-center gap-4 text-sm">
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
          <span class="text-zinc-400">
            <span class="text-white font-medium">{libraryStore.accessibleCount}</span> доступно
          </span>
        </div>
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-red-400"></span>
          <span class="text-zinc-400">
            <span class="text-white font-medium">{libraryStore.blockedCount}</span> заблокировано
          </span>
        </div>
      </div>

      <!-- Add button -->
      <button
        type="button"
        class="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white
               bg-emerald-600 hover:bg-emerald-500 rounded-lg
               transition-colors duration-150"
        onclick={() => showAddModal = true}
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Добавить
      </button>
    </div>
  </div>

  <!-- Filters -->
  <LibraryFilters />

  <!-- Content -->
  <div class="flex-1 overflow-auto">
    {#if libraryStore.loading}
      <!-- Skeleton loading -->
      <div class="space-y-3" role="status" aria-label="Загрузка...">
        {#each Array(5) as _}
          <div class="flex items-center gap-4 p-4 bg-zinc-900/50 border border-zinc-800 rounded-xl animate-pulse">
            <div class="w-12 h-12 bg-zinc-800 rounded-xl"></div>
            <div class="flex-1 space-y-2">
              <div class="h-4 w-32 bg-zinc-800 rounded"></div>
              <div class="h-3 w-48 bg-zinc-800 rounded"></div>
            </div>
            <div class="h-8 w-32 bg-zinc-800 rounded-lg"></div>
          </div>
        {/each}
      </div>
    {:else if libraryStore.error}
      <!-- Error state -->
      <div class="flex flex-col items-center justify-center py-16 text-center">
        <div class="w-16 h-16 flex items-center justify-center bg-red-500/10 rounded-full mb-4">
          <svg class="w-8 h-8 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        <h3 class="text-lg font-medium text-white mb-2">Ошибка загрузки</h3>
        <p class="text-sm text-zinc-400 mb-4">{libraryStore.error}</p>
        <button
          type="button"
          class="px-4 py-2 text-sm font-medium text-white bg-zinc-800 hover:bg-zinc-700 rounded-lg"
          onclick={() => libraryStore.load()}
        >
          Повторить
        </button>
      </div>
    {:else if libraryStore.filteredRules.length === 0}
      <!-- Empty state -->
      <div class="flex flex-col items-center justify-center py-16 text-center">
        <div class="w-16 h-16 flex items-center justify-center bg-zinc-800 rounded-full mb-4">
          <svg class="w-8 h-8 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
        </div>
        <h3 class="text-lg font-medium text-white mb-2">
          {libraryStore.rules.length === 0 ? 'Библиотека пуста' : 'Ничего не найдено'}
        </h3>
        <p class="text-sm text-zinc-400 mb-4">
          {libraryStore.rules.length === 0 
            ? 'Добавьте первый сервис для начала работы' 
            : 'Попробуйте изменить параметры фильтрации'}
        </p>
        {#if libraryStore.rules.length === 0}
          <button
            type="button"
            class="px-4 py-2 text-sm font-medium text-white bg-emerald-600 hover:bg-emerald-500 rounded-lg"
            onclick={() => showAddModal = true}
          >
            Добавить сервис
          </button>
        {:else}
          <button
            type="button"
            class="px-4 py-2 text-sm font-medium text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-lg"
            onclick={() => libraryStore.clearFilters()}
          >
            Сбросить фильтры
          </button>
        {/if}
      </div>
    {:else}
      <!-- Rules list -->
      <div class="space-y-3" role="list" aria-label="Список сервисов">
        {#each libraryStore.filteredRules as rule, index (rule.id)}
          <ServiceRuleCard 
            {rule} 
            isSelected={index === selectedIndex}
            data-rule-id={rule.id}
            data-rule-index={index}
            onclick={() => selectedIndex = index}
          />
        {/each}
      </div>
    {/if}
  </div>

  <!-- Add Rule Modal -->
  <AddRuleModal isOpen={showAddModal} onClose={() => showAddModal = false} />

  <!-- Keyboard shortcuts hint -->
  {#if libraryStore.filteredRules.length > 0 && !libraryStore.loading}
    <div class="flex items-center justify-center gap-6 py-3 border-t border-zinc-800 text-xs text-zinc-500">
      <div class="flex items-center gap-1.5">
        <kbd class="px-1.5 py-0.5 bg-zinc-800 border border-zinc-700 rounded text-zinc-400">↑</kbd>
        <kbd class="px-1.5 py-0.5 bg-zinc-800 border border-zinc-700 rounded text-zinc-400">↓</kbd>
        <span>Навигация</span>
      </div>
      <div class="flex items-center gap-1.5">
        <kbd class="px-1.5 py-0.5 bg-zinc-800 border border-zinc-700 rounded text-zinc-400">Enter</kbd>
        <span>Открыть</span>
      </div>
      <div class="flex items-center gap-1.5">
        <kbd class="px-1.5 py-0.5 bg-zinc-800 border border-zinc-700 rounded text-zinc-400">C</kbd>
        <span>Проверить</span>
      </div>
    </div>
  {/if}
</div>
