# Isolate — План разработки

## Фаза 0: Инициализация проекта
- [x] **0.1** Инициализировать Git репозиторий ✅
- [x] **0.2** Создать `.gitignore` (node_modules, target, thirdparty, бинарники) ✅
- [x] **0.3** Инициализировать Tauri 2.0 + SvelteKit проект ✅
- [x] **0.4** Настроить TypeScript strict mode ✅
- [x] **0.5** Настроить Tailwind CSS ✅
- [x] **0.6** Настроить базовую структуру Rust backend ✅
- [x] **0.7** Первый коммит с базовой структурой ✅

---

## Фаза 1: Core Backend (Rust) — без GUI

### 1.1 Модели данных
- [x] **1.1.1** Создать `src-tauri/src/core/models.rs` — Strategy, Service, Test, Result ✅
- [x] **1.1.2** Создать `src-tauri/src/core/errors.rs` — IsolateError enum ✅
- [x] **1.1.3** Создать YAML схемы для стратегий (`configs/strategies/schema.yaml`) ✅
- [x] **1.1.4** Создать YAML схемы для сервисов (`configs/services/schema.yaml`) ✅

### 1.2 Config Manager
- [x] **1.2.1** Создать `src-tauri/src/core/config.rs` — загрузка YAML конфигов ✅
- [x] **1.2.2** Парсинг стратегий из `configs/strategies/*.yaml` ✅
- [x] **1.2.3** Парсинг сервисов из `configs/services/*.yaml` ✅
- [x] **1.2.4** Валидация конфигов при загрузке ✅

### 1.3 Process Runner
- [x] **1.3.1** Создать `src-tauri/src/core/process_runner.rs` ✅
- [x] **1.3.2** Запуск внешнего процесса с аргументами ✅
- [x] **1.3.3** Захват stdout/stderr ✅
- [x] **1.3.4** Graceful shutdown (SIGTERM → SIGKILL) ✅
- [x] **1.3.5** Таймауты запуска ✅

### 1.4 Environment Info
- [x] **1.4.1** Создать `src-tauri/src/core/env_info.rs` ✅
- [x] **1.4.2** Определение ASN провайдера (через IP-API) ✅
- [x] **1.4.3** Определение страны ✅
- [x] **1.4.4** Получение Wi-Fi SSID (Windows API) ✅
- [x] **1.4.5** Проверка прав администратора ✅

### 1.5 Diagnostics Module
- [x] **1.5.1** Создать `src-tauri/src/core/diagnostics.rs` ✅
- [x] **1.5.2** DNS resolve тест ✅
- [x] **1.5.3** TCP connect тест (порт 443) ✅
- [x] **1.5.4** TLS handshake тест ✅
- [x] **1.5.5** Классификация типа блокировки (DnsBlock, SniTlsBlock, IpBlock, NoBlock) ✅
- [x] **1.5.6** Формирование DpiProfile ✅

### 1.6 Test Engine
- [x] **1.6.1** Создать `src-tauri/src/core/test_engine.rs` ✅
- [x] **1.6.2** HTTP GET/HEAD тест (reqwest) ✅
- [x] **1.6.3** WebSocket тест (tokio-tungstenite) ✅
- [x] **1.6.4** TCP connect тест ✅
- [x] **1.6.5** Поддержка SOCKS5 прокси для всех тестов ✅
- [x] **1.6.6** Параллельное выполнение тестов (tokio::join) ✅
- [x] **1.6.7** Агрегация результатов → ServiceTestSummary ✅

### 1.7 Strategy Engine
- [x] **1.7.1** Создать `src-tauri/src/core/strategy_engine.rs` ✅
- [x] **1.7.2** SOCKS-режим: запуск стратегии на локальном порту ✅
- [x] **1.7.3** GLOBAL-режим: запуск стратегии глобально ✅
- [x] **1.7.4** Управление портами (выделение, освобождение) ✅
- [x] **1.7.5** Последовательный запуск driver_exclusive стратегий ✅
- [x] **1.7.6** Параллельный запуск parallel_safe стратегий ✅

### 1.8 Scoring System
- [x] **1.8.1** Создать `src-tauri/src/core/scoring.rs` ✅
- [x] **1.8.2** Расчёт success_rate ✅
- [x] **1.8.3** Расчёт latency_avg и jitter ✅
- [x] **1.8.4** Формула итогового score ✅
- [x] **1.8.5** Фильтрация по порогу (success_rate >= 0.8) ✅

### 1.9 Storage
- [x] **1.9.1** Создать `src-tauri/src/core/storage.rs` ✅
- [x] **1.9.2** SQLite для настроек пользователя ✅
- [x] **1.9.3** Кэш стратегий (env_key → strategy_id) ✅
- [x] **1.9.4** CRUD операции для настроек ✅

### 1.10 Orchestrator
- [x] **1.10.1** Создать `src-tauri/src/core/orchestrator.rs` ✅
- [x] **1.10.2** Шаг 1: Проверка кэша ✅
- [x] **1.10.3** Шаг 2: DPI-диагностика ✅
- [x] **1.10.4** Шаг 3: Выбор кандидатов ✅
- [x] **1.10.5** Шаг 4: Параллельные SOCKS-тесты (VLESS) ✅
- [x] **1.10.6** Шаг 4b: Последовательные driver-тесты (Zapret) ✅
- [x] **1.10.7** Шаг 5: Выбор лучшей стратегии ✅
- [x] **1.10.8** Шаг 6: Применение в GLOBAL-режиме ✅
- [x] **1.10.9** Event-based progress reporting ✅

---

## Фаза 2: Tauri Integration

### 2.1 Tauri Commands
- [x] **2.1.1** Создать `src-tauri/src/commands/mod.rs` ✅
- [x] **2.1.2** `get_strategies` — список стратегий ✅
- [x] **2.1.3** `get_services` — список сервисов ✅
- [x] **2.1.4** `run_optimization` — запуск оптимизации ✅
- [x] **2.1.5** `cancel_optimization` — отмена ✅
- [x] **2.1.6** `apply_strategy` — применить стратегию ✅
- [x] **2.1.7** `stop_strategy` — остановить текущую ✅
- [x] **2.1.8** `get_status` — текущий статус ✅
- [x] **2.1.9** `diagnose` — запуск диагностики ✅
- [x] **2.1.10** `panic_reset` — аварийный сброс ✅

### 2.2 Tauri Events
- [x] **2.2.1** `optimization:progress` — прогресс оптимизации ✅
- [x] **2.2.2** `optimization:complete` — завершение ✅
- [x] **2.2.3** `strategy:status` — изменение статуса стратегии ✅
- [x] **2.2.4** `error` — ошибки ✅

### 2.3 System Tray
- [x] **2.3.1** Настроить system tray в Tauri ✅
- [x] **2.3.2** Иконка статуса (вкл/выкл) ✅
- [x] **2.3.3** Контекстное меню (Open, Optimize, Toggle, Exit) ✅

---

## Фаза 3: Frontend (SvelteKit)

### 3.1 Базовая структура
- [x] **3.1.1** Layout с навигацией ✅
- [x] **3.1.2** Svelte stores для состояния ✅
- [x] **3.1.3** Tauri API wrapper (`src/lib/api.ts`) ✅
- [x] **3.1.4** Типы TypeScript (`src/lib/stores.ts`) ✅

### 3.2 Главный экран
- [x] **3.2.1** Компонент статуса (вкл/выкл, текущая стратегия) ✅
- [x] **3.2.2** Кнопка "Оптимизировать" ✅
- [x] **3.2.3** Список сервисов со статусами (зелёный/красный) ✅
- [x] **3.2.4** Quick actions (Toggle, Panic Reset) ✅

### 3.3 Экран оптимизации
- [x] **3.3.1** Progress bar ✅
- [x] **3.3.2** Шаги с анимацией (Диагностика → Тестирование → Применение) ✅
- [x] **3.3.3** Лог текущих действий ✅
- [x] **3.3.4** Кнопка отмены ✅
- [x] **3.3.5** Результат (успех/ошибка) ✅

### 3.4 Настройки
- [x] **3.4.1** Страница настроек ✅
- [x] **3.4.2** Автозапуск при старте Windows ✅
- [x] **3.4.3** Автооптимизация при запуске ✅
- [x] **3.4.4** Выбор сервисов (критичные/некритичные) ✅
- [x] **3.4.5** Block QUIC toggle ✅
- [ ] **3.4.6** Телеметрия toggle
- [ ] **3.4.7** VLESS конфигурация (импорт vless://)

### 3.5 Логи
- [x] **3.5.1** Страница логов ✅
- [x] **3.5.2** Фильтрация по модулям ✅
- [x] **3.5.3** Поиск ✅
- [x] **3.5.4** Экспорт логов ✅

---

## Фаза 4: Интеграция реальных бинарников

### 4.1 Zapret/winws
- [ ] **4.1.1** Скопировать winws.exe и WinDivert64.sys в `src-tauri/binaries/`
- [ ] **4.1.2** Создать конфиги стратегий для Discord
- [ ] **4.1.3** Создать конфиги стратегий для YouTube
- [ ] **4.1.4** Тестирование на реальном DPI

### 4.2 Sing-box
- [ ] **4.2.1** Скопировать sing-box.exe в `src-tauri/binaries/`
- [ ] **4.2.2** Шаблон конфига для VLESS
- [ ] **4.2.3** Парсинг vless:// ссылок
- [ ] **4.2.4** Интеграция с Strategy Engine

### 4.3 Hostlists
- [ ] **4.3.1** Скопировать списки доменов из zapret-discord-youtube
- [ ] **4.3.2** Механизм автообновления списков
- [ ] **4.3.3** Пользовательские домены

---

## Фаза 5: Polish & Release

### 5.1 Дополнительные функции
- [ ] **5.1.1** Block QUIC (Windows Firewall rule)
- [ ] **5.1.2** Panic Button (полный сброс сети)
- [ ] **5.1.3** Dual-stack IPv4/IPv6 тестирование
- [ ] **5.1.4** Portable mode (--portable flag)
- [ ] **5.1.5** Silent mode (--silent flag)

### 5.2 Телеметрия (opt-in)
- [x] **5.2.1** Создать `src-tauri/src/core/telemetry.rs` ✅
- [x] **5.2.2** Анонимный payload (ASN, strategy_id, success_rate) ✅
- [x] **5.2.3** HTTP endpoint для отправки ✅
- [ ] **5.2.4** UI для включения/отключения

### 5.3 Сборка и релиз
- [x] **5.3.1** Настроить Tauri bundler для Windows ✅
- [ ] **5.3.2** Подписание бинарников (опционально)
- [x] **5.3.3** GitHub Actions для CI/CD ✅
- [ ] **5.3.4** Создать installer (NSIS/WiX)
- [x] **5.3.5** Автообновление приложения ✅

### 5.4 Тестирование
- [ ] **5.4.1** Unit тесты для scoring
- [ ] **5.4.2** Unit тесты для config parsing
- [ ] **5.4.3** Integration тесты для process_runner
- [ ] **5.4.4** E2E тесты UI (Playwright)

---

## Параллельные задачи для агентов

### Агент 1: Rust Core (Фаза 1.1-1.5)
Модели данных, Config Manager, Process Runner, Env Info, Diagnostics

### Агент 2: Rust Engine (Фаза 1.6-1.10)
Test Engine, Strategy Engine, Scoring, Storage, Orchestrator

### Агент 3: Tauri + Frontend (Фаза 2-3)
Tauri Commands, Events, System Tray, SvelteKit UI

### Агент 4: Integration (Фаза 4)
Бинарники, конфиги стратегий, hostlists

---

## Текущий статус

**Фаза:** 1-3 — Core Backend, Tauri Integration, Frontend (завершены), 4-5 — в процессе
**Прогресс:** ~90%
**Следующая задача:** 4.1 — Интеграция реальных бинарников (winws, sing-box), 3.4.6-3.4.7 — Телеметрия и VLESS UI
