# Hotkeys Architecture

## Текущее состояние

### Что реализовано
- **Frontend store**: `src/lib/stores/hotkeys.ts` — хранение конфигурации в localStorage
- **UI настройки**: `src/routes/settings/+page.svelte` — вкладка Hotkeys с записью комбинаций
- **Модель данных**: `HotkeyConfig` с полями `key`, `ctrlKey`, `shiftKey`, `altKey`

### Ограничения текущей реализации
⚠️ **Hotkeys работают ТОЛЬКО когда приложение в фокусе** — это обычные DOM keyboard events.

Для работы в свёрнутом состоянии требуется интеграция с **Tauri Global Shortcut Plugin**.

---

## Архитектура глобальных hotkeys

### 1. Tauri Global Shortcut Plugin

#### Установка
```bash
pnpm tauri add global-shortcut
```

#### Зависимости
```toml
# src-tauri/Cargo.toml
[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".dependencies]
tauri-plugin-global-shortcut = "2"
```

```json
// package.json
{
  "dependencies": {
    "@tauri-apps/plugin-global-shortcut": "^2.0.0"
  }
}
```

#### Инициализация плагина
```rust
// src-tauri/src/lib.rs
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(handle_shortcut)
                    .build()
            )?;
            Ok(())
        })
        // ...
}
```

#### Permissions
```json
// src-tauri/capabilities/default.json
{
  "permissions": [
    "global-shortcut:allow-is-registered",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-register-all",
    "global-shortcut:allow-unregister-all"
  ]
}
```

---

### 2. Модель данных

#### HotkeyConfig (расширенная)
```typescript
// src/lib/stores/hotkeys.ts
export interface HotkeyConfig {
  key: string;           // "S", "T", "F1", etc.
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey?: boolean;     // Windows key (для будущего)
  enabled: boolean;      // Можно отключить отдельный hotkey
}

export interface HotkeysState {
  toggleStrategy: HotkeyConfig;
  openSettings: HotkeyConfig;
  quickTest: HotkeyConfig;
  showWindow: HotkeyConfig;      // NEW: показать/скрыть окно
  switchStrategy: HotkeyConfig;  // NEW: переключить стратегию
}
```

#### Rust модель
```rust
// src-tauri/src/core/hotkeys.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub key: String,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeysConfig {
    pub toggle_strategy: HotkeyConfig,
    pub open_settings: HotkeyConfig,
    pub quick_test: HotkeyConfig,
    pub show_window: HotkeyConfig,
    pub switch_strategy: HotkeyConfig,
}

impl Default for HotkeysConfig {
    fn default() -> Self {
        Self {
            toggle_strategy: HotkeyConfig {
                key: "S".into(),
                ctrl_key: true,
                shift_key: true,
                alt_key: false,
                enabled: true,
            },
            open_settings: HotkeyConfig {
                key: "Comma".into(),
                ctrl_key: true,
                shift_key: false,
                alt_key: false,
                enabled: true,
            },
            quick_test: HotkeyConfig {
                key: "T".into(),
                ctrl_key: true,
                shift_key: false,
                alt_key: false,
                enabled: true,
            },
            show_window: HotkeyConfig {
                key: "I".into(),
                ctrl_key: true,
                shift_key: true,
                alt_key: false,
                enabled: true,
            },
            switch_strategy: HotkeyConfig {
                key: "Tab".into(),
                ctrl_key: true,
                shift_key: true,
                alt_key: false,
                enabled: true,
            },
        }
    }
}
```

---

### 3. Доступные действия

| Action | Default Hotkey | Описание |
|--------|---------------|----------|
| `toggleStrategy` | `Ctrl+Shift+S` | Включить/выключить текущую стратегию |
| `showWindow` | `Ctrl+Shift+I` | Показать/скрыть главное окно |
| `quickTest` | `Ctrl+T` | Запустить быстрый тест подключения |
| `openSettings` | `Ctrl+,` | Открыть настройки |
| `switchStrategy` | `Ctrl+Shift+Tab` | Переключить на следующую стратегию |

---

### 4. Backend реализация

#### Hotkey Manager
```rust
// src-tauri/src/core/hotkeys.rs
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub struct HotkeyManager {
    app: AppHandle,
    config: HotkeysConfig,
}

impl HotkeyManager {
    pub fn new(app: AppHandle) -> Self {
        let config = load_hotkeys_config(&app).unwrap_or_default();
        Self { app, config }
    }

    /// Регистрирует все hotkeys
    pub fn register_all(&self) -> Result<(), String> {
        let gs = self.app.global_shortcut();
        
        // Сначала отменяем все предыдущие
        let _ = gs.unregister_all();
        
        // Регистрируем каждый hotkey
        if self.config.toggle_strategy.enabled {
            let shortcut = self.config_to_shortcut(&self.config.toggle_strategy)?;
            gs.register(shortcut).map_err(|e| e.to_string())?;
        }
        
        // ... аналогично для остальных
        
        Ok(())
    }

    /// Обновляет один hotkey
    pub fn update_hotkey(&mut self, action: &str, config: HotkeyConfig) -> Result<(), String> {
        // Отменяем старый
        // Регистрируем новый
        // Сохраняем в storage
        Ok(())
    }

    fn config_to_shortcut(&self, config: &HotkeyConfig) -> Result<Shortcut, String> {
        let mut modifiers = Modifiers::empty();
        if config.ctrl_key { modifiers |= Modifiers::CONTROL; }
        if config.shift_key { modifiers |= Modifiers::SHIFT; }
        if config.alt_key { modifiers |= Modifiers::ALT; }
        
        let code = key_to_code(&config.key)?;
        Ok(Shortcut::new(Some(modifiers), code))
    }
}

fn key_to_code(key: &str) -> Result<Code, String> {
    match key.to_uppercase().as_str() {
        "A" => Ok(Code::KeyA),
        "B" => Ok(Code::KeyB),
        // ... остальные буквы
        "S" => Ok(Code::KeyS),
        "T" => Ok(Code::KeyT),
        "TAB" => Ok(Code::Tab),
        "COMMA" | "," => Ok(Code::Comma),
        "F1" => Ok(Code::F1),
        // ... остальные F-клавиши
        _ => Err(format!("Unknown key: {}", key)),
    }
}
```

#### Обработчик событий
```rust
// src-tauri/src/core/hotkeys.rs
pub fn handle_shortcut(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    
    let state = app.state::<Arc<AppState>>();
    let config = &state.hotkeys_config;
    
    // Определяем какое действие вызвано
    if matches_shortcut(shortcut, &config.toggle_strategy) {
        handle_toggle_strategy(app);
    } else if matches_shortcut(shortcut, &config.show_window) {
        handle_show_window(app);
    } else if matches_shortcut(shortcut, &config.quick_test) {
        handle_quick_test(app);
    }
    // ...
}

fn handle_toggle_strategy(app: &AppHandle) {
    // Emit event to frontend
    app.emit("hotkey:toggle-strategy", ()).ok();
    
    // Or execute directly via AppState
    // let state = app.state::<Arc<AppState>>();
    // state.strategy_engine.toggle();
}

fn handle_show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide().ok();
        } else {
            window.show().ok();
            window.set_focus().ok();
        }
    }
}
```

---

### 5. Tauri Commands

```rust
// src-tauri/src/commands/hotkeys.rs
use tauri::State;
use std::sync::Arc;
use crate::core::AppState;

#[tauri::command]
pub async fn get_hotkeys(state: State<'_, Arc<AppState>>) -> Result<HotkeysConfig, String> {
    Ok(state.hotkeys_config.read().await.clone())
}

#[tauri::command]
pub async fn set_hotkey(
    state: State<'_, Arc<AppState>>,
    action: String,
    config: HotkeyConfig,
) -> Result<(), String> {
    state.hotkey_manager.update_hotkey(&action, config).await
}

#[tauri::command]
pub async fn reset_hotkeys(state: State<'_, Arc<AppState>>) -> Result<HotkeysConfig, String> {
    state.hotkey_manager.reset_to_defaults().await
}

#[tauri::command]
pub fn is_hotkey_registered(shortcut: String) -> Result<bool, String> {
    // Check via global_shortcut plugin
    Ok(false)
}
```

---

### 6. Frontend интеграция

#### Обновлённый store
```typescript
// src/lib/stores/hotkeys.ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export const hotkeysStore = {
  // ... существующие методы ...
  
  /**
   * Синхронизация с backend (Tauri global shortcuts)
   */
  async syncWithBackend(): Promise<void> {
    if (!browser || !('__TAURI__' in window)) return;
    
    try {
      const backendConfig = await invoke<HotkeysState>('get_hotkeys');
      currentHotkeys = backendConfig;
      saveHotkeys(currentHotkeys);
      subscribers.forEach(cb => cb(currentHotkeys));
    } catch (e) {
      console.warn('Failed to sync hotkeys with backend:', e);
    }
  },
  
  /**
   * Сохранить hotkey в backend
   */
  async setHotkeyAsync(action: keyof HotkeysState, config: HotkeyConfig): Promise<void> {
    // Сначала обновляем локально
    this.setHotkey(action, config);
    
    // Затем синхронизируем с backend
    if (browser && '__TAURI__' in window) {
      try {
        await invoke('set_hotkey', { action, config });
      } catch (e) {
        console.error('Failed to register global hotkey:', e);
        // Можно показать уведомление о конфликте
      }
    }
  }
};

// Слушаем события от backend
if (browser && '__TAURI__' in window) {
  listen('hotkey:toggle-strategy', () => {
    // Dispatch to app
    window.dispatchEvent(new CustomEvent('isolate:toggle-strategy'));
  });
  
  listen('hotkey:quick-test', () => {
    window.dispatchEvent(new CustomEvent('isolate:quick-test'));
  });
}
```

---

### 7. UI компонент для записи hotkey

Текущая реализация в `+page.svelte` уже хорошая. Рекомендуемые улучшения:

```svelte
<!-- HotkeyRecorder.svelte -->
<script lang="ts">
  import { formatHotkey, parseKeyboardEvent, type HotkeyConfig } from '$lib/stores/hotkeys';
  
  interface Props {
    value: HotkeyConfig;
    onchange: (config: HotkeyConfig) => void;
    disabled?: boolean;
  }
  
  let { value, onchange, disabled = false }: Props = $props();
  
  let recording = $state(false);
  let inputEl: HTMLInputElement;
  
  function startRecording() {
    if (disabled) return;
    recording = true;
    // Focus input after state update
    setTimeout(() => inputEl?.focus(), 0);
  }
  
  function handleKeydown(e: KeyboardEvent) {
    if (!recording) return;
    
    e.preventDefault();
    e.stopPropagation();
    
    const config = parseKeyboardEvent(e);
    if (!config) return; // Modifier-only press
    
    // Validate: require at least one modifier
    if (!config.ctrlKey && !config.altKey && !config.shiftKey) {
      // Show error: "Hotkey must include Ctrl, Alt, or Shift"
      return;
    }
    
    recording = false;
    onchange(config);
  }
</script>

{#if recording}
  <input
    bind:this={inputEl}
    type="text"
    readonly
    placeholder="Press keys..."
    onkeydown={handleKeydown}
    onblur={() => recording = false}
    class="hotkey-input recording"
  />
{:else}
  <button
    onclick={startRecording}
    {disabled}
    class="hotkey-input"
  >
    {formatHotkey(value)}
  </button>
{/if}

<style>
  .hotkey-input {
    min-width: 140px;
    padding: 0.5rem 1rem;
    font-family: monospace;
    text-align: center;
  }
  
  .recording {
    animation: pulse 1s infinite;
    border-color: var(--color-indigo-500);
  }
</style>
```

---

### 8. Конфликты с системными hotkeys

#### Зарезервированные комбинации (Windows)

| Комбинация | Системное действие |
|------------|-------------------|
| `Ctrl+C/V/X/Z` | Копировать/Вставить/Вырезать/Отменить |
| `Ctrl+A` | Выделить всё |
| `Ctrl+S` | Сохранить |
| `Ctrl+W` | Закрыть вкладку/окно |
| `Ctrl+Tab` | Переключение вкладок |
| `Alt+Tab` | Переключение окон |
| `Alt+F4` | Закрыть приложение |
| `Win+*` | Системные функции Windows |
| `Ctrl+Alt+Del` | Системное меню |
| `Ctrl+Shift+Esc` | Диспетчер задач |

#### Валидация при записи
```typescript
const RESERVED_SHORTCUTS = [
  { key: 'C', ctrlKey: true, shiftKey: false, altKey: false },
  { key: 'V', ctrlKey: true, shiftKey: false, altKey: false },
  { key: 'X', ctrlKey: true, shiftKey: false, altKey: false },
  { key: 'Z', ctrlKey: true, shiftKey: false, altKey: false },
  { key: 'A', ctrlKey: true, shiftKey: false, altKey: false },
  { key: 'W', ctrlKey: true, shiftKey: false, altKey: false },
  { key: 'Tab', altKey: true },
  { key: 'F4', altKey: true },
  { key: 'Escape', ctrlKey: true, shiftKey: true }, // Task Manager
];

export function isReservedShortcut(config: HotkeyConfig): boolean {
  return RESERVED_SHORTCUTS.some(reserved => 
    reserved.key.toUpperCase() === config.key.toUpperCase() &&
    (reserved.ctrlKey === undefined || reserved.ctrlKey === config.ctrlKey) &&
    (reserved.shiftKey === undefined || reserved.shiftKey === config.shiftKey) &&
    (reserved.altKey === undefined || reserved.altKey === config.altKey)
  );
}
```

#### Обработка ошибок регистрации
```rust
// Tauri может вернуть ошибку если shortcut уже занят другим приложением
match gs.register(shortcut) {
    Ok(_) => Ok(()),
    Err(e) => {
        tracing::warn!("Failed to register shortcut: {}", e);
        // Emit event to frontend about conflict
        app.emit("hotkey:registration-failed", &HotkeyError {
            action: action.to_string(),
            reason: e.to_string(),
        }).ok();
        Err(format!("Shortcut may be used by another application: {}", e))
    }
}
```

---

### 9. Хранение в Storage

#### Ключ в settings_keys
```rust
// src-tauri/src/core/storage/types.rs
pub mod settings_keys {
    // ... existing keys ...
    
    /// Конфигурация горячих клавиш
    pub const HOTKEYS: &str = "hotkeys";
}
```

#### Сохранение/загрузка
```rust
// src-tauri/src/core/hotkeys.rs
pub async fn load_hotkeys_config(storage: &Storage) -> Result<HotkeysConfig> {
    match storage.get_setting::<HotkeysConfig>(settings_keys::HOTKEYS).await? {
        Some(config) => Ok(config),
        None => Ok(HotkeysConfig::default()),
    }
}

pub async fn save_hotkeys_config(storage: &Storage, config: &HotkeysConfig) -> Result<()> {
    storage.set_setting(settings_keys::HOTKEYS, config).await
}
```

---

### 10. Миграция с localStorage

При первом запуске после обновления:

```typescript
// src/lib/stores/hotkeys.ts
async function migrateFromLocalStorage(): Promise<void> {
  if (!browser || !('__TAURI__' in window)) return;
  
  const localConfig = localStorage.getItem(STORAGE_KEY);
  if (!localConfig) return;
  
  try {
    const parsed = JSON.parse(localConfig);
    
    // Проверяем, есть ли уже конфиг в backend
    const backendConfig = await invoke<HotkeysConfig | null>('get_hotkeys');
    
    if (!backendConfig) {
      // Мигрируем localStorage → backend
      await invoke('import_hotkeys', { config: parsed });
    }
    
    // Удаляем localStorage после успешной миграции
    localStorage.removeItem(STORAGE_KEY);
  } catch (e) {
    console.warn('Hotkeys migration failed:', e);
  }
}
```

---

## Roadmap

### Phase 1: Backend интеграция
- [ ] Добавить `tauri-plugin-global-shortcut`
- [ ] Создать `src-tauri/src/core/hotkeys.rs`
- [ ] Добавить Tauri commands
- [ ] Обновить permissions

### Phase 2: Frontend синхронизация
- [ ] Обновить `hotkeysStore` для работы с backend
- [ ] Добавить обработку событий от backend
- [ ] Миграция с localStorage

### Phase 3: UX улучшения
- [ ] Показывать предупреждения о конфликтах
- [ ] Добавить новые действия (showWindow, switchStrategy)
- [ ] Локализация названий действий

---

## Ссылки

- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri Events](https://v2.tauri.app/develop/calling-frontend/)
- Текущая реализация: `src/lib/stores/hotkeys.ts`
