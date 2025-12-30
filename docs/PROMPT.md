# AI Development Prompt для Isolate

Этот промт используется для генерации кода проекта с помощью AI-агентов.

---

## Prompt

Ты — опытный системный разработчик (Rust + Tauri + сетевой стек).
Твоя задача — спроектировать и реализовать desktop‑приложение **Isolate** для Windows, которое:

- автоматически **подбирает стратегию обхода DPI** для пользователя;
- использует внешние бинарники (например, zapret/winws/WinDivert, sing-box/xray-core);
- тестирует несколько сложных стратегий **параллельно через SOCKS**, а потом включает лучшую в **глобальном режиме**;
- даёт удобный GUI (иконка в трее, кнопка «Оптимизировать», настройки наподобие Throne);
- умеет работать с VLESS (через sing-box/xray).

---

## 0. Общие требования и ограничения

1. **Основной таргет:** Windows 10/11 x64.
2. **Ядро:** Rust (stable), async‑рантайм — Tokio.
3. **GUI:** Tauri (Rust backend + фронт на TypeScript/React или Svelte; конкретный фреймворк выбери сам, но опиши структуру).
4. **Внешние бинарники (используем как «чёрные ящики»):**
   - zapret / winws / WinDivert‑обвязка — для DPI‑обхода;
   - sing-box или xray-core — для VLESS/прокси.
5. **Важное архитектурное требование:**
   - Стратегии/сервисы/тесты должны описываться **во внешних конфиг‑файлах** (JSON/YAML), чтобы можно было обновлять их без перекомпиляции.
6. **Нельзя полагаться на наличие администратора всегда:**
   - часть функционала (тесты через локальный SOCKS, работа UI) должна работать и без admin;
   - включение глобального DPI‑обхода (WinDivert/iptables) естественно требует повышенных прав — это нужно корректно обрабатывать и показывать в UI.

---

## 1. Структура репозитория и общая архитектура

```
isolate/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                 # Точка входа Tauri
│   │   ├── lib.rs
│   │   └── core/
│   │       ├── mod.rs
│   │       ├── orchestrator.rs     # Управление сценариями работы
│   │       ├── diagnostics.rs      # Модуль DPI‑диагностики
│   │       ├── strategy_engine.rs  # Запуск/остановка стратегий
│   │       ├── test_engine.rs      # Запуск тестов сервисов
│   │       ├── models.rs           # Типы данных
│   │       ├── storage.rs          # Кэш, настройки пользователя
│   │       ├── telemetry.rs        # Опциональная телеметрия
│   │       ├── process_runner.rs   # Запуск внешних процессов
│   │       └── env_info.rs         # Сбор информации об окружении
│   ├── binaries/                   # winws, sing-box
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                            # SvelteKit frontend
│   ├── lib/
│   │   ├── components/
│   │   └── stores/
│   ├── routes/
│   └── app.html
├── configs/
│   ├── strategies/                 # YAML описания стратегий
│   └── services/                   # YAML описания сервисов
├── data/
│   ├── cache/                      # Кэш подобранных стратегий
│   └── logs/                       # Логи
├── thirdparty/                     # Референсные проекты
└── docs/                           # Документация
```

---

## 2. Модель данных

### 2.1. Strategy

```yaml
id: "zapret_sni_frag_alt3"
family: "SNI_FRAG"           # DNS_BYPASS, SNI_FRAG, TLS_FRAG, VLESS, HYBRID
engine: "zapret"             # zapret, sing-box, xray, hybrid

mode_capabilities:
  supports_socks: true
  supports_global: true

socks_template:
  binary: "winws.exe"
  args:
    - "--wf-tcp=443"
    - "--dpi-desync=disorder2"
    - "--socks-port={port}"
  env: {}
  log_file: "winws_{id}.log"

global_template:
  binary: "winws.exe"
  args:
    - "--wf-tcp=443"
    - "--dpi-desync=disorder2"
  requires_admin: true

requirements:
  min_rights: "user"         # user или admin
  os: ["windows"]
  binaries: ["winws.exe", "WinDivert64.sys"]

weight_hint: 10
```

### 2.2. Service & Test

```yaml
id: "discord"
name: "Discord"
enabled_by_default: true
critical: true

tests:
  - id: "discord_api"
    type: "HttpsGetTest"
    url: "https://discord.com/api/v10/gateway"
    timeout_ms: 5000
    success_criteria:
      status_codes: [200]
      min_body_size: 10

  - id: "discord_ws"
    type: "WebSocketTest"
    url: "wss://gateway.discord.gg/?v=10&encoding=json"
    timeout_ms: 5000

  - id: "discord_tcp"
    type: "TcpConnectTest"
    host: "discord.com"
    port: 443
    timeout_ms: 3000
```

### 2.3. Results

```rust
struct TestResult {
    test_id: String,
    success: bool,
    latency_ms: Option<u32>,
    error_type: Option<ErrorType>, // Dns, Tcp, Tls, Http, Timeout
}

struct ServiceTestSummary {
    service_id: String,
    total_tests: u32,
    success_rate: f64,
    avg_latency_ms: f64,
    errors: Vec<ErrorType>,
}

struct StrategyScore {
    strategy_id: String,
    success_rate: f64,
    critical_success_rate: f64,
    latency_avg: f64,
    latency_jitter: f64,
    score: f64, // Итоговый балл
}
```

**Формула score:**
```
score = (success_rate * 0.5) + (critical_success_rate * 0.3) + (1.0 - normalized_latency) * 0.15 + (1.0 - jitter) * 0.05
```

---

## 3. DPI-диагностика

### Типы блокировок

| Тип | Признаки | Рекомендуемые семейства |
|-----|----------|------------------------|
| DnsBlock | NXDOMAIN, SERVFAIL, подмена IP | DNS_BYPASS, VLESS |
| SniTlsBlock | TCP OK, TLS RST/timeout | SNI_FRAG, TLS_FRAG |
| IpBlock | TCP timeout/RST | VLESS, HYBRID |
| NoBlock | Всё работает | — |

### Алгоритм

```rust
async fn diagnose(services: &[Service]) -> DpiProfile {
    // 1. DNS resolve
    // 2. TCP connect :443
    // 3. TLS handshake (без полного HTTP)
    // Классифицировать по паттерну ошибок
}
```

---

## 4. Strategy Engine

### SOCKS-режим (параллельные тесты)

**КРИТИЧНО:** Только для `parallel_safe` стратегий (VLESS/Sing-box).
Драйверные стратегии (zapret/winws) — строго последовательно!

```rust
// Параллельный запуск VLESS стратегий
async fn start_socks_instances(strategies: Vec<Strategy>) -> HashMap<String, SocketAddr> {
    let parallel_safe: Vec<_> = strategies.iter()
        .filter(|s| s.mode_capabilities.supports_socks && s.family != "driver_exclusive")
        .collect();
    
    // Запустить каждую на своём порту
    // Вернуть map: strategy_id -> 127.0.0.1:port
}

// Последовательный тест драйверных стратегий
async fn test_driver_strategies(strategies: Vec<Strategy>) -> Vec<StrategyScore> {
    for strategy in strategies.filter(|s| s.engine == "zapret") {
        start_strategy(&strategy).await;
        let score = run_tests_direct().await; // Без SOCKS
        stop_strategy(&strategy).await;
        results.push(score);
    }
}
```

### GLOBAL-режим

```rust
async fn apply_global(strategy: &Strategy) -> Result<()> {
    stop_all_processes().await?;
    
    if strategy.global_template.requires_admin {
        request_elevation()?;
    }
    
    start_process(&strategy.global_template).await?;
    verify_connectivity().await?;
    save_to_cache(&strategy).await?;
    
    Ok(())
}
```

---

## 5. Test Engine

### Поддерживаемые тесты

- `HttpsGetTest` — GET запрос, проверка статуса и размера
- `HttpsHeadTest` — HEAD запрос
- `WebSocketTest` — WSS подключение
- `TcpConnectTest` — TCP handshake
- `DnsTest` — DNS resolve

### Прокси-поддержка

```rust
async fn run_test(test: &TestDefinition, proxy: Option<ProxyConfig>) -> TestResult {
    let client = match proxy {
        Some(p) => reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(format!("socks5://127.0.0.1:{}", p.port))?)
            .build()?,
        None => reqwest::Client::new(),
    };
    // ...
}
```

---

## 6. Алгоритм оптимизации (Orchestrator)

```
┌─────────────────────────────────────────────────────────────┐
│                    START OPTIMIZATION                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 1: Check Cache                                         │
│ - Build env_key (ASN + Country + SSID)                      │
│ - If cached strategy exists → try it → if OK → DONE         │
└─────────────────────────────────────────────────────────────┘
                              │ (cache miss or failed)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 2: DPI Diagnostics                                     │
│ - Run quick tests without bypass                            │
│ - Classify: DnsBlock / SniTlsBlock / IpBlock / NoBlock      │
│ - Get candidate_families                                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 3: Select Candidates                                   │
│ - Filter by family, OS, socks_support                       │
│ - Sort by weight_hint                                       │
│ - Limit to max 10-20                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 4: Parallel SOCKS Tests (VLESS only)                   │
│ - Start SOCKS instances on different ports                  │
│ - Run test burst through each proxy                         │
│ - Collect StrategyScore                                     │
│ - Stop SOCKS instances                                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 4b: Sequential Driver Tests (Zapret)                   │
│ - For each zapret strategy:                                 │
│   - Start → Test → Stop → Next                              │
│ - Timeout: 2-3 sec per strategy                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 5: Select Best Strategy                                │
│ - Filter: success_rate >= 0.8                               │
│ - Sort by score DESC                                        │
│ - Pick top                                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 6: Apply Global                                        │
│ - Start strategy in global mode                             │
│ - Verify connectivity                                       │
│ - Save to cache                                             │
│ - (Optional) Send telemetry                                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                           DONE
```

---

## 7. UI Screens

### Main Screen
- Статус: «Обход включен/выключен»
- Текущая стратегия
- Кнопка «Оптимизировать»
- Список сервисов со статусами

### Optimization Screen
- Прогресс-бар
- Шаги: Диагностика → Запуск → Тестирование → Применение
- Кнопка отмены

### Settings
- Автоподбор при старте
- Выбор сервисов (критичные/некритичные)
- Телеметрия вкл/выкл
- VLESS конфигурация
- Block QUIC

### System Tray
- Иконка статуса
- «Открыть Isolate»
- «Оптимизировать»
- «Вкл/Выкл обход»
- «Panic Reset»

---

## 8. Этапы разработки

### Этап 1 — Core без GUI
- [ ] Модели данных
- [ ] Process runner
- [ ] Diagnostics
- [ ] Strategy Engine (SOCKS + GLOBAL)
- [ ] Test Engine
- [ ] Orchestrator (CLI)

### Этап 2 — Tauri интеграция
- [ ] Tauri commands
- [ ] Базовый UI с кнопкой «Оптимизировать»

### Этап 3 — Полный UI
- [ ] Все экраны
- [ ] System tray
- [ ] Настройки

### Этап 4 — Реальные бинарники
- [ ] Интеграция winws/zapret
- [ ] Интеграция sing-box
- [ ] Конфиги стратегий

### Этап 5 — Polish
- [ ] Кэш
- [ ] Телеметрия
- [ ] Panic Button
- [ ] Block QUIC
