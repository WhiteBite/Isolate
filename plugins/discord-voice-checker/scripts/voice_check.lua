-- Discord Voice Checker
-- Проверяет доступность voice серверов Discord и измеряет задержку по регионам

-- Список voice регионов Discord
local VOICE_REGIONS = {
    { id = "eu-west", name = "EU West", host = "eu-west.discord.gg" },
    { id = "eu-central", name = "EU Central", host = "eu-central.discord.gg" },
    { id = "us-east", name = "US East", host = "us-east.discord.gg" },
    { id = "us-west", name = "US West", host = "us-west.discord.gg" },
    { id = "us-central", name = "US Central", host = "us-central.discord.gg" },
    { id = "us-south", name = "US South", host = "us-south.discord.gg" },
    { id = "singapore", name = "Singapore", host = "singapore.discord.gg" },
    { id = "brazil", name = "Brazil", host = "brazil.discord.gg" },
    { id = "russia", name = "Russia", host = "russia.discord.gg" }
}

-- Проверка доступности Gateway API
local function check_gateway()
    log_debug("Checking Discord Gateway API...")
    
    local response = http_get("https://discord.com/api/v10/gateway")
    
    if response.status ~= 200 then
        log_error("Gateway API returned status: " .. tostring(response.status))
        return nil, "Gateway API unavailable (status: " .. tostring(response.status) .. ")"
    end
    
    local data = json_decode(response.body)
    if not data or not data.url then
        log_error("Invalid gateway response")
        return nil, "Invalid gateway response"
    end
    
    log_info("Gateway URL: " .. data.url)
    return data.url, nil
end

-- Проверка одного voice региона
local function check_region(region)
    local url = "https://" .. region.host .. "/"
    
    log_debug("Testing region: " .. region.name .. " (" .. url .. ")")
    
    local response = http_head(url)
    
    return {
        id = region.id,
        name = region.name,
        host = region.host,
        latency_ms = response.latency_ms or 0,
        available = response.status < 400 or response.status == 404,
        status = response.status
    }
end

-- Проверка всех voice регионов
local function check_all_regions()
    local results = {}
    local available_count = 0
    
    for _, region in ipairs(VOICE_REGIONS) do
        local result = check_region(region)
        table.insert(results, result)
        
        if result.available then
            available_count = available_count + 1
            log_info(string.format(
                "Region %s: %dms (available)",
                result.name,
                result.latency_ms
            ))
        else
            log_warn(string.format(
                "Region %s: unavailable (status %d)",
                result.name,
                result.status
            ))
        end
    end
    
    return results, available_count
end

-- Найти лучший регион по задержке
local function find_best_region(results)
    local best = nil
    local best_latency = math.huge
    
    for _, result in ipairs(results) do
        if result.available and result.latency_ms < best_latency then
            best = result
            best_latency = result.latency_ms
        end
    end
    
    return best
end

-- Вычислить среднюю задержку
local function calculate_average_latency(results)
    local sum = 0
    local count = 0
    
    for _, result in ipairs(results) do
        if result.available then
            sum = sum + result.latency_ms
            count = count + 1
        end
    end
    
    if count == 0 then
        return 0
    end
    
    return math.floor(sum / count)
end

-- Основная функция проверки
function check()
    log_info("Starting Discord Voice check...")
    
    -- Шаг 1: Проверяем Gateway API
    local gateway_url, gateway_error = check_gateway()
    if gateway_error then
        return {
            success = false,
            error = gateway_error,
            details = {
                gateway_available = false
            }
        }
    end
    
    -- Шаг 2: Проверяем все voice регионы
    local results, available_count = check_all_regions()
    
    if available_count == 0 then
        log_error("No voice regions available!")
        return {
            success = false,
            error = "All voice regions are unavailable",
            details = {
                gateway_available = true,
                gateway_url = gateway_url,
                regions_tested = #VOICE_REGIONS,
                regions_available = 0,
                regions = results
            }
        }
    end
    
    -- Шаг 3: Находим лучший регион
    local best_region = find_best_region(results)
    local avg_latency = calculate_average_latency(results)
    
    log_info(string.format(
        "Best region: %s (%dms), Average: %dms, Available: %d/%d",
        best_region.name,
        best_region.latency_ms,
        avg_latency,
        available_count,
        #VOICE_REGIONS
    ))
    
    -- Сохраняем результат в storage для истории
    local history = storage_get("latency_history") or {}
    table.insert(history, {
        timestamp = os.time(),
        best_region = best_region.id,
        best_latency = best_region.latency_ms,
        avg_latency = avg_latency
    })
    
    -- Храним только последние 100 записей
    if #history > 100 then
        table.remove(history, 1)
    end
    storage_set("latency_history", history)
    
    -- Отправляем событие на фронтенд
    emit_event("voice-check-complete", {
        best_region = best_region.name,
        latency = best_region.latency_ms
    })
    
    return {
        success = true,
        latency = best_region.latency_ms,
        details = {
            gateway_available = true,
            gateway_url = gateway_url,
            best_region = best_region.name,
            best_region_id = best_region.id,
            best_latency_ms = best_region.latency_ms,
            average_latency_ms = avg_latency,
            regions_tested = #VOICE_REGIONS,
            regions_available = available_count,
            regions = results
        }
    }
end
