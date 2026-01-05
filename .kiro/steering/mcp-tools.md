# MCP Tools Usage Rules

## Context7 — Документация библиотек

### Когда использовать:
- При работе с внешними библиотеками (Tauri API, Svelte, Tokio, Serde и т.д.)
- Когда нужна актуальная документация по API
- При неуверенности в синтаксисе или параметрах функций
- Для поиска примеров использования

### Как использовать:
1. Сначала вызвать `mcp_context7_resolve_library_id` для получения ID библиотеки
2. Затем `mcp_context7_query_docs` с конкретным вопросом

### Примеры:
```
# Tauri API
resolve_library_id: "tauri", query: "How to invoke commands from frontend"

# Svelte 5
resolve_library_id: "svelte", query: "How to use $state and $effect runes"

# Tokio
resolve_library_id: "tokio", query: "How to spawn async tasks"
```

### НЕ использовать для:
- Внутреннего кода проекта (используй readFile/grepSearch)
- Базовых концепций Rust/TypeScript
- Вопросов не связанных с библиотеками

---

## Memory MCP — Граф знаний

### Когда использовать:
- Для сохранения важной информации о проекте между сессиями
- Для запоминания решений, архитектурных решений, найденных багов
- Для связывания сущностей (файлы ↔ функции ↔ баги)
- При начале новой сессии — проверить что уже известно

### Основные операции:

#### Создание сущностей:
```
mcp_memory_create_entities:
  - name: "orchestrator.rs"
    entityType: "file"
    observations: ["Координатор оптимизации стратегий", "Содержит test_strategy_global"]
```

#### Создание связей:
```
mcp_memory_create_relations:
  - from: "BUG-001"
    to: "orchestrator.rs"
    relationType: "located_in"
```

#### Поиск:
```
mcp_memory_search_nodes: query="strategy"
mcp_memory_read_graph  # Весь граф
```

### Что сохранять:
- Найденные баги и их решения
- Архитектурные решения
- Связи между модулями
- Важные паттерны кода
- Информацию о пользователе/проекте

### Структура сущностей для Isolate:
- **file**: исходные файлы с описанием назначения
- **bug**: найденные баги (BUG-XXX)
- **feature**: функциональность
- **module**: логические модули (strategy_engine, orchestrator)
- **decision**: архитектурные решения

---

## Приоритет инструментов

1. **Внутренний код** → readFile, grepSearch, fileSearch
2. **Внешние библиотеки** → Context7
3. **Долгосрочная память** → Memory MCP
4. **Актуальная информация из интернета** → web_search, webFetch
