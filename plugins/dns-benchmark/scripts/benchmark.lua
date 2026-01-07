-- DNS Benchmark
-- Тестирует производительность DNS серверов через DoH (DNS over HTTPS)

-- Список DNS серверов для тестирования
local DNS_SERVERS = {
    {
        name = "Google",
        provider = "Google Public DNS",
        doh_url = "https://dns.google/dns-query",
        ip = "8.8.8.8",
        secondary_ip = "8.8.4.4"
    },
    {
        name = "Cloudflare",
        provider = "Cloudflare DNS",
        doh_url = "https://cloudflare-dns.com/dns-query",
        ip = "1.1.1.1",
        secondary_ip = "1.0.0.1"
    },
    {
        name = "Quad9",
        provider = "Quad9 (Security)",
        doh_url = "https://dns.quad9.net/dns-query",
        ip = "9.9.9.9",
        secondary_ip = "149.112.112.112"
    },
    {
        name = "OpenDNS",
        provider = "Cisco OpenDNS",
        doh_url = "https://doh.opendns.com/dns-query",
        ip = "208.67.222.222",
        secondary_ip = "208.67.220.220"
    },
    {
        name = "AdGuard",
        provider = "AdGuard DNS",
        doh_url = "https://dns.adguard-dns.com/dns-query",
        ip = "94.140.14.14",
        secondary_ip = "94.140.15.15"
    },
    {
        name = "NextDNS",
        provider = "NextDNS",
        doh_url = "https://dns.nextdns.io/dns-query",
        ip = "45.90.28.0",
        secondary_ip = "45.90.30.0"
    },
    {
        name = "Comodo",
        provider = "Comodo Secure DNS",
        doh_url = "https://doh.securedns.eu/dns-query",
        ip = "8.26.56.26",
        secondary_ip = "8.20.247.20"
    },
    {
        name = "CleanBrowsing",
        provider = "CleanBrowsing (Family)",
        doh_url = "https://doh.cleanbrowsing.org/doh/family-filter/",
        ip = "185.228.168.168",
        secondary_ip = "185.228.169.168"
    }
}

-- Тестовые домены для резолвинга
local TEST_DOMAINS = {
    "google.com",
    "youtube.com",
    "discord.com",
    "github.com",
    "cloudflare.com"
}

-- Количество тестов на сервер
local TESTS_PER_SERVER = 3

-- Тестирование одного DNS сервера через DoH
local function test_dns_server(dns_server)
    log_debug("Testing DNS server: " .. dns_server.name)
    
    local latencies = {}
    local successful_tests = 0
    local failed_tests = 0
    
    -- Выполняем несколько тестов
    for i = 1, TESTS_PER_SERVER do
        -- Выбираем случайный домен для теста
        local domain = TEST_DOMAINS[(i % #TEST_DOMAINS) + 1]
        
        -- Формируем DoH запрос (простой GET с параметром name)
        local url = dns_server.doh_url .. "?name=" .. domain .. "&type=A"
        
        local headers = {
            ["Accept"] = "application/dns-json"
        }
        
        local response = http_get(url, headers)
        
        if response.status == 200 then
            successful_tests = successful_tests + 1
            table.insert(latencies, response.latency_ms or 0)
            
            log_debug(string.format(
                "  Test %d: %s -> %dms",
                i,
                domain,
                response.latency_ms or 0
            ))
        else
            failed_tests = failed_tests + 1
            log_warn(string.format(
                "  Test %d failed: %s (status %d)",
                i,
                domain,
                response.status
            ))
        end
    end
    
    -- Вычисляем статистику
    local min_latency = math.huge
    local max_latency = 0
    local sum_latency = 0
    
    for _, lat in ipairs(latencies) do
        min_latency = math.min(min_latency, lat)
        max_latency = math.max(max_latency, lat)
        sum_latency = sum_latency + lat
    end
    
    local avg_latency = 0
    if #latencies > 0 then
        avg_latency = math.floor(sum_latency / #latencies)
        min_latency = math.floor(min_latency)
        max_latency = math.floor(max_latency)
    else
        min_latency = 0
    end
    
    local available = successful_tests > 0
    local reliability = math.floor((successful_tests / TESTS_PER_SERVER) * 100)
    
    return {
        name = dns_server.name,
        provider = dns_server.provider,
        ip = dns_server.ip,
        secondary_ip = dns_server.secondary_ip,
        doh_url = dns_server.doh_url,
        available = available,
        latency_ms = avg_latency,
        min_latency_ms = min_latency,
        max_latency_ms = max_latency,
        reliability_percent = reliability,
        successful_tests = successful_tests,
        failed_tests = failed_tests,
        total_tests = TESTS_PER_SERVER
    }
end

-- Сортировка результатов по задержке
local function sort_by_latency(results)
    table.sort(results, function(a, b)
        -- Недоступные серверы в конец
        if not a.available and b.available then return false end
        if a.available and not b.available then return true end
        if not a.available and not b.available then return false end
        
        -- Сортируем по задержке
        return a.latency_ms < b.latency_ms
    end)
    return results
end

-- Присвоение рейтинга
local function assign_ratings(results)
    for i, result in ipairs(results) do
        if not result.available then
            result.rating = "❌"
            result.rating_text = "Unavailable"
        elseif result.latency_ms < 30 then
            result.rating = "⭐⭐⭐⭐⭐"
            result.rating_text = "Excellent"
        elseif result.latency_ms < 50 then
            result.rating = "⭐⭐⭐⭐"
            result.rating_text = "Very Good"
        elseif result.latency_ms < 100 then
            result.rating = "⭐⭐⭐"
            result.rating_text = "Good"
        elseif result.latency_ms < 200 then
            result.rating = "⭐⭐"
            result.rating_text = "Fair"
        else
            result.rating = "⭐"
            result.rating_text = "Poor"
        end
        
        result.rank = i
    end
    return results
end

-- Основная функция проверки
function check()
    log_info("Starting DNS Benchmark...")
    log_info(string.format(
        "Testing %d DNS servers with %d tests each",
        #DNS_SERVERS,
        TESTS_PER_SERVER
    ))
    
    local results = {}
    local available_count = 0
    
    -- Тестируем каждый DNS сервер
    for _, dns_server in ipairs(DNS_SERVERS) do
        local result = test_dns_server(dns_server)
        table.insert(results, result)
        
        if result.available then
            available_count = available_count + 1
            log_info(string.format(
                "%s: %dms (min: %d, max: %d, reliability: %d%%)",
                result.name,
                result.latency_ms,
                result.min_latency_ms,
                result.max_latency_ms,
                result.reliability_percent
            ))
        else
            log_warn(string.format(
                "%s: unavailable (%d/%d tests failed)",
                result.name,
                result.failed_tests,
                result.total_tests
            ))
        end
    end
    
    -- Сортируем и присваиваем рейтинги
    results = sort_by_latency(results)
    results = assign_ratings(results)
    
    -- Определяем лучший сервер
    local best_server = nil
    for _, result in ipairs(results) do
        if result.available then
            best_server = result
            break
        end
    end
    
    if not best_server then
        log_error("No DNS servers available!")
        return {
            success = false,
            error = "All DNS servers are unavailable",
            details = {
                servers_tested = #DNS_SERVERS,
                servers_available = 0,
                servers = results
            }
        }
    end
    
    -- Вычисляем общую статистику
    local total_latency = 0
    for _, result in ipairs(results) do
        if result.available then
            total_latency = total_latency + result.latency_ms
        end
    end
    local avg_latency = math.floor(total_latency / available_count)
    
    log_info(string.format(
        "Benchmark complete! Best: %s (%dms), Average: %dms, Available: %d/%d",
        best_server.name,
        best_server.latency_ms,
        avg_latency,
        available_count,
        #DNS_SERVERS
    ))
    
    -- Сохраняем результат в storage
    local history = storage_get("benchmark_history") or {}
    table.insert(history, {
        timestamp = os.time(),
        best_server = best_server.name,
        best_latency = best_server.latency_ms,
        avg_latency = avg_latency,
        available_count = available_count
    })
    
    -- Храним только последние 50 записей
    if #history > 50 then
        table.remove(history, 1)
    end
    storage_set("benchmark_history", history)
    
    -- Отправляем событие на фронтенд
    emit_event("dns-benchmark-complete", {
        best_server = best_server.name,
        latency = best_server.latency_ms,
        servers_available = available_count
    })
    
    -- Формируем рекомендации
    local recommendations = {}
    if best_server.latency_ms < 30 then
        table.insert(recommendations, "Your DNS performance is excellent!")
    elseif best_server.latency_ms < 100 then
        table.insert(recommendations, string.format(
            "Consider using %s (%s) for best performance",
            best_server.name,
            best_server.ip
        ))
    else
        table.insert(recommendations, "DNS latency is high. Check your network connection.")
    end
    
    -- Добавляем рекомендацию по резервному DNS
    local second_best = nil
    for i, result in ipairs(results) do
        if i > 1 and result.available then
            second_best = result
            break
        end
    end
    
    if second_best then
        table.insert(recommendations, string.format(
            "Recommended backup DNS: %s (%s)",
            second_best.name,
            second_best.ip
        ))
    end
    
    return {
        success = true,
        latency = best_server.latency_ms,
        details = {
            best_server = best_server.name,
            best_server_ip = best_server.ip,
            best_latency_ms = best_server.latency_ms,
            average_latency_ms = avg_latency,
            servers_tested = #DNS_SERVERS,
            servers_available = available_count,
            servers = results,
            recommendations = recommendations,
            test_domains = TEST_DOMAINS,
            tests_per_server = TESTS_PER_SERVER
        }
    }
end
