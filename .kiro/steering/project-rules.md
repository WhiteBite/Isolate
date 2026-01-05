# Isolate Project Rules

## Agent Behavior Rules

### Autonomy Principle
- **НИКОГДА не предлагай пользователю сделать что-то самостоятельно** — выполняй задачу сам
- Пользователь даёт задачи, агент их выполняет. Не наоборот.
- Если требуется установка инструментов (Rust, Node, etc.) — устанавливай автоматически
- Если команда не работает — ищи альтернативный способ, не спрашивай пользователя
- При ошибках — исправляй и продолжай, не останавливайся на вопросах
- Минимизируй диалог: делай, а не обсуждай

### Decision Making
- Принимай решения самостоятельно в рамках задачи
- Выбирай лучший технический подход без запроса подтверждения
- При неоднозначности — выбирай наиболее распространённый/безопасный вариант
- Сообщай о результате, а не о планах

## Project Overview

Isolate — десктопное Windows-приложение для автоматического обхода DPI-блокировок.
Стек: Rust + Tauri 2.0 + SvelteKit + TypeScript + Tailwind CSS.

## Architecture Principles

### Code Organization
- Backend (Rust): `src-tauri/src/`
  - `core/` — бизнес-логика (strategy_engine, scoring, models)
  - `commands/` — Tauri IPC команды
  - `services/` — сервисы (registry, checker)
  - `plugins/` — система плагинов
- Frontend (Svelte): `src/`
  - `routes/` — страницы SvelteKit
  - `lib/` — компоненты, stores, утилиты
- Configs: `configs/strategies/`, `configs/services/`
- Documentation: `docs/` (уже создана, НЕ добавлять новые .md без запроса)
- Reference projects: `thirdparty/` (read-only, только для изучения)

### Tauri IPC Architecture

#### AppState Initialization (КРИТИЧНО!)
- `AppState` инициализируется **асинхронно** в `setup()` через `tauri::async_runtime::spawn`
- Фронтенд может загрузиться **ДО** готовности AppState (~300-500ms race condition)
- **ВСЕГДА** используй `is_backend_ready` для проверки готовности перед вызовом команд
- Команды без State (например `is_backend_ready`) работают сразу

```rust
// Команда БЕЗ State — работает сразу
#[tauri::command]
pub fn is_backend_ready(app: AppHandle) -> bool {
    app.try_state::<Arc<AppState>>().is_some()
}

// Команда С State — требует готовности AppState
#[tauri::command]
pub async fn get_services(state: State<'_, Arc<AppState>>) -> Result<Vec<Service>, String> {
    // ...
}
```

#### Frontend Pattern для IPC
```typescript
// ПРАВИЛЬНО: ждём готовности бэкенда
async function loadData(retries = 10) {
  for (let i = 0; i < retries; i++) {
    try {
      const ready = await invoke<boolean>('is_backend_ready');
      if (!ready) {
        await new Promise(r => setTimeout(r, 200));
        continue;
      }
      return await invoke<Data>('get_data');
    } catch {
      await new Promise(r => setTimeout(r, 200));
    }
  }
}

// НЕПРАВИЛЬНО: вызов без проверки готовности
const data = await invoke<Data>('get_data'); // может упасть!
```

### Rust Backend Rules
- Async runtime: Tokio
- Error handling: `thiserror` + `anyhow`
- Serialization: `serde` + `serde_yaml` для конфигов
- Logging: `tracing` crate
- Модули должны быть изолированы с чёткими интерфейсами
- Внешние процессы (winws, sing-box) запускать ТОЛЬКО через `process_runner.rs`
- Новые Tauri commands регистрировать в `lib.rs` invoke_handler

### Frontend Rules (Svelte 5 Runes)
- SvelteKit с TypeScript (strict mode)
- **Svelte 5 runes mode** — использовать `$state`, `$derived`, `$effect`
- НЕ использовать устаревший синтаксис: `let x = value`, `$:`, `onMount` для реактивности
- Стили: Tailwind CSS, никаких inline styles
- State management: Svelte stores + runes
- Tauri API: использовать `@tauri-apps/api/core` для IPC

```svelte
<script lang="ts">
  // ПРАВИЛЬНО: Svelte 5 runes
  let services = $state<Service[]>([]);
  let loading = $state(true);
  let selected = $derived(services.find(s => s.id === selectedId));
  
  $effect(() => {
    loadServices();
  });

  // НЕПРАВИЛЬНО: устаревший синтаксис
  let services: Service[] = [];  // не реактивно в runes mode
  $: selected = services.find(...);  // ошибка в runes mode
  onMount(() => { ... });  // не вызывается при навигации
</script>
```

### Strategy Engine Critical Rules
- **НИКОГДА** не запускать несколько winws/WinDivert процессов параллельно — это вызовет BSOD
- Параллельный запуск разрешён ТОЛЬКО для VLESS/Sing-box стратегий (разные SOCKS-порты)
- Zapret стратегии — строго последовательно с таймаутом 2-3 сек

## File Creation Rules

### ЗАПРЕЩЕНО создавать без явного запроса:
- Любые `.md` файлы (документация уже есть в `docs/`)
- `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`
- Файлы в корне проекта кроме стандартных (`package.json`, `Cargo.toml`, etc.)
- Тестовые файлы без запроса
- Примеры и демо-файлы

### РАЗРЕШЕНО создавать:
- Исходный код в `src-tauri/src/` и `src/`
- Конфиги стратегий в `configs/`
- Tauri конфигурацию

## Code Style

### Rust
```rust
// Используй Result для ошибок
pub async fn run_strategy(id: &str) -> Result<(), IsolateError> { }

// Структуры с derive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy { }

// Документируй публичные API
/// Запускает стратегию в глобальном режиме
pub async fn apply_global(strategy: &Strategy) -> Result<()> { }
```

### TypeScript/Svelte
```typescript
// Строгая типизация
interface Strategy {
  id: string;
  name: string;
  family: StrategyFamily;
}

// Async/await для Tauri commands с проверкой готовности
async function loadStrategies() {
  const ready = await invoke<boolean>('is_backend_ready');
  if (ready) {
    return await invoke<Strategy[]>('get_strategies');
  }
}
```

## Naming Conventions

- Rust: `snake_case` для функций и переменных, `PascalCase` для типов
- TypeScript: `camelCase` для функций и переменных, `PascalCase` для типов/интерфейсов
- Файлы: `snake_case.rs`, `kebab-case.svelte`, `kebab-case.ts`
- Tauri commands: `snake_case` (Rust) → автоматически `camelCase` (JS)

## Git Workflow

- Коммиты на русском или английском, краткие и по делу
- Не коммитить `thirdparty/` (добавить в `.gitignore`)
- Не коммитить бинарники winws/sing-box напрямую (использовать releases)

## Testing Strategy

- Unit тесты для core логики (scoring, parsing, models)
- НЕ создавать тесты автоматически — только по запросу
- Тесты должны искать реальные баги, а не подгоняться под код
- Интеграционные тесты для process_runner
- `#[ignore]` для тестов требующих реальные файлы/сеть

## Dependencies Policy

### Rust (Cargo.toml)
- Минимизировать количество зависимостей
- Предпочитать проверенные crates с активной поддержкой
- Фиксировать версии

### Frontend (package.json)
- Использовать pnpm
- Минимум зависимостей, Tailwind покрывает стили
- Никаких UI-библиотек (чистый Svelte + Tailwind)

## Security

- Никогда не хардкодить пути к бинарникам
- Проверять хэши внешних бинарников перед запуском
- Логи не должны содержать IP пользователя
- UAC запрашивать только когда реально нужен admin

## Performance

- Таймауты на все сетевые операции (max 5 сек)
- Graceful shutdown для всех процессов
- Не блокировать UI во время тестирования стратегий
- AppState инициализация ~300-500ms — учитывать в UX

## Common Bugs & Solutions

### "0 / 0 services" или пустые данные при загрузке
**Причина:** Race condition — фронтенд вызывает команду до готовности AppState
**Решение:** Использовать `is_backend_ready` + retry логику

### Svelte компонент не обновляется при навигации
**Причина:** `onMount` не вызывается при client-side навигации в SvelteKit
**Решение:** Использовать `$effect` вместо `onMount`

### WebView2 ошибки при HMR
**Причина:** WebView2 теряет соединение при hot reload
**Решение:** Перезапустить `pnpm tauri dev`

## Reference Projects (thirdparty/)

Используй для изучения, НЕ копируй код напрямую:
- `zapret/` — параметры winws, документация по DPI
- `throne/` — UI/UX паттерны
- `zapret-discord-youtube/` — готовые конфиги и списки доменов
