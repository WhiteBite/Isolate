# Техническая спецификация Isolate

> См. также: [PROMPT.md](./PROMPT.md) — детальный промт для AI-разработки
> См. также: [REFERENCES.md](./REFERENCES.md) — референсные проекты

## Системные требования

- **ОС**: Windows 10 (1903+) / Windows 11
- **Архитектура**: x64
- **RAM**: 100 MB
- **Диск**: 50 MB
- **Права**: Администратор (для WinDivert)

## Зависимости

### Внешние бинарники (bundled)

| Компонент | Версия | Назначение |
|-----------|--------|------------|
| winws.exe | latest | DPI-обход через WinDivert |
| WinDivert64.sys | 2.2 | Драйвер перехвата пакетов |
| sing-box.exe | 1.8+ | Прокси-ядро |

### Rust crates

```toml
[dependencies]
tauri = "2.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
rusqlite = "0.31"
reqwest = { version = "0.12", features = ["json"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Структура проекта

```
isolate/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/    # Tauri commands
│   │   ├── strategy/    # Strategy engine
│   │   ├── diagnostic/  # Diagnostic module
│   │   ├── process/     # Process manager
│   │   └── config/      # Config manager
│   ├── binaries/        # winws, sing-box
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                 # SvelteKit frontend
│   ├── lib/
│   │   ├── components/
│   │   └── stores/
│   ├── routes/
│   └── app.html
├── docs/                # Документация
├── strategies/          # YAML стратегии
└── hostlists/           # Списки доменов
```

## API (Tauri Commands)

### Стратегии

```typescript
// Получить список стратегий
invoke('get_strategies'): Promise<Strategy[]>

// Запустить тестирование
invoke('run_test', { mode: 'turbo' | 'deep' }): Promise<TestResult[]>

// Применить стратегию
invoke('apply_strategy', { id: string }): Promise<void>

// Остановить текущую стратегию
invoke('stop_strategy'): Promise<void>
```

### Диагностика

```typescript
// Проверить тип блокировки
invoke('diagnose', { domain: string }): Promise<DiagnosticResult>

// Проверить статус сервиса
invoke('check_service', { service: string }): Promise<ServiceStatus>
```

### Система

```typescript
// Panic reset
invoke('panic_reset'): Promise<void>

// Получить логи
invoke('get_logs', { lines: number }): Promise<string[]>

// Block/Unblock QUIC
invoke('set_quic_block', { enabled: boolean }): Promise<void>
```

## Формат стратегий (YAML)

```yaml
strategies:
  - id: "zapret_disorder"
    name: "Disorder + Split"
    description: "Фрагментация с перемешиванием пакетов"
    type: "driver_exclusive"
    engine: "winws"
    params:
      - "--wf-tcp=443"
      - "--dpi-desync=disorder2"
      - "--dpi-desync-split-pos=2"
    services:
      - discord
      - youtube
    priority: 10
    
  - id: "vless_proxy"
    name: "VLESS Proxy"
    description: "Проксирование через VLESS"
    type: "parallel_safe"
    engine: "singbox"
    config_template: "vless_template.json"
    priority: 5
```

## Формат конфигурации (SQLite)

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE custom_services (
    id INTEGER PRIMARY KEY,
    domain TEXT NOT NULL,
    ip_v4 TEXT,
    ip_v6 TEXT,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE strategy_history (
    id INTEGER PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    success BOOLEAN,
    latency_ms INTEGER,
    tested_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Безопасность

### Проверка бинарников

```rust
const WINWS_SHA256: &str = "abc123...";
const SINGBOX_SHA256: &str = "def456...";

fn verify_binary(path: &Path, expected_hash: &str) -> Result<bool> {
    let hash = sha256_file(path)?;
    Ok(hash == expected_hash)
}
```

### Права доступа

- Приложение запрашивает UAC при старте
- WinDivert требует SeLoadDriverPrivilege
- Логи не содержат IP-адресов пользователя

## Обработка ошибок

```rust
#[derive(Debug, thiserror::Error)]
pub enum IsolateError {
    #[error("WinDivert driver not loaded")]
    DriverNotLoaded,
    
    #[error("Strategy timeout after {0}ms")]
    StrategyTimeout(u32),
    
    #[error("Process failed: {0}")]
    ProcessError(String),
    
    #[error("Network reset failed")]
    ResetFailed,
}
```
