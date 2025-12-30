# Isolate — Полный список задач для Production

## ✅ ВЫПОЛНЕНО

### Этап 1: Модели и парсеры
- [x] **1.1** Расширение `models.rs` — ProxyProtocol enum, ProxyConfig, DomainRoute, AppRoute
- [x] **1.2** Создание `proxy_parser.rs` — парсинг всех типов прокси URL (VLESS, VMess, SS, Trojan, TUIC, Hysteria, SOCKS, HTTP)
- [x] **1.3** Создание `singbox_config.rs` — генерация JSON конфигов sing-box для всех протоколов

### Этап 2: Routing модули
- [x] **2.1** Создание `domain_routing.rs` — DomainRouter с CRUD и генерацией правил
- [x] **2.2** Создание `app_routing.rs` — AppRouter с CRUD, сканированием приложений Windows, генерацией правил
- [x] **2.3** Обновление `storage.rs` — таблицы domain_routes, app_routes, методы CRUD

### Этап 3: UI страницы
- [x] **3.1** Layout с боковой навигацией (`+layout.svelte`)
- [x] **3.2** Dashboard (`+page.svelte`) — Status Card, Quick Actions, Active Proxies, Recent Activity
- [x] **3.3** Proxies страница (`proxies/+page.svelte`) — таблица, модалы добавления/редактирования
- [x] **3.4** Routing страница (`routing/+page.svelte`) — табы per-domain/per-app
- [x] **3.5** Strategies страница (`strategies/+page.svelte`) — фильтры, карточки стратегий
- [x] **3.6** Testing страница (`testing/+page.svelte`) — UI для тестирования
- [x] **3.7** Settings страница (`settings/+page.svelte`) — секции настроек
- [x] **3.8** Logs страница (`logs/+page.svelte`) — фильтры, виртуализация

### Этап 4: Tauri Commands
- [x] **4.1** Proxy Commands — get_proxies, add_proxy, update_proxy, delete_proxy, apply_proxy, test_proxy, import_proxy_url, import_subscription
- [x] **4.2** Routing Commands — get_domain_routes, add_domain_route, remove_domain_route, get_app_routes, add_app_route, remove_app_route, get_installed_apps
- [x] **4.3** Testing Commands — run_tests, cancel_tests + события test:progress, test:result, test:complete
- [x] **4.4** Log Commands — get_logs (с фильтрацией), clear_logs, export_logs + событие log:entry

### Backend (уже было)
- [x] Strategy Engine, Orchestrator, Test Engine, Diagnostics
- [x] Storage (SQLite), Config Manager
- [x] VLESS Engine, Sing-box Manager
- [x] QUIC Blocker, Hostlists
- [x] System Tray, Tauri Commands (базовые)
- [x] Log Capture System (`log_capture.rs`) — захват логов в память с фильтрацией

---

## 🔴 КРИТИЧНО — Нужно сделать

### Этап 1: Интеграция Sing-box с routing

#### 1.1 Обновить SingboxManager
```rust
// src-tauri/src/core/singbox_manager.rs — ОБНОВИТЬ:
start_with_routing(
    proxy: &ProxyConfig,
    domain_routes: &[DomainRoute],
    app_routes: &[AppRoute],
    socks_port: u16
) -> Result<SingboxInstance>
```

---

## 🟠 ВЫСОКИЙ ПРИОРИТЕТ

### Этап 2: System Tray улучшения

#### 2.1 Обновить меню трея
```rust
// src-tauri/src/tray.rs — ОБНОВИТЬ:
- Статус: "Активен: {strategy_name}" / "Неактивен"
- Разделитель
- Открыть Isolate
- Оптимизировать (Turbo)
- Оптимизировать (Deep)
- Разделитель
- Включить/Отключить обход
- Разделитель
- Panic Reset (красный)
- Разделитель
- Настройки
- Выход
```

#### 2.2 Динамические иконки трея
- Зеленая: стратегия активна
- Серая: неактивна
- Желтая: оптимизация
- Красная: ошибка

### Этап 3: Onboarding улучшения

#### 3.1 Обновить `src/routes/onboarding/+page.svelte`
- Шаг 1: Добро пожаловать
- Шаг 2: Выбор сервисов (чекбоксы)
- Шаг 3: Диагностика (автоматическая)
- Шаг 4: Первая оптимизация
- Шаг 5: Завершение

### Этап 8: Monitor (фоновый мониторинг)

#### 8.1 Реализовать `src-tauri/src/core/monitor.rs`
```rust
pub struct Monitor {
    interval: Duration,
    strategy_engine: SharedStrategyEngine,
    storage: Arc<Storage>,
}

impl Monitor {
    pub async fn start(&self) -> Result<()>
    pub async fn stop(&self)
    async fn check_strategy_health(&self) -> Result<bool>
    async fn on_degradation(&self)
}
```

### Этап 9: Telemetry (opt-in)

#### 9.1 Реализовать `src-tauri/src/core/telemetry.rs`
```rust
pub struct TelemetryService {
    enabled: bool,
    endpoint: String,
    batch: Vec<TelemetryEvent>,
}

impl TelemetryService {
    pub async fn report_optimization(&self, result: &OptimizationResult)
    pub async fn report_strategy_usage(&self, strategy_id: &str, success: bool)
    async fn flush(&self)
}
```

---

## 🟡 СРЕДНИЙ ПРИОРИТЕТ

### Этап 10: Автообновление

#### 10.1 Конфигурация в `tauri.conf.json`
```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/user/isolate/releases/latest/download/latest.json"
      ],
      "pubkey": "..."
    }
  }
}
```

#### 10.2 UI для обновлений
- Уведомление о новой версии
- Кнопка "Обновить сейчас"
- Progress bar скачивания

### Этап 11: Автообновление конфигов

#### 11.1 Создать `src-tauri/src/core/config_updater.rs`
```rust
pub async fn check_config_updates() -> Result<Vec<String>>
pub async fn download_config_updates() -> Result<()>
```

### Этап 12: Crash Reporting (Sentry)

#### 12.1 Интеграция Sentry
```rust
// src-tauri/src/lib.rs
let _guard = sentry::init(("DSN", sentry::ClientOptions {
    release: Some(env!("CARGO_PKG_VERSION").into()),
    ..Default::default()
}));
```

### Этап 13: Логирование в файл

#### 13.1 Настройка tracing-appender
```rust
// src-tauri/src/lib.rs
let file_appender = tracing_appender::rolling::daily(log_dir, "isolate.log");
let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
```

#### 13.2 Ротация логов
- Хранить последние 7 дней
- Максимум 50MB на файл

---

## 🟢 НИЗКИЙ ПРИОРИТЕТ (Улучшения)

### Этап 14: Per-App Routing через WinDivert

#### 14.1 Фильтрация по PID
```rust
// src-tauri/src/core/app_filter.rs
pub fn get_process_pids(name: &str) -> Vec<u32>
pub fn apply_windivert_filter(pids: &[u32]) -> Result<()>
```

### Этап 15: Расписание

#### 15.1 Создать `src-tauri/src/core/scheduler.rs`
```rust
pub struct Scheduler {
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
    enabled: bool,
}
```

### Этап 16: Browser Extension

#### 16.1 WebSocket сервер в Isolate
```rust
// src-tauri/src/core/ws_server.rs
pub async fn start_ws_server(port: u16) -> Result<()>
```

#### 16.2 Extension (Manifest V3)
- Popup с статусом
- Кнопка оптимизации

### Этап 17: CLI режим

#### 17.1 Создать `src-tauri/src/cli.rs`
```bash
isolate optimize --mode turbo
isolate apply --strategy zapret_universal
isolate stop
isolate status
isolate diagnose
isolate reset
```

### Этап 18: Плагины (пользовательские стратегии)

#### 18.1 Загрузка из `%APPDATA%/Isolate/plugins/`
- Валидация YAML
- Метка "Custom" в UI

---

## 📦 DEPLOYMENT

### CI/CD Pipeline

#### GitHub Actions `.github/workflows/release.yml`
```yaml
- Build Windows x64
- Run tests
- Sign binaries (Code Signing Certificate)
- Create GitHub Release
- Upload .msi, .exe
- Update latest.json
```

### Installer (WiX)

#### Настройки в `tauri.conf.json`
```json
{
  "bundle": {
    "windows": {
      "wix": {
        "language": ["en-US", "ru-RU"]
      }
    }
  }
}
```

---

## 📊 МЕТРИКИ УСПЕХА

| Метрика | Цель |
|---------|------|
| Success rate оптимизации | > 90% |
| Время Turbo оптимизации | < 15 сек |
| Crash rate | < 0.1% |
| Покрытие тестами | > 80% |

---

## 🚀 ПОРЯДОК ВЫПОЛНЕНИЯ

### Фаза 1 (Критично) — 5-7 дней
1. Proxy Commands + Storage
2. Routing Commands
3. AppState расширение
4. Testing страница

### Фаза 2 (Высокий) — 3-5 дней
5. System Tray улучшения
6. Onboarding
7. Monitor
8. Telemetry

### Фаза 3 (Средний) — 3-4 дня
9. Автообновление
10. Crash Reporting
11. Логирование в файл

### Фаза 4 (Низкий) — по желанию
12. Per-App WinDivert
13. Расписание
14. Browser Extension
15. CLI
16. Плагины

---

## 📝 ПРИМЕЧАНИЯ

- **НИКОГДА** не запускать несколько winws/WinDivert процессов параллельно (BSOD!)
- Параллельный запуск разрешён ТОЛЬКО для VLESS/Sing-box (разные SOCKS-порты)
- Zapret стратегии — строго последовательно с таймаутом 2-3 сек
- Все пути к бинарникам через `paths.rs`, не хардкодить
- Логи не должны содержать IP пользователя
