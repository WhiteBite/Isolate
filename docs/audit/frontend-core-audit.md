# Frontend Core Audit Report

**Дата:** 2025-01-07  
**Scope:** Core frontend компоненты и stores  
**Файлы:** `+layout.svelte`, `+page.svelte`, `services/+page.svelte`, stores, hooks, API modules

---

## 🔴 Критичные проблемы

### 1. Memory Leak в `+page.svelte` — неправильный cleanup в $effect

**Файл:** `src/routes/+page.svelte`  
**Строки:** 145-175

```typescript
// ПРОБЛЕМА: initialized = false внутри $effect сбрасывается при каждом запуске
$effect(() => {
  initialized = false;  // ❌ Это вызывает повторную инициализацию!
  initializeDashboard();
  
  return () => {
    cleanupFns.forEach(fn => fn());
    cleanupFns = [];
    clearAllIntervals();
    initialized = false;
  };
});
```

**Проблема:** Переменная `initialized` объявлена как обычная `let` (не `$state`), но сбрасывается внутри `$effect`. При каждом запуске эффекта `initialized = false` вызывает повторную инициализацию, что может привести к:
- Множественным подпискам на stores
- Множественным event listeners
- Утечкам памяти

**Решение:**
```typescript
// Вынести guard за пределы $effect
let initialized = false; // НЕ $state - это правильно

$effect(() => {
  if (initialized) return; // Guard в начале
  initialized = true;
  
  initializeDashboard();
  
  return () => {
    cleanupFns.forEach(fn => fn());
    cleanupFns = [];
    clearAllIntervals();
    initialized = false; // Сброс только в cleanup
  };
});
```

---

### 2. Race Condition в `+layout.svelte` — двойная инициализация

**Файл:** `src/routes/+layout.svelte`  
**Строки:** 85-95

```typescript
// ПРОБЛЕМА: checkOnboarding может вызваться дважды
$effect(() => {
  if (!initialized) {
    checkOnboarding();
  }
});
```

**Проблема:** `initialized` проверяется внутри `$effect`, но `checkOnboarding()` — async функция. Пока она выполняется, `$effect` может запуститься повторно (например, при изменении зависимостей), и `initialized` всё ещё будет `false`.

**Решение:**
```typescript
$effect(() => {
  if (initialized) return;
  initialized = true; // Сразу ставим флаг
  checkOnboarding();
});
```

---

### 3. Отсутствие Error Boundaries в API вызовах

**Файл:** `src/lib/api/core.ts`

```typescript
// ПРОБЛЕМА: Нет обработки ошибок, все ошибки пробрасываются наверх
export async function getStatus(): Promise<AppStatus> {
    return invoke('get_status'); // ❌ Ошибка не обрабатывается
}
```

**Проблема:** Все API функции просто пробрасывают ошибки. Если backend недоступен или команда не существует, приложение может упасть.

**Решение:** Добавить wrapper с retry логикой:
```typescript
import { invokeWhenReady } from '$lib/hooks/useBackendReady';

export async function getStatus(): Promise<AppStatus> {
    return invokeWhenReady<AppStatus>('get_status');
}
```

---

### 4. Потенциальный Memory Leak — intervals без cleanup

**Файл:** `src/routes/+page.svelte`  
**Строки:** 200-230

```typescript
// ПРОБЛЕМА: Если initializeDashboard вызывается повторно, старые intervals не очищаются
healthCheckInterval = setInterval(() => {
  checkServicesHealth();
}, 30000);

networkStatsInterval = setInterval(() => {
  // ...
}, 1000);
```

**Проблема:** Хотя `clearAllIntervals()` вызывается в cleanup, если `initializeDashboard()` вызовется повторно до cleanup (из-за бага #1), создадутся дублирующиеся intervals.

**Решение:** Добавить проверку перед созданием:
```typescript
if (!healthCheckInterval) {
  healthCheckInterval = setInterval(() => {
    checkServicesHealth();
  }, 30000);
}
```

---

## 🟠 Важные улучшения

### 5. Дублирование логики waitForBackend

**Файлы:** 
- `src/lib/hooks/useBackendReady.ts` — `waitForBackend(options)`
- `src/lib/utils/backend.ts` — `waitForBackend(retries, delay)` (предположительно)

**Проблема:** В `+page.svelte` используется `waitForBackend` из `$lib/utils/backend`, а в `+layout.svelte` — из `$lib/hooks/useBackendReady`. Разные сигнатуры и возможно разная логика.

**Решение:** Унифицировать в один модуль `$lib/hooks/useBackendReady.ts` и удалить дубликат.

---

### 6. Отсутствие типизации в event handlers

**Файл:** `src/routes/+page.svelte`  
**Строки:** 240-260

```typescript
unlistenProgress = await listen('automation:progress', (event) => {
  const payload = event.payload as { stage: string; percent: number; message: string };
  // ❌ Небезопасный cast, нет валидации
});
```

**Проблема:** Payload из событий кастуется без валидации. Если backend изменит формат, приложение упадёт.

**Решение:** Добавить runtime валидацию или использовать zod:
```typescript
import { z } from 'zod';

const ProgressPayloadSchema = z.object({
  stage: z.string(),
  percent: z.number(),
  message: z.string(),
});

unlistenProgress = await listen('automation:progress', (event) => {
  const result = ProgressPayloadSchema.safeParse(event.payload);
  if (!result.success) {
    console.error('Invalid progress payload:', result.error);
    return;
  }
  const payload = result.data;
  // ...
});
```

---

### 7. Симулированные данные без индикации

**Файл:** `src/routes/+page.svelte`  
**Строки:** 210-230

```typescript
networkStats = {
  downloadSpeed: Math.round(baseDownload + (Math.random() - 0.5) * 20000),
  // ...
  isSimulated: true // ✓ Флаг есть
};
```

**Проблема:** Флаг `isSimulated` есть, но в UI он не отображается. Пользователь видит "реальные" данные, которые на самом деле симулированы.

**Решение:** Добавить индикатор в `NetworkStatsWidget`:
```svelte
{#if stats.isSimulated}
  <span class="text-xs text-zinc-500">(simulated)</span>
{/if}
```

---

### 8. Большой файл `+page.svelte` (500+ строк)

**Файл:** `src/routes/+page.svelte`

**Проблема:** Dashboard содержит слишком много логики:
- Store subscriptions
- Event listeners
- Health checks
- Network stats simulation
- Quick actions handlers

**Решение:** Вынести логику в composables/hooks:
```typescript
// src/lib/hooks/useDashboard.ts
export function useDashboard() {
  // Store subscriptions
  // Event listeners
  // Health checks
  return { appStatus, services, networkStats, ... };
}
```

---

### 9. Hardcoded таймауты

**Файлы:** Множественные

```typescript
// +page.svelte
healthCheckInterval = setInterval(() => { ... }, 30000); // 30 сек

// services/+page.svelte
setTimeout(() => reject(new Error('Timeout')), 30000); // 30 сек
setTimeout(() => reject(new Error('Timeout')), 10000); // 10 сек
```

**Проблема:** Таймауты захардкожены в разных местах. Сложно настраивать и тестировать.

**Решение:** Вынести в конфиг:
```typescript
// src/lib/config/timeouts.ts
export const TIMEOUTS = {
  HEALTH_CHECK_INTERVAL: 30_000,
  SERVICE_CHECK_TIMEOUT: 10_000,
  BACKEND_READY_TIMEOUT: 30_000,
} as const;
```

---

## 🟡 Рекомендации

### 10. Accessibility: отсутствие ARIA labels

**Файл:** `src/routes/+layout.svelte`

```svelte
<!-- ❌ Кнопки без aria-label -->
<button onclick={() => { ... }}>
  <svg>...</svg>
  <span>Search</span>
</button>
```

**Рекомендация:** Добавить `aria-label` для всех интерактивных элементов:
```svelte
<button 
  onclick={() => { ... }}
  aria-label="Open command palette (Ctrl+K)"
>
```

---

### 11. Keyboard Navigation: отсутствие focus trap в модалках

**Файл:** `src/routes/services/+page.svelte`

**Рекомендация:** Использовать focus trap в модальных окнах:
```svelte
<BaseModal open={showDeleteConfirm} trapFocus={true}>
```

---

### 12. Toast Store: отсутствие лимита

**Файл:** `src/lib/stores/toast.ts`

```typescript
update(toasts => [...toasts, { id, type, message, duration }]);
// ❌ Нет лимита на количество toasts
```

**Рекомендация:** Добавить максимальное количество:
```typescript
const MAX_TOASTS = 5;

update(toasts => {
  const newToasts = [...toasts, { id, type, message, duration }];
  return newToasts.slice(-MAX_TOASTS);
});
```

---

### 13. Отсутствие loading states в API модуле

**Файл:** `src/lib/api/core.ts`

**Рекомендация:** Добавить wrapper с loading state:
```typescript
export function createApiCall<T>(fn: () => Promise<T>) {
  let loading = $state(false);
  let error = $state<Error | null>(null);
  let data = $state<T | null>(null);
  
  async function execute() {
    loading = true;
    error = null;
    try {
      data = await fn();
    } catch (e) {
      error = e instanceof Error ? e : new Error(String(e));
    } finally {
      loading = false;
    }
  }
  
  return { loading, error, data, execute };
}
```

---

### 14. Console.log в production

**Файлы:** Множественные

```typescript
console.warn('[Layout] Backend not ready after retries');
console.error('Failed to check onboarding status:', e);
```

**Рекомендация:** Использовать централизованный logger с уровнями:
```typescript
import { logs } from '$lib/stores/logs';

// Вместо console.warn
logs.warn('layout', 'Backend not ready after retries');
```

---

### 15. Отсутствие debounce в search

**Файл:** `src/routes/services/+page.svelte`

```typescript
let searchQuery = $state('');
// ❌ Фильтрация происходит на каждый keystroke
let filteredServices = $derived(
  services.filter(s => s.name.toLowerCase().includes(searchQuery.toLowerCase()))
);
```

**Рекомендация:** Добавить debounce:
```typescript
import { debounce } from '$lib/utils/debounce';

let searchQuery = $state('');
let debouncedQuery = $state('');

$effect(() => {
  const update = debounce(() => { debouncedQuery = searchQuery; }, 300);
  update();
});

let filteredServices = $derived(
  services.filter(s => s.name.toLowerCase().includes(debouncedQuery.toLowerCase()))
);
```

---

## 🟢 Идеи нового функционала

### 16. Offline Support

**Идея:** Добавить поддержку offline режима с кэшированием последнего состояния:
```typescript
// При загрузке - показать кэшированные данные
// При восстановлении связи - синхронизировать
```

---

### 17. Undo/Redo для действий

**Идея:** Добавить возможность отмены последних действий:
```typescript
// После остановки стратегии
toasts.success('Strategy stopped', {
  action: { label: 'Undo', onClick: () => applyStrategy(lastStrategy) }
});
```

---

### 18. Keyboard Shortcuts Help Overlay

**Идея:** Показывать подсказки по горячим клавишам при удержании Ctrl:
```svelte
{#if ctrlHeld}
  <div class="shortcuts-overlay">
    <kbd>1</kbd> Dashboard
    <kbd>2</kbd> Services
    <kbd>S</kbd> Toggle Protection
  </div>
{/if}
```

---

### 19. Service Health History

**Идея:** Хранить историю статусов сервисов для отображения трендов:
```typescript
interface ServiceHealthHistory {
  serviceId: string;
  history: Array<{
    timestamp: number;
    status: 'working' | 'blocked';
    latency?: number;
  }>;
}
```

---

### 20. Auto-recovery при деградации

**Идея:** Автоматически переключаться на backup стратегию при деградации:
```typescript
// При обнаружении деградации
if (healthCheckFailed && backupStrategy) {
  await applyStrategy(backupStrategy);
  toasts.info('Switched to backup strategy due to degradation');
}
```

---

## Сводка

| Категория | Количество |
|-----------|------------|
| 🔴 Критичные | 4 |
| 🟠 Важные | 5 |
| 🟡 Рекомендации | 6 |
| 🟢 Идеи | 5 |

### Приоритет исправлений

1. **Срочно:** Memory leaks (#1, #4) — могут вызвать деградацию производительности
2. **Высокий:** Race conditions (#2) — могут вызвать непредсказуемое поведение
3. **Средний:** Error handling (#3, #6) — улучшит стабильность
4. **Низкий:** Рефакторинг (#5, #8, #9) — улучшит maintainability
