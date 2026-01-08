# 🎨 Isolate v3 — Полный план редизайна UI

> Дата создания: 8 января 2026
> Статус: Планирование
> Общая оценка: **8-12 недель** работы

---

## 📋 Оглавление

1. [Dashboard Redesign](#-dashboard-redesign)
2. [Library (Services + Strategies)](#-library-services--strategies)
3. [Orchestra → Troubleshooter](#-orchestra--troubleshooter)
4. [Network → Proxy & VPN](#-network--proxy--vpn)
5. [UX Features](#-ux-features)
6. [Architecture & Infrastructure](#-architecture--infrastructure)
7. [Приоритеты и порядок выполнения](#-приоритеты-и-порядок-выполнения)

---

## 🏠 Dashboard Redesign

### Концепция
Центр управления полетом с большим индикатором состояния, live activity и быстрым переключением режимов.

### Компоненты

#### Новые
- [x] **[L]** `ShieldIndicator.svelte` — центральная кнопка-индикатор ✅
  - Статусы: 🟢 Protected, 🟡 Bypassing, 🔴 Issues
  - Анимации: pulse-glow, wave-ripple, shake-attention
  - Размер: ~200x200px, адаптивный

- [x] **[M]** `LiveActivityPanel.svelte` — панель активности ✅
  - Мини-график трафика (Download/Upload)
  - Список активных соединений
  - Обновление каждые 1-2 секунды

- [x] **[S]** `ModeSelector.svelte` — переключатель режимов ✅
  - Auto | TUN | Proxy Only
  - Segmented control с подсветкой

- [x] **[S]** `TrafficChart.svelte` — компактный график трафика ✅
  - Area chart с градиентом
  - Чистый SVG без зависимостей

- [x] **[S]** `ActiveConnectionItem.svelte` — элемент списка соединений ✅

#### Модификация
- [ ] **[M]** Рефакторинг `StatusWidget.svelte` → интеграция с ShieldIndicator
- [ ] **[S]** Обновить `+page.svelte` (Dashboard) — новый layout

### Stores
- [x] **[M]** `src/lib/stores/dashboard.ts` ✅
  ```typescript
  interface DashboardState {
    protectionStatus: 'protected' | 'bypassing' | 'issues' | 'disabled';
    issues: Issue[];
    currentMode: 'auto' | 'tun' | 'proxy';
    activeConnections: ActiveConnection[];
    trafficHistory: TrafficPoint[];
  }
  ```

- [x] **[S]** `src/lib/stores/trafficMonitor.ts` — real-time данные ✅

### Backend (Tauri Commands)
- [ ] **[L]** `get_live_connections` — активные соединения с методами обхода
- [ ] **[M]** `get_traffic_stats` — статистика трафика
- [ ] **[M]** `get_protection_issues` — проблемы требующие внимания
- [ ] **[S]** `set_operation_mode` — установка режима (Auto/TUN/Proxy)
- [ ] **[S]** `fix_issue` — автоматическое исправление проблемы

### События
- [ ] **[M]** `traffic:update` — обновление трафика (каждые 1-2 сек)
- [ ] **[S]** `connection:opened` / `connection:closed`
- [ ] **[S]** `issue:detected` / `issue:resolved`

---

## 📚 Library (Services + Strategies)

### Концепция
Объединение Services и Strategies в единый интерфейс. Каждый сервис — это "правило" с выбором метода доступа.

### Компоненты

#### Новые
- [x] **[L]** `LibraryPage.svelte` — новая страница `/library` ✅
  - Поиск и фильтры
  - Список ServiceRuleCard
  - Кнопка "Add Rule"

- [x] **[L]** `ServiceRuleCard.svelte` — карточка сервиса/правила ✅
  ```
  [Icon] [Name + Status]                    [Method Dropdown ▼]
         └─ Accessible • 45ms               Direct | Auto | Strategy...
  ```

- [x] **[M]** `MethodDropdown.svelte` — выбор метода доступа ✅
  - Группы: Direct, Auto-Strategy, Strategies, VLESS, Proxies, Tor

- [x] **[M]** `AddRuleModal.svelte` — добавление нового правила ✅
  - Ввод домена с валидацией
  - Smart Strategy suggestion

- [ ] **[S]** `SmartStrategySuggestion.svelte` — предложение стратегии
- [x] **[S]** `ServiceStatusBadge.svelte` — бейдж статуса ✅
- [x] **[S]** `LibraryFilters.svelte` — фильтры по статусу/методу/категории ✅

### Stores
- [x] **[L]** `src/lib/stores/library.svelte.ts` ✅
  ```typescript
  interface LibraryState {
    rules: ServiceRule[];
    filters: { search, status, method, category };
    availableStrategies: Strategy[];
    availableProxies: Proxy[];
    availableVlessProfiles: VlessProfile[];
  }
  ```

### Backend (Tauri Commands)
- [ ] **[L]** `get_library_rules` — все правила с методами
- [ ] **[M]** `set_rule_method` — установка метода для сервиса
- [ ] **[M]** `add_library_rule` — добавление нового правила
- [ ] **[S]** `remove_library_rule` — удаление правила
- [ ] **[M]** `suggest_strategy_for_domain` — умное предложение стратегии

### Миграция
- [ ] **[M]** Миграция конфигурации сервисов → LibraryRule
- [ ] **[S]** Настроить redirects: `/services` → `/library`

---

## 🚀 Orchestra → Troubleshooter

### Концепция
Превращение Orchestra в user-friendly Troubleshooter с двумя сценариями:
1. **"У меня не работает"** — визуальный мастер диагностики
2. **"AI Pilot"** — фоновая автоматическая оптимизация

### Компоненты

#### Troubleshoot Wizard
- [x] **[XL]** `TroubleshootWizard.svelte` — пошаговый мастер ✅
  - Step 1: Выбор проблемного сервиса
  - Step 2: Визуальный тест стратегий (как спидтест)
  - Step 3: Результаты с рекомендацией

- [x] **[L]** `ProblemSelector.svelte` — выбор проблемы ✅
  - Карточки: "YouTube тормозит", "Discord не подключается"
  - Группировка: Видео, Мессенджеры, Соцсети, Игры

- [x] **[XL]** `StrategySpeedtest.svelte` — визуальный тест стратегий ✅
  - Анимация "гонки" стратегий
  - Прогресс-бар с градиентом
  - Реалтайм latency

- [x] **[M]** `StrategyRaceItem.svelte` — элемент "гонки" ✅
- [x] **[M]** `ResultsRecommendation.svelte` — результаты и рекомендация ✅

#### AI Pilot
- [x] **[L]** `AIPilotPanel.svelte` — панель AI Pilot ✅
  - Toggle "Фоновая оптимизация"
  - Настройки интервала (30мин/1час/2часа)
  - Лог последних действий

- [x] **[M]** `AIPilotNotification.svelte` — уведомление о переключении ✅

### Stores
- [x] **[M]** `troubleshoot.svelte.ts` — состояние Troubleshooter ✅
- [x] **[M]** `aiPilot.svelte.ts` — состояние AI Pilot ✅

### Backend (Tauri Commands)
- [x] **[L]** `troubleshoot_service` — запуск диагностики ✅
- [x] **[M]** `apply_troubleshoot_result` — применить результат ✅
- [x] **[S]** `get_troubleshoot_problems` — список проблем ✅
- [ ] **[L]** `start_ai_pilot` / `stop_ai_pilot` — управление AI Pilot
- [ ] **[M]** `get_ai_pilot_history` — история действий

### События
- [ ] **[S]** `troubleshoot:strategy_progress` / `troubleshoot:strategy_result`
- [ ] **[S]** `ai_pilot:strategy_changed` / `ai_pilot:check_complete`

---

## 🌐 Network → Proxy & VPN

### Концепция
1. **Карточки с флагами** вместо таблицы прокси
2. **Большая зона импорта** — "Paste key here"
3. **Визуальный конструктор цепочек** — Chain Builder

### Компоненты

#### Proxy Grid
- [x] **[L]** `ProxyCardGrid.svelte` — сетка карточек прокси ✅
- [x] **[M]** `ProxyCountryCard.svelte` — карточка с флагом страны ✅
- [x] **[M]** `CountryFlag.svelte` — компонент флага ✅
- [x] **[S]** `ProtocolBadge.svelte` — badge протокола (VLESS/VMess/SS) ✅
- [x] **[M]** `LatencyIndicator.svelte` — индикатор задержки ✅

#### Import Zone
- [x] **[XL]** `ImportZone.svelte` — большая зона импорта ✅
  - Drag & drop область
  - Textarea "Paste key here"
  - Автодетект: vless://, ss://, vmess://, Sing-box JSON
  - Batch import

- [x] **[S]** `ImportPreview.svelte` — превью импортируемого прокси ✅

#### Chain Builder
- [x] **[XL]** `ChainBuilder.svelte` — визуальный конструктор цепочек ✅
  - Drag & drop блоки
  - Типы: DPI Bypass → Proxy → Internet
  - Соединительные линии

- [x] **[M]** `ChainBlock.svelte` — блок в конструкторе ✅
- [x] **[M]** `ChainConnection.svelte` — соединение между блоками ✅
- [x] **[L]** `ChainPresets.svelte` — пресеты цепочек ✅

### Stores
- [x] **[M]** `proxyChain.svelte.ts` — состояние конструктора ✅
- [x] **[S]** `proxyImport.svelte.ts` — состояние импорта ✅

### Backend (Tauri Commands)
- [ ] **[M]** `parse_proxy_url` — парсинг URL без сохранения
- [ ] **[M]** `batch_import_proxies` — batch импорт
- [ ] **[L]** `save_proxy_chain` / `apply_proxy_chain` — цепочки
- [ ] **[M]** `detect_proxy_country` — определение страны

---

## 🎮 UX Features

### Command Palette (Ctrl+K)
- [ ] **[M]** Расширить `CommandPalette.svelte`
  - Категории: services, strategies, profiles
  - Динамическая загрузка сервисов/стратегий
  - Быстрые действия: "Switch to TUN mode", "Game Mode On/Off"

- [ ] **[S]** Подсветка совпадений в fuzzy search
- [x] **[S]** Store `commandPalette.svelte.ts` ✅

### Tray Menu
- [ ] **[L]** Расширить `tray.rs`
  - Submenu сервисов с checkbox (топ-5)
  - Submenu профилей: Game Mode / Work Mode
  - "Rescan Network" пункт

- [ ] **[M]** Backend команды для управления защитой сервисов

### Toast Notifications
- [ ] **[M]** Расширить toast store
  - Дедупликация и группировка ошибок
  - Progress toast с обновлением

- [x] **[S]** `errorMessages.ts` — маппинг технических ошибок ✅
- [x] **[M]** `SmartToast.svelte` — toast с action buttons ✅

### Game Mode
- [ ] **[L]** `game_detector.rs` — определение запущенных игр
  - Список: cs2.exe, dota2.exe, valorant.exe, etc.
  - Использовать `sysinfo` crate

- [ ] **[M]** `game_monitor.rs` — фоновый мониторинг (каждые 5 сек)
- [ ] **[M]** Автоматическое переключение режима
- [x] **[S]** `GameModeIndicator.svelte` — иконка 🎮 в header ✅
- [x] **[S]** Store `gameMode.svelte.ts` ✅
- [ ] **[M]** Настройки Game Mode в Settings

### Sidebar + Bottom Drawer
- [x] **[M]** Создан `SidebarNew.svelte` с новой структурой ✅
  ```
  Main:    Dashboard, Library
  Tools:   Boost, Proxy & VPN
  System:  Plugins, Settings
  ```

- [x] **[L]** `BottomDrawer.svelte` — выезжающая панель для логов ✅
  - Drag handle для изменения размера
  - Интеграция с LogsContent

- [x] **[S]** Store `bottomDrawer.svelte.ts` ✅
- [x] **[S]** Store `navigation.svelte.ts` ✅
- [x] **[S]** `NavItem.svelte`, `NavGroup.svelte`, `LogsButton.svelte` ✅

---

## 🏗️ Architecture & Infrastructure

### State Machines
- [x] **[L]** `stateMachine.ts` — универсальный state machine ✅
  - States: Idle → Loading → Active → Error → Recovering
  - Transitions с валидацией

- [x] **[M]** `protectionMachine.ts` — state machine для защиты ✅
- [x] **[M]** `serviceMachine.ts` — state machine для сервисов ✅
- [x] **[M]** `useStateMachine.svelte.ts` — хук для Svelte 5 ✅

### Компоненты-Slot'ы
- [ ] **[L]** `LibraryCard.svelte` — универсальная карточка с Snippets
- [ ] **[M]** `PresetCard.svelte` / `CustomRuleCard.svelte` — на основе LibraryCard
- [ ] **[S]** `StatusIndicator.svelte` — индикатор с поддержкой всех состояний

### Виртуализация
- [x] **[L]** `VirtualList.svelte` — виртуальный список ✅
- [x] **[M]** `VirtualGrid.svelte` — виртуальная сетка ✅
- [x] **[S]** `useVirtualScroll.svelte.ts` — хук для виртуализации ✅

### Event Bus
- [x] **[M]** `eventBus.svelte.ts` — централизованный Event Bus ✅
- [x] **[M]** `useEvent.svelte.ts` — хук для подписок ✅
- [ ] **[S]** Расширить типы событий

### Миграция данных
- [ ] **[L]** `src-tauri/src/core/library/` — новый модуль Library
- [ ] **[M]** Модели `LibraryItem`, `LibraryItemType`
- [ ] **[M]** Миграция существующих данных

### Рефакторинг Stores → Svelte 5 Runes
- [ ] **[M]** `stores.ts` → class с $state
- [ ] **[S]** `logs.ts`, `plugins.ts`, `toast.ts`, `theme.ts` → runes

---

## 📊 Приоритеты и порядок выполнения

### Фаза 1: Инфраструктура (1-2 недели)
1. State Machines
2. Event Bus
3. Базовые компоненты Library
4. Рефакторинг stores на runes

### Фаза 2: Dashboard + Library (2-3 недели)
1. ShieldIndicator + ModeSelector
2. LiveActivityPanel + TrafficChart
3. ServiceRuleCard + MethodDropdown
4. LibraryPage + AddRuleModal
5. Backend commands

### Фаза 3: Troubleshooter (2 недели)
1. ProblemSelector + StrategySpeedtest
2. TroubleshootWizard
3. AIPilotPanel
4. Backend commands

### Фаза 4: Proxy & VPN (2 недели)
1. ProxyCardGrid + ProxyCountryCard
2. ImportZone
3. ChainBuilder
4. Backend commands

### Фаза 5: UX Features (1-2 недели)
1. Command Palette улучшения
2. Tray Menu расширение
3. Toast Notifications
4. Game Mode
5. Sidebar + Bottom Drawer

### Фаза 6: Полировка (1 неделя)
1. Анимации и transitions
2. Тесты
3. Документация
4. Bug fixes

---

## 📈 Сводка по задачам

| Раздел | S | M | L | XL | Часы |
|--------|---|---|---|----|----|
| Dashboard | 4 | 4 | 2 | 0 | ~20 |
| Library | 4 | 4 | 3 | 0 | ~26 |
| Troubleshooter | 4 | 6 | 4 | 2 | ~40 |
| Proxy & VPN | 3 | 6 | 3 | 2 | ~35 |
| UX Features | 5 | 6 | 3 | 0 | ~28 |
| Architecture | 4 | 8 | 4 | 0 | ~30 |
| **Итого** | **24** | **34** | **19** | **4** | **~180** |

**Общая оценка: 180-220 часов** (8-12 недель при 20ч/неделю)

---

## 📝 Файлы для создания

### Frontend Components
```
src/lib/components/
├── dashboard/
│   ├── ShieldIndicator.svelte
│   ├── LiveActivityPanel.svelte
│   ├── ModeSelector.svelte
│   ├── TrafficChart.svelte
│   └── ActiveConnectionItem.svelte
├── library/
│   ├── LibraryCard.svelte
│   ├── ServiceRuleCard.svelte
│   ├── MethodDropdown.svelte
│   ├── AddRuleModal.svelte
│   ├── SmartStrategySuggestion.svelte
│   └── LibraryFilters.svelte
├── troubleshoot/
│   ├── TroubleshootWizard.svelte
│   ├── ProblemSelector.svelte
│   ├── StrategySpeedtest.svelte
│   ├── StrategyRaceItem.svelte
│   ├── ResultsRecommendation.svelte
│   ├── AIPilotPanel.svelte
│   └── AIPilotNotification.svelte
├── proxy/
│   ├── ProxyCardGrid.svelte
│   ├── ProxyCountryCard.svelte
│   ├── ImportZone.svelte
│   ├── ChainBuilder.svelte
│   ├── ChainBlock.svelte
│   └── ChainPresets.svelte
├── virtual/
│   ├── VirtualList.svelte
│   └── VirtualGrid.svelte
├── BottomDrawer.svelte
├── SmartToast.svelte
└── GameModeIndicator.svelte
```

### Frontend Stores
```
src/lib/stores/
├── dashboard.ts
├── library.svelte.ts
├── troubleshoot.svelte.ts
├── aiPilot.svelte.ts
├── proxyChain.svelte.ts
├── commandPalette.svelte.ts
└── bottomDrawer.svelte.ts
```

### Frontend State
```
src/lib/state/
├── stateMachine.ts
├── protectionMachine.ts
├── serviceMachine.ts
└── types.ts
```

### Backend (Rust)
```
src-tauri/src/core/
├── library/
│   ├── mod.rs
│   ├── models.rs
│   ├── manager.rs
│   └── migration.rs
├── game_detector.rs
└── game_monitor.rs

src-tauri/src/commands/
├── library.rs
├── troubleshoot.rs
├── ai_pilot.rs
├── chain.rs
└── game_mode.rs
```

---

*Документ создан: 8 января 2026*
*Последнее обновление: 8 января 2026*
