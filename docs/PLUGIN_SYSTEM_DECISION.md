# Архитектурное решение: Система плагинов Isolate

> **Дата:** 2026-01-06
> **Статус:** ПРИНЯТО
> **Автор:** Архитектурный анализ

---

## 1. Контекст и проблема

Isolate нуждается в расширяемой системе плагинов для:
- Добавления новых сервисов (Discord, YouTube, Telegram...)
- Добавления стратегий обхода DPI
- Кастомных UI виджетов на Dashboard
- Интеграций с внешними сервисами

**Ключевые требования:**
1. Безопасность — плагины не должны иметь полный доступ к системе
2. Простота разработки — минимальный порог входа для авторов плагинов
3. Производительность — не замедлять основное приложение
4. Изоляция — падение плагина не должно ронять приложение

---

## 2. Анализ вариантов

### Вариант A: Декларативные JSON/YAML манифесты (текущий)

```
plugins/
  discord-checker/
    plugin.json       ← Только декларация
```

**Как работает:**
- Плагин = JSON файл с описанием endpoints, стратегий, hostlists
- Backend читает манифест и регистрирует сущности
- Вся логика выполняется в core Isolate

**Плюсы:**
- ✅ Максимальная безопасность (нет исполняемого кода)
- ✅ Простота создания плагинов (только JSON)
- ✅ Нулевой overhead производительности
- ✅ Уже частично реализовано

**Минусы:**
- ❌ Нет кастомной логики (только HTTP checks)
- ❌ Нельзя добавить UI виджеты
- ❌ Ограниченная расширяемость

**Оценка:** ⭐⭐⭐ (3/5) — Хорошо для простых случаев

---

### Вариант B: WASM Runtime (WebAssembly)

```
plugins/
  discord-checker/
    plugin.wasm       ← Скомпилированный WASM модуль
    manifest.json
```

**Как работает:**
- Плагины компилируются в WASM (из Rust, Go, AssemblyScript)
- Isolate запускает WASM в песочнице (wasmtime/wasmer)
- Плагин получает доступ к Host API через imports

**Плюсы:**
- ✅ Полная изоляция (sandbox)
- ✅ Кастомная логика любой сложности
- ✅ Кроссплатформенность
- ✅ Безопасность (WASM capabilities)

**Минусы:**
- ❌ Сложность разработки плагинов (нужен Rust/Go)
- ❌ Overhead на WASM runtime (~5-10ms на вызов)
- ❌ Большой размер плагинов (~100KB+)
- ❌ Сложная отладка

**Оценка:** ⭐⭐⭐⭐ (4/5) — Мощно, но сложно

---

### Вариант C: JavaScript Runtime (Deno/Bun)

```
plugins/
  discord-checker/
    index.ts          ← TypeScript код
    manifest.json
```

**Как работает:**
- Встроенный JS runtime (deno_core или QuickJS)
- Плагины пишутся на TypeScript/JavaScript
- Sandbox через Deno permissions или V8 isolates

**Плюсы:**
- ✅ Простота разработки (JS/TS знают все)
- ✅ Богатая экосистема npm
- ✅ Хорошая изоляция (Deno permissions)
- ✅ Hot reload плагинов

**Минусы:**
- ❌ Большой размер runtime (~15-30MB)
- ❌ Потребление памяти (~50MB на runtime)
- ❌ Сложность интеграции с Rust
- ❌ Не нативная производительность

**Оценка:** ⭐⭐⭐ (3/5) — Удобно, но тяжело

---

### Вариант D: Гибридная система (РЕКОМЕНДУЕТСЯ)

```
plugins/
  discord-checker/           ← Тип 1: Декларативный
    plugin.json
    
  speed-test/                ← Тип 2: С UI компонентом
    plugin.json
    ui/
      widget.svelte          ← Svelte компонент
      
  custom-strategy/           ← Тип 3: С Lua скриптом
    plugin.json
    scripts/
      check.lua              ← Лёгкий скрипт
```

**Как работает:**
- **Уровень 1 (Декларативный):** JSON манифесты для простых плагинов
- **Уровень 2 (UI):** Svelte компоненты для виджетов (загружаются динамически)
- **Уровень 3 (Скрипты):** Lua/Rhai для кастомной логики (лёгкий runtime)

**Плюсы:**
- ✅ Простые плагины остаются простыми (JSON)
- ✅ UI плагины на знакомом Svelte
- ✅ Скрипты для сложной логики (Lua ~500KB)
- ✅ Постепенное усложнение по мере необходимости
- ✅ Минимальный overhead для большинства плагинов

**Минусы:**
- ❌ Три разных подхода = больше документации
- ❌ Нужно поддерживать несколько систем

**Оценка:** ⭐⭐⭐⭐⭐ (5/5) — Лучший баланс

---

## 3. Сравнительная таблица

| Критерий | A: JSON | B: WASM | C: JS | D: Гибрид |
|----------|---------|---------|-------|-----------|
| Безопасность | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Простота разработки | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Производительность | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Расширяемость | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Размер бандла | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Сложность реализации | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **ИТОГО** | **22** | **20** | **21** | **27** |

---

## 4. РЕШЕНИЕ: Гибридная трёхуровневая система

### Архитектура

```
┌─────────────────────────────────────────────────────────────────┐
│                        ISOLATE CORE                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Plugin Manager                         │   │
│  │  - Загрузка манифестов                                   │   │
│  │  - Валидация permissions                                 │   │
│  │  - Lifecycle management                                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐            │
│         ▼                    ▼                    ▼            │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐      │
│  │   Level 1   │     │   Level 2   │     │   Level 3   │      │
│  │ Declarative │     │  UI Plugins │     │   Scripts   │      │
│  │   (JSON)    │     │  (Svelte)   │     │   (Lua)     │      │
│  └─────────────┘     └─────────────┘     └─────────────┘      │
│                                                                 │
│  Примеры:            Примеры:            Примеры:              │
│  - service-checker   - dashboard-widget  - custom-check        │
│  - hostlist-provider - settings-panel    - strategy-script     │
│  - strategy-config   - toolbar-button    - transform-data      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Level 1: Декларативные плагины (JSON/YAML)

**Назначение:** Простые плагины без кастомной логики

**Типы:**
- `service-checker` — проверка доступности сервисов
- `hostlist-provider` — списки доменов
- `strategy-config` — конфигурации стратегий

**Структура:**
```
plugins/discord-checker/
  plugin.json
```

**Манифест:**
```json
{
  "id": "discord-checker",
  "name": "Discord Checker",
  "version": "1.0.0",
  "type": "service-checker",
  "contributes": {
    "services": [{
      "id": "discord",
      "name": "Discord",
      "icon": "🎮",
      "endpoints": [
        { "url": "https://discord.com/api/v10/gateway", "method": "GET" }
      ]
    }]
  }
}
```

**Runtime:** Нет (только парсинг JSON)

---

### Level 2: UI плагины (Svelte компоненты)

**Назначение:** Виджеты для Dashboard, панели настроек, toolbar кнопки

**Типы:**
- `dashboard-widget` — виджет на Dashboard
- `settings-panel` — панель в Settings
- `toolbar-action` — кнопка в toolbar
- `sidebar-item` — элемент в sidebar

**Структура:**
```
plugins/speed-test/
  plugin.json
  ui/
    SpeedTestWidget.svelte    ← Svelte 5 компонент
    SpeedTestSettings.svelte
```

**Манифест:**
```json
{
  "id": "speed-test",
  "name": "Speed Test",
  "version": "1.0.0",
  "type": "ui-plugin",
  "contributes": {
    "widgets": [{
      "id": "speed-widget",
      "name": "Speed Test",
      "slot": "dashboard",
      "component": "ui/SpeedTestWidget.svelte",
      "defaultSize": { "cols": 2, "rows": 1 }
    }],
    "settings": [{
      "id": "speed-settings",
      "component": "ui/SpeedTestSettings.svelte"
    }]
  }
}
```

**Runtime:** Динамическая загрузка Svelte компонентов

---

### Level 3: Скриптовые плагины (Lua/Rhai)

**Назначение:** Кастомная логика проверок, трансформации данных

**Типы:**
- `custom-checker` — кастомная логика проверки
- `data-transformer` — обработка данных
- `strategy-script` — динамические стратегии

**Структура:**
```
plugins/custom-discord/
  plugin.json
  scripts/
    check.lua             ← Lua скрипт
```

**Манифест:**
```json
{
  "id": "custom-discord",
  "name": "Custom Discord Check",
  "version": "1.0.0",
  "type": "script-plugin",
  "contributes": {
    "checkers": [{
      "id": "discord-voice",
      "name": "Discord Voice Check",
      "script": "scripts/check.lua",
      "trigger": "on-demand"
    }]
  },
  "permissions": {
    "http": ["discord.com", "*.discord.gg"],
    "timeout": 10000
  }
}
```

**Lua скрипт:**
```lua
-- scripts/check.lua
function check()
  -- API доступно через глобальные функции
  local response = http_get("https://discord.com/api/v10/gateway")
  
  if response.status == 200 then
    local data = json_decode(response.body)
    return {
      success = true,
      latency = response.latency_ms,
      details = { gateway = data.url }
    }
  else
    return {
      success = false,
      error = "API returned " .. response.status
    }
  end
end
```

**Runtime:** mlua (Lua 5.4) — ~500KB, быстрый, безопасный

---

## 5. API для плагинов

### Host API (доступно из Lua скриптов)

```lua
-- HTTP
http_get(url, headers?)           → Response
http_post(url, body, headers?)    → Response
http_head(url, headers?)          → Response

-- JSON
json_encode(table)                → string
json_decode(string)               → table

-- Logging
log_info(message)
log_warn(message)
log_error(message)
log_debug(message)

-- Storage (per-plugin isolated)
storage_get(key)                  → value | nil
storage_set(key, value)
storage_delete(key)

-- Config
config_get(key)                   → value
plugin_id()                       → string
plugin_version()                  → string

-- Events (emit to frontend)
emit_event(name, data)
```

### Frontend API (для Svelte компонентов)

```typescript
// Доступно через props
interface PluginContext {
  pluginId: string;
  pluginVersion: string;
  
  // Storage
  storage: {
    get<T>(key: string): Promise<T | null>;
    set<T>(key: string, value: T): Promise<void>;
    delete(key: string): Promise<void>;
  };
  
  // Events
  emit(event: string, data: any): void;
  on(event: string, handler: (data: any) => void): () => void;
  
  // Tauri invoke (sandboxed)
  invoke<T>(command: string, args?: object): Promise<T>;
}
```

---

## 6. Безопасность

### Модель разрешений

```json
{
  "permissions": {
    "http": ["discord.com", "*.discordapp.com"],  // Whitelist доменов
    "storage": true,                               // Доступ к storage
    "events": ["status-changed", "check-*"],       // Whitelist событий
    "timeout": 10000,                              // Max время выполнения
    "memory": 10485760                             // Max память (10MB)
  }
}
```

### Изоляция

| Уровень | Изоляция | Доступ к FS | Доступ к сети | Доступ к UI |
|---------|----------|-------------|---------------|-------------|
| Level 1 | Полная | ❌ | Через core | ❌ |
| Level 2 | Sandbox | ❌ | Через API | ✅ (свой slot) |
| Level 3 | Lua VM | ❌ | Whitelist | ❌ |

### Валидация

1. **При установке:** Проверка манифеста, подпись (опционально)
2. **При загрузке:** Проверка permissions, sandbox setup
3. **При выполнении:** Timeout, memory limits, rate limiting

---

## 7. Marketplace и распространение

### Источники плагинов

```
1. Встроенные (builtin/)
   - Поставляются с приложением
   - Не могут быть удалены
   - Обновляются с приложением

2. Официальные (marketplace)
   - Проверены командой Isolate
   - Автообновление
   - Подписаны

3. Community (GitHub)
   - Устанавливаются по URL
   - Требуют подтверждения пользователя
   - Без автообновления

4. Локальные (plugins/)
   - Для разработки
   - Без проверок
```

### Формат распространения

```
plugin-name-1.0.0.isolate-plugin
  ├── plugin.json
  ├── ui/
  │   └── *.svelte
  ├── scripts/
  │   └── *.lua
  ├── assets/
  │   └── icon.png
  └── SIGNATURE          ← Опциональная подпись
```

---

## 8. План реализации

### Фаза 1: Level 1 — Декларативные (2-3 дня)
- [x] Plugin manifest schema
- [x] Plugin loader (JSON)
- [x] Service registry
- [ ] Hostlist registry
- [ ] Strategy registry
- [ ] Marketplace UI (базовый)

### Фаза 2: Level 2 — UI плагины (3-4 дня)
- [ ] Dynamic Svelte component loader
- [ ] PluginSlot improvements
- [ ] Plugin context API
- [ ] Dashboard widget system
- [ ] Settings panel integration

### Фаза 3: Level 3 — Скрипты (2-3 дня)
- [ ] Lua runtime integration (mlua)
- [ ] Host API implementation
- [ ] Sandbox configuration
- [ ] Script execution engine

### Фаза 4: Polish (2-3 дня)
- [ ] Plugin installation flow
- [ ] Plugin updates
- [ ] Error handling & recovery
- [ ] Documentation & examples

---

## 9. Примеры плагинов

### Пример 1: Service Checker (Level 1)

```json
// plugins/instagram-checker/plugin.json
{
  "id": "instagram-checker",
  "name": "Instagram Checker",
  "version": "1.0.0",
  "author": "Community",
  "type": "service-checker",
  "icon": "📷",
  "contributes": {
    "services": [{
      "id": "instagram",
      "name": "Instagram",
      "icon": "📷",
      "category": "social",
      "endpoints": [
        { "id": "web", "url": "https://www.instagram.com/", "method": "HEAD" },
        { "id": "api", "url": "https://i.instagram.com/api/v1/", "method": "HEAD" }
      ]
    }]
  },
  "permissions": {
    "http": ["instagram.com", "*.instagram.com"]
  }
}
```

### Пример 2: Dashboard Widget (Level 2)

```json
// plugins/latency-monitor/plugin.json
{
  "id": "latency-monitor",
  "name": "Latency Monitor",
  "version": "1.0.0",
  "type": "ui-plugin",
  "icon": "📊",
  "contributes": {
    "widgets": [{
      "id": "latency-graph",
      "name": "Latency Graph",
      "slot": "dashboard",
      "component": "ui/LatencyGraph.svelte",
      "defaultSize": { "cols": 2, "rows": 2 }
    }]
  }
}
```

```svelte
<!-- plugins/latency-monitor/ui/LatencyGraph.svelte -->
<script lang="ts">
  import type { PluginContext } from '$lib/types/plugin';
  
  let { context }: { context: PluginContext } = $props();
  
  let history = $state<number[]>([]);
  let currentPing = $state(0);
  
  $effect(() => {
    const interval = setInterval(async () => {
      const result = await context.invoke<{latency: number}>('ping_service', {
        serviceId: 'discord'
      });
      currentPing = result.latency;
      history = [...history.slice(-29), result.latency];
    }, 2000);
    
    return () => clearInterval(interval);
  });
</script>

<div class="p-4 bg-zinc-900/40 rounded-xl">
  <h3 class="text-sm font-medium text-zinc-400 mb-2">Latency Monitor</h3>
  <div class="text-3xl font-bold text-white">{currentPing}ms</div>
  <!-- Canvas graph here -->
</div>
```

### Пример 3: Custom Checker (Level 3)

```json
// plugins/discord-voice/plugin.json
{
  "id": "discord-voice",
  "name": "Discord Voice Check",
  "version": "1.0.0",
  "type": "script-plugin",
  "icon": "🎤",
  "contributes": {
    "checkers": [{
      "id": "voice-latency",
      "name": "Voice Latency",
      "script": "scripts/voice_check.lua",
      "targetService": "discord"
    }]
  },
  "permissions": {
    "http": ["discord.com", "*.discord.gg", "*.discordapp.net"],
    "timeout": 15000
  }
}
```

```lua
-- plugins/discord-voice/scripts/voice_check.lua

function check()
  -- Получаем список voice серверов
  local gateway = http_get("https://discord.com/api/v10/gateway")
  if gateway.status ~= 200 then
    return { success = false, error = "Gateway unavailable" }
  end
  
  -- Тестируем несколько voice регионов
  local regions = {"eu-west", "eu-central", "us-east"}
  local results = {}
  
  for _, region in ipairs(regions) do
    local url = "https://" .. region .. ".discord.gg/"
    local response = http_head(url)
    
    table.insert(results, {
      region = region,
      latency = response.latency_ms,
      available = response.status < 400
    })
  end
  
  -- Находим лучший регион
  table.sort(results, function(a, b) return a.latency < b.latency end)
  local best = results[1]
  
  return {
    success = best.available,
    latency = best.latency,
    details = {
      best_region = best.region,
      all_regions = results
    }
  }
end
```

---

## 10. Миграция существующего кода

### Что вынести в плагины

| Текущее расположение | Тип плагина | Приоритет |
|---------------------|-------------|-----------|
| `configs/services/*.yaml` | service-checker (L1) | P0 |
| `configs/strategies/*.yaml` | strategy-config (L1) | P0 |
| `configs/hostlists/*.txt` | hostlist-provider (L1) | P0 |
| Dashboard виджеты | ui-plugin (L2) | P1 |
| Speed Test | ui-plugin + script (L2+L3) | P1 |
| Latency Monitor | ui-plugin (L2) | P1 |

### Встроенные плагины (builtin/)

```
builtin/
  discord-checker/        ← service-checker
  youtube-checker/        ← service-checker
  telegram-checker/       ← service-checker
  zapret-strategies/      ← strategy-config
  common-hostlists/       ← hostlist-provider
  status-widget/          ← ui-plugin (Dashboard)
  health-widget/          ← ui-plugin (Dashboard)
```

---

## 11. Альтернативы, которые отвергнуты

### ❌ Electron-style Node.js плагины
- Слишком тяжёлый runtime
- Проблемы с безопасностью
- Не соответствует философии Tauri

### ❌ Полный WASM для всего
- Слишком сложно для простых плагинов
- Высокий порог входа
- Overkill для service-checker

### ❌ Только декларативные плагины
- Недостаточно гибко
- Нельзя добавить UI
- Ограничивает экосистему

### ❌ Native Rust плагины (dylib)
- Проблемы с ABI совместимостью
- Сложность распространения
- Риски безопасности

---

## 12. Метрики успеха

1. **Простота:** 80% плагинов должны быть Level 1 (только JSON)
2. **Производительность:** Загрузка 10 плагинов < 100ms
3. **Безопасность:** 0 инцидентов с вредоносными плагинами
4. **Экосистема:** 20+ community плагинов за 6 месяцев

---

## 13. Заключение

**Выбрано: Гибридная трёхуровневая система**

Это решение обеспечивает:
- Простоту для 80% случаев (JSON манифесты)
- Гибкость для UI расширений (Svelte компоненты)
- Мощность для сложной логики (Lua скрипты)
- Безопасность на всех уровнях

**Следующие шаги:**
1. Доработать Level 1 (hostlist + strategy registry)
2. Реализовать динамическую загрузку Svelte компонентов
3. Интегрировать mlua для Level 3
4. Создать 5+ примеров плагинов
5. Написать документацию для разработчиков плагинов

---

*Документ создан: 2026-01-06*
*Последнее обновление: 2026-01-06*
