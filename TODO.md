# Isolate — Master Plan & Progress

> Дата: 2026-01-05
> Концепция: **Modular Ecosystem** — OS-подобный интерфейс уровня VS Code/Figma

---

# 🚀 ISOLATE 2.0: THE MODULAR ECOSYSTEM

## Концепция
Не "админка с вкладками", а **кокпит космического корабля** с системой плагинов.
Референсы: VS Code, Linear, Figma, Arc Browser, Star Citizen UI.

---

# 🎨 PHASE 1: Visual Language — "Glass & Void"

## 1.1 Цветовая палитра (tailwind.config.js)
```
void:
  DEFAULT: #050505    ← База (почти чёрный)
  50:      #0F1012    ← Surface (карточки)
  100:     #1A1D24    ← Elevated (модалки)
  200:     #252830    ← Hover states
  300:     #2E323C    ← Active states

glass:
  border:        rgba(255,255,255,0.03)   ← Едва заметные
  border-active: rgba(255,255,255,0.08)   ← При hover/focus
  surface:       rgba(15,16,18,0.8)       ← Полупрозрачные панели

electric:
  DEFAULT: #3B82F6                        ← Primary Blue
  glow:    rgba(59,130,246,0.5)           ← Glow эффект
  dim:     rgba(59,130,246,0.2)           ← Subtle accent

neon:
  green:  #22C55E    ← Success
  red:    #EF4444    ← Error
  yellow: #F59E0B    ← Warning
  cyan:   #00D4FF    ← Accent

text:
  primary:   #F3F4F6
  secondary: #9CA3AF
  muted:     #6B7280
```

### ✅ Реализовано:
- [x] tailwind.config.js обновлён с полной палитрой
- [x] Glow shadows (shadow-glow, shadow-glow-lg, shadow-glow-green/red/cyan)
- [x] Анимации (pulse-glow, slide-up, slide-down, fade-in)
- [x] Шрифты: Inter/Geist (sans), JetBrains Mono/Geist Mono (mono)

### ⏳ TODO:
- [ ] Windows Mica/Acrylic эффект для sidebar (требует Tauri window API)
- [ ] Кастомные window controls (убрать системные)

---

## 1.2 Типографика
- **UI:** `font-sans` — Inter / Geist Sans
- **Code/Data:** `font-mono` — JetBrains Mono / Geist Mono (IP, порты, логи)
- **Weights:** 600 (заголовки), 500 (кнопки), 400 (текст)

---

## 1.3 Эффекты
- **Glow:** `shadow-glow` = `0 0 20px -5px rgba(59, 130, 246, 0.5)`
- **Borders:** `border-glass-border` = `1px solid rgba(255,255,255,0.03)`
- **Radius:** `rounded-lg` (8px), `rounded-xl` (12px), `rounded-2xl` (16px)
- **Backdrop blur:** `backdrop-blur-md` для модалок

---

# 🏗 PHASE 2: Layout Architecture — "Three-Pane Layout"

## 2.1 Структура
```
┌──────┬─────────────────────────────────────────────────┐
│      │                                                 │
│  S   │              Main Content Area                  │
│  I   │                                                 │
│  D   │   Dashboard / Services / Routing / Proxies     │
│  E   │                                                 │
│  B   │                                                 │
│  A   │                                                 │
│  R   │                                                 │
│      │                                                 │
├──────┴─────────────────────────────────────────────────┤
│              Terminal / Logs Panel (collapsible)       │
└────────────────────────────────────────────────────────┘
```

### ✅ Реализовано:
- [x] `+layout.svelte` — новый Three-Pane layout
- [x] `Sidebar.svelte` — collapsible (60px/200px), три секции
- [x] `TerminalPanel.svelte` — выезжающая панель логов (Ctrl+`)
- [x] `ResizablePanelGroup.svelte` — система resizable панелей
- [x] `ResizablePanel.svelte` — панель с min/max/collapse
- [x] `ResizableHandle.svelte` — ручка для resize
- [x] Top Bar с breadcrumb и status indicator

### ✅ Реализовано:
- [x] Master-Detail view для Services страницы
- [x] Configure modal для сервисов

### ⏳ TODO:
- [ ] Сохранение layout в localStorage (частично работает)

---

## 2.2 Sidebar
**Верх (Navigation):**
- Dashboard, Services, Routing, Proxies

**Центр (Plugins Area):**
- Динамические иконки из `installedPlugins` store

**Низ (System):**
- Marketplace, Settings, Logs

### ✅ Реализовано:
- [x] Collapsed/Expanded toggle с анимацией
- [x] Active state для текущего route
- [x] Lucide-style SVG иконки
- [x] Plugins section из store

---

# 🖥 PHASE 3: Functional Blocks

## 3.1 Dashboard — "Bento Grid System"

### ✅ Реализовано:
- [x] `BentoGrid.svelte` — контейнер сетки (columns, gap)
- [x] `BentoWidget.svelte` — виджет с colspan/rowspan
- [x] `StatusWidget.svelte` — Global Status с BigToggleButton + glow
- [x] `HealthWidget.svelte` — Health Monitor с ping и индикаторами
- [x] `MethodWidget.svelte` — Active Method (zapret/vless/proxy/direct)
- [x] `QuickActionsWidget.svelte` — Quick Actions grid 2x2

### ⏳ TODO:
- [ ] Drag-n-Drop для виджетов (dnd-kit аналог)
- [ ] Сохранение layout виджетов в localStorage
- [ ] Plugin Widgets slot

---

## 3.2 Services — "Master-Detail View"

### ✅ Реализовано:
- [x] Master-Detail layout (левая панель — список, правая — детали)
- [x] Список сервисов с иконками и статусами
- [x] Detail panel с информацией о выбранном сервисе
- [x] Configure modal с настройками сервиса
- [x] Add Custom Service modal

### ⏳ TODO:
- [ ] Real-time ping график
- [ ] Логи только этого сервиса

---

## 3.3 Routing — "Visual Flow Builder"

### ✅ Реализовано:
- [x] Visual Flow карточки (Source → Condition → Action)
- [x] Add/Edit Rule modal с preview
- [x] Toggle enable/disable для правил
- [x] Stats bar (Total, Active, Proxied, Blocked)

### ⏳ TODO:
- [ ] Drag-n-drop сортировка правил
- [ ] JSON editor с подсветкой

---

## 3.4 Proxies — "Wallet View"

### ✅ Реализовано:
- [x] `ProxyCard.svelte` — карточка прокси
- [x] Card-based list view
- [x] Add Modal с табами
- [x] Auto-paste из буфера

### ✅ Реализовано:
- [x] Флаги стран (emoji flags)
- [x] Subscription import modal

### ⏳ TODO:
- [ ] Drag-n-drop сортировка

---

## 3.5 Terminal — "Developer Console"

### ✅ Реализовано:
- [x] `TerminalPanel.svelte` — выезжающая панель
- [x] `logs` store с методами error/warn/info/debug/success
- [x] Цветная подсветка уровней
- [x] Фильтрация по level/source/search
- [x] Auto-scroll с toggle
- [x] Copy/Clear функции
- [x] Resize по высоте
- [x] Keyboard shortcut (Ctrl+`)

---

# ⚡️ PHASE 4: UX Patterns (AAA Quality)

## 4.1 Command Palette (Ctrl+K)

### ✅ Реализовано:
- [x] `CommandPalette.svelte` — модальное окно
- [x] Fuzzy search по командам
- [x] Категории: Navigation, Actions, Settings
- [x] Keyboard navigation (↑↓ + Enter)
- [x] Shortcut hints справа
- [x] Glass & Void дизайн

### Команды:
- Go to Dashboard/Diagnostics/Proxies/Settings
- Start/Stop Protection
- Add Proxy
- Test Connection
- Panic Reset
- Toggle Theme

---

## 4.2 Context Menus

### ✅ Реализовано:
- [x] `ContextMenu.svelte` — контейнер меню
- [x] `ContextMenuItem.svelte` — элемент с icon/shortcut
- [x] `ContextMenuSeparator.svelte` — разделитель
- [x] Позиционирование у курсора
- [x] Закрытие по Escape/клик вне
- [x] Danger variant
- [x] Анимация появления (scale + opacity)

---

## 4.3 States & Feedback

### ✅ Реализовано:
- [x] Toast notifications (`Toast.svelte`, `ToastContainer.svelte`)
- [x] Loading states в BigToggleButton
- [x] Glow эффекты при активации
- [x] Skeleton loaders (`Skeleton.svelte`, `SkeletonCard.svelte`, `SkeletonList.svelte`)
- [x] Page transitions (`PageTransition.svelte` — fly + fade)
- [x] Dashboard skeleton (`DashboardSkeleton.svelte`)

### ⏳ TODO:
- [ ] Scanning states с бегущими строками

---

## 4.4 Keyboard Shortcuts

### ✅ Реализовано:
- [x] `Ctrl+K` — Command Palette
- [x] `Ctrl+\`` — Toggle Terminal
- [x] `Escape` — Close modals/panels
- [x] `Ctrl+1-4` — Switch panels (Dashboard, Services, Routing, Proxies)

---

# � PHASEК 5: Plugin Architecture

### ✅ Реализовано:
- [x] `installedPlugins` store с демо-данными
- [x] Sidebar отображает плагины из store

### ⏳ TODO:
- [ ] `PluginSlot.svelte` — слоты для UI плагинов
- [ ] Plugin Manifest система
- [ ] Marketplace UI
- [ ] Plugin settings интеграция

---

# 🔧 ТЕХНИЧЕСКИЙ АУДИТ

## 🔴 КРИТИЧЕСКИЕ ПРОБЛЕМЫ

### ~~1. Блокирующий `std::sync::Mutex` в async контексте~~ ✅ ИСПРАВЛЕНО
**Файл:** `src-tauri/src/core/storage.rs`
**Решение:** Заменён `std::sync::Mutex` на `tokio::sync::Mutex`

### 2. Две системы стратегий (JSON vs YAML)
**Статус:** ⏳ TODO — унифицировать в JSON

---

## 🟠 ВЫСОКИЙ ПРИОРИТЕТ

### 3. Монолитный commands/mod.rs
**Было:** 2,777 строк, 78 команд
**Стало:** Разбит на модули

### ✅ Созданные модули:
- [x] `commands/vless.rs` — 13 VLESS команд
- [x] `commands/proxies.rs` — 9 proxy команд
- [x] `commands/hostlists.rs` — 8 hostlist команд
- [x] `commands/settings.rs` — 6 settings команд
- [x] `commands/diagnostics.rs`
- [x] `commands/logs.rs`
- [x] `commands/quic.rs`
- [x] `commands/routing.rs`
- [x] `commands/system.rs`
- [x] `commands/tray.rs`
- [x] `commands/updates.rs`

### ✅ Реализовано:
- [x] Унифицированный `IsolateError` тип (`src-tauri/src/core/errors.rs`)
- [x] Serialize для Tauri IPC с kind + message
- [x] From implementations для std::io, serde_yaml, serde_json, reqwest, rusqlite, anyhow
- [x] Helper constructors (config, strategy, process, network, validation, tauri, other)
- [x] Unit тесты для всех вариантов

---

## 🟡 СРЕДНИЙ ПРИОРИТЕТ

### 4. Блокирующие `std::fs::*` в async функциях
**Статус:** ⏳ TODO — ~30 мест

### 5. Тестовое покрытие
- **Rust:** 78% модулей с тестами
- **Frontend:** 1 файл
- **E2E:** 2 файла

---

# 📊 ПРОГРЕСС MASTER PLAN

## UI Components
| Компонент | Статус | Файл |
|-----------|--------|------|
| Sidebar | ✅ | `src/lib/components/Sidebar.svelte` |
| CommandPalette | ✅ | `src/lib/components/CommandPalette.svelte` |
| TerminalPanel | ✅ | `src/lib/components/TerminalPanel.svelte` |
| BentoGrid | ✅ | `src/lib/components/BentoGrid.svelte` |
| BentoWidget | ✅ | `src/lib/components/BentoWidget.svelte` |
| StatusWidget | ✅ | `src/lib/components/widgets/StatusWidget.svelte` |
| HealthWidget | ✅ | `src/lib/components/widgets/HealthWidget.svelte` |
| MethodWidget | ✅ | `src/lib/components/widgets/MethodWidget.svelte` |
| QuickActionsWidget | ✅ | `src/lib/components/widgets/QuickActionsWidget.svelte` |
| ContextMenu | ✅ | `src/lib/components/ContextMenu.svelte` |
| ResizablePanelGroup | ✅ | `src/lib/components/ResizablePanelGroup.svelte` |
| ResizablePanel | ✅ | `src/lib/components/ResizablePanel.svelte` |
| ResizableHandle | ✅ | `src/lib/components/ResizableHandle.svelte` |

## Stores
| Store | Статус | Файл |
|-------|--------|------|
| logs | ✅ | `src/lib/stores/logs.ts` |
| plugins | ✅ | `src/lib/stores/plugins.ts` |
| toast | ✅ | `src/lib/stores/toast.ts` |
| appStatus | ✅ | `src/lib/stores/index.ts` |

## Config
| Файл | Статус |
|------|--------|
| tailwind.config.js | ✅ Glass & Void палитра |
| +layout.svelte | ✅ Three-Pane Layout |

---

# 🎯 NEXT STEPS

## Immediate (сейчас)
1. [ ] Обновить Dashboard (`+page.svelte`) с BentoGrid виджетами
2. [ ] Применить Glass & Void стили ко всем страницам
3. [ ] Интегрировать ContextMenu в списки

## Short-term (эта неделя)
4. [ ] Services Master-Detail view
5. [ ] Skeleton loaders
6. [ ] Page transitions

## Medium-term (следующая неделя)
7. [ ] Visual Flow Builder для Routing
8. [ ] Plugin Slots система
9. [ ] Windows Mica эффект

---

# ✅ Definition of Done (AAA Quality)

1. **Power User:** Ctrl+K работает ✅, все действия доступны с клавиатуры
2. **Visual:** Glow эффекты ✅, плавные анимации ⏳, Mica blur ⏳
3. **Modular:** Плагины могут добавлять UI элементы ⏳
4. **Professional:** Выглядит как VS Code / Linear / Figma ⏳
5. **Responsive:** Panels resizable ✅, layout сохраняется ⏳
