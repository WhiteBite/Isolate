# Events Contract Documentation

Документация контрактов событий в приложении Isolate.

## Архитектура событий

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Rust Backend                                   │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │   Commands      │    │    Services     │    │    Monitor      │     │
│  │  (strategies,   │    │  (ai_pilot,     │    │  (health,       │     │
│  │   proxy_chain)  │    │   automation)   │    │   degradation)  │     │
│  └────────┬────────┘    └────────┬────────┘    └────────┬────────┘     │
│           │                      │                      │               │
│           └──────────────────────┼──────────────────────┘               │
│                                  │                                       │
│                          app.emit(event, payload)                        │
└──────────────────────────────────┼───────────────────────────────────────┘
                                   │
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                        Tauri Event Bridge                                │
│                    (WebSocket IPC to Frontend)                           │
└──────────────────────────────────┬───────────────────────────────────────┘
                                   │
                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                         Frontend (SvelteKit)                             │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Tauri Event Listeners                         │    │
│  │              listen('event:name', handler)                       │    │
│  └────────────────────────────┬────────────────────────────────────┘    │
│                               │                                          │
│                               ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        EventBus                                  │    │
│  │              (src/lib/stores/eventBus.svelte.ts)                │    │
│  │                                                                  │    │
│  │   eventBus.emit('event:name', payload)                          │    │
│  │   eventBus.on('event:name', handler)                            │    │
│  └────────────────────────────┬────────────────────────────────────┘    │
│                               │                                          │
│                               ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Stores & Components                           │    │
│  │   dashboardStore, aiPilotStore, proxyChainStore, etc.           │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────────────┘
```

## Паттерн проброса событий

Tauri события пробрасываются в EventBus для единообразия:

```typescript
// В store (например dashboard.svelte.ts)
const unlistenStrategyApplied = await listen<{ strategy_id?: string }>('strategy:applied', (event) => {
  // Пробрасываем в EventBus
  eventBus.emit('strategy:applied', {
    strategyId: event.payload?.strategy_id ?? 'unknown',
    timestamp: Date.now(),
  });
});
```

---

## Категории событий

### 1. Strategy Events

События связанные с применением и остановкой стратегий обхода DPI.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `strategy:applied` | `{ strategy_id, strategy_name?, service_id?, timestamp }` | Backend: `strategies.rs` | `dashboardStore`, UI компоненты | Стратегия успешно применена |
| `strategy:stopped` | `{ strategy_id?, service_id?, timestamp }` | Backend: `strategies.rs` | `dashboardStore`, UI компоненты | Стратегия остановлена |
| `strategy:changed` | `{ serviceId, strategyId }` | Frontend: EventBus | `dashboardStore` | Стратегия изменена для сервиса |
| `strategy:failover` | `{ failed_strategy, backup_strategy, reason }` | Backend: `strategies.rs` | UI компоненты | Автоматическое переключение на backup |
| `strategy:health` | `HealthCheckResult` | Backend: `monitor.rs` | Monitoring UI | Результат health check |
| `strategy:degraded` | `{ strategy_id, degradation_percent, metrics }` | Backend: `monitor.rs` | Monitoring UI | Деградация производительности |
| `strategy:recovered` | `{ strategy_id, recovery_time_ms }` | Backend: `monitor.rs` | Monitoring UI | Восстановление после деградации |
| `strategy:restart_requested` | `strategy_id: string` | Backend: `monitor.rs` | Strategy manager | Запрос на перезапуск стратегии |

**Payload Types:**

```typescript
// strategy:applied (EventBus)
interface StrategyAppliedPayload {
  strategyId: string;
  serviceId?: string;
  timestamp: number;
}

// strategy:stopped (EventBus)
interface StrategyStoppedPayload {
  strategyId?: string;
  serviceId?: string;
  timestamp: number;
}

// strategy:changed (EventBus)
interface StrategyChangedPayload {
  serviceId: string;
  strategyId: string;
}
```

**Гарантии:**
- `strategy:applied` всегда эмитится после успешного запуска winws/sing-box
- `strategy:stopped` эмитится при любой остановке (ручной или автоматической)
- Порядок: `applied` → (работа) → `stopped`

---

### 2. AI Pilot Events

События автоматической оптимизации стратегий.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `ai_pilot:started` | `{ started_at: number }` | Backend: `ai_pilot.rs` | `aiPilotStore` | AI Pilot запущен |
| `ai_pilot:stopped` | `{ stopped_at, total_checks, total_actions }` | Backend: `ai_pilot.rs` | `aiPilotStore` | AI Pilot остановлен |
| `ai_pilot:check_complete` | `{ service_id, result, latency?, strategy_id? }` | Backend: `ai_pilot.rs` | `aiPilotStore`, Dashboard | Проверка сервиса завершена |
| `ai_pilot:action` | `{ action: AiPilotAction }` | Backend: `ai_pilot.rs` | `aiPilotStore`, Dashboard, Notifications | AI Pilot выполнил действие |

**Payload Types:**

```typescript
// ai_pilot:started
interface AiPilotStartedEvent {
  started_at: number; // Unix timestamp ms
}

// ai_pilot:stopped
interface AiPilotStoppedEvent {
  stopped_at: number;
  total_checks: number;
  total_actions: number;
}

// ai_pilot:check_complete (EventBus)
interface AiPilotCheckCompletePayload {
  service_id: string;
  result: 'success' | 'failed' | 'timeout';
  latency?: number;
  strategy_id?: string;
}

// ai_pilot:action
interface AiPilotActionEvent {
  action: {
    id: string;
    action_type: 'switch_strategy' | 'restart' | 'fallback';
    service_id: string;
    from_strategy: string | null;
    to_strategy: string | null;
    reason: string;
    timestamp: number;
    success: boolean;
    is_plugin_strategy: boolean;
    source_plugin: string | null;
  };
}
```

**Гарантии:**
- `ai_pilot:started` эмитится один раз при запуске
- `ai_pilot:action` эмитится для каждого действия (включая undo)
- Частота проверок зависит от настроенного интервала (30/60/120 мин)

---

### 3. Proxy Chain Events

События управления цепочками прокси.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `proxy:chain_activated` | `{ chain_id, name }` | Backend: `proxy_chain.rs` | `proxyChainStore`, `dashboardStore` | Цепочка активирована |
| `proxy:chain_deactivated` | `{ chain_id }` | Backend: `proxy_chain.rs` | `proxyChainStore`, `dashboardStore` | Цепочка деактивирована |
| `proxy:imported` | `{ count, source }` | Frontend: `proxyChainStore` | UI компоненты | Прокси импортированы |
| `proxy:test_complete` | `{ proxy_id, latency, success }` | Frontend: EventBus | Proxy testing UI | Тест прокси завершён |

**Payload Types:**

```typescript
// proxy:chain_activated (EventBus)
interface ProxyChainActivatedPayload {
  chain_id: string;
  name: string;
}

// proxy:chain_deactivated (EventBus)
interface ProxyChainDeactivatedPayload {
  chain_id: string;
}

// proxy:imported (EventBus)
interface ProxyImportedPayload {
  count: number;
  source: string;
}
```

**Гарантии:**
- Только одна цепочка может быть активна одновременно
- При активации новой цепочки сначала эмитится `deactivated` для старой

---

### 4. Traffic & Connection Events

События мониторинга трафика и соединений.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `traffic:update` | `{ download, upload }` | Backend → EventBus | `dashboardStore` | Обновление статистики трафика |
| `connection:opened` | `{ serviceId, method }` | Backend → EventBus | `dashboardStore` | Новое соединение открыто |
| `connection:closed` | `{ serviceId }` | Backend → EventBus | `dashboardStore` | Соединение закрыто |

**Payload Types:**

```typescript
// traffic:update (EventBus)
interface TrafficUpdatePayload {
  download: number; // bytes
  upload: number;   // bytes
}

// connection:opened (EventBus)
interface ConnectionOpenedPayload {
  serviceId: string;
  method: string; // 'direct' | 'strategy' | 'proxy' | 'vless'
}

// connection:closed (EventBus)
interface ConnectionClosedPayload {
  serviceId: string;
}
```

**Гарантии:**
- `traffic:update` эмитится периодически (каждые 1-5 сек)
- `connection:opened` всегда предшествует `connection:closed` для того же соединения

---

### 5. Issue Events

События обнаружения и решения проблем.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `issue:detected` | `{ id, message, severity }` | Backend → EventBus | `dashboardStore` | Обнаружена проблема |
| `issue:resolved` | `{ id }` | Backend → EventBus | `dashboardStore` | Проблема решена |

**Payload Types:**

```typescript
// issue:detected (EventBus)
interface IssueDetectedPayload {
  id: string;
  message: string;
  severity: 'warning' | 'error';
}

// issue:resolved (EventBus)
interface IssueResolvedPayload {
  id: string;
}
```

---

### 6. Plugin Events

События системы плагинов.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `plugin:state_changed` | `{ plugin_id, enabled }` | Backend: `scripts.rs` | `pluginsStore` | Состояние плагина изменено |

**Payload Types:**

```typescript
// plugin:state_changed (Tauri)
interface PluginStateChangedPayload {
  plugin_id: string;
  enabled: boolean;
}
```

**Plugin Events API (для плагинов):**

```typescript
// Плагины могут эмитить и слушать события через context.events
interface PluginEvents {
  emit(event: string, data?: unknown): void;
  on(event: string, handler: (data: unknown) => void): () => void;
}

// Разрешённые события для плагинов (whitelist в manifest)
// Пример: ['status:changed', 'data:update']
```

---

### 7. Library Events

События библиотеки правил.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `library:rule_added` | `{ rule_id, pattern, method }` | Frontend: EventBus | Library UI | Правило добавлено |
| `library:rule_removed` | `{ rule_id }` | Frontend: EventBus | Library UI | Правило удалено |
| `library:rule_updated` | `{ rule_id, changes }` | Frontend: EventBus | Library UI | Правило обновлено |
| `library:method_changed` | `{ service_id, old_method, new_method }` | Frontend: EventBus | Library UI | Метод сервиса изменён |

---

### 8. Ipset Events

События управления IP-списками.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `ipset:updated` | `{ ip_count, source }` | Frontend: EventBus | Ipset UI | IP-список обновлён |
| `ipset:mode_changed` | `{ old_mode, new_mode }` | Frontend: EventBus | Ipset UI | Режим ipset изменён |

---

### 9. Tray Events

События системного трея.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `tray:optimize` | `'turbo' \| 'deep'` | Backend: `tray.rs` | Main window | Запрос оптимизации |
| `tray:toggle` | `void` | Backend: `tray.rs` | Main window | Переключение защиты |
| `tray:stop` | `void` | Backend: `tray.rs` | Main window | Остановка защиты |
| `tray:panic_reset` | `void` | Backend: `tray.rs` | Main window | Аварийный сброс |
| `tray:navigate` | `route: string` | Backend: `tray.rs` | Router | Навигация к странице |
| `tray:quit` | `void` | Backend: `tray.rs` | App lifecycle | Выход из приложения |
| `tray:quic_block` | `block: boolean` | Backend: `tray.rs` | Settings | Блокировка QUIC |
| `tray:service_toggle` | `{ service, active }` | Backend: `tray.rs` | Services UI | Переключение сервиса |
| `tray:profile_changed` | `{ profile, name }` | Backend: `tray.rs` | Profile UI | Профиль изменён |

---

### 10. Testing Events

События тестирования стратегий и прокси.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `test:progress` | `TestProgress` | Backend: `testing.rs` | Testing UI | Прогресс тестирования |
| `test:result` | `TestResult` | Backend: `testing.rs` | Testing UI | Результат теста |
| `test:complete` | `TestResult[]` | Backend: `testing.rs` | Testing UI | Все тесты завершены |

**Payload Types:**

```typescript
interface TestProgress {
  current: number;
  total: number;
  current_item: string;
  percent: number;
}

interface TestResult {
  id: string;
  name: string;
  success: boolean;
  latency_ms?: number;
  error?: string;
}
```

---

### 11. Automation Events

События автоматизации и оптимизации.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `automation:progress` | `OptimizationProgress` | Backend: `automation.rs` | Optimization UI | Прогресс оптимизации |
| `automation:complete` | `OptimizationResult` | Backend: `automation.rs` | Optimization UI | Оптимизация завершена |
| `automation:error` | `string` | Backend: `automation.rs` | Optimization UI | Ошибка оптимизации |
| `automation:domain_locked` | `{ domain, strategy_id, protocol }` | Backend: `automation.rs` | Monitor UI | Домен заблокирован на стратегии |
| `automation:domain_unlocked` | `{ domain, protocol }` | Backend: `automation.rs` | Monitor UI | Блокировка домена снята |
| `automation:monitor_started` | `{ domains: string[] }` | Backend: `automation.rs` | Monitor UI | Мониторинг запущен |
| `automation:monitor_stopped` | `void` | Backend: `automation.rs` | Monitor UI | Мониторинг остановлен |

---

### 12. Troubleshoot Events

События диагностики.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `troubleshoot:progress` | `TroubleshootProgress` | Backend: `troubleshoot.rs` | Troubleshoot UI | Прогресс диагностики |
| `troubleshoot:strategy_result` | `StrategyTestResult` | Backend: `troubleshoot.rs` | Troubleshoot UI | Результат теста стратегии |
| `troubleshoot:complete` | `TroubleshootResult` | Backend: `troubleshoot.rs` | Troubleshoot UI | Диагностика завершена |

---

### 13. System Events

Системные события.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `binaries:progress` | `DownloadProgress` | Backend: `system.rs` | Setup UI | Прогресс загрузки бинарников |
| `binaries:complete` | `void` | Backend: `system.rs` | Setup UI | Загрузка завершена |
| `integrity:warning` | `IntegrityResult` | Backend: `lib.rs` | Security UI | Предупреждение о целостности |
| `log:entry` | `LogEntry` | Backend: `log_capture.rs` | Logs UI | Новая запись в логе |

---

### 14. A/B Testing Events

События A/B тестирования.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `ab_test:started` | `testId: string` | Backend | A/B Testing UI | Тест запущен |
| `ab_test:completed` | `ABTestResult` | Backend | A/B Testing UI | Тест завершён |
| `ab_test:error` | `error: string` | Backend | A/B Testing UI | Ошибка теста |

---

### 15. Failover Events

События автоматического переключения.

| Event | Payload | Emitter | Listeners | Описание |
|-------|---------|---------|-----------|----------|
| `failover:triggered` | `{ previousStrategy, newStrategy }` | Backend: `services.rs` | Dashboard, Notifications | Failover сработал |
| `failover:apply_strategy` | `strategy_id: string` | Backend: `services.rs` | Strategy manager | Применить backup стратегию |

---

## Примеры использования

### Подписка на события в Store

```typescript
// src/lib/stores/dashboard.svelte.ts
import { listen } from '@tauri-apps/api/event';
import { eventBus } from './eventBus.svelte';

class DashboardStore {
  private unlisteners: UnlistenFn[] = [];

  async setupEventListeners() {
    // Подписка на Tauri событие
    const unlisten = await listen<{ strategy_id: string }>('strategy:applied', (event) => {
      console.log('Strategy applied:', event.payload.strategy_id);
      
      // Проброс в EventBus для других компонентов
      eventBus.emit('strategy:applied', {
        strategyId: event.payload.strategy_id,
        timestamp: Date.now(),
      });
    });
    this.unlisteners.push(unlisten);
    
    // Подписка на EventBus
    const unsubscribe = eventBus.onStrategyApplied((payload) => {
      this.protectionStatus = 'protected';
    });
    this.eventBusUnsubscribers.push(unsubscribe);
  }

  async cleanup() {
    for (const unlisten of this.unlisteners) {
      unlisten();
    }
  }
}
```

### Подписка в Svelte компоненте

```svelte
<script lang="ts">
  import { eventBus } from '$lib/stores/eventBus.svelte';
  import { onDestroy } from 'svelte';

  let lastAction = $state<string | null>(null);

  // Подписка с автоматической отпиской
  const unsubscribe = eventBus.onAiPilotAction((payload) => {
    lastAction = `${payload.action_type}: ${payload.service_id}`;
  });

  onDestroy(() => {
    unsubscribe();
  });
</script>

{#if lastAction}
  <p>Last AI Pilot action: {lastAction}</p>
{/if}
```

### Использование хука useEvent

```typescript
// src/lib/hooks/useEvent.svelte.ts
import { eventBus, type EventType, type EventPayload } from '$lib/stores/eventBus.svelte';

export function useEventEmitter<T extends EventType>(
  event: T
): (payload: EventPayload[T]) => void {
  return (payload: EventPayload[T]) => {
    eventBus.emit(event, payload);
  };
}

// Использование
const emitStrategyChanged = useEventEmitter('strategy:changed');
emitStrategyChanged({ serviceId: 'youtube', strategyId: 'fake-tls' });
```

### Эмит события из Rust

```rust
// src-tauri/src/commands/strategies.rs
use tauri::Emitter;

#[tauri::command]
pub async fn apply_strategy(
    app: AppHandle,
    strategy_id: String,
) -> Result<(), String> {
    // ... применение стратегии ...
    
    // Эмит события для frontend
    let _ = app.emit("strategy:applied", serde_json::json!({
        "strategy_id": strategy_id,
        "strategy_name": strategy_name,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }));
    
    Ok(())
}
```

---

## Гарантии и ограничения

### Порядок событий

1. **Strategy lifecycle:**
   ```
   strategy:applied → (работа) → strategy:stopped
   ```

2. **AI Pilot lifecycle:**
   ```
   ai_pilot:started → ai_pilot:check_complete* → ai_pilot:action* → ai_pilot:stopped
   ```

3. **Proxy chain lifecycle:**
   ```
   proxy:chain_activated → (работа) → proxy:chain_deactivated
   ```

### Частота событий

| Event | Частота |
|-------|---------|
| `traffic:update` | 1-5 сек |
| `ai_pilot:check_complete` | По интервалу (30/60/120 мин) |
| `test:progress` | Каждый тест |
| `log:entry` | По мере появления логов |

### Потеря событий

- События НЕ буферизуются — если нет подписчиков, событие теряется
- При переподключении WebSocket события за время отключения не восстанавливаются
- Рекомендуется синхронизировать состояние через команды после переподключения

### Thread Safety

- Tauri события эмитятся из любого потока (async runtime)
- EventBus работает в main thread (UI)
- Handlers вызываются синхронно в порядке подписки
