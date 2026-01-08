# Isolate — Полный аудит функционала

> Дата аудита: 8 января 2026
> Версия: 2.x

## Обзор

Isolate — десктопное Windows-приложение для автоматического обхода DPI-блокировок.
Стек: Rust + Tauri 2.0 + SvelteKit + TypeScript + Tailwind CSS.

---

## 1. Frontend (SvelteKit)

### 1.1 Страницы (12)

| Страница | Путь | Описание |
|----------|------|----------|
| Dashboard | `/` | Главная с виджетами статуса, быстрыми действиями |
| Services | `/services` | Управление сервисами (YouTube, Discord, etc.) |
| Strategies | `/strategies` | Просмотр и управление стратегиями обхода |
| Network | `/network` | Сетевые настройки, прокси, TUN |
| Settings | `/settings` | Общие настройки приложения |
| Diagnostics | `/diagnostics` | Диагностика системы и конфликтов |
| Logs | `/logs` | Просмотр логов в реальном времени |
| Orchestra | `/orchestra` | Оркестрация и автоматизация |
| Testing | `/testing` | Тестирование стратегий |
| Plugins | `/plugins` | Управление плагинами |
| Onboarding | `/onboarding` | Мастер первоначальной настройки |
| Provider Select | `/provider-select` | Выбор интернет-провайдера |

### 1.2 Компоненты (~60)

#### UI Primitives
- `Button`, `Card`, `Badge`, `Tooltip`, `Modal`, `Dropdown`
- `Input`, `Select`, `Checkbox`, `Switch`, `Slider`
- `Tabs`, `Accordion`, `Progress`, `Spinner`, `Skeleton`
- `Toast`, `Alert`, `ConfirmDialog`

#### Layout
- `Sidebar`, `Header`, `Footer`, `PageContainer`
- `SplitPane`, `ResizablePanel`, `ScrollArea`

#### Dashboard Widgets
- `StatusWidget` — текущий статус защиты
- `QuickActions` — быстрые действия
- `ServiceStatus` — статус сервисов
- `ConnectionStats` — статистика соединений
- `RecentActivity` — последняя активность

#### Services
- `ServiceCard`, `ServiceList`, `ServiceDetails`
- `ServiceHealth`, `PingHistory`, `HealthBadge`

#### Strategies
- `StrategyCard`, `StrategyList`, `StrategyDetails`
- `StrategyEditor`, `ParameterEditor`
- `StrategyComparison`, `StrategyMetrics`

#### Network
- `ProxyList`, `ProxyEditor`, `ProxyImport`
- `TunStatus`, `TunConfig`
- `RoutingRules`, `DomainRouter`

#### Testing
- `TestRunner`, `TestResults`, `TestProgress`
- `ABTestPanel`, `TestComparison`

#### Orchestra
- `AutomationRules`, `ScheduleEditor`
- `FailoverConfig`, `EventLog`

### 1.3 Stores (10)

| Store | Назначение |
|-------|------------|
| `appStatus` | Глобальный статус приложения |
| `services` | Список сервисов и их состояние |
| `settings` | Пользовательские настройки |
| `logs` | Буфер логов |
| `toasts` | Уведомления |
| `plugins` | Загруженные плагины |
| `theme` | Тема оформления |
| `hotkeys` | Горячие клавиши |
| `connectionStats` | Статистика соединений |
| `providers` | ISP провайдеры |

### 1.4 API Модули (15)

| Модуль | Функции |
|--------|---------|
| `core` | `isBackendReady`, `getAppVersion`, `getSystemInfo` |
| `optimization` | `startOptimization`, `stopOptimization`, `getProgress` |
| `vless` | `getProxies`, `addProxy`, `testProxy`, `importSubscription` |
| `proxy` | `setSystemProxy`, `getSystemProxy`, `clearProxy` |
| `routing` | `getRoutes`, `addRoute`, `deleteRoute`, `getDomainRouting` |
| `testing` | `runTest`, `getTestResults`, `abTest` |
| `settings` | `getSettings`, `saveSettings`, `resetSettings` |
| `logs` | `getLogs`, `clearLogs`, `exportLogs` |
| `tray` | `updateTrayMenu`, `showNotification` |
| `tun` | `startTun`, `stopTun`, `getTunStatus` |
| `monitor` | `getConnectionStats`, `getProcessList` |
| `telemetry` | `sendEvent`, `getMetrics` |
| `dns` | `getDnsServers`, `setDns`, `flushDns` |
| `failover` | `getFailoverConfig`, `setFailoverConfig` |
| `crashReporting` | `reportCrash`, `sendFeedback` |

---

## 2. Backend (Rust)

### 2.1 Core Modules (65 файлов)

#### Движки стратегий
| Модуль | Описание |
|--------|----------|
| `strategy_engine.rs` | Основной движок запуска стратегий |
| `nodpi_engine.rs` | Движок NoDPI/Zapret стратегий |
| `vless_engine.rs` | Движок VLESS/Sing-box стратегий |
| `multi_strategy.rs` | Мульти-стратегии (комбинации) |
| `strategy_composition.rs` | Композиция стратегий |
| `strategy_combiner.rs` | Комбинирование параметров |
| `strategy_analyzer.rs` | Анализ эффективности |
| `strategy_prewarming.rs` | Предварительный прогрев |

#### Загрузка и валидация
| Модуль | Описание |
|--------|----------|
| `strategy_loader.rs` | Загрузка стратегий из YAML |
| `unified_strategy_loader.rs` | Унифицированный загрузчик |
| `config_validation.rs` | Валидация конфигураций |
| `config_updater.rs` | Обновление конфигов |

#### Тестирование
| Модуль | Описание |
|--------|----------|
| `strategy_tester.rs` | Тестирование стратегий |
| `test_engine.rs` | Движок тестов |
| `checker.rs` | Проверка доступности |
| `ab_testing.rs` | A/B тестирование |
| `scoring.rs` | Скоринг результатов |
| `strategy_metrics.rs` | Метрики стратегий |

#### Автоматизация
| Модуль | Описание |
|--------|----------|
| `automation/optimizer.rs` | Автоматическая оптимизация |
| `automation/monitor.rs` | Мониторинг состояния |
| `automation/events.rs` | Система событий |
| `auto_failover.rs` | Автоматическое переключение |
| `auto_restart.rs` | Автоперезапуск при сбоях |
| `autorun.rs` | Автозапуск с системой |

#### Сеть и прокси
| Модуль | Описание |
|--------|----------|
| `proxy_parser.rs` | Парсинг прокси-ссылок |
| `proxy_tester.rs` | Тестирование прокси |
| `system_proxy.rs` | Системный прокси Windows |
| `domain_routing.rs` | Маршрутизация по доменам |
| `app_routing.rs` | Маршрутизация по приложениям |
| `routing_converter.rs` | Конвертация правил |
| `quic_blocker.rs` | Блокировка QUIC |
| `dns_manager.rs` | Управление DNS |
| `tun_manager.rs` | TUN-интерфейс |

#### Sing-box интеграция
| Модуль | Описание |
|--------|----------|
| `singbox_manager.rs` | Управление Sing-box |
| `singbox_config.rs` | Генерация конфигов |

#### Процессы и ресурсы
| Модуль | Описание |
|--------|----------|
| `process_runner.rs` | Запуск внешних процессов |
| `process_manager.rs` | Управление процессами |
| `global_runner.rs` | Глобальный запуск |
| `resource_limits.rs` | Лимиты ресурсов |

#### Хранение данных
| Модуль | Описание |
|--------|----------|
| `storage/database.rs` | SQLite база данных |
| `storage/migrations.rs` | Миграции БД |
| `storage/queries.rs` | SQL запросы |
| `storage/health_history.rs` | История здоровья сервисов |
| `storage/routing.rs` | Хранение маршрутов |
| `storage/subscriptions.rs` | Подписки на прокси |

#### Менеджеры состояния
| Модуль | Описание |
|--------|----------|
| `managers/blocked.rs` | Заблокированные стратегии |
| `managers/locked.rs` | Залоченные стратегии |
| `managers/cache.rs` | Кэширование |
| `managers/history.rs` | История операций |

#### Hostlists и IP
| Модуль | Описание |
|--------|----------|
| `hostlists.rs` | Работа со списками хостов |
| `hostlist_updater.rs` | Автообновление списков |
| `dynamic_hostlist.rs` | Динамические списки |
| `hosts_manager.rs` | Управление /etc/hosts |
| `ipset_updater.rs` | Обновление IP-сетов |

#### Утилиты
| Модуль | Описание |
|--------|----------|
| `binaries.rs` | Управление бинарниками |
| `integrity.rs` | Проверка целостности |
| `crypto.rs` | Криптографические функции |
| `paths.rs` | Пути к файлам |
| `constants.rs` | Константы |
| `errors.rs` | Типы ошибок |
| `retry.rs` | Логика повторов |

#### Логирование и мониторинг
| Модуль | Описание |
|--------|----------|
| `logging.rs` | Настройка логирования |
| `log_capture.rs` | Захват логов процессов |
| `log_rotation.rs` | Ротация логов |
| `monitor.rs` | Мониторинг системы |
| `telemetry.rs` | Телеметрия |
| `sentry_integration.rs` | Интеграция с Sentry |

#### Диагностика
| Модуль | Описание |
|--------|----------|
| `diagnostics.rs` | Системная диагностика |
| `conflict_detector.rs` | Детектор конфликтов |
| `env_info.rs` | Информация о среде |
| `tcp_timestamps.rs` | TCP timestamps |

#### Прочее
| Модуль | Описание |
|--------|----------|
| `config.rs` | Конфигурация приложения |
| `providers.rs` | ISP провайдеры |
| `hotkeys.rs` | Горячие клавиши |
| `event_bus.rs` | Шина событий |
| `update_checker.rs` | Проверка обновлений |

### 2.2 Models (7 файлов)

| Модель | Описание |
|--------|----------|
| `strategy.rs` | Strategy, StrategyFamily, StrategyParams |
| `service.rs` | Service, ServiceHealth, CheckResult |
| `proxy.rs` | Proxy, ProxyProtocol, Subscription |
| `config.rs` | AppConfig, Settings |
| `diagnostic.rs` | DiagnosticResult, ConflictInfo |
| `subscription.rs` | ProxySubscription |

### 2.3 Commands (38 модулей, ~250 команд)

#### Основные команды
| Модуль | Ключевые команды |
|--------|------------------|
| `strategies.rs` | `get_strategies`, `get_strategy`, `apply_strategy`, `stop_strategy` |
| `services.rs` | `get_services`, `check_service`, `get_service_health` |
| `automation.rs` | `start_optimization`, `stop_optimization`, `get_optimization_status` |
| `testing.rs` | `test_strategy`, `run_ab_test`, `get_test_results` |
| `vless.rs` | `get_proxies`, `add_proxy`, `test_proxy`, `import_subscription` |
| `routing.rs` | `get_routes`, `add_route`, `get_domain_routing` |
| `proxies.rs` | `set_system_proxy`, `clear_system_proxy` |

#### Сетевые команды
| Модуль | Ключевые команды |
|--------|------------------|
| `network.rs` | `get_network_info`, `get_interfaces` |
| `dns.rs` | `get_dns_servers`, `set_dns`, `flush_dns` |
| `tun.rs` | `start_tun`, `stop_tun`, `get_tun_status` |
| `quic.rs` | `block_quic`, `unblock_quic`, `get_quic_status` |

#### Диагностика и мониторинг
| Модуль | Ключевые команды |
|--------|------------------|
| `diagnostics.rs` | `run_diagnostics`, `check_conflicts`, `get_system_info` |
| `monitor.rs` | `get_connection_stats`, `get_process_list` |
| `health_history.rs` | `get_health_history`, `get_service_uptime` |
| `metrics.rs` | `get_strategy_metrics`, `reset_metrics` |

#### Настройки и конфигурация
| Модуль | Ключевые команды |
|--------|------------------|
| `settings.rs` | `get_settings`, `save_settings`, `reset_settings` |
| `providers.rs` | `get_providers`, `set_provider`, `detect_provider` |
| `hostlists.rs` | `get_hostlists`, `update_hostlist`, `add_custom_host` |
| `hosts.rs` | `get_hosts_entries`, `add_hosts_entry` |
| `ipset.rs` | `get_ipsets`, `update_ipset` |

#### Плагины и скрипты
| Модуль | Ключевые команды |
|--------|------------------|
| `plugins.rs` | `get_plugins`, `enable_plugin`, `run_plugin` |
| `scripts.rs` | `run_script`, `get_scripts` |
| `plugin_hostlists.rs` | `get_plugin_hostlists` |

#### Автоматизация
| Модуль | Ключевые команды |
|--------|------------------|
| `failover.rs` | `get_failover_config`, `set_failover_config`, `trigger_failover` |
| `ab_testing.rs` | `start_ab_test`, `get_ab_results` |
| `prewarming.rs` | `prewarm_strategies`, `get_prewarming_status` |
| `composition.rs` | `compose_strategies`, `get_compositions` |

#### Системные
| Модуль | Ключевые команды |
|--------|------------------|
| `system.rs` | `is_admin`, `request_admin`, `get_system_info` |
| `logs.rs` | `get_logs`, `clear_logs`, `export_logs` |
| `tray.rs` | `update_tray`, `show_notification` |
| `updates.rs` | `check_updates`, `download_update` |
| `validation.rs` | `validate_config`, `validate_strategy` |
| `resources.rs` | `get_resource_usage`, `set_resource_limits` |
| `tcp_timestamps.rs` | `disable_tcp_timestamps`, `restore_tcp_timestamps` |
| `speedtest.rs` | `run_speedtest` |

### 2.4 Services (2 файла)

| Сервис | Описание |
|--------|----------|
| `registry.rs` | Реестр сервисов, загрузка из YAML |

### 2.5 Plugins (система плагинов)

| Тип | Описание |
|-----|----------|
| `checker` | Проверка доступности (curl, browser, dns, tcp) |
| `script` | Пользовательские скрипты |
| `widget` | Виджеты для Dashboard |

---

## 3. Конфигурации

### 3.1 Стратегии (35)

#### Zapret стратегии (33)

| Стратегия | Целевой сервис | Техника |
|-----------|----------------|---------|
| `general_multisplit` | Универсальная | Multi-split |
| `general_fakedsplit` | Универсальная | Fake + Split |
| `general_fake_tls` | Универсальная | Fake TLS |
| `general_fake_tls_auto_alt` | Универсальная | Fake TLS Auto |
| `general_fake_tls_auto_alt2` | Универсальная | Fake TLS Auto v2 |
| `general_fake_tls_auto_alt3` | Универсальная | Fake TLS Auto v3 |
| `general_simple_fake` | Универсальная | Simple Fake |
| `general_simple_fake_alt` | Универсальная | Simple Fake Alt |
| `general_simple_fake_alt2` | Универсальная | Simple Fake Alt2 |
| `general_cutoff_n2` | Универсальная | Cutoff N2 |
| `general_cutoff_n3` | Универсальная | Cutoff N3 |
| `general_alt2` - `general_alt11` | Универсальная | Альтернативные варианты |
| `youtube_zapret` | YouTube | Zapret |
| `youtube_split` | YouTube | Split |
| `youtube_google` | YouTube + Google | Combined |
| `discord_zapret` | Discord | Zapret |
| `discord_fake` | Discord | Fake |
| `telegram_multisplit` | Telegram | Multi-split |
| `telegram_fake` | Telegram | Fake |
| `twitter_multisplit` | Twitter/X | Multi-split |
| `meta_multisplit` | Meta (FB, IG) | Multi-split |
| `streaming_multisplit` | Streaming | Multi-split |
| `streaming_fake` | Streaming | Fake |
| `ai_multisplit` | AI сервисы | Multi-split |
| `gaming_multisplit` | Gaming | Multi-split |
| `universal_zapret` | Все сервисы | Universal |

#### VLESS стратегии (1+)
| Стратегия | Описание |
|-----------|----------|
| `vless_proxy` | VLESS через Sing-box |

### 3.2 Сервисы (8)

| Сервис | Файл | Домены |
|--------|------|--------|
| YouTube | `youtube.yaml` | youtube.com, googlevideo.com, ytimg.com |
| Discord | `discord.yaml` | discord.com, discord.gg, discordapp.com |
| Google | `google.yaml` | google.com, googleapis.com, gstatic.com |
| Telegram | `telegram.yaml` | telegram.org, t.me, tg.dev |
| ChatGPT | `chatgpt.yaml` | openai.com, chat.openai.com |
| Meta | `meta.yaml` | facebook.com, instagram.com, fb.com |
| Spotify | `spotify.yaml` | spotify.com, scdn.co |
| Twitter | `twitter.yaml` | twitter.com, x.com, twimg.com |

### 3.3 Hostlists (14)

| Список | Описание |
|--------|----------|
| `all.txt` | Все домены |
| `youtube.txt` | YouTube домены |
| `discord.txt` | Discord домены |
| `google.txt` | Google домены |
| `telegram.txt` | Telegram домены |
| `twitter.txt` | Twitter/X домены |
| `meta.txt` | Meta (FB, IG) домены |
| `streaming.txt` | Стриминговые сервисы |
| `gaming.txt` | Игровые сервисы |
| `ai.txt` | AI сервисы |
| `general.txt` | Общие домены |
| `exclude.txt` | Исключения |
| `ipset-all.txt` | IP-адреса (все) |
| `ipset-exclude.txt` | IP исключения |

### 3.4 ISP Провайдеры (6)

| Провайдер | Файл | Рекомендуемые стратегии |
|-----------|------|------------------------|
| Ростелеком | `rostelecom.yaml` | general_multisplit, youtube_zapret |
| МТС | `mts.yaml` | general_fake_tls, discord_fake |
| Мегафон | `megafon.yaml` | general_fakedsplit |
| Билайн | `beeline.yaml` | general_simple_fake |
| Дом.ру | `dom_ru.yaml` | general_multisplit |
| ТТК | `ttk.yaml` | general_alt2 |

### 3.5 Плагины (12)

#### Checkers (6)
| Плагин | Описание |
|--------|----------|
| `youtube-checker` | Проверка доступности YouTube |
| `discord-checker` | Проверка Discord |
| `discord-voice-checker` | Проверка голосовых каналов Discord |
| `telegram-checker` | Проверка Telegram |
| `twitter-checker` | Проверка Twitter/X |
| `instagram-checker` | Проверка Instagram |
| `steam-checker` | Проверка Steam |

#### Widgets (3)
| Плагин | Описание |
|--------|----------|
| `speed-widget` | Виджет скорости |
| `speed-test-widget` | Виджет тестирования скорости |
| `latency-monitor` | Мониторинг задержки |

#### Utilities (2)
| Плагин | Описание |
|--------|----------|
| `dns-benchmark` | Бенчмарк DNS серверов |
| `templates` | Шаблоны для создания плагинов |

---

## 4. Инфраструктура

### 4.1 Сборка и CI/CD

| Файл | Назначение |
|------|------------|
| `package.json` | Frontend зависимости (pnpm) |
| `src-tauri/Cargo.toml` | Rust зависимости |
| `vite.config.ts` | Vite конфигурация |
| `svelte.config.js` | SvelteKit конфигурация |
| `tailwind.config.js` | Tailwind CSS |
| `tsconfig.json` | TypeScript |
| `vitest.config.ts` | Unit тесты |
| `playwright.config.ts` | E2E тесты |
| `wdio.conf.ts` | WebdriverIO тесты |

### 4.2 Tauri конфигурация

| Файл | Назначение |
|------|------------|
| `src-tauri/tauri.conf.json` | Основная конфигурация |
| `src-tauri/capabilities/` | Capabilities (permissions) |
| `src-tauri/build.rs` | Build script |

### 4.3 Внешние бинарники

| Бинарник | Назначение |
|----------|------------|
| `winws.exe` | Zapret/WinDivert обход DPI |
| `sing-box.exe` | VLESS/Sing-box прокси |
| `WinDivert.dll` | Драйвер перехвата пакетов |
| `WinDivert64.sys` | Kernel драйвер |

---

## 5. Ключевые фичи

### 5.1 Реализованные (P0-P3)

- ✅ Автоматическая оптимизация стратегий
- ✅ A/B тестирование стратегий
- ✅ Автоматический failover
- ✅ VLESS/Sing-box интеграция
- ✅ TUN режим
- ✅ Система плагинов
- ✅ Профили ISP провайдеров
- ✅ Горячие клавиши
- ✅ Автообновление hostlists
- ✅ Диагностика конфликтов
- ✅ Service health history
- ✅ TCP timestamps управление
- ✅ Strategy prewarming
- ✅ Strategy composition
- ✅ Process resource limits
- ✅ IPSet updater
- ✅ Hosts manager
- ✅ GameFilter (gaming стратегии)
- ✅ Smoke тесты в CI
- ✅ E2E тесты

### 5.2 Архитектурные особенности

- **Асинхронный backend** на Tokio
- **Svelte 5 runes** для реактивности
- **SQLite** для хранения данных
- **Event-driven** архитектура
- **Plugin system** для расширяемости
- **Graceful shutdown** для всех процессов
- **Race condition protection** при инициализации

---

## 6. Статистика

| Метрика | Значение |
|---------|----------|
| Frontend страниц | 12 |
| Frontend компонентов | ~60 |
| Frontend stores | 10 |
| Frontend API модулей | 15 |
| Backend core модулей | 65 |
| Backend commands | ~250 |
| Стратегий | 36 |
| Сервисов | 8 |
| Hostlists | 14 |
| ISP провайдеров | 6 |
| Плагинов | 12 |

---

*Документ сгенерирован автоматически на основе анализа кодовой базы.*
