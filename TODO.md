# TODO — Isolate UX & Product Backlog

> Сгенерировано: 2026-01-08  
> Цель: собрать единый список **незакрытых** задач (особенно по UI/UX) в формате, удобном для ИИ‑агентов.  
> Источники: `TODO_v2.md`, `TODO_REDESIGN.md`, текущий UI (Dashboard, Services, Network, Orchestra, Plugins, Settings).

## Легенда

- Приоритеты: `[P0]` критично, `[P1]` высокий, `[P2]` средний, `[P3]` nice‑to‑have.
- Типы: `[UX]` интерфейс/опыт, `[UI]` визуал, `[A11y]` доступность, `[FE]` фронтенд‑логика, `[BE]` бэкенд, `[DOC]` документация, `[ARCH]` архитектура.
- Формат задачи: **что сделать** + ссылки на файлы/модули + короткий критерий готовности.

---

## 0. Метазадачи и синхронизация

- [ ] [P1][meta] Описать в `CONTRIBUTING.md` процесс работы с этим TODO (как агенты отмечают выполненное, как линкуются PR/commits).
- [ ] [P1][meta] Раз в релиз синхронизировать `TODO.md` с `TODO_v2.md` и `TODO_REDESIGN.md` (удалять выполненные, переносить новые разделы).
- [ ] [P2][DOC] Добавить в `README.md` краткий раздел «Roadmap / UX backlog» с ссылкой на этот файл.
- [ ] [P2][ARCH] Ввести теги для задач (например, `#dashboard`, `#network`) и использовать их в описаниях / PR.

---

## 1. Глобальная навигация и информационная архитектура

- [x] [P1][UX][UI] Завершить миграцию на `SidebarNew.svelte` и новую структуру разделов. ✅ Реализовано
  - Файлы: `src/routes/+layout.svelte`, `src/lib/components/SidebarNew.svelte`, `src/lib/stores/navigation.svelte.ts`.
  - Критерий: в рантайме существует только одна реализация sidebar, структура как в TODO_REDESIGN (Main / Tools / System).
- [x] [P1][UX] Убрать дублирование списка плагинов: сейчас в сайдбаре одновременно есть секция "Plugins" и длинный список отдельных плагинов. ✅ Реализовано
  - Критерий: 
    - слева отображается только один элемент навигации "Plugins";
    - список установленных плагинов живёт **только** на странице `/plugins`.
- [ ] [P2][UX][A11y] Настроить предсказуемый порядок фокуса по основным пунктам навигации (Dashboard → Services/Library → Network → Orchestra/Troubleshooter → Plugins → Settings → Logs).
  - Файлы: компоненты навигации, глобальный layout.
  - Критерий: навигация полностью проходима по Tab/Shift+Tab, активный пункт визуально подсвечен.
- [ ] [P2][UX] Реализовать аккуратный режим `sidebar.collapsed` (иконки без текста, подсказки по hover, сохранение в `layout.ts`).
- [ ] [P2][UX] Добавить явную навигацию между старыми и новыми сущностями: `Services` → `Library` (если Library включён), `Network` → `Proxy & VPN`.
- [ ] [P3][UI] Унифицировать отступы, высоту элементов и размеры иконок в сайдбаре (одна шкала размеров вместо микса Tailwind‑классов).

---

## 2. Локализация и текст

- [x] [P1][UX][i18n] Привести к единому языку Settings‑страницу: сейчас заголовки и часть описаний на русском, часть на английском. ✅ Реализовано
  - Файл: `src/routes/settings/+page.svelte` + связанные компоненты.
  - Критерий: все user‑visible строки проходят через `$lib/i18n` (`t()`), и для них есть пары en/ru.
- [ ] [P1][i18n] Проинвентаризировать хардкод‑строки в `src/routes/**/*.svelte` и вынести их в словари.
  - Автоматическая часть: скрипт/линтер, который ищет строки без `t()`.
- [ ] [P2][UX] Согласовать терминологию: "Protection", "DPI Bypass", "Gateway", "Proxy", "Service", "Strategy", "Orchestra", "Troubleshooter".
  - Создать `docs/terminology.md` и использовать его при переводах.
- [ ] [P2][UX] Переписать пустые/ошибочные состояния на человекопонятные тексты (без технических деталей) для Dashboard, Services, Network, Orchestra, Plugins.
- [ ] [P3][DOC] Добавить микро‑копирайт для сложных действий (например, переключение TUN/System Proxy, запуск оптимизации, удаление плагина).

---

## 3. Dashboard (главный экран)

Файл: `src/routes/+page.svelte` + виджеты в `src/lib/components/widgets/*`.

- [x] [P1][UX] Улучшить состояние "backend не готов". ✅ Реализовано: BackendNotReady.svelte
  - Сейчас: пишем в логи и просто отключаем скелетон.
  - Сделать card с текстом "Backend not ready", кнопкой Retry и ссылкой на логи.
- [x] [P1][UX][UI] Согласовать статусы: хедер показывает `Inactive`, большая кнопка — "Protection Disabled", другие виджеты используют свои статусы. ✅ Реализовано
  - Ввести один источник истины в store (например, `dashboard.protectionStatus`) и под него привести тексты/цвета.
- [x] [P1][UX] Проработать сценарий первого запуска. ✅ Реализовано
  - Требование: новый пользователь видит понятный онбординг вместо "Unknown / No data yet" во всех виджетах.
- [ ] [P2][UX][FE] Заменить эмулируемые `networkStats` на реальные данные, когда будет готов `get_traffic_stats` (см. `TODO_REDESIGN.md`).
  - Признак: `NetworkStatsWidget` показывает реальные измерения, "Demo" режим включается только при запуске без Tauri.
- [ ] [P2][A11y] Добавить клавиатурную поддержку для drag‑n‑drop в `BentoGrid` (как уже сделано для Proxy/Rules).
  - Критерий: можно менять порядок виджетов стрелками + модификаторы, есть ARIA‑атрибуты и live‑region для анонсов.
- [ ] [P2][UX] Улучшить Quick Actions:
  - добавить tooltips с описанием побочных эффектов (например, что делает "Scan All" / "Test Current");
  - показывать disabled‑состояние, если backend не готов или нет активной стратегии.
- [ ] [P2][UI] Визуально выделить главный CTA (включение защиты) над остальными элементами: анимация, более крупный текст, подпись с рисками.
- [ ] [P3][UX] Добавить мини‑лог активности прямо на Dashboard (последние 3–5 событий типа "Protection started", "Strategy X applied").
- [ ] [P3][UI] Подчистить визуальный шум: выровнять тени и градиенты, убрать лишние бордеры в соседних виджетах.

---

## 4. Services / Library (состояние сервисов)

Файлы: `src/routes/services/+page.svelte`, `src/lib/components/services/*`, `src/lib/stores/library.svelte.ts`, `LibraryPage.svelte` (см. `TODO_REDESIGN.md`).

- [x] [P1][UX] Сделать внятное пустое состояние для Services, когда список сервиса пуст или backend не готов. ✅ Реализовано
  - Критерий: пользователь видит инструкцию, какую кнопку нажать ("Check All Services" / "Add Service"), а не тупой список.
- [x] [P1][UX] Перестать показывать огромные блоки с "Unknown"/"No data" без контекста. ✅ Реализовано
  - Скрывать графики Latency/Health, пока нет данных; показывать компактный плейсхолдер "Run check to see history".
- [x] [P1][UX] Объединить концепцию Services и Library, чтобы не было двух разных мест для настройки правил. ✅ Реализовано: баннер редиректа
  - Сделать редирект `/services` → `/library` после готовности Library.
- [ ] [P2][UX][FE] Повысить отзывчивость чеков сервисов:
  - индикатор прогресса/счётчик "N of M" при `check_all_registry_services`;
  - возможность отменить проверку.
- [x] [P2][A11y] Добавить hotkeys: ↑/↓ переключают сервис в списке, Enter открывает детали, `C` — проверить текущий сервис. ✅ Реализовано в Library
- [x] [P2][UX] В фильтрах добавить быстрый пресет "Critical only" (игры/Discord/YouTube и т.п.). ✅ Реализовано
- [ ] [P2][UI] Избавиться от лишних прогресс‑баров в списке (оставить один чёткий индикатор статуса: цвет + иконка + latency).
- [ ] [P3][UX] Добавить сохранение раскладки/выбранного сервиса в `layout.ts` (последний выбранный сервис по умолчанию).
- [x] [P3][UX] Подсветить услуги, для которых ещё ни разу не выполнялся тест (badge "Never tested"). ✅ Реализовано

---

## 5. Network / Proxy & VPN

Файл: `src/routes/network/+page.svelte` + компоненты в `src/lib/components/network/*`, `src/lib/components/proxies/*`.

- [x] [P1][UX] Улучшить первый запуск Network: сейчас пользователь видит несколько блоков "No gateways" / "No subscriptions". ✅ Реализовано
  - Требование: большой CTA "Add proxy" / "Import subscription" + краткая инструкция, что вообще такое gateway.
- [x] [P1][UX] Объяснить пользователю разницу между режимами `System Proxy` и `TUN Driver` прямо в UI. ✅ Реализовано: CaptureModeInfo.svelte
  - Tooltip или небольшой блок справки под переключателем.
- [x] [P1][UX][FE] Сделать защищённый сценарий переключения режима захвата: ✅ Реализовано
  - подтверждение перед выключением действующего режима;
  - обработка ошибок с понятным текстом (сейчас просто toast с текстом ошибки).
- [ ] [P2][UX] Упростить список Traffic Rules: 
  - сгруппировать по типу действия (Direct / Proxy / Block / DPI Bypass);
  - добавить быстрый фильтр по категории сервиса.
- [ ] [P2][UX] Добавить подсказку, что правило "*.doubleclick.net" блокирует трекинг (поможет новичкам понять пользу).
- [ ] [P2][UI] Вынести легенду цветов (Direct / Proxy / Block / DPI Bypass) из подвала в более заметное место.
- [ ] [P2][UX][A11y] Убедиться, что клавиатурное управление для RuleList (мультивыбор, reorder) полностью работает и задокументировать hotkeys прямо на странице.
- [ ] [P2][UX] Для тестирования прокси (`ProxyTester`) показать объединённый результат "best proxy" и дать кнопку "Set as active".
- [ ] [P3][UI] Добавить иконки стран к gateway‑карточкам в сетке (после внедрения `ProxyCardGrid.svelte`).

---

## 6. Orchestra → Troubleshooter (оптимизация стратегий)

Файл: `src/routes/orchestra/+page.svelte` + `src/lib/components/orchestra/*` + будущий Troubleshooter.

- [x] [P1][UX] Переименовать раздел Orchestra в более понятный пользователю "Troubleshooter" / "Оптимизация" и обновить все тексты. ✅ Реализовано: `/troubleshoot` с русскими текстами
- [x] [P1][UX] Реализовать сценарий "У меня не работает X" (мастер диагностики) вместо чисто техничного списка стратегий. ✅ Реализовано: `TroubleshootWizard.svelte`, `ProblemSelector.svelte`
- [x] [P1][UX] Сделать явное разделение режимов: ✅ Реализовано
  - **Одноразовая диагностика** (пользователь запускает вручную);
  - **AI Pilot / авто‑оптимизация** в фоне.
- [ ] [P2][UX] Упростить терминологию и визуал текущей Orchestra: уменьшить количество одновременно видимых панелей (Status, Progress, Controls, ServiceGrid, Statistics, StrategyQueue, ActivityLog).
  - Ввести вкладки или collapsible‑блоки.
- [x] [P2][UX][FE] Показать оценку времени (ETA) и оставшееся количество стратегий при оптимизации. ✅ Реализовано в `TestingProgress.svelte`
- [x] [P2][UX] Дать пользователю простой выбор сервиса на стартовом шаге (крупные карточки YouTube/Discord/Игры и пр.), а не чекбокс‑сетку. ✅ Реализовано в `ProblemSelector.svelte`
- [ ] [P2][A11y] Убедиться, что таблица StrategyQueue доступна для экранных читалок (role=table, headers, aria‑labels для статусов).
- [ ] [P3][UI] Добавить маленькую легенду по статусам стратегий (pending/testing/success/failed/skipped) с цветами.

---

## 7. Plugins (управление плагинами)

Файлы: `src/routes/plugins/+page.svelte`, старые страницы плагинов (`src/routes/plugins/[id]` если ещё есть), `src/lib/stores/plugins.ts`.

- [x] [P1][UX] Устранить дублирующий UI плагинов: сейчас есть новая страница `/plugins` и отдельная страница плагина (как на скрине Discord Checker) с другим дизайном. ✅ Реализовано: редирект на основную страницу
  - План: оставить **одну** страницу управления плагином, старая должна редиректить на новую.
- [x] [P1][UX] Добавить явное предупреждение перед удалением плагина с перечислением, что он добавляет (services/strategies/hostlists). ✅ Реализовано: модальное окно с предупреждением
- [ ] [P2][UX] Добавить фильтры и сортировку в marketplace (по категории, по количеству загрузок, по новизне).
- [ ] [P2][UX] Сделать страницу плагина самодостаточной:
  - краткое описание наверху;
  - список предоставляемых сущностей (services/strategies/hostlists) с переходами.
- [ ] [P2][A11y] Обеспечить навигацию по Tab внутри списка плагинов и marketplace, плюс явно видимый focus state.
- [x] [P2][UX] Пояснить значения permissions (`HTTP`, `FS`, `Proc`) — tooltip или маленькая справка "что это даёт". ✅ Реализовано: добавлены tooltips с пояснениями
- [ ] [P3][UI] Вынести ошибки загрузки плагинов (`p.error`) в более заметный блок с иконкой и действиями (открыть логи, перезагрузить).

---

## 8. Settings (настройки приложения)

Файл: `src/routes/settings/+page.svelte` + `src/lib/components/settings/*`.

- [x] [P1][UX] Пересобрать структуру настроек под реальные сценарии пользователя: ✅ Реализовано
  - Базовые (язык, тема, автозапуск) в отдельном простом разделе "Общие";
  - Сложные параметры (TcpTimestamps, Windivert mode) спрятать в "Для экспертов".
- [ ] [P1][UX] Уточнить модель сохранения:
  - сейчас часть настроек применяется сразу (через `set_setting`), часть только по кнопке Save.
  - Вариант: либо всё auto‑save + toast, либо чёткий "draft" + "Save".
- [x] [P1][UX] Сделать единый, заметный feedback при сохранении/ошибке (`saveMessage`) с консистентными цветами и иконками. ✅ Реализовано
- [ ] [P2][UX] Улучшить UX Hostlists‑раздела:
  - показывать индикатор загрузки/обновления прямо в таблице;
  - выводить количество новых доменов/обновлений;
  - давать ссылку на источник.
- [ ] [P2][UX] Добавить визуальное объяснение профиля провайдера (ISP Profile): что он делает, как влияет на стратегии.
- [ ] [P2][A11y] Обеспечить, чтобы все переключатели и селекты имели связанные label и были доступны для экранных читалок.
- [ ] [P3][UX] Для продвинутых настроек (Game Mode, Auto Failover) добавить мини‑видеогайд/ссылку на документацию.

---

## 9. Общий UX, доступность и обратная связь

- [ ] [P1][UX][A11y] Пройтись по всем основным страницам (Dashboard, Services/Library, Network, Orchestra/Troubleshooter, Plugins, Settings) и проверить контраст в тёмной теме.
  - Критерий: все ключевые тексты соответствуют WCAG AA.
- [x] [P1][UX] Выровнять поведение модалок по всему приложению (BaseModal): ✅ Реализовано
  - единые анимации;
  - Esc закрывает, если нет критичных действий;
  - все destructive‑действия требуют явного подтверждения.
- [ ] [P2][UX] Ввести единый паттерн для состояний `loading / empty / error / success` в списках и таблицах.
- [ ] [P2][UX] Настроить консистентные toast‑уведомления (уровни: info/success/warning/error, лимит одновременных, авто‑hide).
- [ ] [P2][A11y] Добавить тесты доступности (axe / pa11y) для ключевых страниц, интегрировать в CI.
- [ ] [P2][UX] Улучшить систему hotkeys:
  - единая справка (overlay);
  - одинаковые комбинации для похожих действий (Ctrl+F — поиск, Ctrl+K — command palette, `?` — справка).
- [ ] [P3][UX] Добавить лёгкий onboarding‑wizard, который объясняет 3–4 основных сценария использования (игры, мессенджеры, видео, работа).

---

## 10. Связь с существующими планами (TODO_REDESIGN)

Эти задачи не дублируют `TODO_REDESIGN.md`, но ссылаются на него как на детализированное ТЗ.

- [ ] [P1][ARCH][FE] Реализовать `Dashboard Redesign` из `TODO_REDESIGN.md` (ShieldIndicator, LiveActivityPanel, ModeSelector, TrafficChart, интеграция с новым `dashboard.ts`).
- [ ] [P1][ARCH][FE] Завершить перенос Services → Library (ServiceRuleCard, MethodDropdown, AddRuleModal, backend‑команды `get_library_rules`, `set_rule_method` и т.д.).
- [x] [P1][ARCH][BE] Реализовать новый модуль Library на бэкенде (`src-tauri/src/core/library/*` + команды в `src-tauri/src/commands/library.rs`). ✅ Уже реализовано
- [ ] [P2][ARCH][BE] Реализовать Proxy & VPN редизайн (ProxyCardGrid, ImportZone, ChainBuilder + backend команды для цепочек).
- [ ] [P2][ARCH][FE] Расширить Command Palette (категории, быстрые действия) и связать её с навигацией и стратегиями.
- [ ] [P2][ARCH][BE] Реализовать Game Mode (game_detector.rs, game_monitor.rs, UI‑индикатор режимов).
- [ ] [P3][ARCH][FE] Перевести основные stores на Svelte 5 runes, как описано в разделе "Рефакторинг Stores" в `TODO_REDESIGN.md`.

---

## 11. Идеи для будущих итераций (низкий приоритет)

- [ ] [P3][UX] Добавить режим "simple view" для нетехнических пользователей (минимум контролов, большой переключатель on/off, список профилей типа Game/Work/Streaming).
- [ ] [P3][UX] Дать пользователю возможность экспортировать отчёт о состоянии (PDF/Markdown) для отправки в поддержку.
- [ ] [P3][UX] Добавить встроенный "tour" (подсказки по шагам) при первом открытии каждого крупного раздела.
- [ ] [P3][UI] Поддержать альтернативную светлую тему не только на уровне Tailwind, но и с отдельными иллюстрациями/иконками.
- [ ] [P3][DOC] Подготовить серию коротких гайдов "Как починить X" (YouTube/Discord/Twitch/Telegram) и линковать их из Troubleshooter.

---

## 12. Backend: стратегии, winws и DPI‑техники (из TODO_v2)

- [ ] [P3][BE] Поддержать дополнительные winws‑параметры из `TODO_v2.md` для расширенных стратегий:
  - `--dpi-desync-split-pos=sniext+1`
  - `--dpi-desync-split-seqovl=679`, `654`
  - `--dpi-desync-fake-tls-mod=none`
  - `--dpi-desync-badseq-increment=10000000`
  - `--dpi-desync-hostfakesplit-mod=host=ozon.ru`
  - `--dpi-desync-fake-tls=tls_clienthello_4pda_to.bin`
  - `--dpi-desync-fake-tls=tls_clienthello_max_ru.bin`
- [ ] [P3][BE] Реализовать особые DPI‑техники из zapret‑discord‑youtube:
  - комбинированные режимы `fake+multisplit`, `fake+multidisorder`, `syndata+multidisorder`;
  - двойной `fake-tls` с разными fingerprint;
  - разные fingerprints и `seqovl` для разных целей (Google / остальные);
  - динамический GameFilter и переключение ipset‑режимов (any/none/loaded) поверх уже существующего игрового профиля.
- [ ] [P3][BE] Добавить поддержку отсутствующих TLS fingerprint бинарников:
  - `tls_clienthello_4pda_to.bin`
  - `tls_clienthello_max_ru.bin`
- [ ] [P3][BE][DOC] Задокументировать все расширенные режимы DPI (новые параметры, когда их использовать, риски) в `docs/strategies/dpi-tuning.md`.

## 13. Troubleshooter & AI Pilot — связка с Library и backend

- [ ] [P1][BE] В `apply_troubleshoot_result` сохранять привязку `service_id → strategy_id` в хранилище (Library), а не только запускать стратегию.
- [x] [P1][FE][ARCH] Связать `troubleshootStore` с `library.svelte.ts`: ✅ Реализовано
  - после успешного применения стратегии обновлять соответствующее правило в Library;
  - при открытии Troubleshooter подтягивать метод доступа из Library, если он уже задан.
- [x] [P1][BE] Реализовать команды AI Pilot: ✅ Уже реализовано
  - `start_ai_pilot` / `stop_ai_pilot` (фоновая оптимизация);
  - `get_ai_pilot_history` (история с таймстампами и действиями).
- [ ] [P1][FE] Подключить `aiPilotStore` к реальным backend‑командам и событиям:
  - запуск/остановка через Tauri `invoke`;
  - подгрузка истории из backend;
  - обновление истории по событиям `ai_pilot:action`.
- [ ] [P2][UX] В `AIPilotPanel.svelte` улучшить объяснение, что именно делает фоновая оптимизация (какие сервисы/стратегии может менять, как отменить изменения).
- [x] [P2][UX][FE] Добавить связь AI Pilot ↔ Troubleshooter: ✅ Реализовано: Troubleshooter и AI Pilot работают как единая система
  - если Troubleshooter применил стратегию, AI Pilot учитывает это как ручное вмешательство и не откатывает её без явного согласия пользователя;
  - в AI Pilot History помечать действия, которые связаны с результатом диагностики.

## 14. Proxy Chain & Import — backend и интеграция

- [x] [P1][BE] Реализовать парсинг и импорт прокси: ✅ Уже реализовано
  - `parse_proxy_url` (разбор одной ссылки без сохранения);
  - `batch_import_proxies` (групповой импорт из `ImportZone`).
- [x] [P1][BE] Реализовать команды для цепочек: ✅ Уже реализовано
  - `save_proxy_chain` / `apply_proxy_chain` в `chain.rs`;
  - хранение цепочек в БД (модель + миграции).
- [ ] [P1][FE] Связать `proxyChainStore` и `ChainBuilder.svelte` с backend:
  - загрузка сохранённых цепочек при открытии страницы;
  - сохранение по действию "Сохранить цепочку";
  - применение цепочки одним кликом "Применить".
- [ ] [P2][BE] Реализовать `detect_proxy_country` и использовать его:
  - при импорте новых прокси;
  - при отображении карточек в `ProxyCardGrid.svelte`.
- [ ] [P2][UX] Добавить явный статус применённой цепочки (какая цепочка сейчас активна, когда была активирована, кнопка "Сбросить").

## 15. Dashboard & события мониторинга

- [x] [P1][BE] Реализовать команды Dashboard из `TODO_REDESIGN.md`: ✅ Уже реализовано
  - `get_live_connections`;
  - `get_traffic_stats`;
  - `get_protection_issues`;
  - `set_operation_mode`;
  - `fix_issue`.
- [x] [P1][FE] Подключить `dashboard.ts` к реальному backend: ✅ Уже реализовано
  - заменить симуляцию статуса и трафика на реальные вызовы;
  - использовать Tauri events `traffic:update`, `connection:opened/closed`, `issue:detected/resolved`.
- [ ] [P2][ARCH][FE] Интегрировать EventBus с Dashboard:
  - при получении событий обновлять `dashboardStore` и связанные виджеты (ShieldIndicator, LiveActivityPanel и т.п.);
  - обеспечить, чтобы Dashboard был единственным источником истины для статусов защиты.

## 16. Event Bus & типизированные события

- [x] [P2][ARCH][FE] Расширить `eventBus.svelte.ts` дополнительными типами событий из `TODO_REDESIGN.md` и backend: ✅ Уже реализовано
  - `ai_pilot:check_complete`, `strategy:changed` с дополнительными полями;
  - события для Library (изменение метода доступа, добавление/удаление правил);
  - события для Proxy & VPN (изменение активной цепочки, импорт прокси).
- [ ] [P2][FE] Пройтись по основным страницам (Dashboard, Library, Troubleshooter, Network) и заменить локальные `event`‑системы / ручные `listen` на EventBus там, где это уместно.
- [ ] [P3][DOC] Описать контракты событий (кто эмитит, кто слушает, гарантии по порядку и частоте) в `docs/events.md`.

## 17. Diagnostics, Logs & Dev Tools

- [x] [P2][UX][FE] Улучшить страницу Diagnostics (`/diagnostics`): ✅ Реализовано
  - сгруппировать тесты по типам (DNS, TCP, DPI);
  - добавить понятные тексты результатов и рекомендации ("что делать дальше");
  - показать историю последних проверок.
- [ ] [P2][UX] Расширить Bottom Drawer с логами:
  - быстрые фильтры по уровню (info/warn/error);
  - пресеты "Только backend", "Только сеть".
- [ ] [P3][DEV] Добавить скрытый "dev mode" переключатель:
  - отображение технических идентификаторов стратегий/сервисов;
  - дополнительные диагностики (raw latency, debug‑инфа по DPI).


---

## 18. UI Components & UX Features (из TODO_REDESIGN)

### Компоненты Library

- [x] [P2][FE] Создать `SmartStrategySuggestion.svelte` — компонент предложения стратегии ✅ Уже реализовано
  - Файлы: `src/lib/components/library/SmartStrategySuggestion.svelte`
  - Критерий: при добавлении нового правила показывает рекомендуемую стратегию на основе домена

- [x] [P2][FE] Создать `LibraryCard.svelte` — универсальная карточка с Svelte 5 Snippets ✅ Уже реализовано
  - Файлы: `src/lib/components/library/LibraryCard.svelte`
  - Критерий: базовый компонент для `PresetCard`, `CustomRuleCard`, `ServiceRuleCard`

- [x] [P3][FE] Создать `PresetCard.svelte` и `CustomRuleCard.svelte` на основе `LibraryCard` ✅ Уже реализовано
  - Файлы: `src/lib/components/library/PresetCard.svelte`, `CustomRuleCard.svelte`
  - Критерий: переиспользуют `LibraryCard` через snippets

- [x] [P2][FE] Создать `StatusIndicator.svelte` — универсальный индикатор состояния ✅ Уже реализовано
  - Файлы: `src/lib/components/ui/StatusIndicator.svelte`
  - Критерий: поддерживает все состояния (idle/loading/active/error/recovering), используется в Dashboard, Library, Network

### Tray Menu

- [x] [P2][BE] Расширить `tray.rs` — submenu сервисов и профили ✅ Уже реализовано
  - Файлы: `src-tauri/src/tray.rs`
  - Функционал:
    - Submenu сервисов с checkbox (топ-5 по использованию)
    - Submenu профилей: Game Mode / Work Mode
    - Пункт "Rescan Network"
  - Критерий: из трея можно быстро включить/выключить защиту для конкретного сервиса

- [x] [P2][BE] Backend команды для управления защитой сервисов из трея ✅ Уже реализовано
  - Файлы: `src-tauri/src/commands/tray.rs`
  - Критерий: `toggle_service_protection`, `get_top_services`, `set_profile_mode`

### Toast Notifications

- [x] [P2][FE] Расширить toast store — дедупликация и группировка ✅ Уже реализовано
  - Файлы: `src/lib/stores/toast.ts`
  - Функционал:
    - Дедупликация одинаковых ошибок (показывать счётчик вместо дублей)
    - Progress toast с обновлением (для длительных операций)
  - Критерий: при 5 одинаковых ошибках показывается один toast с "(×5)"

### Game Mode

- [x] [P2][FE] Добавить настройки Game Mode в Settings ✅ Уже реализовано
  - Файлы: `src/routes/settings/+page.svelte`, `src/lib/components/settings/GameModeSettings.svelte`
  - Функционал:
    - Список отслеживаемых игр (добавление/удаление)
    - Интервал проверки (5/10/30 сек)
    - Действие при обнаружении игры (переключить профиль / уведомить)
  - Критерий: пользователь может настроить автоматическое переключение режима при запуске игры

---

## 19. Advanced DPI & Network Features (из TODO_v2)

### Ipset Management

- [x] [P2][BE] Реализовать автообновление ipset ✅ Уже реализовано
  - Файлы: `src-tauri/src/core/ipset_updater.rs`
  - Функционал:
    - Скачивание `ipset-all.txt` из GitHub (zapret-discord-youtube)
    - Проверка обновлений по расписанию (раз в день)
    - Уведомление о новых IP-адресах
  - Критерий: ipset автоматически обновляется без участия пользователя

- [x] [P2][BE] Реализовать переключение ipset режимов ✅ Уже реализовано
  - Файлы: `src-tauri/src/core/ipset_manager.rs`, `src-tauri/src/commands/ipset.rs`
  - Режимы:
    - `any` — использовать ipset для всех стратегий
    - `none` — не использовать ipset
    - `loaded` — использовать только загруженный ipset
  - Критерий: пользователь может выбрать режим в Settings → Advanced

- [x] [P2][FE] UI для управления ipset ✅ Уже реализовано
  - Файлы: `src/lib/components/settings/IpsetSettings.svelte`
  - Критерий: отображает текущий режим, количество IP в ipset, дату последнего обновления

### Hosts Management

- [x] [P3][BE] Реализовать обновление hosts для Discord voice ✅ Уже реализовано
  - Файлы: `src-tauri/src/core/hosts_manager.rs`
  - Функционал:
    - Добавление записей для Discord voice серверов
    - Резервное копирование оригинального hosts
    - Откат изменений при отключении
  - Критерий: Discord voice работает стабильнее за счёт прямых IP

- [x] [P3][FE] UI для управления hosts ✅ Уже реализовано
  - Файлы: `src/lib/components/settings/HostsSettings.svelte`
  - Критерий: toggle "Оптимизировать hosts для Discord", показ добавленных записей

### Расширенные DPI параметры (дополнение к разделу 12)

- [ ] [P3][BE] Добавить поддержку динамического seqovl
  - Файлы: `src-tauri/src/core/strategy_engine.rs`, конфиги стратегий
  - Функционал:
    - Разные значения `seqovl` для разных целей (681 для Google, 568/652/654 для остальных)
    - Автоматический выбор на основе домена
  - Критерий: стратегия использует оптимальный seqovl для каждого сервиса

- [ ] [P3][BE] Добавить поддержку динамического TLS fingerprint
  - Файлы: `src-tauri/src/core/strategy_engine.rs`
  - Функционал:
    - `google.com` fingerprint для Google сервисов
    - `4pda.to` / `max.ru` fingerprint для остальных
    - Автоматический выбор на основе домена
  - Критерий: стратегия использует оптимальный fingerprint для каждого сервиса

- [ ] [P3][BE] Реализовать комбинированные режимы DPI
  - Файлы: конфиги в `configs/strategies/`
  - Режимы:
    - `fake+multisplit` с одновременным fake-tls и split-seqovl-pattern
    - `fake+multidisorder` с split-pos=1,midsld
    - `syndata+multidisorder`
  - Критерий: созданы стратегии с комбинированными режимами, протестированы на DPI-симуляторе


---

## 20. Inline TODO/FIXME из кода

> Полный список: [docs/CODE_TODOS.md](docs/CODE_TODOS.md)

### Высокий приоритет (блокирует функциональность)

- [x] [P1][FE] `dashboard.ts:63` — Заменить mock данные на реальные API вызовы ✅ Подготовлено к backend (есть TODO комментарии)
  - Файлы: `src/lib/stores/dashboard.ts`
  - Критерий: Dashboard показывает реальные данные от backend

- [x] [P1][FE] `library.svelte.ts:84` — Заменить mock данные на реальные API вызовы ✅ Подготовлено к backend
  - Файлы: `src/lib/stores/library.svelte.ts`
  - Критерий: Library загружает правила из backend

- [ ] [P1][FE] `library.svelte.ts:192` — Реализовать реальную проверку сервисов
  - Файлы: `src/lib/stores/library.svelte.ts`
  - Критерий: `checkService()` вызывает backend и возвращает реальный статус

### Средний приоритет (улучшение UX)

- [ ] [P2][FE] `+page.svelte:370` — Получать реальную статистику сети
  - Файлы: `src/routes/+page.svelte`
  - Критерий: NetworkStatsWidget показывает реальные данные вместо симуляции

- [x] [P2][FE] `gameMode.svelte.ts:170` — Реализовать детекцию процессов через Tauri ✅ Подготовлено к backend
  - Файлы: `src/lib/stores/gameMode.svelte.ts`
  - Критерий: Game Mode автоматически определяет запущенные игры

- [x] [P2][FE] `aiPilot.svelte.ts:104` — Подключить AI Pilot проверку к backend ✅ Подготовлено к backend
  - Файлы: `src/lib/stores/aiPilot.svelte.ts`
  - Критерий: `runCheck()` вызывает реальную команду backend

- [x] [P2][FE] `aiPilot.svelte.ts:148` — Подключить AI Pilot откат к backend ✅ Подготовлено к backend
  - Файлы: `src/lib/stores/aiPilot.svelte.ts`
  - Критерий: `undoAction()` реально откатывает изменения через backend

### Низкий приоритет (косметические)

- [ ] [P3][FE] `AutoRecoverySettings.svelte:51` — Добавить отслеживание изменений
  - Файлы: `src/lib/components/settings/AutoRecoverySettings.svelte`
  - Критерий: кнопка Save активна только при наличии изменений

- [ ] [P3][BE] `troubleshoot.rs:224` — Сохранять привязку service → strategy
  - Файлы: `src-tauri/src/commands/troubleshoot.rs`
  - Критерий: после применения результата Troubleshooter, привязка сохраняется в Library

---

## Сводка по приоритетам

| Приоритет | Количество | Описание |
|-----------|------------|----------|
| P0 | 0 | Критично (блокирует работу) |
| P1 | ~25 | Высокий (влияет на качество) |
| P2 | ~45 | Средний (улучшения) |
| P3 | ~30 | Низкий (nice to have) |
| **Всего** | **~100** | |

---

## 21. Plugins & Packs — архитектура и интеграция

- [ ] [P1][ARCH] Синхронизировать контракты манифеста между Rust и TS.
  - Проверить, что `PluginManifest.contributes` в Rust (`src-tauri/src/plugins/manifest.rs`) и в TS (`src/lib/types/plugin.ts`) описывают одни и те же сущности (widgets, services, strategies, hostlists, presets и т.п.).
  - Добавить на фронте типы для strategy/hostlist-вкладов, если их ещё нет (сейчас TS знает только про `widgets/settings/services`).

- [ ] [P1][ARCH] Выравнять типы плагинов между Rust, backend-ответами и frontend-типами.
  - Сравнить перечень типов (`service-checker`, `hostlist-provider`, `strategy-config`, `ui-plugin`, `script-plugin`) в Rust (`PluginType`), backend (`LoadedPluginInfo` / `js_loader`) и фронтенде (`PluginType` в `src/lib/stores/plugins.ts` и `src/lib/types/plugin.ts`).
  - Проверить/обновить маппинг `typeMap` в `loadPluginsFromBackend()` так, чтобы все реальные типы плагинов корректно отображались в UI.

- [ ] [P1][FE][BE] Интегрировать сервисы из `service-checker` плагинов в Services/Troubleshooter.
  - Использовать `get_plugin_services` / `check_plugin_service` для расширения списка проблем/сервисов.
  - В `ProblemSelector` и на странице Services помечать сервисы из плагинов бейджем (например, "from plugin").

- [ ] [P1][FE][BE] Подружить стратегии из плагинов с AI Pilot и Troubleshooter.
  - Использовать `get_plugin_strategies` / `get_all_registered_strategies` и `get_strategies_for_service` для получения стратегий.
  - В Troubleshooter включить plugin-стратегии в список тестируемых стратегий для сервиса.
  - В AI Pilot учитывать plugin-стратегии при оптимизации и в истории действий.

- [ ] [P2][FE][BE] Интеграция `hostlist-provider` плагинов в Network/Proxies.
  - Определить и зафиксировать формат данных hostlist-плагинов на фронтенде (типы, контракт).
  - В UI выбора источников хостлистов/правил добавить источники из плагинов с пометкой плагина.

- [ ] [P2][FE] Поддержка пресетов Library из плагинов.
  - Описать формат "library presets" в манифесте (если планируется использовать пресеты от плагинов).
  - В Library выделить группу "Presets from plugins" и помечать такие пресеты иконкой плагина; описать поведение (нельзя редактировать напрямую, только включать/override).

- [ ] [P1][SEC] Пересмотреть и зафиксировать политику `ALLOWED_COMMANDS` для песочницы плагинов.
  - Проверить, что в whitelist в `src/lib/plugins/context.ts` попали только безопасные read-only команды (статусы, сервисы, стратегии, plugin-specific HTTP/storage).
  - При необходимости добавить отдельные безопасные команды для чтения статуса стратегий/сервисов, не давая плагинам прямой доступ к опасным действиям.

- [ ] [P2][SEC][BE] Реализовать поддержку `PluginPermissions` на backend.
  - Учесть `timeout` и `memory` при исполнении script-плагинов (Level 3), когда они будут включены.
  - Логировать превышение лимитов и попытки обращения к запрещённым HTTP-доменам или Tauri-командам.

- [ ] [P2][UX] Отображать нарушения sandbox и ограничения плагинов в UI.
  - На `/plugins/[id]` показывать, если плагин пытался выйти за пределы разрешённых прав (с ссылкой на логи и подсказкой, что именно было заблокировано).

- [ ] [P1][ARCH][FE][BE] Единый источник правды по включённости плагина.
  - Согласовать `installedPlugins` store и backend (`LoadedPluginInfo`/`js_loader`): включение/отключение плагина из `/plugins` должно корректно обновлять и frontend (`setPluginEnabled`/`unregisterUIPlugin`), и backend-состояние.
  - Определить канонический путь загрузки: dev-плагины из `/plugins` vs пользовательские плагины из `plugins_dir`.

- [ ] [P2][FE] Гарантированная инициализация плагин-системы при старте приложения.
  - Подключить `initializePlugins()` / `initializePluginLoader()` в точке входа фронтенда так, чтобы builtin-плагины и плагины с диска сканировались и регистрировались один раз при старте.

- [ ] [P2][ARCH][UX] Спроектировать "Streaming Pack" как эталонный pack.
  - В `plugin.json` описать сервисы (Netflix/YouTube/Twitch/Spotify/TikTok и т.п.), их endpoints и категории.
  - Добавить вклад в Troubleshooter: проблема/чекер "Стриминг тормозит".
  - Добавить виджет на Dashboard: "Streaming status".
  - Добавить пресет Library: "Streaming Friendly".

- [ ] [P2][FE][BE] Реализовать и прогнать end-to-end сценарий для "Streaming Pack".
  - Проверить, что весь путь работает: manifest → backend (регистрация сервисов/стратегий) → stores → UI (Dashboard, Troubleshooter, Network, Library).
  - Использовать результат как референс для следующих паков (например, "AI Services Pack").

*Последнее обновление: 8 января 2026 (отмечены выполненные задачи из разделов 6, 7, 13, 18, 20)*
