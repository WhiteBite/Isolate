# Найденные баги — E2E тестирование

## Критические (Backend)

### ~~BUG-001: Zapret стратегии не поддерживают SOCKS mode~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src-tauri/src/core/orchestrator.rs`
**Решение:** Создана функция `test_strategy_global()` для тестирования Zapret в GLOBAL режиме. `test_zapret_sequential` теперь использует её вместо `test_strategy_socks`.

### ~~BUG-002: VLESS стратегия не поддерживает GLOBAL mode~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src-tauri/src/core/orchestrator.rs`
**Решение:** Финальное применение стратегии теперь учитывает тип движка:
- VLESS (SingBox/Xray) → `start_socks()`
- Zapret → `start_global()`

## UI/UX баги

### ~~BUG-003: Смешивание on:click и onclick в Svelte 5 runes mode~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src/routes/services/+page.svelte`, `src/lib/components/BigToggleButton.svelte`
**Проблема:** Использование устаревшего `on:click` вместо `onclick` в runes mode вызывало ошибку "Mixing old and new syntaxes"
**Решение:** Заменены все `on:click` на `onclick` во всех .svelte файлах

### ~~BUG-004: Кнопка "Configure" на странице Services показывает только toast~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src/routes/services/+page.svelte`
**Решение:** Создано модальное окно конфигурации сервиса с настройками: auto-check, интервал проверки, уведомления, приоритет.

### ~~BUG-005: Нет индикатора загрузки при "Scan All" на Dashboard~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src/routes/+page.svelte`
**Решение:** Добавлены loading states через `$state` и `$derived`. Кнопка показывает spinner и disabled во время сканирования.

### ~~BUG-006: Marketplace страница не существует (404)~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src/routes/marketplace/+page.svelte`
**Решение:** Создана страница маркетплейса с mock данными плагинов

### ~~BUG-007: Settings страница вызывает infinite loop~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src/routes/settings/+page.svelte`
**Проблема:** `$effect` подписывался на settings store, что вызывало бесконечный цикл перерендеров
**Решение:** Использован флаг `initialized` и `get(settings)` для одноразовой инициализации

### ~~BUG-008: Add Proxy не поддерживает несколько ссылок~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src/routes/proxies/+page.svelte`
**Проблема:** Модальное окно "Add Proxy" → "Paste Link" принимало только одну ссылку
**Решение:** `handlePasteImport()` теперь разбивает текст по строкам и импортирует каждую ссылку отдельно

### ~~BUG-009: Onboarding не скачивает бинарники~~ ✅ ИСПРАВЛЕНО
**Файлы:** `src/routes/onboarding/+page.svelte`
**Проблема:** Onboarding только проверял наличие бинарников, но не скачивал их при отсутствии
**Решение:** Добавлен вызов `download_binaries` с прогрессом через Tauri events

## Требуют проверки

### TODO-001: Проверить модальное окно "Добавить правило" на странице Routing
**Файлы:** `src/routes/routing/+page.svelte`
**Статус:** Кнопка найдена, нужно проверить открытие модалки

### TODO-002: Протестировать страницу Proxies
**Файлы:** `src/routes/proxies/+page.svelte`

### TODO-003: Протестировать Marketplace
**Файлы:** `src/routes/marketplace/` (если есть)

### TODO-004: Протестировать плагины Discord Fix и Speed Test
**Файлы:** `src/routes/plugins/`

## Архитектурные проблемы

### ARCH-001: Несоответствие режимов стратегий
**Проблема:** Orchestrator пытается тестировать Zapret стратегии в SOCKS mode, хотя они работают только в GLOBAL mode. И наоборот — VLESS только в SOCKS mode.
**Решение:** Нужна логика выбора правильного режима тестирования в зависимости от типа стратегии (family)

---
*Последнее обновление: 2026-01-06*
