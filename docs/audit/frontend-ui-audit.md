# Frontend UI Components Audit

**Дата:** 2025-01-XX  
**Версия:** 1.0  
**Проанализировано компонентов:** 16

---

## Содержание

1. [Критичные проблемы (🔴)](#критичные-проблемы-)
2. [Важные улучшения (🟠)](#важные-улучшения-)
3. [Рекомендации (🟡)](#рекомендации-)
4. [Идеи нового функционала (🟢)](#идеи-нового-функционала-)

---

## Критичные проблемы (🔴)

### 1. ProxyCard.svelte — Отсутствие ARIA labels на интерактивных элементах

**Файл:** `src/lib/components/ProxyCard.svelte`

**Проблема:** Кнопки действий (Share, Copy, Edit, Delete) не имеют `aria-label`, только `title`. Screen readers не читают `title` атрибуты.

```svelte
<!-- Текущий код -->
<button
  class="p-2 rounded-lg..."
  onclick={(e) => { e.stopPropagation(); onShare?.(); }}
  title="Share QR code"  <!-- ❌ Не читается screen reader -->
>
```

**Решение:**
```svelte
<button
  class="p-2 rounded-lg..."
  onclick={(e) => { e.stopPropagation(); onShare?.(); }}
  title="Share QR code"
  aria-label="Share QR code for {name}"
>
```

**Затронутые кнопки:** Share, Copy, Edit, Delete

---

### 2. HealthWidget.svelte — Синтаксическая ошибка в шаблоне

**Файл:** `src/lib/components/widgets/HealthWidget.svelte`

**Проблема:** Лишний символ `>` в строке 42:

```svelte
<!-- Строка 42 — ОШИБКА -->
{service.status === 'down' ? 'text-zinc-400' : ''}">>
                                                  ^^ лишний символ
```

**Решение:** Удалить лишний `>`:
```svelte
{service.status === 'down' ? 'text-zinc-400' : ''}">
```

---

### 3. BaseModal.svelte — Отсутствие body scroll lock

**Файл:** `src/lib/components/BaseModal.svelte`

**Проблема:** При открытии модального окна страница за ним остаётся прокручиваемой, что создаёт плохой UX и проблемы на мобильных устройствах.

**Решение:**
```svelte
$effect(() => {
  if (open) {
    document.body.style.overflow = 'hidden';
  }
  return () => {
    document.body.style.overflow = '';
  };
});
```

---

### 4. CommandPalette.svelte — Неправильная обработка focus trap

**Файл:** `src/lib/components/CommandPalette.svelte`

**Проблема:** Focus trap реализован некорректно — при Tab фокус остаётся только на input, но должен циклически переходить по всем интерактивным элементам (input → команды → input).

**Текущий код:**
```svelte
function handleFocusTrap(e: KeyboardEvent) {
  if (!isOpen || e.key !== 'Tab') return;
  e.preventDefault();
  inputRef?.focus();  // ❌ Всегда возвращает на input
}
```

**Решение:** Реализовать полноценный focus trap как в BaseModal.svelte.

---

## Важные улучшения (🟠)

### 1. Sidebar.svelte — Отсутствие keyboard navigation

**Файл:** `src/lib/components/Sidebar.svelte`

**Проблема:** Навигация по sidebar возможна только мышью. Нет поддержки:
- Arrow Up/Down для перемещения между пунктами
- Home/End для перехода к первому/последнему пункту

**Рекомендация:** Добавить `role="navigation"` и обработчики клавиатуры:
```svelte
<nav 
  class="flex-1 flex flex-col py-3 overflow-hidden" 
  aria-label="Primary"
  onkeydown={handleNavKeydown}
>
```

---

### 2. Toast.svelte — Отсутствие auto-dismiss и aria-live

**Файл:** `src/lib/components/Toast.svelte`

**Проблемы:**
1. Нет `aria-live="polite"` для анонсирования screen readers
2. Нет визуального индикатора времени до закрытия
3. Нет паузы таймера при hover

**Решение:**
```svelte
<div
  class="..."
  role="alert"
  aria-live="polite"
  aria-atomic="true"
  onmouseenter={pauseTimer}
  onmouseleave={resumeTimer}
>
```

---

### 3. ProxyCard.svelte — Кнопки действий скрыты по умолчанию

**Файл:** `src/lib/components/ProxyCard.svelte`

**Проблема:** Кнопки действий появляются только при hover (`opacity-0 group-hover:opacity-100`). Это:
- Недоступно для keyboard-only пользователей
- Недоступно на touch устройствах
- Нарушает принцип discoverability

**Решение:** Показывать кнопки при focus-within:
```svelte
<div class="flex items-center gap-1 flex-shrink-0 
            opacity-0 group-hover:opacity-100 group-focus-within:opacity-100
            transition-opacity duration-200">
```

---

### 4. NetworkStatsWidget.svelte — Потенциальная утечка памяти

**Файл:** `src/lib/components/widgets/NetworkStatsWidget.svelte`

**Проблема:** История sparkline обновляется в `$effect`, но массивы мутируются напрямую, что может вызвать проблемы с реактивностью и памятью при длительной работе.

**Текущий код:**
```svelte
downloadHistory.shift();
downloadHistory.push(newDownload);
downloadHistory = downloadHistory; // trigger reactivity
```

**Решение:** Использовать иммутабельное обновление:
```svelte
downloadHistory = [...downloadHistory.slice(1), newDownload];
```

---

### 5. Widgets — Отсутствие reduced motion support

**Затронутые файлы:**
- `StatusWidget.svelte` — `animate-pulse-glow`
- `HealthWidget.svelte` — `animate-pulse`
- `NetworkStatsWidget.svelte` — `animate-pulse`
- `ConnectionStatsWidget.svelte` — `animate-pulse`

**Проблема:** Анимации не отключаются для пользователей с `prefers-reduced-motion: reduce`.

**Решение:** Добавить в глобальные стили:
```css
@media (prefers-reduced-motion: reduce) {
  .animate-pulse,
  .animate-pulse-glow,
  .animate-spin {
    animation: none !important;
  }
}
```

---

### 6. Sidebar.svelte — Inline SVG icons создают bloat

**Файл:** `src/lib/components/Sidebar.svelte`

**Проблема:** Все иконки хранятся как inline SVG строки в объекте `icons`. Это:
- Увеличивает размер компонента
- Затрудняет поддержку
- Не позволяет кэшировать иконки

**Рекомендация:** Вынести иконки в отдельный компонент `Icon.svelte` или использовать sprite:
```svelte
<Icon name="layout-dashboard" class="w-5 h-5" />
```

---

## Рекомендации (🟡)

### 1. Консистентность цветовой схемы

**Проблема:** Разные компоненты используют разные способы задания цветов:
- `text-neon-green`, `text-neon-cyan` (custom tokens)
- `text-emerald-400`, `text-green-400` (Tailwind)
- `rgb(34, 211, 238)` (hardcoded)

**Рекомендация:** Стандартизировать на Tailwind tokens + CSS variables:
```css
:root {
  --color-success: theme('colors.emerald.400');
  --color-warning: theme('colors.amber.400');
  --color-error: theme('colors.red.400');
}
```

---

### 2. ProxyCard.svelte — Улучшение touch targets

**Файл:** `src/lib/components/ProxyCard.svelte`

**Проблема:** Кнопки действий имеют размер `p-2` (32x32px), что меньше рекомендуемого минимума 44x44px для touch устройств.

**Решение:**
```svelte
<button class="p-2.5 min-w-[44px] min-h-[44px] ...">
```

---

### 3. CommandPalette.svelte — Добавить recent commands

**Файл:** `src/lib/components/CommandPalette.svelte`

**Рекомендация:** Показывать недавно использованные команды в начале списка:
```svelte
let recentCommands = $state<string[]>([]);

// При выполнении команды
function executeCommand(cmd: Command) {
  recentCommands = [cmd.id, ...recentCommands.filter(id => id !== cmd.id)].slice(0, 5);
  localStorage.setItem('recentCommands', JSON.stringify(recentCommands));
}
```

---

### 4. BaseModal.svelte — Добавить размеры по умолчанию

**Файл:** `src/lib/components/BaseModal.svelte`

**Рекомендация:** Добавить preset размеров:
```svelte
interface Props {
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
}

const sizeClasses = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  full: 'max-w-[90vw] max-h-[90vh]'
};
```

---

### 5. LatencyWidget.svelte — Добавить threshold alerts

**Файл:** `src/lib/components/widgets/LatencyWidget.svelte`

**Рекомендация:** Визуально выделять критические значения:
```svelte
{#if currentLatency && currentLatency > 300}
  <div class="absolute inset-0 border-2 border-red-500/50 rounded-xl animate-pulse pointer-events-none"></div>
{/if}
```

---

### 6. DashboardSkeleton.svelte — Добавить shimmer эффект

**Файл:** `src/lib/components/widgets/DashboardSkeleton.svelte`

**Рекомендация:** Skeleton компоненты выглядят статично. Добавить shimmer анимацию:
```css
.skeleton-shimmer {
  background: linear-gradient(
    90deg,
    rgba(255,255,255,0) 0%,
    rgba(255,255,255,0.05) 50%,
    rgba(255,255,255,0) 100%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
```

---

### 7. Network Page — Слишком много состояний

**Файл:** `src/routes/network/+page.svelte`

**Проблема:** Страница содержит 20+ `$state` переменных, что затрудняет поддержку.

**Рекомендация:** Вынести состояние в отдельный store:
```typescript
// src/lib/stores/network.ts
export const networkStore = createNetworkStore();

// В компоненте
let { gateways, rules, loading } = $derived(networkStore);
```

---

## Идеи нового функционала (🟢)

### 1. Drag & Drop для ProxyCard

Позволить пользователям менять порядок прокси перетаскиванием:
```svelte
<div
  draggable="true"
  ondragstart={handleDragStart}
  ondragover={handleDragOver}
  ondrop={handleDrop}
>
```

---

### 2. Keyboard shortcuts overlay

Добавить overlay с горячими клавишами (по нажатию `?`):
- `Ctrl+K` — Command Palette
- `Ctrl+1-4` — Навигация
- `Ctrl+Shift+R` — Panic Reset
- `Space` — Toggle protection

---

### 3. Widget customization

Позволить пользователям:
- Менять порядок виджетов на dashboard
- Скрывать ненужные виджеты
- Изменять размер виджетов

---

### 4. Dark/Light theme toggle

Добавить поддержку светлой темы:
```svelte
<button onclick={toggleTheme} aria-label="Toggle theme">
  {#if isDark}
    <SunIcon />
  {:else}
    <MoonIcon />
  {/if}
</button>
```

---

### 5. Connection quality indicator

Добавить визуальный индикатор качества соединения в header:
```svelte
<div class="flex items-center gap-1">
  <div class="w-1 h-3 rounded-full {quality > 80 ? 'bg-green-500' : 'bg-zinc-600'}"></div>
  <div class="w-1 h-4 rounded-full {quality > 60 ? 'bg-green-500' : 'bg-zinc-600'}"></div>
  <div class="w-1 h-5 rounded-full {quality > 40 ? 'bg-green-500' : 'bg-zinc-600'}"></div>
  <div class="w-1 h-6 rounded-full {quality > 20 ? 'bg-green-500' : 'bg-zinc-600'}"></div>
</div>
```

---

### 6. Proxy groups

Группировка прокси по категориям:
- По стране
- По протоколу
- По скорости
- Избранные

---

### 7. Real-time notifications

Push-уведомления о событиях:
- Прокси отключился
- Высокий latency
- Обновление доступно

---

## Сводка по приоритетам

| Приоритет | Количество | Примерное время |
|-----------|------------|-----------------|
| 🔴 Критичные | 4 | 2-3 часа |
| 🟠 Важные | 6 | 4-6 часов |
| 🟡 Рекомендации | 7 | 6-8 часов |
| 🟢 Новый функционал | 7 | 2-3 дня |

---

## Следующие шаги

1. **Немедленно:** Исправить синтаксическую ошибку в HealthWidget.svelte
2. **Высокий приоритет:** Добавить ARIA labels в ProxyCard.svelte
3. **Средний приоритет:** Реализовать body scroll lock в BaseModal.svelte
4. **Планирование:** Создать задачи для улучшений accessibility

---

*Аудит выполнен автоматически. Рекомендуется ручная проверка критичных проблем.*
