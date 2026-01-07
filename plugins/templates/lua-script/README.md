# Lua Script Plugin Template

Шаблон для создания Lua скрипта автоматизации (Level 3).

## Быстрый старт

1. Скопируйте эту директорию:
   ```bash
   cp -r plugins/templates/lua-script plugins/my-script
   ```

2. Отредактируйте `plugin.json`:
   - Измените `id` на уникальный идентификатор
   - Настройте триггеры запуска
   - Укажите необходимые разрешения

3. Напишите логику в `main.lua`

4. Перезапустите Isolate — скрипт загрузится автоматически

## Структура плагина

```
my-script/
├── plugin.json     # Манифест плагина
├── README.md       # Документация
├── main.lua        # Точка входа
└── lib/            # Дополнительные модули (опционально)
    └── utils.lua
```

## Конфигурация plugin.json

### Обязательные поля

| Поле | Описание |
|------|----------|
| `id` | Уникальный идентификатор (kebab-case) |
| `name` | Отображаемое имя |
| `version` | Версия в формате semver |
| `type` | Должен быть `"lua-script"` |
| `script.entry` | Путь к главному Lua файлу |

### Триггеры запуска

```json
{
  "script": {
    "entry": "main.lua",
    "triggers": {
      "events": ["strategy-applied", "status-changed"],
      "schedule": "*/5 * * * *",
      "manual": true
    }
  }
}
```

| Триггер | Описание |
|---------|----------|
| `events` | Запуск при событиях |
| `schedule` | Cron-выражение для периодического запуска |
| `manual` | Разрешить ручной запуск из UI |

### События для триггеров

- `app-started` — Запуск приложения
- `strategy-applied` — Применена стратегия
- `strategy-stopped` — Остановлена стратегия
- `status-changed` — Изменился статус сервиса
- `test-completed` — Завершено тестирование

### Cron-выражения

```
*/5 * * * *     # Каждые 5 минут
0 * * * *       # Каждый час
0 0 * * *       # Каждый день в полночь
0 0 * * 1       # Каждый понедельник
```

### Песочница (Sandbox)

```json
{
  "script": {
    "sandbox": {
      "memoryLimit": 10485760,
      "timeout": 30000,
      "allowedModules": ["string", "table", "math"]
    }
  }
}
```

| Параметр | Описание |
|----------|----------|
| `memoryLimit` | Лимит памяти в байтах (10MB по умолчанию) |
| `timeout` | Таймаут выполнения в мс |
| `allowedModules` | Разрешённые стандартные модули Lua |

## Lua API

### Глобальные функции

```lua
-- Логирование
log.info("Information message")
log.warn("Warning message")
log.error("Error message")
log.debug("Debug message")

-- Конфигурация плагина
local config = plugin.config()
local value = config.threshold

-- Хранилище
storage.set("key", "value")
local value = storage.get("key")
storage.remove("key")
```

### HTTP запросы

```lua
-- GET запрос
local response = http.get("https://api.example.com/data")
if response.ok then
    local data = json.decode(response.body)
    log.info("Got data: " .. data.value)
end

-- POST запрос
local response = http.post("https://api.example.com/action", {
    headers = { ["Content-Type"] = "application/json" },
    body = json.encode({ action = "test" })
})

-- С таймаутом
local response = http.get("https://api.example.com/data", {
    timeout = 5000
})
```

### События

```lua
-- Отправка события
events.emit("my-script-completed", { result = "success" })

-- Подписка на события (в init)
events.on("status-changed", function(data)
    log.info("Status changed: " .. data.service)
end)
```

### Уведомления

```lua
-- Показать уведомление (требует permission)
notify.show({
    title = "My Script",
    body = "Task completed successfully",
    icon = "success"  -- success | warning | error | info
})
```

### Isolate API

```lua
-- Получить текущую стратегию
local strategy = isolate.current_strategy()
if strategy then
    log.info("Current strategy: " .. strategy.name)
end

-- Получить статус сервисов
local services = isolate.services()
for _, service in ipairs(services) do
    log.info(service.name .. ": " .. service.status)
end

-- Применить стратегию
isolate.apply_strategy("youtube-fix")

-- Остановить стратегию
isolate.stop_strategy()
```

## Примеры

### main.lua — Базовый шаблон

```lua
-- ============================================
-- My Lua Script
-- ============================================

local config = plugin.config()

-- Инициализация (вызывается один раз при загрузке)
function init()
    log.info("Script initialized")
    
    -- Подписка на события
    events.on("status-changed", on_status_changed)
end

-- Главная функция (вызывается по триггерам)
function main(trigger)
    log.info("Script triggered by: " .. trigger.type)
    
    if not config.enabled then
        log.info("Script is disabled")
        return
    end
    
    -- Ваша логика здесь
    do_work()
end

-- Обработчик события
function on_status_changed(data)
    log.info("Service " .. data.service .. " is now " .. data.status)
end

-- Основная логика
function do_work()
    -- Пример: проверка и уведомление
    local services = isolate.services()
    local blocked_count = 0
    
    for _, service in ipairs(services) do
        if service.status == "blocked" then
            blocked_count = blocked_count + 1
        end
    end
    
    if blocked_count > config.threshold then
        notify.show({
            title = "Alert",
            body = blocked_count .. " services are blocked!",
            icon = "warning"
        })
    end
    
    -- Сохранение результата
    storage.set("last_check", {
        time = os.time(),
        blocked = blocked_count
    })
    
    events.emit("my-script-completed", { blocked = blocked_count })
end
```

### Автоматическое переключение стратегий

```lua
function main(trigger)
    local services = isolate.services()
    
    -- Проверяем YouTube
    local youtube = find_service(services, "youtube")
    if youtube and youtube.status == "blocked" then
        log.info("YouTube blocked, applying fix...")
        isolate.apply_strategy("youtube-fix")
        return
    end
    
    -- Проверяем Discord
    local discord = find_service(services, "discord")
    if discord and discord.status == "blocked" then
        log.info("Discord blocked, applying fix...")
        isolate.apply_strategy("discord-fix")
        return
    end
    
    log.info("All services OK")
end

function find_service(services, id)
    for _, s in ipairs(services) do
        if s.id == id then return s end
    end
    return nil
end
```

### Мониторинг с историей

```lua
local MAX_HISTORY = 100

function main()
    local history = storage.get("history") or {}
    
    -- Собираем метрики
    local metrics = collect_metrics()
    
    -- Добавляем в историю
    table.insert(history, {
        time = os.time(),
        metrics = metrics
    })
    
    -- Ограничиваем размер
    while #history > MAX_HISTORY do
        table.remove(history, 1)
    end
    
    storage.set("history", history)
    
    -- Анализ тренда
    if #history >= 10 then
        analyze_trend(history)
    end
end

function collect_metrics()
    local services = isolate.services()
    local available = 0
    local total = #services
    
    for _, s in ipairs(services) do
        if s.status == "available" then
            available = available + 1
        end
    end
    
    return {
        available = available,
        total = total,
        ratio = available / total
    }
end

function analyze_trend(history)
    -- Последние 10 записей
    local recent = {}
    for i = #history - 9, #history do
        table.insert(recent, history[i].metrics.ratio)
    end
    
    -- Средняя доступность
    local sum = 0
    for _, r in ipairs(recent) do sum = sum + r end
    local avg = sum / #recent
    
    if avg < 0.5 then
        notify.show({
            title = "Low Availability",
            body = string.format("Average availability: %.0f%%", avg * 100),
            icon = "warning"
        })
    end
end
```

## Отладка

### Логирование

```lua
log.debug("Variable value: " .. tostring(value))
log.info("Processing started")
log.warn("Unexpected state")
log.error("Failed to connect")
```

### Просмотр логов

Логи скриптов доступны в:
- UI: Settings → Plugins → Logs
- Файл: `%APPDATA%/isolate/logs/plugins.log`

### Тестирование

1. Установите `manual: true` в триггерах
2. Запустите скрипт вручную из UI
3. Проверьте логи на ошибки

## Ограничения

- Нет доступа к файловой системе
- Нет выполнения системных команд
- HTTP только к разрешённым доменам
- Ограничение памяти и времени выполнения
- Нет многопоточности

## Разрешения

```json
{
  "permissions": {
    "http": ["api.example.com"],
    "storage": true,
    "events": ["my-script-*"],
    "system": {
      "notifications": true,
      "clipboard": false
    }
  }
}
```

| Разрешение | Описание |
|------------|----------|
| `http` | Домены для HTTP запросов |
| `storage` | Локальное хранилище |
| `events` | Отправка/получение событий |
| `system.notifications` | Показ уведомлений |
| `system.clipboard` | Доступ к буферу обмена |

## См. также

- [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/)
- [JSON в Lua](https://github.com/rxi/json.lua)
