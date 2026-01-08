<script lang="ts">
  import LibraryCard from './LibraryCard.svelte';

  /** Тип пресета */
  export interface Preset {
    id: string;
    name: string;
    description: string;
    rulesCount: number;
    isActive: boolean;
  }

  interface Props {
    /** Данные пресета */
    preset: Preset;
    /** Выделена ли карточка */
    selected?: boolean;
    /** Заблокирована ли карточка */
    disabled?: boolean;
    /** Обработчик применения пресета */
    onApply?: (preset: Preset) => void;
    /** Обработчик редактирования пресета */
    onEdit?: (preset: Preset) => void;
  }

  let {
    preset,
    selected = false,
    disabled = false,
    onApply,
    onEdit
  }: Props = $props();

  // Статус карточки на основе активности пресета
  let status = $derived<'idle' | 'loading' | 'success' | 'error'>(preset.isActive ? 'success' : 'idle');

  function handleApply(event: MouseEvent) {
    event.stopPropagation();
    onApply?.(preset);
  }

  function handleEdit(event: MouseEvent) {
    event.stopPropagation();
    onEdit?.(preset);
  }
</script>

<LibraryCard {selected} {disabled} {status}>
  {#snippet icon()}
    <!-- Иконка пресета -->
    <svg class="w-6 h-6 text-violet-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
      <path stroke-linecap="round" stroke-linejoin="round" d="M6.429 9.75L2.25 12l4.179 2.25m0-4.5l5.571 3 5.571-3m-11.142 0L2.25 7.5 12 2.25l9.75 5.25-4.179 2.25m0 0L21.75 12l-4.179 2.25m0 0l4.179 2.25L12 21.75 2.25 16.5l4.179-2.25m11.142 0l-5.571 3-5.571-3" />
    </svg>
    
    <!-- Индикатор активности -->
    {#if preset.isActive}
      <div class="absolute -top-1 -right-1 w-3 h-3 bg-emerald-500 rounded-full border-2 border-zinc-900"></div>
    {/if}
  {/snippet}

  {#snippet content()}
    <div class="space-y-1">
      <!-- Название и статус -->
      <div class="flex items-center gap-2">
        <h3 class="font-medium text-zinc-100 truncate">{preset.name}</h3>
        {#if preset.isActive}
          <span class="px-1.5 py-0.5 text-xs font-medium bg-emerald-500/20 text-emerald-400 rounded">
            Активен
          </span>
        {/if}
      </div>
      
      <!-- Описание -->
      <p class="text-sm text-zinc-400 line-clamp-2">{preset.description}</p>
      
      <!-- Количество правил -->
      <div class="flex items-center gap-1 text-xs text-zinc-500">
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 6.75h12M8.25 12h12m-12 5.25h12M3.75 6.75h.007v.008H3.75V6.75zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zM3.75 12h.007v.008H3.75V12zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm-.375 5.25h.007v.008H3.75v-.008zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z" />
        </svg>
        <span>{preset.rulesCount} {preset.rulesCount === 1 ? 'правило' : preset.rulesCount < 5 ? 'правила' : 'правил'}</span>
      </div>
    </div>
  {/snippet}

  {#snippet actions()}
    <!-- Кнопка редактирования -->
    <button
      type="button"
      class="p-2 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-700/50 rounded-lg transition-colors"
      onclick={handleEdit}
      disabled={disabled}
      title="Редактировать"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" />
      </svg>
    </button>
    
    <!-- Кнопка применения -->
    <button
      type="button"
      class="px-3 py-1.5 text-sm font-medium rounded-lg transition-colors
             {preset.isActive 
               ? 'bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30' 
               : 'bg-violet-500/20 text-violet-400 hover:bg-violet-500/30'}"
      onclick={handleApply}
      disabled={disabled}
    >
      {preset.isActive ? 'Активен' : 'Применить'}
    </button>
  {/snippet}
</LibraryCard>
