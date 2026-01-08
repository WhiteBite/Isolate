# Modal Unification Summary

## Цель
Унифицировать поведение всех модальных окон в приложении Isolate через использование `BaseModal.svelte`.

## Проблемы найденные

### 1. Несогласованные анимации
- **BaseModal**: использует `fade` (200ms) для backdrop и `scale` (200ms, start: 0.95) для контента
- **Network modals** (EditRuleModal, AddRuleModal, etc.): используют `fade` для backdrop и `fly` для drawer-панелей
- **StrategyDetails**: использовал кастомную реализацию без transitions
- **AddRuleModal (library)**: использовал кастомную реализацию без transitions

### 2. Несогласованное закрытие по Esc
- Некоторые модалки закрываются по Esc всегда
- Некоторые имеют `preventClose` но не используют его корректно
- Drawer-панели (network modals) закрываются по Esc через window listener

### 3. Отсутствие поддержки destructive actions
- Нет визуальной индикации для опасных действий (удаление, сброс)
- Нет единого паттерна для подтверждения destructive actions

### 4. Разные подходы к accessibility
- BaseModal имеет полную поддержку: focus trap, aria-labels, keyboard navigation
- Кастомные модалки имеют частичную или отсутствующую поддержку

## Решения реализованные

### 1. Обновлен BaseModal.svelte
Добавлен новый prop:
```typescript
/** Show warning for destructive actions */
destructive?: boolean;
```

Это позволяет визуально выделять модалки с опасными действиями (будущее улучшение).

### 2. Конвертированы в BaseModal
- ✅ **StrategyDetails.svelte** - теперь использует BaseModal
- ✅ **AddRuleModal.svelte** (library) - теперь использует BaseModal

### 3. Уже используют BaseModal корректно
- ✅ ABTestModal.svelte
- ✅ ServiceConfigModal.svelte
- ✅ AddServiceModal.svelte
- ✅ SubscriptionManager.svelte (2 модалки внутри)
- ✅ PluginSettings.svelte
- ✅ PluginDetailsModal.svelte
- ✅ KeyboardShortcutsModal.svelte
- ✅ Modal.svelte (обёртка над BaseModal)

### 4. Требуют особого подхода (Drawer-панели)
Эти компоненты используют drawer-паттерн (slide from right) и должны оставаться как есть:
- ⚠️ **EditRuleModal.svelte** - drawer с fly transition
- ⚠️ **AddRuleModal.svelte** (network) - drawer с fly transition
- ⚠️ **EditGatewayModal.svelte** - drawer с fly transition
- ⚠️ **AddGatewayModal.svelte** - drawer с fly transition
- ⚠️ **AdvancedDrawer.svelte** - drawer с fly transition

**Решение**: Создать `BaseDrawer.svelte` компонент для унификации drawer-панелей.

### 5. Специальные компоненты
- ⚠️ **CommandPalette.svelte** - специальный UI, не модалка
- ⚠️ **KeyboardOverlay.svelte** - overlay для shortcuts, не модалка
- ⚠️ **BottomDrawer.svelte** - resizable drawer, специальная логика
- ⚠️ **StrategyCompositionSettings.svelte** - имеет встроенную модалку для add rule

## Стандарты для модальных окон

### Когда использовать BaseModal
- Центрированные модальные окна
- Формы добавления/редактирования
- Диалоги подтверждения
- Детальная информация

### Когда использовать BaseDrawer (будущее)
- Панели редактирования с большим количеством полей
- Настройки с вкладками
- Формы импорта/экспорта

### Обязательные props для BaseModal
```typescript
open: boolean;           // Состояние открытия
onclose: () => void;     // Callback закрытия
ariaLabel?: string;      // Для accessibility
preventClose?: boolean;  // Для критичных действий
destructive?: boolean;   // Для опасных действий (удаление, сброс)
```

### Паттерн использования
```svelte
<script lang="ts">
  import BaseModal from '$lib/components/BaseModal.svelte';
  
  let open = $state(false);
  
  function handleClose() {
    open = false;
    // cleanup if needed
  }
</script>

<BaseModal 
  bind:open 
  onclose={handleClose}
  ariaLabel="My Modal Title"
  preventClose={isProcessing}
>
  <div class="p-6">
    <!-- Content -->
  </div>
</BaseModal>
```

### Destructive Actions Pattern
Для опасных действий (удаление, сброс) использовать двухэтапное подтверждение:

```svelte
<!-- Main modal -->
<BaseModal bind:open onclose={handleClose}>
  <button onclick={() => showDeleteConfirm = true}>Delete</button>
</BaseModal>

<!-- Confirmation modal -->
<BaseModal 
  bind:open={showDeleteConfirm} 
  onclose={() => showDeleteConfirm = false}
  destructive={true}
  class="max-w-sm"
>
  <div class="p-6 text-center">
    <div class="w-14 h-14 mx-auto mb-4 rounded-full bg-red-500/10">
      <svg class="w-7 h-7 text-red-400">...</svg>
    </div>
    <h3>Are you sure?</h3>
    <p>This action cannot be undone.</p>
    <div class="flex gap-3 mt-6">
      <button onclick={cancel}>Cancel</button>
      <button onclick={confirmDelete} class="bg-red-500">Delete</button>
    </div>
  </div>
</BaseModal>
```

## Следующие шаги

### Немедленно
1. ✅ Обновить BaseModal с `destructive` prop
2. ✅ Конвертировать StrategyDetails в BaseModal
3. ✅ Конвертировать library/AddRuleModal в BaseModal

### Краткосрочно
4. Создать BaseDrawer.svelte для drawer-панелей
5. Конвертировать network modals в BaseDrawer
6. Обновить StrategyCompositionSettings для использования BaseModal

### Долгосрочно
7. Добавить визуальную индикацию для `destructive` модалок
8. Создать библиотеку готовых confirmation dialogs
9. Документировать паттерны в Storybook (если будет добавлен)

## Проверка готовности

### Критерии успеха
- [ ] Все модалки используют единые анимации (fade + scale или fly для drawers)
- [ ] Esc закрывает все модалки (кроме preventClose)
- [ ] Все destructive действия требуют подтверждения
- [ ] Все модалки имеют правильные aria-labels
- [ ] Focus trap работает во всех модалках
- [ ] Код компилируется без ошибок

### Тестирование
1. Открыть каждую модалку в приложении
2. Проверить закрытие по Esc
3. Проверить закрытие по клику на backdrop
4. Проверить keyboard navigation (Tab, Shift+Tab)
5. Проверить анимации открытия/закрытия
6. Проверить preventClose для критичных действий

## Файлы изменены
- ✅ `src/lib/components/BaseModal.svelte` - добавлен `destructive` prop
- ✅ `src/lib/components/strategies/StrategyDetails.svelte` - конвертирован в BaseModal
- ✅ `src/lib/components/library/AddRuleModal.svelte` - конвертирован в BaseModal
- ✅ `src/routes/strategies/+page.svelte` - обновлен вызов StrategyDetails

## Примечания
- Network drawer-панели намеренно оставлены с fly transition - это часть UX
- CommandPalette и KeyboardOverlay - специальные компоненты, не требуют изменений
- BottomDrawer - resizable drawer с уникальной логикой, не требует изменений
