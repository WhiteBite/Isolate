# Isolate — План разработки

## Фаза 0: Инициализация проекта
- [ ] **0.1** Инициализировать Git репозиторий
- [ ] **0.2** Создать `.gitignore` (node_modules, target, thirdparty, бинарники)
- [ ] **0.3** Инициализировать Tauri 2.0 + SvelteKit проект
- [ ] **0.4** Настроить TypeScript strict mode
- [ ] **0.5** Настроить Tailwind CSS
- [ ] **0.6** Настроить базовую структуру Rust backend
- [ ] **0.7** Первый коммит с базовой структурой

---

## Фаза 1: Core Backend (Rust) — без GUI

### 1.1 Модели данных
- [ ] **1.1.1** Создать `src-tauri/src/core/models.rs` — Strategy, Service, Test, Result
- [ ] **1.1.2** Создать `src-tauri/src/core/errors.rs` — IsolateError enum
- [ ] **1.1.3** Создать YAML схемы для стратегий (`configs/strategies/schema.yaml`)
- [ ] **1.1.4** Создать YAML схемы для сервисов (`configs/services/schema.yaml`)

### 1.2 Config Manager
- [ ] **1.2.1** Создать `src-tauri/src/core/config.rs` — загрузка YAML конфигов
- [ ] **1.2.2** Парсинг стратегий из `configs/strategies/*.yaml`
- [ ] **1.2.3** Парсинг сервисов из `configs/services/*.yaml`
- [ ] **1.2.4** Валидация конфигов при загрузке

### 1.3 Process Runner
- [ ] **1.3.1** Создать `src-tauri/src/core/process_runner.rs`
- [ ] **1.3.2** Запуск внешнего процесса с аргументами
- [ ] **1.3.3** Захват stdout/stderr
- [ ] **1.3.4** Graceful shutdown (SIGTERM → SIGKILL)
- [ ] **1.3.5** Таймауты запуска

### 1.4 Environment Info
- [ ] **1.4.1** Создать `src-tauri/src/core/env_info.rs`
- [ ] **1.4.2** Определение ASN провайдера (через IP-API)
- [ ] **1.4.3** Определение страны
- [ ] **1.4.4** Получение Wi-Fi SSID (Windows API)
- [ ] **1.4.5** Проверка прав администратора

### 1.5 Diagnostics Module
- [ ] **1.5.1** Создать `src-tauri/src/core/diagnostics.rs`
- [ ] **1.5.2** DNS resolve тест
- [ ] **1.5.3** TCP connect тест (порт 443)
- [ ] **1.5.4** TLS handshake тест
- [ ] **1.5.5** Классификация типа блокировки (DnsBlock, SniTlsBlock, IpBlock, NoBlock)
- [ ] **1.5.6** Формирование DpiProfile

### 1.6 Test Engine
- [ ] **1.6.1** Создать `src-tauri/src/core/test_engine.rs`
- [ ] **1.6.2** HTTP GET/HEAD тест (reqwest)
- [ ] **1.6.3** WebSocket тест (tokio-tungstenite)
- [ ] **1.6.4** TCP connect тест
- [ ] **1.6.5** Поддержка SOCKS5 прокси для всех тестов
- [ ] **1.6.6** Параллельное выполнение тестов (tokio::join)
- [ ] **1.6.7** Агрегация результатов → ServiceTestSummary

### 1.7 Strategy Engine
- [ ] **1.7.1** Создать `src-tauri/src/core/strategy_engine.rs`
- [ ] **1.7.2** SOCKS-режим: запуск стратегии на локальном порту
- [ ] **1.7.3** GLOBAL-режим: запуск стратегии глобально
- [ ] **1.7.4** Управление портами (выделение, освобождение)
- [ ] **1.7.5** Последовательный запуск driver_exclusive стратегий
- [ ] **1.7.6** Параллельный запуск parallel_safe стратегий

### 1.8 Scoring System
- [ ] **1.8.1** Создать `src-tauri/src/core/scoring.rs`
- [ ] **1.8.2** Расчёт success_rate
- [ ] **1.8.3** Расчёт latency_avg и jitter
- [ ] **1.8.4** Формула итогового score
- [ ] **1.8.5** Фильтрация по порогу (success_rate >= 0.8)

### 1.9 Storage
- [ ] **1.9.1** Создать `src-tauri/src/core/storage.rs`
- [ ] **1.9.2** SQLite для настроек пользователя
- [ ] **1.9.3** Кэш стратегий (env_key → strategy_id)
- [ ] **1.9.4** CRUD операции для настроек

### 1.10 Orchestrator
- [ ] **1.10.1** Создать `src-tauri/src/core/orchestrator.rs`
- [ ] **1.10.2** Шаг 1: Проверка кэша
- [ ] **1.10.3** Шаг 2: DPI-диагностика
- [ ] **1.10.4** Шаг 3: Выбор кандидатов
- [ ] **1.10.5** Шаг 4: Параллельные SOCKS-тесты (VLESS)
- [ ] **1.10.6** Шаг 4b: Последовательные driver-тесты (Zapret)
- [ ] **1.10.7** Шаг 5: Выбор лучшей стратегии
- [ ] **1.10.8** Шаг 6: Применение в GLOBAL-режиме
- [ ] **1.10.9** Event-based progress reporting

---

## Фаза 2: Tauri Integration

### 2.1 Tauri Commands
- [ ] **2.1.1** Создать `src-tauri/src/commands/mod.rs`
- [ ] **2.1.2** `get_strategies` — список стратегий
- [ ] **2.1.3** `get_services` — список сервисов
- [ ] **2.1.4** `run_optimization` — запуск оптимизации
- [ ] **2.1.5** `cancel_optimization` — отмена
- [ ] **2.1.6** `apply_strategy` — применить стратегию
- [ ] **2.1.7** `stop_strategy` — остановить текущую
- [ ] **2.1.8** `get_status` — текущий статус
- [ ] **2.1.9** `diagnose` — запуск диагностики
- [ ] **2.1.10** `panic_reset` — аварийный сброс

### 2.2 Tauri Events
- [ ] **2.2.1** `optimization:progress` — прогресс оптимизации
- [ ] **2.2.2** `optimization:complete` — завершение
- [ ] **2.2.3** `strategy:status` — изменение статуса стратегии
- [ ] **2.2.4** `error` — ошибки

### 2.3 System Tray
- [ ] **2.3.1** Настроить system tray в Tauri
- [ ] **2.3.2** Иконка статуса (вкл/выкл)
- [ ] **2.3.3** Контекстное меню (Open, Optimize, Toggle, Exit)

---

## Фаза 3: Frontend (SvelteKit)

### 3.1 Базовая структура
- [ ] **3.1.1** Layout с навигацией
- [ ] **3.1.2** Svelte stores для состояния
- [ ] **3.1.3** Tauri API wrapper (`src/lib/tauri.ts`)
- [ ] **3.1.4** Типы TypeScript (`src/lib/types.ts`)

### 3.2 Главный экран
- [ ] **3.2.1** Компонент статуса (вкл/выкл, текущая стратегия)
- [ ] **3.2.2** Кнопка "Оптимизировать"
- [ ] **3.2.3** Список сервисов со статусами (зелёный/красный)
- [ ] **3.2.4** Quick actions (Toggle, Panic Reset)

### 3.3 Экран оптимизации
- [ ] **3.3.1** Progress bar
- [ ] **3.3.2** Шаги с анимацией (Диагностика → Тестирование → Применение)
- [ ] **3.3.3** Лог текущих действий
- [ ] **3.3.4** Кнопка отмены
- [ ] **3.3.5** Результат (успех/ошибка)

### 3.4 Настройки
- [ ] **3.4.1** Страница настроек
- [ ] **3.4.2** Автозапуск при старте Windows
- [ ] **3.4.3** Автооптимизация при запуске
- [ ] **3.4.4** Выбор сервисов (критичные/некритичные)
- [ ] **3.4.5** Block QUIC toggle
- [ ] **3.4.6** Телеметрия toggle
- [ ] **3.4.7** VLESS конфигурация (импорт vless://)

### 3.5 Логи
- [ ] **3.5.1** Страница логов
- [ ] **3.5.2** Фильтрация по модулям
- [ ] **3.5.3** Поиск
- [ ] **3.5.4** Экспорт логов

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
- [ ] **5.2.1** Создать `src-tauri/src/core/telemetry.rs`
- [ ] **5.2.2** Анонимный payload (ASN, strategy_id, success_rate)
- [ ] **5.2.3** HTTP endpoint для отправки
- [ ] **5.2.4** UI для включения/отключения

### 5.3 Сборка и релиз
- [ ] **5.3.1** Настроить Tauri bundler для Windows
- [ ] **5.3.2** Подписание бинарников (опционально)
- [ ] **5.3.3** GitHub Actions для CI/CD
- [ ] **5.3.4** Создать installer (NSIS/WiX)
- [ ] **5.3.5** Автообновление приложения

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

**Фаза:** 0 — Инициализация
**Прогресс:** 0%
**Следующая задача:** 0.1 — Инициализировать Git
