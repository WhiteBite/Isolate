# UI Widget Plugin Template

Шаблон для создания UI виджета для дашборда (Level 2).

## Быстрый старт

1. Скопируйте эту директорию:
   ```bash
   cp -r plugins/templates/ui-widget plugins/my-widget
   ```

2. Отредактируйте `plugin.json`:
   - Измените `id` на уникальный идентификатор
   - Настройте виджеты в `contributes.widgets`
   - Укажите необходимые разрешения

3. Создайте Svelte компонент в `ui/MyWidget.svelte`

4. Перезапустите Isolate — виджет появится на дашборде

## Структура плагина

```
my-widget/
├── plugin.json          # Манифест плагина
├── README.md            # Документация
└── ui/
    ├── MyWidget.svelte  # Основной виджет
    └── MySettings.svelte # Настройки (опционально)
```

## Конфигурация plugin.json

### Обязательные поля

| Поле | Описание |
|------|----------|
| `id` | Уникальный идентификатор (kebab-case) |
| `name` | Отображаемое имя |
| `version` | Версия в формате semver |
| `type` | Должен быть `"ui-widget"` |
| `contributes.widgets` | Массив виджетов |

### Конфигурация виджета

```json
{
  "contributes": {
    "widgets": [
      {
        "id": "my-widget",
        "name": "My Widget",
        "slot": "dashboard",
        "component": "ui/MyWidget.svelte",
        "defaultSize": { "cols": 2, "rows": 1 },
        "order": 50,
        "icon": "📊"
      }
    ]
  }
}
```

| Поле | Тип | Описание |
|------|-----|----------|
| `id` | string | Уникальный ID виджета |
| `name` | string | Отображаемое имя |
| `slot` | string | `dashboard`, `sidebar`, `statusbar` |
| `component` | string | Путь к Svelte компоненту |
| `defaultSize` | object | Размер в grid-ячейках |
| `order` | number | Порядок отображения |
| `icon` | string | Emoji иконка |

### Слоты размещения

- `dashboard` — Основная область дашборда (grid layout)
- `sidebar` — Боковая панель
- `statusbar` — Статус-бар внизу

### Размеры виджетов

```json
{ "cols": 1, "rows": 1 }  // Маленький (1x1)
{ "cols": 2, "rows": 1 }  // Широкий (2x1)
{ "cols": 2, "rows": 2 }  // Большой (2x2)
{ "cols": 4, "rows": 2 }  // Полная ширина
```

## Svelte компонент

### Базовая структура

```svelte
<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  interface Props {
    context: PluginContext;
  }
  
  let { context }: Props = $props();
  
  // Состояние виджета
  let data = $state<string>('');
  let loading = $state(false);
  
  // Загрузка данных при монтировании
  $effect(() => {
    loadData();
  });
  
  async function loadData() {
    loading = true;
    try {
      // Загрузка из storage
      const saved = await context.storage.get<string>('my-data');
      if (saved) data = saved;
    } finally {
      loading = false;
    }
  }
</script>

<div class="p-4 bg-zinc-900/40 rounded-xl border border-white/5">
  <h3 class="text-xs text-zinc-400 uppercase tracking-wider mb-2">
    My Widget
  </h3>
  
  {#if loading}
    <p class="text-zinc-500">Loading...</p>
  {:else}
    <p class="text-white">{data || 'No data'}</p>
  {/if}
</div>
```

### PluginContext API

```typescript
interface PluginContext {
  // Информация о плагине
  pluginId: string;
  
  // Локальное хранилище
  storage: {
    get<T>(key: string): Promise<T | null>;
    set<T>(key: string, value: T): Promise<void>;
    remove(key: string): Promise<void>;
  };
  
  // События
  events: {
    emit(event: string, data?: any): void;
    on(event: string, handler: (data: any) => void): () => void;
  };
  
  // HTTP запросы (только к разрешённым доменам)
  http: {
    get(url: string): Promise<Response>;
    post(url: string, body: any): Promise<Response>;
  };
}
```

### Работа с хранилищем

```typescript
// Сохранение данных
await context.storage.set('settings', { theme: 'dark' });

// Загрузка данных
const settings = await context.storage.get<{ theme: string }>('settings');

// Удаление
await context.storage.remove('settings');
```

### Работа с событиями

```typescript
// Отправка события
context.events.emit('my-widget-updated', { value: 42 });

// Подписка на события
const unsubscribe = context.events.on('status-changed', (data) => {
  console.log('Status changed:', data);
});

// Отписка при размонтировании
$effect(() => {
  return () => unsubscribe();
});
```

### HTTP запросы

```typescript
// GET запрос
const response = await context.http.get('https://api.example.com/data');
const data = await response.json();

// POST запрос
const response = await context.http.post('https://api.example.com/action', {
  action: 'test'
});
```

## Стилизация

Используйте Tailwind CSS классы:

```svelte
<div class="
  p-4 
  bg-zinc-900/40 
  backdrop-blur-md 
  border border-white/5 
  rounded-xl
  hover:border-white/10
  transition-all
">
  <!-- Контент -->
</div>
```

### Цветовая схема

- Фон: `bg-zinc-900/40`, `bg-zinc-800`
- Текст: `text-white`, `text-zinc-400`, `text-zinc-500`
- Акценты: `text-cyan-400`, `text-emerald-400`, `text-amber-400`
- Границы: `border-white/5`, `border-white/10`

## Разрешения

```json
{
  "permissions": {
    "http": ["api.example.com"],
    "storage": true,
    "events": ["my-widget-*", "status-changed"],
    "timeout": 10000
  }
}
```

| Разрешение | Описание |
|------------|----------|
| `http` | Домены для HTTP запросов |
| `storage` | Доступ к локальному хранилищу |
| `events` | Паттерны событий (wildcards) |
| `timeout` | Максимальный таймаут операций |

## Примеры

### Виджет статуса

```svelte
<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  let { context }: { context: PluginContext } = $props();
  
  let status = $state<'online' | 'offline' | 'checking'>('checking');
  
  $effect(() => {
    checkStatus();
    const interval = setInterval(checkStatus, 30000);
    return () => clearInterval(interval);
  });
  
  async function checkStatus() {
    status = 'checking';
    try {
      await context.http.get('https://api.example.com/health');
      status = 'online';
    } catch {
      status = 'offline';
    }
  }
</script>

<div class="p-3 bg-zinc-900/40 rounded-lg">
  <div class="flex items-center gap-2">
    <span class="w-2 h-2 rounded-full {
      status === 'online' ? 'bg-emerald-400' :
      status === 'offline' ? 'bg-red-400' : 'bg-amber-400 animate-pulse'
    }"></span>
    <span class="text-sm text-zinc-300">
      {status === 'online' ? 'Online' : status === 'offline' ? 'Offline' : 'Checking...'}
    </span>
  </div>
</div>
```

## См. также

- [plugins/speed-widget](../speed-widget/) — пример виджета скорости
- [plugins/latency-monitor](../latency-monitor/) — пример виджета с графиком
