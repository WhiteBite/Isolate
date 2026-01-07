# Isolate Plugin SDK

Система плагинов для расширения функциональности Isolate.

## Обзор

Isolate поддерживает трёхуровневую систему плагинов:

| Уровень | Тип | Сложность | Описание |
|---------|-----|-----------|----------|
| **Level 1** | `service-checker` | Простой | Декларативные JSON-конфиги для проверки сервисов |
| **Level 2** | `ui-widget` | Средний | Svelte компоненты для UI |
| **Level 3** | `lua-script` | Продвинутый | Lua скрипты для автоматизации |

## Быстрый старт

### 1. Выберите шаблон

```bash
# Level 1: Проверка сервиса
cp -r plugins/templates/service-checker plugins/my-service

# Level 2: UI виджет
cp -r plugins/templates/ui-widget plugins/my-widget

# Level 3: Lua скрипт
cp -r plugins/templates/lua-script plugins/my-script
```

### 2. Отредактируйте plugin.json

### 3. Перезапустите Isolate

Плагины загружаются автоматически из директории `plugins/`.

---

## Level 1: Service Checker

Самый простой тип плагина — декларативный JSON для добавления проверки сервиса.

### Структура

```
my-service-checker/
└── plugin.json
```

### Пример plugin.json

```json
{
  "id": "github-checker",
  "name": "GitHub Checker",
  "version": "1.0.0",
  "type": "service-checker",
  "service": {
    "id": "github",
    "name": "GitHub",
    "icon": "🐙",
    "category": "other",
    "description": "Платформа для разработчиков",
    "endpoints": [
      {
        "id": "main",
        "name": "GitHub",
        "url": "https://github.com/",
        "method": "HEAD"
      }
    ]
  },
  "permissions": {
    "http": ["github.com"]
  }
}
```

### Категории сервисов

- `social` — Социальные сети
- `media` — Медиа и стриминг
- `gaming` — Игровые платформы
- `messaging` — Мессенджеры
- `other` — Прочие

📖 **Подробнее:** [templates/service-checker/README.md](templates/service-checker/README.md)

---

## Level 2: UI Widget

Svelte компоненты для дашборда и других частей UI.

### Структура

```
my-widget/
├── plugin.json
└── ui/
    ├── MyWidget.svelte
    └── MySettings.svelte  # опционально
```

### Пример plugin.json

```json
{
  "id": "my-widget",
  "name": "My Widget",
  "version": "1.0.0",
  "type": "ui-widget",
  "icon": "📊",
  "contributes": {
    "widgets": [{
      "id": "my-main-widget",
      "name": "My Widget",
      "slot": "dashboard",
      "component": "ui/MyWidget.svelte",
      "defaultSize": { "cols": 2, "rows": 1 }
    }]
  },
  "permissions": {
    "http": ["api.example.com"],
    "storage": true,
    "events": ["my-widget-*"]
  }
}
```

### Svelte компонент

```svelte
<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  let { context }: { context: PluginContext } = $props();
  
  let data = $state('');
  
  $effect(() => {
    loadData();
  });
  
  async function loadData() {
    const saved = await context.storage.get<string>('data');
    if (saved) data = saved;
  }
</script>

<div class="p-4 bg-zinc-900/40 rounded-xl">
  <h3 class="text-xs text-zinc-400 uppercase">My Widget</h3>
  <p class="text-white">{data}</p>
</div>
```

### PluginContext API

```typescript
interface PluginContext {
  pluginId: string;
  
  storage: {
    get<T>(key: string): Promise<T | null>;
    set<T>(key: string, value: T): Promise<void>;
    remove(key: string): Promise<void>;
  };
  
  events: {
    emit(event: string, data?: any): void;
    on(event: string, handler: (data: any) => void): () => void;
  };
  
  http: {
    get(url: string): Promise<Response>;
    post(url: string, body: any): Promise<Response>;
  };
}
```

📖 **Подробнее:** [templates/ui-widget/README.md](templates/ui-widget/README.md)

---

## Level 3: Lua Script

Скрипты для автоматизации и сложной логики.

### Структура

```
my-script/
├── plugin.json
├── main.lua
└── lib/           # опционально
    └── utils.lua
```

### Пример plugin.json

```json
{
  "id": "my-script",
  "name": "My Script",
  "version": "1.0.0",
  "type": "lua-script",
  "script": {
    "entry": "main.lua",
    "triggers": {
      "events": ["status-changed"],
      "schedule": "*/5 * * * *",
      "manual": true
    }
  },
  "permissions": {
    "http": ["api.example.com"],
    "storage": true,
    "system": { "notifications": true }
  }
}
```

### Пример main.lua

```lua
local config = plugin.config()

function init()
    log.info("Script initialized")
    events.on("status-changed", on_status_changed)
end

function main(trigger)
    if not config.enabled then return end
    
    local services = isolate.services()
    local blocked = 0
    
    for _, s in ipairs(services) do
        if s.status == "blocked" then
            blocked = blocked + 1
        end
    end
    
    if blocked > 0 then
        notify.show({
            title = "Alert",
            body = blocked .. " services blocked",
            icon = "warning"
        })
    end
end

function on_status_changed(data)
    log.info("Service " .. data.service .. ": " .. data.new_status)
end
```

### Lua API

| Модуль | Функции |
|--------|---------|
| `log` | `info()`, `warn()`, `error()`, `debug()` |
| `storage` | `get()`, `set()`, `remove()` |
| `events` | `emit()`, `on()` |
| `http` | `get()`, `post()` |
| `notify` | `show()` |
| `isolate` | `services()`, `current_strategy()`, `apply_strategy()`, `stop_strategy()` |
| `json` | `encode()`, `decode()` |

📖 **Подробнее:** [templates/lua-script/README.md](templates/lua-script/README.md)

---

## Разрешения

Все плагины работают в песочнице с ограниченными правами.

### HTTP

```json
{
  "permissions": {
    "http": ["example.com", "*.example.com"]
  }
}
```

### Storage

```json
{
  "permissions": {
    "storage": true
  }
}
```

### Events

```json
{
  "permissions": {
    "events": ["my-plugin-*", "status-changed"]
  }
}
```

### System (только Lua)

```json
{
  "permissions": {
    "system": {
      "notifications": true,
      "clipboard": false
    }
  }
}
```

---

## Шаблоны

| Шаблон | Путь | Описание |
|--------|------|----------|
| Service Checker | `templates/service-checker/` | Level 1 — проверка сервиса |
| UI Widget | `templates/ui-widget/` | Level 2 — виджет дашборда |
| Lua Script | `templates/lua-script/` | Level 3 — автоматизация |

---

## Существующие плагины

### Service Checkers

| Плагин | Сервис | Категория |
|--------|--------|-----------|
| `youtube-checker` | YouTube | media |
| `discord-checker` | Discord | gaming |
| `discord-voice-checker` | Discord Voice | gaming |
| `telegram-checker` | Telegram | messaging |
| `instagram-checker` | Instagram | social |
| `twitter-checker` | Twitter/X | social |
| `steam-checker` | Steam | gaming |

### UI Widgets

| Плагин | Описание |
|--------|----------|
| `speed-widget` | Мониторинг скорости соединения |
| `latency-monitor` | График задержки сети |
| `dns-benchmark` | Бенчмарк DNS серверов |

---

## Отладка

### Проверка JSON

```bash
cat plugins/my-plugin/plugin.json | jq .
```

### Логи плагинов

- UI: Settings → Plugins → Logs
- Файл: `%APPDATA%/isolate/logs/plugins.log`

---

## Безопасность

- Плагины работают в изолированной песочнице
- HTTP запросы только к разрешённым доменам
- Нет доступа к файловой системе
- Нет выполнения системных команд
- Ограничения памяти и времени выполнения для Lua
