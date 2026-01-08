# 🎮 UX Features — Детальный план реализации

> Дата создания: Январь 2026
> Статус: Планирование

---

## Обзор

Этот документ содержит детальный план реализации UX-улучшений для Isolate:
1. **Command Palette** — расширенный поиск и быстрые действия
2. **Tray Menu** — улучшенное меню в трее с сервисами и профилями
3. **Toast Notifications** — умные уведомления без спама
4. **Game Mode** — автоматический режим для игр
5. **Sidebar Navigation** — новая структура навигации + Bottom Drawer для логов

---

## 1. Command Palette (Ctrl+K)

### Текущее состояние
- ✅ Базовый компонент `CommandPalette.svelte` существует
- ✅ Fuzzy search реализован
- ✅ Категории: navigation, actions, settings
- ❌ Нет поиска по сервисам и стратегиям
- ❌ Нет подсветки совпадений
- ❌ Нет быстрых действий типа "Enable YouTube"

### Задачи

#### Frontend

- [ ] **[M]** Расширить `CommandPalette.svelte` — добавить категории
  - Файл: `src/lib/components/CommandPalette.svelte`
  - Добавить категории: `services`, `strategies`, `profiles`
  - Интерфейс `Command` расширить полями: `serviceId?`, `strategyId?`
  ```typescript
  interface Command {
    id: string;
    label: string;
    category: 'navigation' | 'actions' | 'settings' | 'services' | 'strategies' | 'profiles';
    shortcut?: string;
    icon?: string;
    serviceId?: string;
    strategyId?: string;
    action: () => void | Promise<void>;
  }
  ```

- [ ] **[S]** Добавить подсветку совпадений в fuzzy search
  - Файл: `src/lib/components/CommandPalette.svelte`
  - Создать функцию `highlightMatches(text: string, query: string): string`
  - Вернуть HTML с `<mark>` тегами для совпадений
  - Использовать `{@html}` для рендера

- [ ] **[M]** Динамическая загрузка сервисов в Command Palette
  - Файл: `src/lib/components/CommandPalette.svelte`
  - При открытии загружать сервисы через `invoke('get_services')`
  - Генерировать команды: "Enable {service}", "Disable {service}", "Test {service}"
  - Кэшировать результат в `$state`

- [ ] **[M]** Динамическая загрузка стратегий
  - Файл: `src/lib/components/CommandPalette.svelte`
  - Загружать стратегии через `invoke('get_strategies')`
  - Генерировать команды: "Apply {strategy}", "Test {strategy}"

- [ ] **[S]** Добавить быстрые действия
  - Файл: `src/lib/components/CommandPalette.svelte`
  - "Switch to TUN mode" → `invoke('start_tun')`
  - "Switch to Proxy mode" → `invoke('set_system_proxy')`
  - "Rescan Network" → `invoke('check_all_registry_services')`
  - "Game Mode On/Off" → `invoke('set_game_filter_mode')`

- [ ] **[S]** Добавить профили (Game Mode / Work Mode)
  - Файл: `src/lib/components/CommandPalette.svelte`
  - "Switch to Game Mode" → применить gaming профиль
  - "Switch to Work Mode" → применить стандартный профиль

#### Stores

- [ ] **[S]** Создать `commandPalette.svelte.ts` store
  - Файл: `src/lib/stores/commandPalette.svelte.ts`
  - Хранить: `isOpen`, `recentCommands`, `favorites`
  - Методы: `open()`, `close()`, `addRecent()`, `toggleFavorite()`

#### Оценка: **M** (Medium) — 3-4 часа

---

## 2. Tray Menu

### Текущее состояние
- ✅ Базовый tray menu в `src-tauri/src/tray.rs`
- ✅ Статус, оптимизация, toggle bypass, TUN/Proxy
- ❌ Нет списка сервисов с галочками
- ❌ Нет быстрой смены профиля
- ❌ Нет "Rescan Network"

### Задачи

#### Backend (Rust)

- [ ] **[L]** Расширить tray menu — добавить submenu сервисов
  - Файл: `src-tauri/src/tray.rs`
  - Создать submenu "Services" с топ-5 сервисами
  - Каждый сервис — checkbox (вкл/выкл защиту)
  - Использовать `CheckMenuItemBuilder` из Tauri
  ```rust
  // В build_tray_menu():
  let services_submenu = SubmenuBuilder::new(app, "🛡️ Services")
      .items(&service_items)
      .build()?;
  ```

- [ ] **[M]** Добавить submenu профилей
  - Файл: `src-tauri/src/tray.rs`
  - Профили: "🎮 Game Mode", "💼 Work Mode", "🌐 Normal"
  - При выборе — применить соответствующие настройки
  - Использовать radio-style selection

- [ ] **[S]** Добавить "Rescan Network" в tray
  - Файл: `src-tauri/src/tray.rs`
  - Пункт меню "🔄 Rescan Network"
  - Emit event `tray:rescan_network`

- [ ] **[M]** Создать команды для управления сервисами из tray
  - Файл: `src-tauri/src/commands/tray.rs`
  - `get_top_services()` — топ-5 сервисов по использованию
  - `toggle_service_protection(service_id: String, enabled: bool)`
  - `get_service_protection_status(service_id: String) -> bool`

- [ ] **[M]** Хранение состояния защиты сервисов
  - Файл: `src-tauri/src/core/models/config.rs`
  - Добавить `service_protection: HashMap<String, bool>` в `AppSettings`
  - Сохранять/загружать из конфига

#### Frontend

- [ ] **[S]** Обработка tray событий для сервисов
  - Файл: `src/lib/api/tray.ts`
  - Добавить `onTrayServiceToggle(callback: (serviceId: string, enabled: boolean) => void)`
  - Добавить `onTrayProfileChange(callback: (profile: string) => void)`
  - Добавить `onTrayRescan(callback: () => void)`

#### Оценка: **L** (Large) — 6-8 часов

---

## 3. Toast Notifications (Умные уведомления)

### Текущее состояние
- ✅ Базовый toast store в `src/lib/stores/toast.ts`
- ✅ Компоненты `Toast.svelte`, `ToastContainer.svelte`
- ❌ Нет группировки похожих ошибок
- ❌ Нет "умных" сообщений с контекстом
- ❌ Нет прогресс-уведомлений

### Задачи

#### Stores

- [ ] **[M]** Расширить toast store — добавить умную логику
  - Файл: `src/lib/stores/toast.ts`
  - Добавить дедупликацию: не показывать одинаковые ошибки подряд
  - Добавить группировку: "3 ошибки подключения" вместо 3 отдельных
  - Добавить `updateToast(id, message)` для обновления существующего
  ```typescript
  interface Toast {
    id: number;
    type: 'success' | 'error' | 'warning' | 'info' | 'progress';
    message: string;
    duration: number;
    progress?: number; // 0-100 для progress type
    groupKey?: string; // для группировки
    count?: number; // количество сгруппированных
  }
  ```

- [ ] **[S]** Добавить progress toast
  - Файл: `src/lib/stores/toast.ts`
  - Метод `showProgress(message, initialProgress)` → возвращает id
  - Метод `updateProgress(id, progress, message?)` → обновляет
  - Метод `completeProgress(id, successMessage)` → завершает

- [ ] **[S]** Добавить умные сообщения об ошибках
  - Файл: `src/lib/utils/errorMessages.ts` (новый)
  - Маппинг технических ошибок → человекочитаемые
  ```typescript
  const errorMap: Record<string, string> = {
    'Connection error 502': 'Сервер временно недоступен',
    'ETIMEDOUT': 'Превышено время ожидания',
    'ECONNREFUSED': 'Не удалось подключиться',
  };
  
  export function humanizeError(error: string): string {
    for (const [pattern, message] of Object.entries(errorMap)) {
      if (error.includes(pattern)) return message;
    }
    return error;
  }
  ```

#### Components

- [ ] **[M]** Обновить `Toast.svelte` — поддержка progress и группировки
  - Файл: `src/lib/components/Toast.svelte`
  - Добавить progress bar для `type: 'progress'`
  - Показывать badge с count для сгруппированных
  - Добавить кнопку "Подробнее" для раскрытия деталей

- [ ] **[S]** Создать `SmartToast.svelte` — toast с действиями
  - Файл: `src/lib/components/SmartToast.svelte` (новый)
  - Поддержка action buttons: "Повторить", "Подробнее", "Отмена"
  - Пример: "YouTube недоступен" + кнопка "Попробовать альтернативу"

#### API

- [ ] **[M]** Создать `src/lib/api/notifications.ts`
  - Файл: `src/lib/api/notifications.ts` (новый)
  - Централизованная логика уведомлений
  - Интеграция с backend событиями
  ```typescript
  export function setupNotificationHandlers() {
    listen('service:unavailable', (event) => {
      const { service, error } = event.payload;
      toasts.warning(`${service} недоступен. Пробуем альтернативу...`);
    });
    
    listen('service:recovered', (event) => {
      toasts.success(`${event.payload.service} восстановлен`);
    });
  }
  ```

#### Оценка: **M** (Medium) — 4-5 часов

---

## 4. Game Mode

### Текущее состояние
- ✅ `GameFilterMode` enum в `src-tauri/src/core/models/config.rs`
- ✅ Команды `get_game_filter_mode`, `set_game_filter_mode`
- ❌ Нет автоматического определения игр
- ❌ Нет UI индикатора Game Mode
- ❌ Нет автоматического переключения

### Задачи

#### Backend (Rust)

- [ ] **[L]** Создать модуль определения запущенных игр
  - Файл: `src-tauri/src/core/game_detector.rs` (новый)
  - Список процессов игр: `cs2.exe`, `dota2.exe`, `valorant.exe`, `steam.exe`, etc.
  - Функция `detect_running_games() -> Vec<String>`
  - Использовать `sysinfo` crate для получения списка процессов
  ```rust
  use sysinfo::{System, ProcessExt, SystemExt};
  
  const GAME_PROCESSES: &[&str] = &[
      "cs2.exe", "csgo.exe", "dota2.exe", "valorant.exe",
      "LeagueClient.exe", "VALORANT-Win64-Shipping.exe",
      "GenshinImpact.exe", "ZenlessZoneZero.exe",
  ];
  
  pub fn detect_running_games() -> Vec<String> {
      let mut sys = System::new_all();
      sys.refresh_processes();
      
      sys.processes()
          .values()
          .filter_map(|p| {
              let name = p.name().to_lowercase();
              GAME_PROCESSES.iter()
                  .find(|&&game| name.contains(&game.to_lowercase()))
                  .map(|&s| s.to_string())
          })
          .collect()
  }
  ```

- [ ] **[M]** Создать фоновый мониторинг игр
  - Файл: `src-tauri/src/core/game_monitor.rs` (новый)
  - Запускать проверку каждые 5 секунд
  - При обнаружении игры — emit event `game:detected`
  - При закрытии игры — emit event `game:closed`
  ```rust
  pub async fn start_game_monitor(app: AppHandle) {
      loop {
          let games = detect_running_games();
          if !games.is_empty() {
              app.emit("game:detected", &games).ok();
          }
          tokio::time::sleep(Duration::from_secs(5)).await;
      }
  }
  ```

- [ ] **[M]** Автоматическое переключение режима
  - Файл: `src-tauri/src/core/game_monitor.rs`
  - При `game:detected`:
    - Сохранить текущие настройки
    - Переключить на Gaming mode
    - Отключить туннелирование для всего кроме Discord
    - Приостановить фоновые проверки пинга
  - При `game:closed`:
    - Восстановить сохранённые настройки

- [ ] **[S]** Команды для Game Mode
  - Файл: `src-tauri/src/commands/game_mode.rs` (новый)
  - `is_game_mode_active() -> bool`
  - `get_detected_games() -> Vec<String>`
  - `set_game_mode_auto(enabled: bool)`
  - `get_game_mode_settings() -> GameModeSettings`

- [ ] **[S]** Конфигурация Game Mode
  - Файл: `src-tauri/src/core/models/config.rs`
  - Добавить в `AppSettings`:
  ```rust
  pub struct GameModeSettings {
      pub auto_detect: bool,
      pub pause_health_checks: bool,
      pub keep_discord: bool,
      pub custom_game_processes: Vec<String>,
  }
  ```

#### Frontend

- [ ] **[S]** Создать `GameModeIndicator.svelte`
  - Файл: `src/lib/components/GameModeIndicator.svelte` (новый)
  - Иконка джойстика 🎮 в header/sidebar
  - Tooltip с названием игры
  - Анимация при активации
  ```svelte
  <script lang="ts">
    let isGameMode = $state(false);
    let gameName = $state<string | null>(null);
    
    $effect(() => {
      const unlisten = listen('game:detected', (e) => {
        isGameMode = true;
        gameName = e.payload[0];
      });
      return () => unlisten.then(fn => fn());
    });
  </script>
  
  {#if isGameMode}
    <div class="game-mode-indicator" title="Game Mode: {gameName}">
      🎮
    </div>
  {/if}
  ```

- [ ] **[M]** Добавить страницу настроек Game Mode
  - Файл: `src/routes/settings/+page.svelte`
  - Секция "Game Mode":
    - Toggle "Auto-detect games"
    - Toggle "Pause health checks"
    - Toggle "Keep Discord active"
    - Список кастомных процессов

- [ ] **[S]** Добавить Game Mode в Command Palette
  - Файл: `src/lib/components/CommandPalette.svelte`
  - "Enable Game Mode" / "Disable Game Mode"
  - "Add game process..."

#### API

- [ ] **[S]** Создать `src/lib/api/gameMode.ts`
  - Файл: `src/lib/api/gameMode.ts` (новый)
  ```typescript
  export async function isGameModeActive(): Promise<boolean>;
  export async function getDetectedGames(): Promise<string[]>;
  export async function setGameModeAuto(enabled: boolean): Promise<void>;
  export function onGameDetected(callback: (games: string[]) => void): Promise<UnlistenFn>;
  export function onGameClosed(callback: () => void): Promise<UnlistenFn>;
  ```

#### Оценка: **XL** (Extra Large) — 10-12 часов

---

## 5. Sidebar Navigation + Bottom Drawer

### Текущее состояние
- ✅ `Sidebar.svelte` с навигацией
- ✅ Страница `/logs` существует
- ❌ Структура не соответствует новому дизайну
- ❌ Нет Bottom Drawer для логов

### Задачи

#### Components

- [ ] **[M]** Реструктурировать `Sidebar.svelte`
  - Файл: `src/lib/components/Sidebar.svelte`
  - Новая структура:
  ```typescript
  const mainItems: NavItem[] = [
    { id: 'dashboard', name: 'Dashboard', icon: 'layout-dashboard', route: '/' },
    { id: 'library', name: 'Library', icon: 'library', route: '/services' },
  ];
  
  const toolsItems: NavItem[] = [
    { id: 'orchestra', name: 'Boost', icon: 'wand', route: '/orchestra' },
    { id: 'proxies', name: 'Proxy & VPN', icon: 'globe', route: '/proxies' },
  ];
  
  const systemItems: NavItem[] = [
    { id: 'plugins', name: 'Plugins', icon: 'puzzle', route: '/plugins' },
    { id: 'settings', name: 'Settings', icon: 'settings', route: '/settings' },
  ];
  // Logs убрать из меню!
  ```
  - Добавить разделители между группами
  - Добавить заголовки групп: "Main", "Tools", "System"

- [ ] **[L]** Создать `BottomDrawer.svelte` для логов
  - Файл: `src/lib/components/BottomDrawer.svelte` (новый)
  - Выезжающая панель снизу (высота 30-50% экрана)
  - Drag handle для изменения размера
  - Кнопка закрытия
  - Интеграция с `TerminalPanel.svelte` для отображения логов
  ```svelte
  <script lang="ts">
    interface Props {
      isOpen?: boolean;
      onClose?: () => void;
      height?: number; // в процентах
    }
    
    let { isOpen = $bindable(false), onClose, height = 35 }: Props = $props();
    let dragging = $state(false);
    let currentHeight = $state(height);
  </script>
  
  {#if isOpen}
    <div 
      class="fixed bottom-0 left-0 right-0 bg-surface border-t border-white/10 z-40"
      style="height: {currentHeight}vh"
    >
      <!-- Drag handle -->
      <div class="h-2 cursor-ns-resize flex justify-center items-center">
        <div class="w-12 h-1 bg-white/20 rounded-full"></div>
      </div>
      
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-2 border-b border-white/10">
        <span class="text-sm font-medium">Logs</span>
        <button onclick={() => onClose?.()}>✕</button>
      </div>
      
      <!-- Content -->
      <div class="flex-1 overflow-hidden">
        <slot />
      </div>
    </div>
  {/if}
  ```

- [ ] **[S]** Добавить кнопку открытия логов в footer/header
  - Файл: `src/routes/+layout.svelte`
  - Кнопка "📋 Logs" в нижней части экрана
  - Или иконка терминала в header
  - При клике — открыть BottomDrawer

- [ ] **[S]** Интегрировать `TerminalPanel.svelte` в BottomDrawer
  - Файл: `src/routes/+layout.svelte`
  - Использовать существующий компонент для отображения логов
  - Добавить фильтры: All, Errors, Warnings

#### Stores

- [ ] **[S]** Создать `bottomDrawer.svelte.ts` store
  - Файл: `src/lib/stores/bottomDrawer.svelte.ts` (новый)
  - Состояние: `isOpen`, `height`, `activeTab`
  - Методы: `open()`, `close()`, `toggle()`, `setHeight()`
  - Сохранение состояния в localStorage

#### Routes

- [ ] **[S]** Переименовать `/services` → `/library` (опционально)
  - Или оставить `/services` но изменить название в sidebar на "Library"
  - Решение: оставить route как есть, изменить только label

- [ ] **[S]** Удалить `/logs` из навигации
  - Файл: `src/lib/components/Sidebar.svelte`
  - Убрать из `systemItems`
  - Страница остаётся для прямого доступа по URL

#### Оценка: **L** (Large) — 5-6 часов

---

## Сводная таблица

| Фича | Сложность | Время | Приоритет |
|------|-----------|-------|-----------|
| Command Palette | M | 3-4ч | P1 |
| Tray Menu | L | 6-8ч | P2 |
| Toast Notifications | M | 4-5ч | P1 |
| Game Mode | XL | 10-12ч | P3 |
| Sidebar + Bottom Drawer | L | 5-6ч | P1 |

**Общее время:** ~28-35 часов

---

## Зависимости между задачами

```
Command Palette
    └── Зависит от: stores/commandPalette.svelte.ts

Tray Menu
    └── Зависит от: backend tray.rs расширения
    └── Зависит от: service protection state

Toast Notifications
    └── Зависит от: stores/toast.ts расширения
    └── Зависит от: utils/errorMessages.ts

Game Mode
    └── Зависит от: core/game_detector.rs
    └── Зависит от: core/game_monitor.rs
    └── Зависит от: models/config.rs расширения

Sidebar + Bottom Drawer
    └── Зависит от: stores/bottomDrawer.svelte.ts
    └── Зависит от: BottomDrawer.svelte компонент
```

---

## Порядок реализации (рекомендуемый)

### Фаза 1: Quick Wins (1-2 дня)
1. Toast Notifications — умные сообщения
2. Sidebar реструктуризация
3. Command Palette — подсветка и базовые улучшения

### Фаза 2: Core Features (2-3 дня)
4. Bottom Drawer для логов
5. Command Palette — динамические сервисы/стратегии
6. Tray Menu — submenu сервисов

### Фаза 3: Advanced (3-4 дня)
7. Tray Menu — профили
8. Game Mode — детектор и мониторинг
9. Game Mode — UI и интеграция

---

## Файлы для создания

### Новые файлы
```
src/lib/components/BottomDrawer.svelte
src/lib/components/SmartToast.svelte
src/lib/components/GameModeIndicator.svelte
src/lib/stores/commandPalette.svelte.ts
src/lib/stores/bottomDrawer.svelte.ts
src/lib/api/notifications.ts
src/lib/api/gameMode.ts
src/lib/utils/errorMessages.ts
src-tauri/src/core/game_detector.rs
src-tauri/src/core/game_monitor.rs
src-tauri/src/commands/game_mode.rs
```

### Файлы для модификации
```
src/lib/components/CommandPalette.svelte
src/lib/components/Sidebar.svelte
src/lib/components/Toast.svelte
src/lib/stores/toast.ts
src/lib/api/tray.ts
src/routes/+layout.svelte
src/routes/settings/+page.svelte
src-tauri/src/tray.rs
src-tauri/src/commands/tray.rs
src-tauri/src/core/models/config.rs
src-tauri/src/core/mod.rs
src-tauri/src/commands/mod.rs
src-tauri/src/lib.rs
```

---

## Tauri Capabilities

### Необходимые permissions
```json
// src-tauri/capabilities/default.json
{
  "permissions": [
    "core:default",
    "shell:allow-open",
    "process:allow-exit",
    "notification:default",
    "tray:default",
    "tray:allow-set-icon",
    "tray:allow-set-menu"
  ]
}
```

### Для Game Mode (sysinfo)
- Добавить `sysinfo` в `Cargo.toml`:
```toml
[dependencies]
sysinfo = "0.30"
```

---

*Документ будет обновляться по мере реализации задач.*
