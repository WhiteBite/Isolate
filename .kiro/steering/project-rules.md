# Isolate Project Rules

## Project Overview

Isolate — десктопное Windows-приложение для автоматического обхода DPI-блокировок.
Стек: Rust + Tauri 2.0 + SvelteKit + TypeScript + Tailwind CSS.

## Architecture Principles

### Code Organization
- Backend (Rust): `src-tauri/src/`
- Frontend (Svelte): `src/`
- Configs: `configs/strategies/`, `configs/services/`
- Documentation: `docs/` (уже создана, НЕ добавлять новые .md без запроса)
- Reference projects: `thirdparty/` (read-only, только для изучения)

### Rust Backend Rules
- Async runtime: Tokio
- Error handling: `thiserror` + `anyhow`
- Serialization: `serde` + `serde_yaml` для конфигов
- Logging: `tracing` crate
- Модули должны быть изолированы с чёткими интерфейсами
- Внешние процессы (winws, sing-box) запускать ТОЛЬКО через `process_runner.rs`

### Frontend Rules
- SvelteKit с TypeScript (strict mode)
- Стили: Tailwind CSS, никаких inline styles
- State management: Svelte stores
- Tauri API: использовать `@tauri-apps/api` для IPC

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

// Async/await для Tauri commands
const strategies = await invoke<Strategy[]>('get_strategies');
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

- Unit тесты для core логики (scoring, parsing)
- НЕ создавать тесты автоматически — только по запросу
- Интеграционные тесты для process_runner

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

## Reference Projects (thirdparty/)

Используй для изучения, НЕ копируй код напрямую:
- `zapret/` — параметры winws, документация по DPI
- `throne/` — UI/UX паттерны
- `zapret-discord-youtube/` — готовые конфиги и списки доменов
