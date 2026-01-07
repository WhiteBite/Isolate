# Аудит Rust Backend Core Модулей

**Дата:** 2025-01  
**Версия:** 1.0  
**Автор:** AI Audit Agent

## Обзор

Проанализированы следующие core модули:
- `strategy_engine.rs` — движок стратегий (2421 строк)
- `nodpi_engine.rs` — NoDPI движок для winws (1234 строки)
- `vless_engine.rs` — VLESS прокси через sing-box (1269 строк)
- `process_runner.rs` — запуск и управление процессами (1015 строк)
- `scoring.rs` — скоринг стратегий (350 строк)

**Примечание:** Файл `config_manager.rs` не существует. Вместо него используются `config.rs` и `config_updater.rs`.

---

## 🔴 Критичные проблемы

### 1. [strategy_engine.rs:1095-1120] Потенциальный race condition при concurrent запуске

**Проблема:** В методе `start_process_with_guard` проверка `processes.contains_key()` и последующая вставка не атомарны. Между проверкой и вставкой другой поток может вставить запись.

```rust
// Проверяем, не запущена ли уже стратегия
{
    let processes = self.processes.read().await;
    if processes.contains_key(&strategy.id) {
        return Err(...);
    }
}
// <-- RACE CONDITION: другой поток может вставить здесь

// ... spawn process ...

{
    let mut processes = self.processes.write().await;
    processes.insert(strategy.id.clone(), process);
}
```

**Решение:** Использовать `entry` API или держать write lock на всё время операции:
```rust
let mut processes = self.processes.write().await;
if processes.contains_key(&strategy.id) {
    return Err(...);
}
// spawn process
processes.insert(strategy.id.clone(), process);
```

---

### 2. [nodpi_engine.rs:680-720] Утечка WinDivert guard при ошибке spawn

**Проблема:** Если `global_runner::spawn()` возвращает ошибку, WinDivert guard освобождается через Drop, но между `WinDivertGuard::acquire()` и ошибкой spawn проходит время, в течение которого другие стратегии не могут запуститься.

```rust
let mut windivert_guard = WinDivertGuard::acquire()?;
// ... много кода ...
match global_runner::spawn(&process_id, process_config).await {
    Ok(_) => { ... }
    Err(e) => {
        // Guard освобождается через Drop, но задержка уже произошла
        error!("Failed to start Zapret strategy '{}': {}", strategy.id, e);
        Err(e)
    }
}
```

**Решение:** Проверять все preconditions (binary exists, etc.) ДО захвата guard:
```rust
// Verify binary exists BEFORE acquiring guard
if !tokio::fs::try_exists(&binary_path).await.unwrap_or(false) {
    return Err(IsolateError::Process(...));
}

// Now acquire guard
let mut windivert_guard = WinDivertGuard::acquire()?;
```

---

### 3. [vless_engine.rs:580-620] Отсутствие cleanup temp файлов при panic

**Проблема:** Временные конфиг-файлы sing-box создаются в `start_vless()`, но если процесс паникует или приложение крашится, файлы остаются в temp директории.

```rust
let config_path = get_temp_config_path(&config.id);
tokio::fs::write(&config_path, &config_json).await?;
// Если panic здесь или позже - файл останется навсегда
```

**Решение:** Использовать RAII wrapper для temp файлов:
```rust
struct TempConfigFile {
    path: PathBuf,
}

impl Drop for TempConfigFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

---

### 4. [process_runner.rs:180-220] Потеря stdout/stderr при быстром завершении

**Проблема:** Если процесс завершается очень быстро (до того как spawn'ятся задачи чтения stdout/stderr), вывод может быть потерян.

```rust
// Capture stdout
if let Some(stdout) = child.stdout.take() {
    tokio::spawn(async move {
        // Если процесс уже завершился, этот код может не успеть прочитать
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            // ...
        }
    });
}
```

**Решение:** Дождаться первого чтения или использовать синхронный буфер:
```rust
let stdout_ready = Arc::new(tokio::sync::Notify::new());
let stdout_ready_clone = stdout_ready.clone();

tokio::spawn(async move {
    stdout_ready_clone.notify_one();
    // ... read loop
});

stdout_ready.notified().await;
```

---

## 🟠 Важные улучшения

### 5. [strategy_engine.rs:250-280] Hardcoded константы портов

**Проблема:** Диапазон SOCKS портов захардкожен:
```rust
const SOCKS_PORT_START: u16 = 10800;
const MAX_SOCKS_PORTS: u16 = 100;
```

**Решение:** Вынести в конфигурацию или сделать настраиваемым:
```rust
impl StrategyEngine {
    pub fn with_port_range(start: u16, count: u16) -> Self { ... }
}
```

---

### 6. [nodpi_engine.rs:100-130] Дублирование логики build_winws_args

**Проблема:** Три функции делают почти одно и то же:
- `build_winws_args_from_template()`
- `build_winws_args_from_template_with_mode()`
- `build_winws_args_with_extra_hostlist()`

**Решение:** Объединить в одну функцию с builder pattern:
```rust
pub struct WinwsArgsBuilder<'a> {
    template: &'a LaunchTemplate,
    mode: Option<WinDivertMode>,
    extra_hostlist: Option<&'a Path>,
}

impl<'a> WinwsArgsBuilder<'a> {
    pub fn new(template: &'a LaunchTemplate) -> Self { ... }
    pub fn with_mode(mut self, mode: WinDivertMode) -> Self { ... }
    pub fn with_extra_hostlist(mut self, path: &'a Path) -> Self { ... }
    pub fn build(self) -> Vec<String> { ... }
}
```

---

### 7. [vless_engine.rs:300-350] Отсутствие валидации UUID в VLESS URL

**Проблема:** `parse_vless_url()` не валидирует UUID формат:
```rust
let uuid = authority_part[..at_pos].to_string();
if uuid.is_empty() {
    return Err(IsolateError::Config("Invalid VLESS URL: empty UUID".into()));
}
// UUID может быть любой строкой, не обязательно валидным UUID
```

**Решение:** Добавить валидацию UUID:
```rust
fn validate_uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
}

if !validate_uuid(&uuid) {
    return Err(IsolateError::Config("Invalid VLESS URL: malformed UUID".into()));
}
```

---

### 8. [scoring.rs:50-80] Magic numbers в формуле скоринга

**Проблема:** Веса захардкожены без объяснения:
```rust
const WEIGHT_SUCCESS_RATE: f64 = 0.5;
const WEIGHT_CRITICAL_SUCCESS: f64 = 0.3;
const WEIGHT_LATENCY: f64 = 0.15;
const WEIGHT_JITTER: f64 = 0.05;
```

**Решение:** Добавить документацию и/или сделать настраиваемыми:
```rust
/// Scoring weights configuration
/// 
/// Default weights are based on empirical testing:
/// - Success rate (50%): Primary indicator of strategy effectiveness
/// - Critical success (30%): Prioritizes strategies for important services
/// - Latency (15%): Lower latency is better for user experience
/// - Jitter (5%): Stability indicator
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub success_rate: f64,
    pub critical_success: f64,
    pub latency: f64,
    pub jitter: f64,
}
```

---

### 9. [process_runner.rs:350-400] Отсутствие retry логики при spawn

**Проблема:** Если spawn процесса не удался из-за временной проблемы (файл занят, etc.), нет retry:
```rust
let mut child = cmd.spawn().map_err(|e| {
    IsolateError::Process(format!("Failed to spawn {}: {}", config.binary.display(), e))
})?;
```

**Решение:** Добавить retry с exponential backoff:
```rust
let mut attempts = 0;
let max_attempts = 3;
let child = loop {
    match cmd.spawn() {
        Ok(c) => break c,
        Err(e) if attempts < max_attempts => {
            attempts += 1;
            tokio::time::sleep(Duration::from_millis(100 * attempts)).await;
        }
        Err(e) => return Err(IsolateError::Process(...)),
    }
};
```

---

### 10. [strategy_engine.rs:700-750] Неэффективная проверка is_running

**Проблема:** `is_running()` делает два отдельных lock'а:
```rust
pub async fn is_running(&self, strategy_id: &str) -> bool {
    // Проверяем Zapret стратегию
    {
        let zapret = self.zapret_strategy.read().await;
        if let Some(ref z) = *zapret {
            if z.strategy_id == strategy_id {
                return z.handle.is_running().await;
            }
        }
    }

    // Проверяем обычные процессы
    let processes = self.processes.read().await;
    processes.contains_key(strategy_id)
}
```

**Решение:** Объединить проверки или кэшировать состояние в отдельной структуре.

---

## 🟡 Рекомендации

### 11. [nodpi_engine.rs:50-70] Улучшить документацию WinDivertGuard

**Текущее состояние:** Документация есть, но не объясняет последствия неправильного использования.

**Рекомендация:** Добавить примеры и warnings:
```rust
/// RAII guard that automatically releases the WinDivert flag when dropped.
/// 
/// # Safety
/// 
/// **CRITICAL:** Only ONE WinDivert process can run at a time!
/// Running multiple WinDivert processes simultaneously WILL cause BSOD.
/// 
/// # Example
/// ```rust,ignore
/// // CORRECT: Guard is held for the duration of the process
/// let guard = WinDivertGuard::acquire()?;
/// let process = start_winws().await?;
/// // guard is dropped when process ends
/// 
/// // WRONG: Guard released before process ends
/// let guard = WinDivertGuard::acquire()?;
/// drop(guard); // DON'T DO THIS
/// let process = start_winws().await?; // BSOD risk!
/// ```
```

---

### 12. [vless_engine.rs:800-850] Добавить настраиваемый health check timeout

**Текущее состояние:** Health check использует фиксированный timeout:
```rust
let timeout_duration = Duration::from_secs(3);
```

**Рекомендация:** Сделать настраиваемым:
```rust
pub async fn health_check_socks_with_timeout(
    socks_port: u16,
    timeout: Duration,
) -> HealthCheckResult { ... }
```

---

### 13. [scoring.rs:100-150] Использовать weighted average для latency

**Текущее состояние:** Latency усредняется без учёта количества тестов:
```rust
let latencies: Vec<f64> = summaries
    .iter()
    .filter(|s| s.passed_tests > 0 && s.avg_latency_ms > 0.0)
    .map(|s| s.avg_latency_ms)
    .collect();
```

**Рекомендация:** Использовать weighted average:
```rust
let (total_weight, weighted_sum) = summaries
    .iter()
    .filter(|s| s.passed_tests > 0 && s.avg_latency_ms > 0.0)
    .fold((0u32, 0.0), |(w, sum), s| {
        (w + s.passed_tests, sum + s.avg_latency_ms * s.passed_tests as f64)
    });

let avg = if total_weight > 0 {
    weighted_sum / total_weight as f64
} else {
    0.0
};
```

---

### 14. [process_runner.rs:500-550] Добавить Job Objects для cleanup на Windows

**Текущее состояние:** Процессы убиваются индивидуально через taskkill.

**Рекомендация:** Использовать Job Objects для гарантированного cleanup всех дочерних процессов:
```rust
#[cfg(windows)]
fn create_job_object() -> Result<HANDLE> {
    // Create job object that kills all processes when closed
}
```

---

### 15. [strategy_engine.rs:1200-1250] Расширить структурированное логирование

**Текущее состояние:**
```rust
info!(strategy_id = %strategy.id, "Started global strategy");
```

**Рекомендация:** Добавить больше контекста:
```rust
info!(
    strategy_id = %strategy.id,
    engine = ?strategy.engine,
    mode = "global",
    windivert_mode = %windivert_mode,
    pid = ?process.pid(),
    "Strategy started"
);
```

---

## 🟢 Идеи нового функционала

### 16. Strategy Prewarming

**Описание:** Предварительный запуск стратегий в фоне для уменьшения времени переключения.

```rust
impl StrategyEngine {
    /// Prewarm a strategy by starting it in standby mode
    pub async fn prewarm(&self, strategy: &Strategy) -> Result<()> {
        // Start process but don't route traffic through it
    }
    
    /// Activate a prewarmed strategy
    pub async fn activate_prewarmed(&self, strategy_id: &str) -> Result<()> {
        // Switch traffic to prewarmed strategy
    }
}
```

---

### 17. Strategy Metrics Collection

**Описание:** Сбор метрик производительности стратегий в реальном времени.

```rust
#[derive(Debug, Clone)]
pub struct StrategyMetrics {
    pub strategy_id: String,
    pub uptime: Duration,
    pub bytes_processed: u64,
    pub connections_handled: u64,
    pub errors_count: u64,
    pub avg_latency_ms: f64,
}

impl StrategyEngine {
    pub async fn get_metrics(&self, strategy_id: &str) -> Option<StrategyMetrics>;
    pub async fn get_all_metrics(&self) -> Vec<StrategyMetrics>;
}
```

---

### 18. Automatic Strategy Failover

**Описание:** Автоматическое переключение на backup стратегию при сбое.

```rust
pub struct FailoverConfig {
    pub primary: String,
    pub backups: Vec<String>,
    pub health_check_interval: Duration,
    pub failover_threshold: u32,
}

impl StrategyEngine {
    pub async fn start_with_failover(&self, config: FailoverConfig) -> Result<()>;
}
```

---

### 19. Strategy Composition

**Описание:** Комбинирование нескольких стратегий для разных сервисов.

```rust
pub struct CompositeStrategy {
    pub id: String,
    pub rules: Vec<RoutingRule>,
}

pub struct RoutingRule {
    pub domains: Vec<String>,
    pub strategy_id: String,
}

// Пример: YouTube через Zapret, Discord через VLESS
```

---

### 20. Process Resource Limits

**Описание:** Ограничение ресурсов для запускаемых процессов.

```rust
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<u32>,
    pub max_connections: Option<u32>,
}

impl ProcessConfig {
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self;
}
```

---

## Сводка

| Категория | Количество |
|-----------|------------|
| 🔴 Критичные | 4 |
| 🟠 Важные | 6 |
| 🟡 Рекомендации | 5 |
| 🟢 Новый функционал | 5 |

### Приоритеты исправления

1. **Немедленно:** #1 (race condition), #2 (WinDivert guard leak)
2. **В ближайшем релизе:** #3 (temp files), #4 (stdout loss)
3. **Планово:** #5-#10 (улучшения)
4. **По возможности:** #11-#15 (рекомендации)
5. **Backlog:** #16-#20 (новый функционал)
