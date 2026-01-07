-- ============================================
-- My Lua Script Template
-- ============================================
-- 
-- Это шаблон Lua скрипта для Isolate.
-- Скрипты могут автоматизировать задачи, реагировать на события
-- и взаимодействовать с системой.
--
-- Документация: plugins/templates/lua-script/README.md
-- ============================================

-- Загружаем конфигурацию плагина
local config = plugin.config()

-- ============================================
-- Инициализация
-- ============================================
-- Вызывается один раз при загрузке скрипта

function init()
    log.info("Script '" .. plugin.id .. "' initialized")
    log.info("Version: " .. plugin.version)
    
    -- Подписка на события
    events.on("status-changed", on_status_changed)
    events.on("strategy-applied", on_strategy_applied)
    
    -- Загрузка сохранённого состояния
    local state = storage.get("state")
    if state then
        log.info("Restored state from storage")
    end
end

-- ============================================
-- Главная функция
-- ============================================
-- Вызывается по триггерам (события, расписание, вручную)
--
-- @param trigger {type: string, data: table}
--   type: "event" | "schedule" | "manual"
--   data: данные события (для type="event")

function main(trigger)
    log.info("Script triggered: " .. trigger.type)
    
    -- Проверяем, включён ли скрипт
    if not config.enabled then
        log.info("Script is disabled in config")
        return
    end
    
    -- Обработка в зависимости от типа триггера
    if trigger.type == "manual" then
        on_manual_run()
    elseif trigger.type == "schedule" then
        on_scheduled_run()
    elseif trigger.type == "event" then
        on_event_trigger(trigger.data)
    end
end

-- ============================================
-- Обработчики триггеров
-- ============================================

function on_manual_run()
    log.info("Manual run started")
    
    -- Пример: проверка всех сервисов
    local result = check_services()
    
    -- Показываем уведомление о результате
    notify.show({
        title = "Check Complete",
        body = string.format("%d/%d services available", 
            result.available, result.total),
        icon = result.available == result.total and "success" or "warning"
    })
    
    -- Отправляем событие о завершении
    events.emit("my-script-completed", result)
end

function on_scheduled_run()
    log.info("Scheduled run started")
    
    local result = check_services()
    
    -- Сохраняем в историю
    save_to_history(result)
    
    -- Проверяем пороговое значение
    if result.blocked > config.threshold then
        log.warn("Blocked services exceed threshold!")
        notify.show({
            title = "Alert",
            body = result.blocked .. " services are blocked",
            icon = "warning"
        })
    end
end

function on_event_trigger(data)
    log.info("Event trigger: " .. json.encode(data))
    -- Обработка события
end

-- ============================================
-- Обработчики событий
-- ============================================

function on_status_changed(data)
    log.info(string.format("Service '%s' status: %s -> %s",
        data.service,
        data.old_status or "unknown",
        data.new_status
    ))
    
    -- Пример: автоматическое применение стратегии
    if data.new_status == "blocked" and config.auto_fix then
        local strategy = get_fix_strategy(data.service)
        if strategy then
            log.info("Auto-applying strategy: " .. strategy)
            isolate.apply_strategy(strategy)
        end
    end
end

function on_strategy_applied(data)
    log.info("Strategy applied: " .. data.strategy)
    
    -- Сохраняем время применения
    storage.set("last_strategy", {
        name = data.strategy,
        time = os.time()
    })
end

-- ============================================
-- Вспомогательные функции
-- ============================================

function check_services()
    local services = isolate.services()
    local result = {
        total = #services,
        available = 0,
        blocked = 0,
        unknown = 0,
        details = {}
    }
    
    for _, service in ipairs(services) do
        if service.status == "available" then
            result.available = result.available + 1
        elseif service.status == "blocked" then
            result.blocked = result.blocked + 1
        else
            result.unknown = result.unknown + 1
        end
        
        table.insert(result.details, {
            id = service.id,
            name = service.name,
            status = service.status
        })
    end
    
    return result
end

function save_to_history(result)
    local history = storage.get("history") or {}
    
    table.insert(history, {
        time = os.time(),
        available = result.available,
        blocked = result.blocked,
        total = result.total
    })
    
    -- Ограничиваем размер истории
    while #history > 100 do
        table.remove(history, 1)
    end
    
    storage.set("history", history)
end

function get_fix_strategy(service_id)
    -- Маппинг сервисов на стратегии
    local strategies = {
        youtube = "youtube-fix",
        discord = "discord-fix",
        instagram = "instagram-fix",
        twitter = "twitter-fix"
    }
    
    return strategies[service_id]
end

-- ============================================
-- Экспорт (опционально)
-- ============================================
-- Функции, доступные другим скриптам

return {
    check_services = check_services,
    get_fix_strategy = get_fix_strategy
}
