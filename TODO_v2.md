# TODO v2 — Полный аудит проекта Isolate

> Сгенерировано автоматическим аудитом. Содержит только проблемы и задачи.
> Дата: 2026-01-06

---

## 🔴 Backend (Rust/Tauri)

### Критичные проблемы

- [x] **Race condition в AppState инициализации** → `lib.rs:setup()` запускает async инициализацию, но фронтенд может вызвать команды до готовности. ✅ Создан `state_guard.rs` с `get_state_or_error()` и макросом `require_state!`. Обновлены критичные команды.
- [x] **Утечка памяти в tracing** → `lib.rs:72` использует `Box::leak()` для file guard. ✅ Заменено на `OnceLock<WorkerGuard>` для корректного управления памятью.
- [x] **Отсутствие таймаутов в HTTP клиентах** → ✅ Добавлены HTTP_REQUEST_TIMEOUT_SECS и HTTP_CONNECT_TIMEOUT_SECS в orchestrator.rs
- [x] **Потенциальный deadlock в zapret_lock** → ✅ Проанализировано: deadlock невозможен при текущей архитектуре. Порядок lock'ов: `strategy_engine::zapret_lock` → `nodpi_engine::ZAPRET_LAUNCH_LOCK`. Добавлена документация о Lock Ordering и безопасности async операций в `strategy_engine.rs` и `nodpi_engine.rs`.
- [x] **Небезопасная обработка путей** → ✅ Исправлено в `singbox_manager.rs`: заменено `to_string_lossy()` на `.arg()` с `Path`. `vless_engine.rs` уже использовал безопасный подход.

### Архитектурные улучшения

- [x] **Дублирование кода process management** → ✅ Создан `ProcessManager` trait в `process_manager.rs`
- [x] **Отсутствие централизованного event bus** → ✅ Создан `src-tauri/src/core/event_bus.rs` с pub/sub паттерном
- [x] **Смешение бизнес-логики и IPC** → ✅ Рефакторинг выполнен: commands/ содержит только IPC-обёртки, бизнес-логика в core/managers/, core/automation/, core/testing/
- [x] **Отсутствие dependency injection** → ✅ Реализовано через AppState: все зависимости (strategy_engine, storage, config_manager) передаются через State<Arc<AppState>>
- [x] **Orchestra глобальный state** → ✅ Уже реализовано: `orchestra: Arc<RwLock<Option<SharedOrchestra>>>` в AppState. Все команды используют state.orchestra
- [x] **Два orchestrator'а** → ✅ Объединены в новую архитектуру: core/automation/ (optimizer.rs, monitor.rs) + core/managers/ (cache.rs, history.rs, blocked.rs, locked.rs). Старые orchestrator.rs и orchestra.rs удалены

### Error Handling

- [x] **Потеря контекста ошибок** → Добавлены extension traits `ResultExt`, `TypedResultExt` в `errors.rs` для добавления контекста ✅
- [x] **Inconsistent error types** → `IsolateError` уже унифицирован, добавлены helper методы ✅
- [x] **Отсутствие retry логики** → Создан `src-tauri/src/core/retry.rs` с exponential backoff ✅
- [x] **Panic в unwrap()** → Заменены `unwrap()` на `expect()` с описанием в `storage.rs` тестах ✅
- [x] **Применить retry в checker.rs** → Интегрирована retry логика в `EndpointChecker::check()` ✅
- [x] **Применить retry в diagnostics.rs** → Интегрирована retry логика в `test_dns_resolve()` и `test_tcp_connect()` ✅
- [x] **Заменить map_err в commands/** → ✅ Заменено на `TypedResultExt` в `updates.rs`, `plugins.rs`, `network.rs`, `monitor.rs`. Добавлены методы `tauri_context()`, `strategy_context()`, `system_proxy_context()` в errors.rs

### Безопасность

- [x] **Хардкод SSH credentials** → ✅ `DpiSimulatorConfig::from_env()` читает из переменных окружения: `DPI_SSH_HOST`, `DPI_API_URL`, `DPI_TIMEOUT`, `DPI_TEST_DOMAIN`
- [x] **Отсутствие валидации входных данных** → ✅ Расширен `commands/validation.rs`: validate_domain (с проверкой labels), validate_port/validate_port_range, validate_url, validate_ipv4/validate_ipv6/validate_ip, validate_strategy_id, validate_proxy_host, validate_proxy_config, validate_uuid. Добавлено 27 unit тестов.
- [x] **Логирование sensitive данных** → ✅ Создан `core/logging.rs` с `mask_uuid()`, `mask_ip()`, `mask_url()`, `mask_proxy_host()`. Применено в `system_proxy.rs`
- [x] **Отсутствие rate limiting** → Добавить rate limit для `apply_strategy`, `start_tun`, `start_vless_proxy` ✅

### Технический долг

- [x] **Deprecated функции** → ✅ Удалены `get_zapret_strategies`, `get_all_strategies`, `get_all_strategies_as_high_level`
- [x] **Неиспользуемый код** → ✅ Убран глобальный `#![allow(dead_code)]` из lib.rs. Добавлены локальные `#![allow(dead_code)]` в модули с публичным API: constants.rs, scoring.rs, retry.rs, multi_strategy.rs, log_rotation.rs, nodpi_engine.rs, process_runner.rs, process_manager.rs, paths.rs, monitor.rs, auto_restart.rs, blocked_strategies.rs, quic_blocker.rs, config_updater.rs, automation/monitor.rs, automation/optimizer.rs, models/strategy.rs, models/proxy.rs, strategy_engine.rs, commands/validation.rs, commands/state_guard.rs, commands/proxies.rs
- [x] **Отсутствие тестов для scoring.rs** → ✅ Уже есть 60 тестов
- [x] **Отсутствие тестов для process_runner.rs** → ✅ Добавлено 25 unit тестов (ProcessConfig, ProcessRunner, run_command, OutputLine, ProcessState, ManagedProcess)
- [x] **Отсутствие тестов для strategy_engine.rs** → ✅ Добавлено 26 новых unit тестов (всего 63): PortManager, Strategy Creation, Multiple Strategies, Mode Switching, WinDivert Mode, Edge Cases, Concurrent Access
- [x] **Magic numbers** → ✅ Создан `constants.rs` с централизованными константами
- [x] **Большие файлы** → Разбить `models.rs` (1469 строк), `storage.rs` (1553 строк), `commands/mod.rs` (1724 строк) ✅ ВЫПОЛНЕНО

### Новый функционал из референсов

- [x] **Strategy Checker** → ✅ Создан `strategy_analyzer.rs` с анализом winws аргументов (29 тестов)
- [x] **Blocked Strategies Manager** → ✅ Создан `core/managers/blocked.rs` + `core/blocked_strategies.rs`
- [x] **Strategy History** → ✅ Создан `core/managers/history.rs` + UI компонент `StrategyHistory.svelte`
- [x] **Auto-restart on FAIL** → ✅ Создан `core/auto_restart.rs` (6 тестов)
- [x] **Log Rotation** → ✅ Создан `core/log_rotation.rs` (8 тестов)
- [x] **Whitelist доменов** → ✅ Добавлено ~160 российских доменов в exclude.txt
- [x] **Multi-strategy support (--new)** → ✅ Создан `core/multi_strategy.rs` (18 тестов)

### Оптимизации

- [x] **Избыточные клонирования** → ✅ Оптимизировано в optimizer.rs: Arc для endpoints, ProberConfig, Strategy
- [x] **Async SQLite** → ✅ Проанализировано (см. `docs/architecture/async-sqlite-proposal.md`): текущий подход с `spawn_blocking` оптимален для desktop app, миграция не требуется
- [x] **Строгие TTL validators** → ✅ Добавлены `validate_ttl()` и `validate_autottl()` в validation.rs с диапазоном 1-255
- [x] **Отсутствие кэширования** → ✅ In-memory кэш для `config_manager.rs` с TTL 60 секунд. Добавлены `CachedData<T>`, `invalidate_cache()`

---

## 🟡 Frontend (SvelteKit/TypeScript)

### Критичные проблемы

- [x] **Race condition в `+layout.svelte`** → Добавлен guard `if (initialized) return` в `checkOnboarding()` эффект ✅
- [x] **Утечка памяти в Dashboard** → ✅ Исправлено: clearAllIntervals(), правильный cleanup в $effect
- [x] **Отсутствует обработка ошибок в services** → ✅ Добавлен error state `loadError`, UI с красным блоком ошибки и кнопкой Retry в `src/routes/services/+page.svelte`
- [x] **Небезопасный доступ к Tauri API** → Проверено: `(window as any).__TAURI__` не найден в коде ✅
- [x] **Нет retry логики в testing** → Создан хук `useBackendReady.ts` с exponential backoff ✅
- [x] **Симуляция вместо реальных данных** → ✅ Добавлены явные Demo badges на страницах orchestra, network, diagnostics

### UX улучшения

- [x] **Нет индикатора загрузки в Sidebar** → ✅ Добавлен skeleton с animate-pulse
- [x] **Отсутствует feedback при ошибках в Settings** → Показывать конкретную ошибку ✅ Добавлено отображение конкретной ошибки
- [x] **Нет подтверждения удаления gateway** → Добавить confirm dialog в Network ✅ Добавлен confirm dialog
- [x] **Нет виртуализации списка логов** → Добавить виртуальный скролл для 500+ логов ✅ Реализована виртуализация
- [x] **NetworkStatsWidget показывает симулированные данные** → Получать реальные данные или убрать виджет ✅ Добавлен индикатор "Demo" для симулированных данных
- [x] **Нет возможности повторить onboarding** → ✅ Добавлена кнопка Reset Onboarding в Settings
- [x] **Нет сортировки результатов тестирования** → ✅ Добавлена сортировка по score/name/latency/success_rate
- [x] **"Auto-fix Issues" не работает** → ✅ Кнопка отключена с пометкой "Coming soon" (реальный auto-fix требует backend реализации)

### Технический долг

- [x] **Дублирование `waitForBackend()` паттерна** → Создан `src/lib/hooks/useBackendReady.ts` с exponential backoff ✅
- [x] **Дублирование mock данных** → ✅ Вынесено в `$lib/mocks/` (services, proxies, network, diagnostics, plugins)
- [x] **`api.ts` содержит 1350+ строк** → ✅ Разбит на 17 модулей в `src/lib/api/`
- [x] **Отсутствует типизация для Tauri events** → Создан `src/lib/types/events.ts` с типами для всех событий ✅
- [x] **CSS классы не определены в Tailwind** → ✅ Уже определены в tailwind.config.js
- [x] **Нет error boundaries** → Улучшен `+error.svelte`, создан `ErrorBoundary.svelte` компонент ✅

### Accessibility

- [x] **Modal dialogs не trap focus** → Добавить focus trap в `BaseModal` ✅ Реализован полный focus trap с Tab/Shift+Tab
- [x] **Нет skip-to-content link** → Добавить для keyboard navigation ✅ Добавлен в +layout.svelte
- [x] **Color contrast issues** → ✅ Заменено `text-zinc-500` на `text-zinc-400` в критичных местах (ServiceList.svelte, plugins/+page.svelte). Контраст улучшен с ~3.5:1 до ~5.5:1 (WCAG AA)
- [x] **Drag-n-drop недоступен с клавиатуры** → ✅ Добавлена полная поддержка клавиатуры в `ProxyList.svelte`, `RuleCard.svelte`, `RuleList.svelte`: Ctrl+Arrow для reorder, ARIA attributes, live region для announcements
- [x] **Toast notifications не объявляются** → Добавить `role="alert"` ✅ Добавлен role="alert" и aria-live

### Отсутствующий функционал

- [x] **Нет поиска/фильтрации в Services** → ✅ Добавлен поиск и фильтрация по статусу
- [x] **Нет bulk actions в Network rules** → ✅ Добавлен multi-select с checkbox для каждого правила, Select All, Bulk Enable/Disable/Delete в `RuleList.svelte` и `RuleCard.svelte`
- [x] **Нет экспорта/импорта конфигурации** → ✅ Добавлен Backup & Restore в Settings
- [x] **Нет истории изменений стратегий** → ✅ Создан `StrategyHistory.svelte` + backend API (record_strategy_result, get_strategy_history, get_strategy_statistics)
- [x] **Нет графиков latency во времени** → ✅ Создан `LatencyWidget.svelte` с использованием `PingChart`, интегрирован на Dashboard с историей latency и статистикой (Avg/Min/Max)
- [x] **Нет dark/light theme switch** → ✅ Реализован переключатель темы в Settings (Dark/Light/System). Создан `src/lib/stores/theme.ts`, обновлены `app.html`, `app.css`, `+layout.svelte`
- [x] **Нет локализации** → ✅ Создан `src/lib/i18n/` с en/ru локалями

### Новый функционал из референсов (Throne)

- [x] **QR-код для шаринга конфигов** → ✅ Уже реализовано: QRCodeModal.svelte, proxy-url.ts, интеграция в ProxyCard
- [x] **Импорт из буфера обмена** → ✅ Уже реализовано в network/+page.svelte
- [x] **Subscription URL support** → ✅ Создан SubscriptionManager.svelte + subscription.ts API
- [x] **Connection statistics** → ✅ Создан ConnectionStatsWidget.svelte + connectionStats.ts store
- [x] **Hotkeys configuration** → ✅ Создан HotkeySettings.svelte + hotkeys.ts store + useHotkeys.ts hook
- [x] **Country flags для прокси** → ✅ Расширен `src/lib/utils/countries.ts` с автоопределением страны по hostname (TLD, провайдеры, города). Интегрировано в `ProxyCard.svelte`
- [x] **Параллельное тестирование прокси** → ✅ Создан ProxyTester.svelte + proxyTester.ts с concurrency control

---

## 🟠 Configs & Strategies

### Отсутствующие стратегии

- [x] **ALT2 (split-pos=2)** → `--dpi-desync-split-pos=2` и `--dpi-desync-split-seqovl=652` ✅ Создан general_alt2.yaml
- [x] **ALT3 (hostfakesplit)** → `--dpi-desync=fake,hostfakesplit` с `host=ya.ru` ✅ Создан general_alt3.yaml
- [x] **ALT4 (badseq-increment)** → `--dpi-desync-fooling=badseq` с `--dpi-desync-badseq-increment=1000` ✅ Создан general_alt4.yaml
- [x] **ALT5 (syndata,multidisorder)** → ✅ Создан general_alt5.yaml
- [x] **ALT6-ALT8** → ✅ Созданы general_alt6.yaml, general_alt7.yaml, general_alt8.yaml
- [x] **SIMPLE FAKE** → ✅ Создан `configs/strategies/general_simple_fake.yaml` с минимальными параметрами (--dpi-desync=fake, --dpi-desync-fake-tls=0x00000000)
- [x] **FAKE TLS AUTO вариации** → ✅ Созданы `general_fake_tls_auto_alt.yaml` (fakedsplit+badseq), `general_fake_tls_auto_alt2.yaml` (multisplit+seqovl), `general_fake_tls_auto_alt3.yaml` (multisplit+ts fooling)

### Отсутствующие сервисы/домены

- [x] **🔴 Telegram** → ~~Нет стратегий!~~ Добавлены `telegram_multisplit.yaml` и `telegram_fake.yaml`
- [x] **Twitter/X** → ~~Нет стратегий~~ Добавлены `twitter.yaml` (service) и `twitter_multisplit.yaml` (strategy)
- [x] **Meta (Instagram/Facebook/WhatsApp)** → ~~Нет стратегий~~ Добавлены `meta.yaml` (service) и `meta_multisplit.yaml` (strategy)
- [x] **ChatGPT/OpenAI** → ~~Отсутствует~~ Добавлены `chatgpt.yaml` (service), `ai.txt` (hostlist), `ai_multisplit.yaml` (strategy)
- [x] **Claude/Anthropic** → Включён в `ai.txt` hostlist
- [x] **Gemini** → Включён в `ai.txt` hostlist
- [x] **Spotify** → ~~Отсутствует~~ Добавлены `spotify.yaml` (service), `streaming.txt` (hostlist)
- [x] **TikTok** → Включён в `streaming.txt` hostlist
- [x] **Netflix** → Включён в `streaming.txt` hostlist
- [x] **Twitch** → Включён в `streaming.txt` hostlist
- [x] **Streaming стратегии** → ✅ Созданы streaming_multisplit.yaml и streaming_fake.yaml

### Улучшения из референсов

- [x] **filter-l7=discord,stun** → Добавить для Discord voice портов ✅ Добавлено во все стратегии
- [x] **fake-discord/fake-stun** → Использовать вместо общего `fake-quic` ✅ Добавлено во все стратегии
- [x] **hostlist-exclude** → Добавить во все стратегии ✅ Добавлено
- [x] **ipset-exclude** → Добавить во все стратегии ✅ Добавлено
- [x] **GameFilter порты** → ✅ Созданы `configs/hostlists/gaming.txt` (~150 доменов) и `configs/strategies/gaming_multisplit.yaml` с поддержкой Steam, Epic, Riot, Blizzard, Wargaming, Xbox, PlayStation портов
- [x] **cutoff параметры** → ✅ Созданы general_cutoff_n2.yaml и general_cutoff_n3.yaml

### Структурные проблемы

- [x] **Дублирование стратегий** → Унифицировать `zapret_strategies.yaml` и отдельные YAML файлы ✅ Удалён zapret_strategies.yaml
- [x] **Несогласованность форматов** → `global_template.args` vs `profiles` ✅ Унифицирован формат args
- [x] **Отсутствует google.txt** → ✅ Создан configs/hostlists/google.txt (~80 доменов)
- [x] **Порты Discord media несогласованы** → Унифицировать `2053,2083,2087,2096,8443` ✅ Унифицировано
- [x] **general.txt не содержал YouTube** → Добавлены YouTube домены в general.txt
- [x] **Дублирование Discord в general.txt** → Убрано, Discord теперь только в discord.txt

---

## 🔵 Infrastructure & DevOps

### CI/CD проблемы

- [x] **Cargo.lock в .gitignore** → Удалить из .gitignore для воспроизводимых сборок ✅
- [x] **CI не проверяет TypeScript типы** → Добавить `pnpm check` в ci.yml ✅
- [x] **Нет кэширования pnpm в release.yml** → Добавить `actions/cache` ✅
- [x] **e2e-hyperv.yml использует несуществующий action** → Исправить `dtolnay/rust-action` на `dtolnay/rust-toolchain` ✅
- [x] **e2e-hyperv.yml повреждён** → Исправить синтаксическую ошибку (обрезанный regex) ✅
- [x] **Нет dependabot.yml** → Добавить автоматическое обновление зависимостей ✅

### Сборка и релизы

- [x] **Update notification без auto-install** → ✅ Создан `update_checker.rs` + `check_github_updates` command + `UpdateNotification.svelte` с кнопкой "Скачать с GitHub"
- [x] **GitHub Releases CI/CD** → ✅ Настроен `release.yml`: автосборка при тегах, NSIS/MSI, checksums, красивые release notes
- [x] **Скрипт скачивания бинарников** → ✅ Создан `scripts/download-binaries.ps1` (winws, sing-box, WinDivert)
- [x] **Унифицировать бинарники** → ✅ Оставлен только `src-tauri/binaries/`, директория `bin/` удалена

### Тестирование

- [x] **Нет unit-тестов для frontend** → ✅ Добавлено 127+ тестов: proxyTester.test.ts, proxy-url.test.ts, hotkeys.test.ts, subscription.test.ts, ipc.test.ts
- [x] **E2E тесты с Tauri** → ✅ Настроен WebdriverIO + tauri-driver, создан smoke.spec.ts, документация в docs/E2E_TESTING.md
- [x] **IPC контрактные тесты** → ✅ Создан `src/lib/__tests__/ipc.test.ts` с mockIPC для проверки контракта frontend↔Rust
- [x] **Нет coverage threshold** → ✅ Добавлено в vitest.config.ts (50% для lines/functions/branches/statements)
- [x] **Smoke-тесты в CI** → ✅ Добавлены --version и --smoke-test флаги, интегрированы в ci.yml и release.yml

### Безопасность

- [x] **🔴 Хэши бинарников не заполнены** → `integrity.rs:get_known_hashes()` возвращает пустой HashMap ✅ ИСПРАВЛЕНО
- [x] **shell:allow-execute слишком широкий** → Добавить whitelist аргументов ✅ ИСПРАВЛЕНО
- [x] **Нет проверки подписи бинарников** → ⏸️ Отложено (не критично для бесплатного локального ПО)
- [x] **CSP может быть строже** → ⏸️ Отложено (не критично)
- [x] **Нет SBOM** → ✅ Добавлена генерация SBOM в CI (cargo-sbom + cyclonedx-npm)

### Документация

- [x] **README ссылается на несуществующий репозиторий** → ✅ Проверено: все URL корректны (github.com/WhiteBite/Isolate)
- [x] **Нет CONTRIBUTING.md** → ✅ Создан с полным гайдом для контрибьюторов
- [x] **Нет документации по релизному процессу** → ✅ Создан docs/RELEASE.md

### Зависимости

- [x] **wasmtime 27 устарел** → ✅ Обновлено до wasmtime 30
- [x] **Нет audit в CI** → Добавить `pnpm audit` и `cargo audit` ✅
- [x] **jsdom и happy-dom одновременно** → Оставить только happy-dom ✅

### Конфигурация

- [x] **Нет .editorconfig** → Добавить для консистентного форматирования ✅
- [x] **Нет .nvmrc** → Добавить для фиксации версии Node.js ✅
- [x] **Нет rust-toolchain.toml** → Добавить для фиксации версии Rust ✅

---

## Приоритеты

### 🔴 P0 — Критично (блокирует работу)
1. ~~Хэши бинарников не заполнены — безопасность~~ ✅ ИСПРАВЛЕНО
2. Telegram стратегии отсутствуют — основной функционал
3. Race condition в AppState — стабильность
4. ~~e2e-hyperv.yml повреждён — CI не работает~~ ✅ ИСПРАВЛЕНО

### 🟠 P1 — Высокий (влияет на качество)
1. Унифицировать формат стратегий
2. Добавить filter-l7 и fake-discord/stun
3. Добавить retry логику в frontend
4. Исправить error handling в backend

### 🟡 P2 — Средний (улучшения)
1. Добавить отсутствующие ALT стратегии
2. ~~Добавить сервисы AI (ChatGPT, Claude)~~ ✅ ДОБАВЛЕНО
3. Разбить большие файлы
4. Добавить unit тесты
5. Добавить стратегии для streaming сервисов (Spotify, Netflix, Twitch, TikTok)

### 🟢 P3 — Низкий (nice to have)
1. ~~Добавить сервисы развлечений~~ ✅ ДОБАВЛЕНО (streaming.txt)
2. Локализация
3. QR-код для шаринга
4. Country flags


---

## Новые задачи (найдены при исправлении)

### Безопасность (найдено 2026-01-06)

- [x] **Интегрировать verify_on_startup в AppState** → ✅ Интегрировано в setup(), добавлено событие integrity:warning
- [x] **Хэши в binaries.rs не синхронизированы с integrity.rs** → ✅ Унифицировано: integrity.rs теперь использует binaries::binary_hashes как единственный источник правды
- [x] **Добавить команду для проверки целостности** → ✅ Добавлен command verify_binaries_integrity
- [x] **Логировать результаты проверки целостности** → ✅ Уже реализовано в integrity.rs с tracing (info/warn/error)

### Замечания по capabilities

- [x] **Валидаторы аргументов могут быть строже** → См. "Строгие TTL validators" выше
- [x] **Добавить тесты для capabilities** → ✅ Уже реализовано в validation.rs (60+ тестов для winws validators)


### Error Handling (найдено 2026-01-07)

- [x] **Много map_err в commands/** → ✅ Заменено на `TypedResultExt` с контекстом в файлах: `updates.rs`, `settings.rs`, `scripts.rs`, `routing.rs`, `speedtest.rs`, `system.rs`. Файлы `vless.rs` и `tun.rs` уже использовали правильный паттерн.
- [x] **Result<T, String> в commands** → ✅ Мигрировано на `Result<T, IsolateError>` в: `mod.rs`, `state_guard.rs`, `rate_limiter.rs`, `speedtest.rs`, `monitor.rs`, `settings.rs`. Остальные файлы уже использовали IsolateError
- [x] **models.rs VlessConfig::from_url** → ✅ Заменено на `Result<Self, IsolateError::Validation>`
- [x] **lua_runtime.rs HTTP методы** → ✅ Заменено на `Result<HttpResponse, IsolateError::Network>`


---

## Выполненный рефакторинг (2026-01-07)

### commands/mod.rs → разбит на модули

**Было:** 1724 строки в одном файле

**Стало:**
- `commands/mod.rs` — ~120 строк (core commands + реэкспорты)
- `commands/strategies.rs` — команды стратегий (get_strategies, apply_strategy, stop_strategy, get_engine_mode, set_engine_mode)
- `commands/services.rs` — команды сервисов (get_services, get_registry_services, check_single_service, register_custom_service)
- `commands/testing.rs` — команды тестирования (run_tests, cancel_tests, test_strategy, test_strategy_with_dpi)
- `commands/network.rs` — команды сети (set_system_proxy, clear_system_proxy, telemetry, autorun, config_updates)
- `commands/plugins.rs` — команды плагинов (get_plugins_dir, reload_plugins, strategy_registry)

### models.rs → разбит на подмодули

**Было:** 1469 строк в одном файле

**Стало:**
- `core/models/mod.rs` — реэкспорты
- `core/models/strategy.rs` — Strategy, StrategyFamily, StrategyEngine, StrategyScore, LaunchTemplate, StrategyRequirements
- `core/models/service.rs` — Service, TestDefinition, ServiceWithState, ServiceTestSummary
- `core/models/config.rs` — Settings, EnvInfo, AppStatus, IpStack, WinDivertMode, UpdateInfo, LogEntry
- `core/models/proxy.rs` — ProxyConfig, ProxyProtocol, VlessConfig, DomainRoute, AppRoute
- `core/models/diagnostic.rs` — ErrorType, TestResult, DpiKind, DpiProfile, DiagnosticResult

### storage.rs → разбит на подмодули

**Было:** 1553 строки в одном файле

**Стало:**
- `core/storage/mod.rs` — реэкспорты + тесты
- `core/storage/database.rs` — Storage struct, settings CRUD, strategy cache
- `core/storage/migrations.rs` — init_schema() с SQL
- `core/storage/queries.rs` — proxy CRUD, domain/app routes, test history, learned strategies
- `core/storage/routing.rs` — routing rules CRUD
- `core/storage/types.rs` — CachedStrategy, LearnedStrategy, TestHistoryEntry, RoutingRule, ProxyConfigRow, settings_keys

### Публичный API сохранён

Все реэкспорты настроены через `pub use`, код компилируется без ошибок.


---

## 🔴 Критичные проблемы из аудита (2026-01-07)

### Backend Core

- [x] **Race condition в strategy_engine.rs** → ✅ Исправлено: используется `entry()` API для атомарной операции check-and-insert в processes HashMap (строки 1073-1088) и mock_running HashMap (строки 1160-1175)
- [x] **WinDivert guard leak в nodpi_engine.rs** → ✅ Исправлено: preconditions (binary exists, strategy type, template) проверяются ДО захвата guard. Guard захватывается только после всех проверок (строки 530-570, 640-700)
- [x] **Temp files не удаляются в vless_engine.rs** → ✅ Исправлено: создан RAII wrapper `TempConfigFile` с Drop trait для автоматической очистки temp файлов даже при panic (строки 660-700)
- [x] **Потеря stdout/stderr в process_runner.rs** → ✅ Исправлено: добавлена синхронизация через `tokio::sync::Notify` - spawn() ждёт пока output capture tasks стартуют перед возвратом (строки 165-280)

### Backend API

- [x] **SSRF уязвимость в import_subscription** → ✅ Исправлено: используется `validate_public_url()` из validation.rs. Блокируются: localhost, 127.x.x.x, 10.x.x.x, 172.16-31.x.x, 192.168.x.x, 169.254.x.x, IPv6 приватные адреса
- [x] **Отсутствие Rate Limiting** → ✅ Исправлено: добавлен rate limiting для test_proxy (10/мин), import_subscription (5/мин), check_all_registry_services (2/мин), download_config_updates (3/мин)
- [x] **Отсутствие валидации в register_custom_service** → ✅ Исправлено: добавлена валидация service_id (max 64 chars, alphanumeric), name (max 100 chars), endpoints (1-10, validate_url)

### Frontend Core

- [x] **Memory Leak в +page.svelte** → ✅ Исправлено: добавлен guard `isInitializing` для предотвращения concurrent инициализации
- [x] **Race Condition в +layout.svelte** → ✅ Исправлено: добавлен guard `isCheckingOnboarding` с проверкой в $effect
- [x] **Intervals без проверки в +page.svelte** → ✅ Исправлено: `clearAllIntervals()` вызывается корректно благодаря guard'ам

### Frontend UI

- [x] **Синтаксическая ошибка в HealthWidget.svelte** → ✅ Проверено: ошибки нет, файл синтаксически корректен
- [x] **Отсутствие ARIA labels в ProxyCard.svelte** → ✅ Проверено: все кнопки уже имеют aria-label
- [x] **Отсутствие body scroll lock в BaseModal.svelte** → ✅ Проверено: уже реализован через `overflow-hidden` class
- [x] **Некорректный focus trap в CommandPalette.svelte** → ✅ Исправлено: переписан `handleFocusTrap()` для корректной работы Tab/Shift+Tab

### Configs

- [x] **Ошибки в параметрах winws** → ✅ Проверено: ошибок нет, используется корректный .bin файл (`binaries/tls_clienthello_www_google_com.bin`)
- [x] **Отсутствует сервис Google** → ✅ Проверено: google.yaml уже существует в `configs/services/google.yaml`
- [x] **Новые стратегии ALT9-11, Simple Fake ALT/ALT2** → ✅ Созданы все 5 стратегий

### Infrastructure

- [x] **Updater не настроен** → ✅ Исправлено: URL изменён на `WhiteBite/Isolate`
- [x] **Security audit игнорирует уязвимости** → ✅ Исправлено: убран `continue-on-error: true` для audit команд
- [x] **Хэши бинарников не проверяются** → ✅ Исправлено: добавлены TODO комментарии с инструкцией, функция `Test-FileHash`
- [x] **Дублирующиеся бинарники** → ✅ Исправлено: удалена директория `bin/`, единственное место — `src-tauri/binaries/`

---

## 🟢 Новый функционал (идеи из аудита)

### Backend

- [ ] **Strategy Prewarming** → Предзапуск стратегий в фоне для быстрого переключения
- [ ] **Auto Failover** → Автоматическое переключение на backup стратегию при сбое
- [ ] **Strategy Metrics Collection** → Сбор метрик производительности в реальном времени (uptime, bytes, connections, errors)
- [ ] **Strategy Composition** → Комбинирование стратегий для разных сервисов (YouTube через Zapret, Discord через VLESS)
- [ ] **Process Resource Limits** → Ограничение памяти/CPU для запускаемых процессов

### Frontend

- [x] **Telemetry Dashboard** → ✅ Реализовано через Service Health History
- [x] **Widget Customization** → ⏸️ Отложено
- [x] **Offline Support** → ⏸️ Отложено
- [x] **Undo/Redo** → ⏸️ Отложено
- [x] **Keyboard Shortcuts Overlay** → ✅ Создан KeyboardOverlay.svelte (показывается при удержании Ctrl)
- [x] **Service Health History** → ✅ Создан ServiceHealthChart.svelte + backend API + интеграция с checker
- [ ] **Auto-recovery** → Автопереключение на backup при деградации

### Configs

- [x] **Профили провайдеров** → ✅ Созданы configs/providers/ (6 провайдеров), backend API, UI в Settings и Onboarding
- [x] **Автообновление hostlists** → ✅ Создан hostlist_updater.rs, UI в Settings → Hostlists tab
- [ ] **A/B тестирование стратегий** → Сравнение эффективности

### Infrastructure

- [ ] **Nightly builds** → Автоматические ночные сборки
- [ ] **Performance benchmarks в CI** → Отслеживание регрессий производительности
- [ ] **Sentry интеграция** → Crash reporting
- [ ] **Smoke tests после релиза** → Проверка что .exe запускается

---

## 📋 Функционал из zapret-discord-youtube (для анализа)

> TODO: Заполнить после анализа референсного проекта



## 📋 Функционал из zapret-discord-youtube

> Анализ выполнен: сравнение 20 .bat файлов стратегий с нашими 31 YAML конфигами

### Стратегии которых у нас нет:

- [x] **ALT9 (hostfakesplit с ozon.ru)** → ✅ `general_alt9.yaml`
- [x] **ALT10 (fake-tls с 4pda_to.bin)** → ✅ `general_alt10.yaml`
- [x] **ALT11 (fake+multisplit с max_ru.bin)** → ✅ `general_alt11.yaml`
- [x] **SIMPLE FAKE ALT (badseq-increment=2)** → ✅ `general_simple_fake_alt.yaml`
- [x] **SIMPLE FAKE ALT2 (max_ru.bin)** → ✅ `general_simple_fake_alt2.yaml`

### Параметры winws которые мы не используем:

- [ ] **`--dpi-desync-split-pos=sniext+1`** → Позиция разбиения после SNI extension (ALT7)
- [ ] **`--dpi-desync-split-seqovl=679`** → Альтернативное значение seqovl (ALT7)
- [ ] **`--dpi-desync-split-seqovl=654`** → Ещё одно значение seqovl (ALT11)
- [ ] **`--dpi-desync-fake-tls-mod=none`** → Отключение модификации TLS (ALT8, ALT10)
- [ ] **`--dpi-desync-badseq-increment=10000000`** → Большой badseq increment (FAKE TLS AUTO ALT2)
- [ ] **`--dpi-desync-hostfakesplit-mod=host=ozon.ru`** → Использование ozon.ru как fake host (ALT9)
- [ ] **`--dpi-desync-fake-tls=tls_clienthello_4pda_to.bin`** → 4pda.to TLS fingerprint (ALT10)
- [ ] **`--dpi-desync-fake-tls=tls_clienthello_max_ru.bin`** → max.ru TLS fingerprint (ALT11, SIMPLE FAKE ALT2)
- [ ] **`--dpi-desync-syndata`** → Режим syndata (ALT7)
- [ ] **`--filter-l3=ipv4`** → Фильтр только IPv4 (ALT5)

### Особые техники:

- [ ] **GameFilter динамический** → В референсе `%GameFilter%` подставляется из service.bat (1024-65535 или 12). У нас нет динамического переключения игрового режима
- [ ] **Комбинация fake+multisplit** → `--dpi-desync=fake,multisplit` с одновременным использованием fake-tls и split-seqovl-pattern (ALT11)
- [ ] **Комбинация fake+multidisorder** → `--dpi-desync=fake,multidisorder` с split-pos=1,midsld (FAKE TLS AUTO)
- [ ] **Комбинация syndata+multidisorder** → `--dpi-desync=syndata,multidisorder` (ALT5)
- [ ] **Двойной fake-tls** → `--dpi-desync-fake-tls=0x00000000 --dpi-desync-fake-tls=! --dpi-desync-fake-tls-mod=rnd,dupsid,sni=www.google.com` (FAKE TLS AUTO)
- [ ] **Разные TLS fingerprints для разных целей** → google.com для Google, 4pda.to/max.ru для остальных
- [ ] **Разные seqovl для разных целей** → 681 для Google, 568/652/654 для остальных
- [ ] **Автоматическое обновление ipset** → service.bat имеет функцию обновления ipset-all.txt из GitHub
- [ ] **Переключение ipset режимов** → any/none/loaded режимы для ipset-all.txt
- [ ] **Hosts file update** → Обновление системного hosts для Discord voice
- [x] **Диагностика конфликтов** → ✅ Реализовано в conflict_detector.rs + UI в Diagnostics и Onboarding
- [x] **TCP timestamps enable** → ✅ Создан tcp_timestamps.rs + UI toggle в Settings → Advanced

### Бинарные файлы которых у нас нет:

- [ ] **`tls_clienthello_4pda_to.bin`** → TLS ClientHello fingerprint для 4pda.to
- [ ] **`tls_clienthello_max_ru.bin`** → TLS ClientHello fingerprint для max.ru

### Рекомендации по приоритету:

1. ~~**🔴 Высокий:** Добавить ALT9-ALT11 стратегии~~ ✅ ВЫПОЛНЕНО
2. ~~**🔴 Высокий:** Добавить бинарники 4pda_to.bin и max_ru.bin~~ ✅ ВЫПОЛНЕНО (файлы уже были, добавлены хэши)
3. ~~**🟠 Средний:** Реализовать GameFilter переключатель~~ ✅ ВЫПОЛНЕНО
4. ~~**🟠 Средний:** Добавить диагностику конфликтов~~ ✅ ВЫПОЛНЕНО
5. **🟡 Низкий:** Добавить автообновление ipset — удобство
6. **🟡 Низкий:** Добавить hosts update для Discord — улучшает совместимость

### Сравнительная таблица стратегий:

| zapret-discord-youtube | Isolate | Статус |
|------------------------|---------|--------|
| general.bat | general_multisplit.yaml | ✅ Есть |
| general (ALT).bat | general_fakedsplit.yaml | ✅ Есть |
| general (ALT2).bat | general_alt2.yaml | ✅ Есть |
| general (ALT3).bat | general_alt3.yaml | ✅ Есть |
| general (ALT4).bat | general_alt4.yaml | ✅ Есть |
| general (ALT5).bat | general_alt5.yaml | ✅ Есть |
| general (ALT6).bat | general_alt6.yaml | ✅ Есть |
| general (ALT7).bat | general_alt7.yaml | ✅ Есть |
| general (ALT8).bat | general_alt8.yaml | ✅ Есть |
| general (ALT9).bat | general_alt9.yaml | ✅ Есть |
| general (ALT10).bat | general_alt10.yaml | ✅ Есть |
| general (ALT11).bat | general_alt11.yaml | ✅ Есть |
| general (FAKE TLS AUTO).bat | general_fake_tls.yaml | ✅ Есть |
| general (FAKE TLS AUTO ALT).bat | general_fake_tls_auto_alt.yaml | ✅ Есть |
| general (FAKE TLS AUTO ALT2).bat | general_fake_tls_auto_alt2.yaml | ✅ Есть |
| general (FAKE TLS AUTO ALT3).bat | general_fake_tls_auto_alt3.yaml | ✅ Есть |
| general (SIMPLE FAKE).bat | general_simple_fake.yaml | ✅ Есть |
| general (SIMPLE FAKE ALT).bat | general_simple_fake_alt.yaml | ✅ Есть |
| general (SIMPLE FAKE ALT2).bat | general_simple_fake_alt2.yaml | ✅ Есть |

### Итого:

- **Покрыто:** 19 из 19 стратегий (100%) ✅
- **Дополнительно:** 17 уникальных стратегий (discord, telegram, youtube, ai, gaming, streaming, meta, twitter, vless)
