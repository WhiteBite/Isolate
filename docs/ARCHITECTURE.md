# Архитектура Isolate

## Общая схема

```
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Frontend                         │
│                   (SvelteKit + Tailwind)                    │
├─────────────────────────────────────────────────────────────┤
│                      Tauri Commands                         │
│              (IPC между Frontend и Backend)                 │
├─────────────────────────────────────────────────────────────┤
│                      Rust Backend                           │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐  │
│  │  Strategy   │  Diagnostic │   Process   │   Config    │  │
│  │   Engine    │   Module    │   Manager   │   Manager   │  │
│  └─────────────┴─────────────┴─────────────┴─────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    External Binaries                        │
│         ┌──────────────┐    ┌──────────────┐               │
│         │    winws     │    │   sing-box   │               │
│         │  (WinDivert) │    │   (proxy)    │               │
│         └──────────────┘    └──────────────┘               │
└─────────────────────────────────────────────────────────────┘
```

## Модули Backend

### 1. Strategy Engine
Ядро приложения. Отвечает за:
- Загрузку стратегий из YAML
- Классификацию: `parallel_safe` (VLESS) vs `driver_exclusive` (zapret)
- Запуск тестирования
- Ранжирование результатов

### 2. Diagnostic Module
Определяет тип блокировки:
- **Timeout** — пакеты дропаются
- **Instant RST** — активный DPI рвёт соединение
- **Wrong Certificate** — MITM-заглушка провайдера

### 3. Process Manager
Управление внешними процессами:
- Запуск/остановка winws и sing-box
- Мониторинг состояния
- Graceful shutdown
- Panic reset (аварийный сброс)

### 4. Config Manager
- Чтение/запись настроек (SQLite)
- Управление стратегиями (YAML)
- Автообновление hostlist'ов

## Алгоритм тестирования стратегий

```
1. Загрузить стратегии из strategies.yaml
2. Разделить на группы:
   - parallel_safe (VLESS/Sing-box)
   - driver_exclusive (Zapret/winws)

3. Фаза 1: Параллельный тест parallel_safe
   - Запустить все VLESS стратегии на разных SOCKS-портах
   - Таймаут: 3 сек на стратегию
   - Собрать результаты

4. Фаза 2: Последовательный тест driver_exclusive
   - Запускать winws стратегии по одной
   - Таймаут: 2-3 сек на стратегию
   - Kill процесс → следующая стратегия

5. Ранжировать по latency и стабильности
6. Применить лучшую стратегию
```

## Структура данных

### Стратегия (Strategy)
```yaml
id: "zapret_split_hello"
name: "Split TLS Hello"
type: "driver_exclusive"  # или "parallel_safe"
engine: "winws"           # или "singbox"
params:
  - "--wf-tcp=443"
  - "--split-pos=2"
priority: 10
```

### Результат теста (TestResult)
```rust
struct TestResult {
    strategy_id: String,
    success: bool,
    latency_ms: Option<u32>,
    error_type: Option<ErrorType>, // Timeout, RST, MITM
}
```

## Безопасность

- WinDivert требует прав администратора
- Все внешние бинарники подписаны или проверяются по хэшу
- Логи не содержат чувствительных данных
