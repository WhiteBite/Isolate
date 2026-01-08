# Backend Commands для интеграции

Документация всех Tauri команд, необходимых для интеграции frontend stores с Rust backend.

> **Статус**: 🔴 Не реализовано | 🟡 Частично | 🟢 Готово

---

## Dashboard

Файл store: `src/lib/stores/dashboard.ts`

### get_protection_status 🔴

**Описание**: Возвращает текущий статус защиты и режим работы

**Параметры**: нет

**Возвращает**:
```typescript
interface ProtectionStatusResponse {
  status: 'protected' | 'bypassing' | 'issues' | 'disabled';
  mode: 'auto' | 'tun' | 'proxy';
  activeStrategyId?: string;
  uptime: number; // секунды с момента активации
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn get_protection_status(
    state: State<'_, Arc<AppState>>
) -> Result<ProtectionStatusResponse, String>
```

---

### get_traffic_stats 🔴

**Описание**: Возвращает статистику трафика (текущая скорость и история)

**Параметры**: нет

**Возвращает**:
```typescript
interface TrafficStatsResponse {
  currentDownload: number; // bytes/sec
  currentUpload: number;   // bytes/sec
  totalDownload: number;   // bytes
  totalUpload: number;     // bytes
  history: TrafficPoint[]; // последние N точек
}

interface TrafficPoint {
  timestamp: number;
  download: number;
  upload: number;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn get_traffic_stats(
    state: State<'_, Arc<AppState>>
) -> Result<TrafficStatsResponse, String>
```

---

### get_active_connections 🔴

**Описание**: Возвращает список активных соединений

**Параметры**: нет

**Возвращает**:
```typescript
interface ActiveConnectionsResponse {
  connections: ActiveConnection[];
  totalCount: number;
}

interface ActiveConnection {
  domain: string;
  method: 'direct' | 'strategy' | 'proxy' | 'vless';
  strategyName?: string;
  proxyName?: string;
  bytesTransferred: number;
  duration: number;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn get_active_connections(
    state: State<'_, Arc<AppState>>
) -> Result<ActiveConnectionsResponse, String>
```

---

### get_issues 🔴

**Описание**: Возвращает список текущих проблем

**Параметры**: нет

**Возвращает**:
```typescript
interface IssuesResponse {
  issues: Issue[];
}

interface Issue {
  id: string;
  type: 'service_blocked' | 'strategy_failed' | 'connection_error';
  message: string;
  serviceId?: string;
  timestamp: number;
  canAutoFix: boolean;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn get_issues(
    state: State<'_, Arc<AppState>>
) -> Result<IssuesResponse, String>
```

---

### set_operation_mode 🔴

**Описание**: Устанавливает режим работы приложения

**Параметры**:
- `mode: 'auto' | 'tun' | 'proxy'` — новый режим работы

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn set_operation_mode(
    state: State<'_, Arc<AppState>>,
    mode: OperationMode
) -> Result<(), String>
```

---

### fix_issue 🔴

**Описание**: Автоматически исправляет проблему

**Параметры**:
- `issue_id: String` — ID проблемы для исправления

**Возвращает**: `Result<bool, String>` — успешность исправления

**Rust signature**:
```rust
#[tauri::command]
pub async fn fix_issue(
    state: State<'_, Arc<AppState>>,
    issue_id: String
) -> Result<bool, String>
```

---

## Library

Файл store: `src/lib/stores/library.svelte.ts`

### get_library_rules 🔴

**Описание**: Загружает все правила библиотеки сервисов

**Параметры**: нет

**Возвращает**:
```typescript
interface ServiceRule {
  id: string;
  name: string;
  domain: string;
  icon: string;
  category: string;
  status: 'accessible' | 'blocked' | 'unknown' | 'checking';
  currentMethod: AccessMethod;
  availableMethods: AccessMethod[];
  isCustom: boolean;
  lastChecked?: number;
  ping?: number;
}

interface AccessMethod {
  type: 'direct' | 'auto' | 'strategy' | 'vless' | 'proxy' | 'tor';
  strategyId?: string;
  strategyName?: string;
  proxyId?: string;
  proxyName?: string;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn get_library_rules(
    state: State<'_, Arc<AppState>>
) -> Result<Vec<ServiceRule>, String>
```

---

### save_library_rule 🔴

**Описание**: Сохраняет правило (создание или обновление)

**Параметры**:
- `rule: ServiceRule` — правило для сохранения

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn save_library_rule(
    state: State<'_, Arc<AppState>>,
    rule: ServiceRule
) -> Result<(), String>
```

---

### delete_library_rule 🔴

**Описание**: Удаляет правило из библиотеки

**Параметры**:
- `rule_id: String` — ID правила для удаления

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn delete_library_rule(
    state: State<'_, Arc<AppState>>,
    rule_id: String
) -> Result<(), String>
```

---

### check_service_availability 🔴

**Описание**: Проверяет доступность сервиса (пинг/блокировка)

**Параметры**:
- `domain: String` — домен для проверки

**Возвращает**:
```typescript
interface ServiceCheckResult {
  status: 'accessible' | 'blocked' | 'unknown';
  ping?: number; // ms
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn check_service_availability(
    state: State<'_, Arc<AppState>>,
    domain: String
) -> Result<ServiceCheckResult, String>
```

---

### set_rule_access_method 🔴

**Описание**: Устанавливает метод доступа для правила

**Параметры**:
- `rule_id: String` — ID правила
- `method: AccessMethod` — новый метод доступа

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn set_rule_access_method(
    state: State<'_, Arc<AppState>>,
    rule_id: String,
    method: AccessMethod
) -> Result<(), String>
```

---

## AI Pilot

Файл store: `src/lib/stores/aiPilot.svelte.ts`

### start_ai_pilot 🔴

**Описание**: Запускает фоновую задачу автоматической оптимизации стратегий

**Параметры**:
- `interval: u32` — интервал проверки в минутах (30, 60, 120)

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn start_ai_pilot(
    state: State<'_, Arc<AppState>>,
    interval: u32
) -> Result<(), String>
```

**Логика**:
- Запускает фоновую Tokio задачу
- Периодически проверяет качество текущих стратегий
- При обнаружении проблем автоматически переключает на лучшую стратегию
- Записывает действия в историю

---

### stop_ai_pilot 🔴

**Описание**: Останавливает фоновую задачу оптимизации

**Параметры**: нет

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn stop_ai_pilot(
    state: State<'_, Arc<AppState>>
) -> Result<(), String>
```

---

### get_ai_pilot_status 🔴

**Описание**: Получает текущий статус AI Pilot

**Параметры**: нет

**Возвращает**:
```typescript
interface AiPilotStatus {
  enabled: boolean;
  interval: number;
  last_check: string | null; // ISO 8601
  is_checking: boolean;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn get_ai_pilot_status(
    state: State<'_, Arc<AppState>>
) -> Result<AiPilotStatus, String>
```

---

### get_ai_pilot_history 🔴

**Описание**: Возвращает историю действий AI Pilot

**Параметры**:
- `limit: Option<u32>` — максимальное количество записей (default: 50)

**Возвращает**:
```typescript
interface AiPilotAction {
  id: string;
  timestamp: string; // ISO 8601
  service_id: string;
  service_name: string;
  old_strategy: string;
  new_strategy: string;
  reason: string;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn get_ai_pilot_history(
    state: State<'_, Arc<AppState>>,
    limit: Option<u32>
) -> Result<Vec<AiPilotAction>, String>
```

---

### undo_ai_pilot_action 🔴

**Описание**: Откатывает действие AI Pilot (восстанавливает предыдущую стратегию)

**Параметры**:
- `action_id: String` — ID действия для отката

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn undo_ai_pilot_action(
    state: State<'_, Arc<AppState>>,
    action_id: String
) -> Result<(), String>
```

---

## Game Mode

Файл store: `src/lib/stores/gameMode.svelte.ts`

### detect_running_games 🔴

**Описание**: Сканирует запущенные процессы и возвращает список обнаруженных игр

**Параметры**: нет

**Возвращает**:
```typescript
interface DetectedGame {
  name: string;
  processName: string;
  pid?: number;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn detect_running_games(
    state: State<'_, Arc<AppState>>
) -> Result<Vec<DetectedGame>, String>
```

**Логика**:
- Использует Windows API для получения списка процессов
- Сравнивает с известным списком игр (KNOWN_GAMES)
- Возвращает найденные совпадения

---

### start_game_monitor 🔴

**Описание**: Запускает фоновый мониторинг процессов для автодетекта игр

**Параметры**: нет

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn start_game_monitor(
    state: State<'_, Arc<AppState>>
) -> Result<(), String>
```

**События**:
- `game-detected` — при обнаружении игры (payload: `DetectedGame`)
- `game-closed` — при закрытии игры (payload: `{ name: string }`)

**Логика**:
- Запускает фоновую задачу с интервалом ~5 сек
- При обнаружении игры эмитит событие `game-detected`
- При закрытии игры эмитит событие `game-closed`

---

### stop_game_monitor 🔴

**Описание**: Останавливает фоновый мониторинг процессов

**Параметры**: нет

**Возвращает**: `Result<(), String>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn stop_game_monitor(
    state: State<'_, Arc<AppState>>
) -> Result<(), String>
```

---

### get_game_mode_status 🔴

**Описание**: Возвращает текущий статус игрового режима

**Параметры**: нет

**Возвращает**:
```typescript
interface GameModeStatus {
  isMonitoring: boolean;
  detectedGame: string | null;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub fn get_game_mode_status(
    state: State<'_, Arc<AppState>>
) -> Result<GameModeStatus, String>
```

---

## Troubleshooter

Файл: `src-tauri/src/commands/troubleshoot.rs`

### troubleshoot_service 🟢

**Описание**: Запускает диагностику для сервиса — тестирует все подходящие стратегии и находит лучшую

**Параметры**:
- `service_id: String` — ID сервиса для диагностики

**Возвращает**:
```typescript
interface TroubleshootResult {
  service_id: string;
  strategies_tested: TroubleshootStrategyResult[];
  best_strategy_id: string | null;
  best_strategy_name: string | null;
  best_latency_ms: number | null;
}

interface TroubleshootStrategyResult {
  strategy_id: string;
  strategy_name: string;
  success: boolean;
  latency_ms: number | null;
  error: string | null;
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn troubleshoot_service(
    window: Window,
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<TroubleshootResult, IsolateError>
```

**События**:
- `troubleshoot:progress` — прогресс тестирования каждой стратегии
- `troubleshoot:strategy_result` — результат тестирования стратегии
- `troubleshoot:complete` — финальный результат

---

### apply_troubleshoot_result 🟢

**Описание**: Применяет результат диагностики — устанавливает лучшую стратегию для сервиса

**Параметры**:
- `service_id: String` — ID сервиса
- `strategy_id: String` — ID стратегии для применения

**Возвращает**: `Result<(), IsolateError>`

**Rust signature**:
```rust
#[tauri::command]
pub async fn apply_troubleshoot_result(
    state: State<'_, Arc<AppState>>,
    service_id: String,
    strategy_id: String,
) -> Result<(), IsolateError>
```

---

### get_troubleshoot_problems 🟢

**Описание**: Возвращает список сервисов для выбора в troubleshooter

**Параметры**: нет

**Возвращает**:
```typescript
interface ServiceProblem {
  service_id: string;
  service_name: string;
  category: string; // video, social, gaming, other
}
```

**Rust signature**:
```rust
#[tauri::command]
pub async fn get_troubleshoot_problems(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ServiceProblem>, IsolateError>
```

---

## Общие команды

### is_backend_ready 🟢

**Описание**: Проверяет готовность AppState (для решения race condition при старте)

**Параметры**: нет

**Возвращает**: `bool`

**Rust signature**:
```rust
#[tauri::command]
pub fn is_backend_ready(app: AppHandle) -> bool {
    app.try_state::<Arc<AppState>>().is_some()
}
```

**Важно**: Эта команда НЕ требует State и работает сразу после запуска приложения.

---

## Регистрация команд

Все команды должны быть зарегистрированы в `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // Общие
    is_backend_ready,
    
    // Dashboard
    get_protection_status,
    get_traffic_stats,
    get_active_connections,
    get_issues,
    set_operation_mode,
    fix_issue,
    
    // Library
    get_library_rules,
    save_library_rule,
    delete_library_rule,
    check_service_availability,
    set_rule_access_method,
    
    // AI Pilot
    start_ai_pilot,
    stop_ai_pilot,
    get_ai_pilot_status,
    get_ai_pilot_history,
    undo_ai_pilot_action,
    
    // Game Mode
    detect_running_games,
    start_game_monitor,
    stop_game_monitor,
    get_game_mode_status,
    
    // Troubleshooter
    troubleshoot_service,
    apply_troubleshoot_result,
    get_troubleshoot_problems,
])
```

---

## Rust Types (для копирования)

```rust
// Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtectionStatus {
    #[serde(rename = "protected")]
    Protected,
    #[serde(rename = "bypassing")]
    Bypassing,
    #[serde(rename = "issues")]
    Issues,
    #[serde(rename = "disabled")]
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationMode {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "tun")]
    Tun,
    #[serde(rename = "proxy")]
    Proxy,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProtectionStatusResponse {
    pub status: ProtectionStatus,
    pub mode: OperationMode,
    pub active_strategy_id: Option<String>,
    pub uptime: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficPoint {
    pub timestamp: u64,
    pub download: u64,
    pub upload: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficStatsResponse {
    pub current_download: u64,
    pub current_upload: u64,
    pub total_download: u64,
    pub total_upload: u64,
    pub history: Vec<TrafficPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub id: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub message: String,
    pub service_id: Option<String>,
    pub timestamp: u64,
    pub can_auto_fix: bool,
}

// Library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessMethod {
    #[serde(rename = "type")]
    pub method_type: String,
    pub strategy_id: Option<String>,
    pub strategy_name: Option<String>,
    pub proxy_id: Option<String>,
    pub proxy_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRule {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub icon: String,
    pub category: String,
    pub status: String,
    pub current_method: AccessMethod,
    pub available_methods: Vec<AccessMethod>,
    pub is_custom: bool,
    pub last_checked: Option<u64>,
    pub ping: Option<u32>,
}

// AI Pilot
#[derive(Debug, Clone, Serialize)]
pub struct AiPilotStatus {
    pub enabled: bool,
    pub interval: u32,
    pub last_check: Option<String>,
    pub is_checking: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiPilotAction {
    pub id: String,
    pub timestamp: String,
    pub service_id: String,
    pub service_name: String,
    pub old_strategy: String,
    pub new_strategy: String,
    pub reason: String,
}

// Game Mode
#[derive(Debug, Clone, Serialize)]
pub struct DetectedGame {
    pub name: String,
    pub process_name: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameModeStatus {
    pub is_monitoring: bool,
    pub detected_game: Option<String>,
}
```

---

## Приоритет реализации

1. **Высокий** (критично для работы):
   - `get_protection_status`
   - `get_library_rules`
   - `check_service_availability`

2. **Средний** (улучшение UX):
   - `get_traffic_stats`
   - `get_active_connections`
   - `detect_running_games`
   - `start_game_monitor`

3. **Низкий** (дополнительные фичи):
   - AI Pilot команды
   - `fix_issue`
   - `set_operation_mode`
