<script lang="ts">
  import LibraryCard from './LibraryCard.svelte';

  /** Метод доступа для правила */
  export type AccessMethod = 'zapret' | 'vless' | 'direct' | 'block';

  /** Тип кастомного правила */
  export interface CustomRule {
    id: string;
    pattern: string;
    method: AccessMethod;
    isEnabled: boolean;
    description?: string;
  }

  interface Props {
    /** Данные правила */
    rule: CustomRule;
    /** Выделена ли карточка */
    selected?: boolean;
    /** Заблокирована ли карточка */
    disabled?: boolean;
    /** Статус тестирования */
    testing?: boolean;
    /** Обработчик тестирования правила */
    onTest?: (rule: CustomRule) => void;
    /** Обработчик удаления правила */
    onDelete?: (rule: CustomRule) => void;
  }

  let {
    rule,
    selected = false,
    disabled = false,
    testing = false,
    onTest,
    onDelete
  }: Props = $props();

  // Статус карточки
  let status = $derived.by(() => {
    if (testing) return 'loading' as const;
    if (!rule.isEnabled) return 'idle' as const;
    return 'success' as const;
  });

  // Конфигурация методов доступа
  const methodConfig: Record<AccessMethod, { label: string; color: string; icon: string }> = {
    zapret: { 
      label: 'Zapret', 
      color: 'text-blue-400 bg-blue-500/20',
      icon: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z'
    },
    vless: { 
      label: 'VLESS', 
      color: 'text-violet-400 bg-violet-500/20',
      icon: 'M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418'
    },
    direct: { 
      label: 'Напрямую', 
      color: 'text-emerald-400 bg-emerald-500/20',
      icon: 'M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3'
    },
    block: { 
      label: 'Блокировать', 
      color: 'text-red-400 bg-red-500/20',
      icon: 'M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636'
    }
  };

  let currentMethod = $derived(methodConfig[rule.method]);

  function handleTest(event: MouseEvent) {
    event.stopPropagation();
    onTest?.(rule);
  }

  function handleDelete(event: MouseEvent) {
    event.stopPropagation();
    onDelete?.(rule);
  }
</script>

<LibraryCard {selected} {disabled} {status}>
  {#snippet icon()}
    <!-- Иконка метода -->
    <svg class="w-6 h-6 {currentMethod.color.split(' ')[0]}" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
      <path stroke-linecap="round" stroke-linejoin="round" d={currentMethod.icon} />
    </svg>
    
    <!-- Индикатор статуса -->
    {#if rule.isEnabled}
      <div class="absolute -top-1 -right-1 w-3 h-3 bg-emerald-500 rounded-full border-2 border-zinc-900"></div>
    {:else}
      <div class="absolute -top-1 -right-1 w-3 h-3 bg-zinc-600 rounded-full border-2 border-zinc-900"></div>
    {/if}
  {/snippet}

  {#snippet content()}
    <div class="space-y-1.5">
      <!-- Паттерн/домен -->
      <div class="flex items-center gap-2">
        <code class="font-mono text-sm text-zinc-100 truncate">{rule.pattern}</code>
        {#if !rule.isEnabled}
          <span class="px-1.5 py-0.5 text-xs font-medium bg-zinc-700 text-zinc-400 rounded">
            Отключено
          </span>
        {/if}
      </div>
      
      <!-- Описание (если есть) -->
      {#if rule.description}
        <p class="text-sm text-zinc-400 truncate">{rule.description}</p>
      {/if}
      
      <!-- Метод доступа -->
      <div class="flex items-center gap-2">
        <span class="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium rounded {currentMethod.color}">
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d={currentMethod.icon} />
          </svg>
          {currentMethod.label}
        </span>
      </div>
    </div>
  {/snippet}

  {#snippet actions()}
    <!-- Кнопка тестирования -->
    <button
      type="button"
      class="p-2 text-zinc-400 hover:text-blue-400 hover:bg-blue-500/10 rounded-lg transition-colors disabled:opacity-50"
      onclick={handleTest}
      disabled={disabled || testing}
      title="Тестировать"
    >
      {#if testing}
        <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      {:else}
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.348a1.125 1.125 0 010 1.971l-11.54 6.347a1.125 1.125 0 01-1.667-.985V5.653z" />
        </svg>
      {/if}
    </button>
    
    <!-- Кнопка удаления -->
    <button
      type="button"
      class="p-2 text-zinc-400 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors disabled:opacity-50"
      onclick={handleDelete}
      disabled={disabled}
      title="Удалить"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
      </svg>
    </button>
  {/snippet}
</LibraryCard>
