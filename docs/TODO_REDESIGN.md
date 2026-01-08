# 🎯 Isolate UI Redesign — TODO

> Дата создания: Январь 2026
> Цель: Редизайн Orchestra → Troubleshooter и Network → Proxy & VPN

---

## 🚀 Orchestra → Troubleshooter Redesign

### Концепция
Превращение Orchestra из технического инструмента оптимизации в user-friendly "Troubleshooter" с двумя сценариями:
1. **"У меня не работает"** — визуальный мастер диагностики
2. **"AI Pilot"** — фоновая автоматическая оптимизация

---

### Компоненты (Frontend)

#### Новые компоненты

- [ ] **[XL]** `TroubleshootWizard.svelte` — пошаговый мастер диагностики
  - Step 1: `ProblemSelector` — выбор проблемного сервиса из списка
    - Карточки: "YouTube тормозит", "Discord не подключается", "Telegram не работает"
    - Иконки сервисов, краткое описание проблемы
  - Step 2: `StrategySpeedtest.svelte` — визуальный тест стратегий (как спидтест)
    - Анимированный прогресс-бар для каждой стратегии
    - Реалтайм показ latency, статуса (✓/✗)
    - Визуализация "гонки" стратегий
  - Step 3: `ResultsRecommendation.svelte` — результаты с рекомендацией
    - Лучшая стратегия с метриками
    - Кнопка "Применить" / "Попробовать другую"
    - Объяснение почему эта стратегия лучше

- [ ] **[L]** `ProblemSelector.svelte` — выбор проблемы
  - Список сервисов с иконками
  - Поиск/фильтрация
  - Группировка: Видео, Мессенджеры, Соцсети, Игры
  - Состояние: idle, selected, testing

- [ ] **[XL]** `StrategySpeedtest.svelte` — визуальный тест стратегий
  - Анимация "спидтеста" для каждой стратегии
  - Прогресс-бар с градиентом (красный → желтый → зеленый)
  - Реалтайм обновление latency
  - Звуковые эффекты (опционально)
  - Состояния: waiting, testing, success, failed, skipped

- [ ] **[M]** `StrategyRaceItem.svelte` — элемент "гонки" стратегий
  - Название стратегии
  - Анимированный прогресс-бар
  - Latency badge
  - Статус иконка

- [ ] **[M]** `ResultsRecommendation.svelte` — результаты и рекомендация
  - Карточка лучшей стратегии
  - Сравнительная таблица топ-3
  - Кнопки действий
  - Объяснение выбора

- [ ] **[L]** `AIPilotPanel.svelte` — панель AI Pilot
  - Toggle "Фоновая оптимизация"
  - Настройки: интервал проверки (30мин/1час/2часа)
  - Лог последних действий
  - Статус: активен/неактивен
  - Уведомления о переключениях

- [ ] **[M]** `AIPilotNotification.svelte` — уведомление о переключении
  - "Мы переключили Discord на новую стратегию"
  - Показ старой и новой стратегии
  - Кнопка "Отменить"

- [ ] **[S]** `ServiceProblemCard.svelte` — карточка проблемы сервиса
  - Иконка сервиса
  - Название проблемы
  - Краткое описание
  - Hover эффект

#### Модификация существующих

- [ ] **[M]** `OrchestraStatus.svelte` → `TroubleshootStatus.svelte`
  - Адаптация под новый flow
  - Добавление состояния "diagnosing"

- [ ] **[S]** `OptimizationProgress.svelte` → переиспользовать в StrategySpeedtest
  - Рефакторинг для универсальности

- [ ] **[M]** `ActivityLog.svelte` → добавить фильтрацию по типу
  - Фильтр: все / AI Pilot / ручные тесты

---

### Stores (Frontend)

- [ ] **[M]** `troubleshoot.svelte.ts` — состояние Troubleshooter
  ```typescript
  interface TroubleshootState {
    step: 'select' | 'testing' | 'results';
    selectedService: string | null;
    selectedProblem: string | null;
    testingStrategies: StrategyTestState[];
    bestStrategy: string | null;
    bestLatency: number | null;
    isRunning: boolean;
  }
  
  interface StrategyTestState {
    id: string;
    name: string;
    status: 'waiting' | 'testing' | 'success' | 'failed' | 'skipped';
    latency: number | null;
    progress: number; // 0-100
  }
  ```

- [ ] **[M]** `aiPilot.svelte.ts` — состояние AI Pilot
  ```typescript
  interface AIPilotState {
    enabled: boolean;
    checkInterval: number; // минуты
    lastCheck: Date | null;
    lastAction: AIPilotAction | null;
    history: AIPilotAction[];
    monitoredServices: string[];
  }
  
  interface AIPilotAction {
    timestamp: Date;
    service: string;
    oldStrategy: string;
    newStrategy: string;
    reason: string;
  }
  ```

---

### Backend (Rust)

#### Новые команды

- [ ] **[L]** `troubleshoot_service` — запуск диагностики для сервиса
  ```rust
  #[tauri::command]
  pub async fn troubleshoot_service(
      state: State<'_, Arc<AppState>>,
      service_id: String,
      problem_type: String, // "slow", "blocked", "unstable"
  ) -> Result<TroubleshootSession, String>
  ```

- [ ] **[M]** `get_service_problems` — получить список проблем для сервиса
  ```rust
  #[tauri::command]
  pub fn get_service_problems(service_id: String) -> Vec<ServiceProblem>
  ```

- [ ] **[L]** `run_strategy_race` — запуск "гонки" стратегий
  ```rust
  #[tauri::command]
  pub async fn run_strategy_race(
      state: State<'_, Arc<AppState>>,
      service_id: String,
      strategy_ids: Vec<String>,
      timeout_ms: u64,
  ) -> Result<Vec<StrategyRaceResult>, String>
  ```
  - Эмитит события `troubleshoot:strategy_progress`
  - Эмитит события `troubleshoot:strategy_result`

- [ ] **[M]** `apply_troubleshoot_result` — применить результат диагностики
  ```rust
  #[tauri::command]
  pub async fn apply_troubleshoot_result(
      state: State<'_, Arc<AppState>>,
      service_id: String,
      strategy_id: String,
  ) -> Result<(), String>
  ```

- [ ] **[L]** `start_ai_pilot` — запуск AI Pilot
  ```rust
  #[tauri::command]
  pub async fn start_ai_pilot(
      state: State<'_, Arc<AppState>>,
      config: AIPilotConfig,
  ) -> Result<(), String>
  ```

- [ ] **[S]** `stop_ai_pilot` — остановка AI Pilot
  ```rust
  #[tauri::command]
  pub async fn stop_ai_pilot(state: State<'_, Arc<AppState>>) -> Result<(), String>
  ```

- [ ] **[S]** `get_ai_pilot_status` — статус AI Pilot
  ```rust
  #[tauri::command]
  pub fn get_ai_pilot_status(state: State<'_, Arc<AppState>>) -> AIPilotStatus
  ```

- [ ] **[M]** `get_ai_pilot_history` — история действий AI Pilot
  ```rust
  #[tauri::command]
  pub fn get_ai_pilot_history(
      state: State<'_, Arc<AppState>>,
      limit: usize,
  ) -> Vec<AIPilotAction>
  ```

#### Новые события

- [ ] **[S]** `troubleshoot:strategy_progress` — прогресс тестирования стратегии
- [ ] **[S]** `troubleshoot:strategy_result` — результат тестирования стратегии
- [ ] **[S]** `troubleshoot:complete` — завершение диагностики
- [ ] **[S]** `ai_pilot:strategy_changed` — AI Pilot сменил стратегию
- [ ] **[S]** `ai_pilot:check_started` — AI Pilot начал проверку
- [ ] **[S]** `ai_pilot:check_complete` — AI Pilot завершил проверку

#### Модификация существующих

- [ ] **[M]** `automation/optimizer.rs` — добавить режим "race"
  - Параллельное тестирование с ранним завершением
  - Эмит событий прогресса для UI

- [ ] **[M]** `automation/monitor.rs` — интеграция с AI Pilot
  - Периодические проверки
  - Автоматическое переключение

---

### API модули (Frontend)

- [ ] **[M]** `src/lib/api/troubleshoot.ts` — API для Troubleshooter
  ```typescript
  export async function troubleshootService(serviceId: string, problemType: string): Promise<TroubleshootSession>;
  export async function getServiceProblems(serviceId: string): Promise<ServiceProblem[]>;
  export async function runStrategyRace(serviceId: string, strategyIds: string[]): Promise<void>;
  export async function applyTroubleshootResult(serviceId: string, strategyId: string): Promise<void>;
  export function onStrategyProgress(callback: (progress: StrategyProgress) => void): Promise<UnlistenFn>;
  export function onStrategyResult(callback: (result: StrategyResult) => void): Promise<UnlistenFn>;
  export function onTroubleshootComplete(callback: (result: TroubleshootResult) => void): Promise<UnlistenFn>;
  ```

- [ ] **[M]** `src/lib/api/aiPilot.ts` — API для AI Pilot
  ```typescript
  export async function startAIPilot(config: AIPilotConfig): Promise<void>;
  export async function stopAIPilot(): Promise<void>;
  export async function getAIPilotStatus(): Promise<AIPilotStatus>;
  export async function getAIPilotHistory(limit?: number): Promise<AIPilotAction[]>;
  export function onStrategyChanged(callback: (action: AIPilotAction) => void): Promise<UnlistenFn>;
  ```

---

### Страница

- [ ] **[L]** `src/routes/orchestra/+page.svelte` → полный редизайн
  - Переименовать в Troubleshooter (или оставить route, изменить UI)
  - Два режима: Wizard / AI Pilot
  - Табы или toggle для переключения

---

## 🌐 Network → Proxy & VPN Redesign

### Концепция
Превращение Network из технической страницы в визуально привлекательный интерфейс:
1. **Карточки с флагами** вместо таблицы прокси
2. **Большая зона импорта** — "Paste key here"
3. **Визуальный конструктор цепочек** — Chain Builder

---

### Компоненты (Frontend)

#### Новые компоненты

- [ ] **[L]** `ProxyCardGrid.svelte` — сетка карточек прокси
  - Responsive grid (2-4 колонки)
  - Drag & drop для сортировки
  - Фильтрация по стране/протоколу

- [ ] **[M]** `ProxyCountryCard.svelte` — карточка прокси с флагом
  - Флаг страны (большой, заметный)
  - Название сервера
  - Протокол badge (VLESS, VMess, SS, Trojan)
  - Latency индикатор (цветной)
  - Статус: active/inactive/testing
  - Кнопки: Test, Edit, Delete, Activate

- [ ] **[XL]** `ImportZone.svelte` — большая зона импорта
  - Drag & drop область
  - Textarea "Paste key here"
  - Автодетект формата: vless://, ss://, vmess://, trojan://, Sing-box JSON
  - Превью импортируемых прокси
  - Batch import (несколько ссылок)
  - Импорт из файла

- [ ] **[S]** `ImportPreview.svelte` — превью импортируемого прокси
  - Парсинг и отображение данных
  - Валидация
  - Редактирование имени перед импортом

- [ ] **[XL]** `ChainBuilder.svelte` — визуальный конструктор цепочек
  - Drag & drop блоки
  - Типы блоков: DPI Bypass, Proxy, Direct, Internet
  - Соединительные линии между блоками
  - Валидация цепочки
  - Сохранение/загрузка пресетов

- [ ] **[M]** `ChainBlock.svelte` — блок в конструкторе цепочек
  - Иконка типа
  - Название
  - Настройки (для Proxy — выбор прокси)
  - Drag handle
  - Delete button

- [ ] **[M]** `ChainConnection.svelte` — соединение между блоками
  - SVG линия
  - Анимация потока данных
  - Статус соединения

- [ ] **[L]** `ChainPresets.svelte` — пресеты цепочек
  - "DPI Bypass → Internet" (базовый)
  - "DPI Bypass → VLESS (NL) → Internet" (для гео-блокировок)
  - "DPI Bypass → VLESS → VLESS → Internet" (double hop)
  - Кастомные пресеты пользователя

- [ ] **[M]** `CountryFlag.svelte` — компонент флага страны
  - SVG флаги или emoji
  - Fallback для неизвестных стран
  - Размеры: sm, md, lg

- [ ] **[S]** `ProtocolBadge.svelte` — badge протокола
  - Цветовая кодировка по протоколу
  - VLESS: синий, VMess: фиолетовый, SS: зеленый, Trojan: оранжевый

- [ ] **[M]** `LatencyIndicator.svelte` — индикатор задержки
  - Цветовая шкала: зеленый (<100ms), желтый (<300ms), красный (>300ms)
  - Анимация при тестировании
  - Tooltip с деталями

#### Модификация существующих

- [ ] **[M]** `GatewayCard.svelte` → `ProxyCountryCard.svelte`
  - Добавить флаг страны
  - Улучшить визуал

- [ ] **[M]** `GatewayList.svelte` → `ProxyCardGrid.svelte`
  - Переход от списка к сетке
  - Добавить фильтры

- [ ] **[S]** `AddGatewayModal.svelte` → интеграция с ImportZone
  - Использовать ImportZone внутри модала
  - Или заменить на inline ImportZone

---

### Stores (Frontend)

- [ ] **[M]** `proxyChain.svelte.ts` — состояние конструктора цепочек
  ```typescript
  interface ChainState {
    blocks: ChainBlock[];
    connections: ChainConnection[];
    isValid: boolean;
    validationErrors: string[];
    activePreset: string | null;
  }
  
  interface ChainBlock {
    id: string;
    type: 'dpi-bypass' | 'proxy' | 'direct' | 'internet';
    position: { x: number; y: number };
    config: Record<string, any>;
  }
  
  interface ChainConnection {
    from: string;
    to: string;
  }
  ```

- [ ] **[S]** `proxyImport.svelte.ts` — состояние импорта
  ```typescript
  interface ImportState {
    rawInput: string;
    parsedProxies: ParsedProxy[];
    validCount: number;
    invalidCount: number;
    isImporting: boolean;
  }
  ```

---

### Backend (Rust)

#### Новые команды

- [ ] **[M]** `parse_proxy_url` — парсинг URL без сохранения
  ```rust
  #[tauri::command]
  pub fn parse_proxy_url(url: String) -> Result<ProxyConfig, String>
  ```

- [ ] **[M]** `batch_import_proxies` — batch импорт прокси
  ```rust
  #[tauri::command]
  pub async fn batch_import_proxies(
      state: State<'_, Arc<AppState>>,
      urls: Vec<String>,
  ) -> Result<BatchImportResult, String>
  ```

- [ ] **[L]** `save_proxy_chain` — сохранение цепочки
  ```rust
  #[tauri::command]
  pub async fn save_proxy_chain(
      state: State<'_, Arc<AppState>>,
      chain: ProxyChain,
  ) -> Result<String, String>
  ```

- [ ] **[M]** `get_proxy_chains` — получение цепочек
  ```rust
  #[tauri::command]
  pub fn get_proxy_chains(state: State<'_, Arc<AppState>>) -> Vec<ProxyChain>
  ```

- [ ] **[L]** `apply_proxy_chain` — применение цепочки
  ```rust
  #[tauri::command]
  pub async fn apply_proxy_chain(
      state: State<'_, Arc<AppState>>,
      chain_id: String,
  ) -> Result<(), String>
  ```

- [ ] **[M]** `detect_proxy_country` — определение страны прокси
  ```rust
  #[tauri::command]
  pub async fn detect_proxy_country(
      state: State<'_, Arc<AppState>>,
      proxy_id: String,
  ) -> Result<CountryInfo, String>
  ```

- [ ] **[S]** `get_chain_presets` — получение пресетов цепочек
  ```rust
  #[tauri::command]
  pub fn get_chain_presets() -> Vec<ChainPreset>
  ```

#### Новые модели

- [ ] **[M]** `src-tauri/src/core/models/chain.rs`
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ProxyChain {
      pub id: String,
      pub name: String,
      pub blocks: Vec<ChainBlock>,
      pub is_active: bool,
  }
  
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ChainBlock {
      pub id: String,
      pub block_type: ChainBlockType,
      pub config: serde_json::Value,
      pub order: usize,
  }
  
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub enum ChainBlockType {
      DpiBypass { strategy_id: String },
      Proxy { proxy_id: String },
      Direct,
  }
  ```

---

### API модули (Frontend)

- [ ] **[M]** `src/lib/api/chain.ts` — API для цепочек
  ```typescript
  export async function saveProxyChain(chain: ProxyChain): Promise<string>;
  export async function getProxyChains(): Promise<ProxyChain[]>;
  export async function deleteProxyChain(id: string): Promise<void>;
  export async function applyProxyChain(id: string): Promise<void>;
  export async function getChainPresets(): Promise<ChainPreset[]>;
  ```

- [ ] **[S]** Расширить `src/lib/api/proxy.ts`
  ```typescript
  export async function parseProxyUrl(url: string): Promise<ProxyConfig>;
  export async function batchImportProxies(urls: string[]): Promise<BatchImportResult>;
  export async function detectProxyCountry(id: string): Promise<CountryInfo>;
  ```

---

### Страница

- [ ] **[XL]** `src/routes/network/+page.svelte` → полный редизайн
  - Три секции: Proxies Grid, Import Zone, Chain Builder
  - Responsive layout
  - Табы или accordion для секций на мобильных

---

## 📊 Оценка сложности

| Размер | Описание | Примерное время |
|--------|----------|-----------------|
| **S** | Простой компонент, минимальная логика | 1-2 часа |
| **M** | Средний компонент, умеренная логика | 2-4 часа |
| **L** | Сложный компонент, много логики/состояний | 4-8 часов |
| **XL** | Очень сложный, много интеграций | 8-16 часов |

---

## 🔄 Порядок реализации

### Фаза 1: Troubleshooter Core (1-2 недели)
1. `troubleshoot.svelte.ts` store
2. `ProblemSelector.svelte`
3. `StrategyRaceItem.svelte`
4. `StrategySpeedtest.svelte`
5. Backend: `troubleshoot_service`, `run_strategy_race`
6. `TroubleshootWizard.svelte`
7. Интеграция в страницу

### Фаза 2: AI Pilot (1 неделя)
1. `aiPilot.svelte.ts` store
2. Backend: `start_ai_pilot`, `stop_ai_pilot`, события
3. `AIPilotPanel.svelte`
4. `AIPilotNotification.svelte`
5. Интеграция в страницу

### Фаза 3: Network Proxies Grid (1 неделя)
1. `CountryFlag.svelte`, `ProtocolBadge.svelte`, `LatencyIndicator.svelte`
2. `ProxyCountryCard.svelte`
3. `ProxyCardGrid.svelte`
4. Backend: `detect_proxy_country`
5. Интеграция в страницу

### Фаза 4: Import Zone (3-5 дней)
1. `ImportPreview.svelte`
2. `ImportZone.svelte`
3. Backend: `parse_proxy_url`, `batch_import_proxies`
4. Интеграция в страницу

### Фаза 5: Chain Builder (1-2 недели)
1. `proxyChain.svelte.ts` store
2. Backend: модели, команды для цепочек
3. `ChainBlock.svelte`
4. `ChainConnection.svelte`
5. `ChainBuilder.svelte`
6. `ChainPresets.svelte`
7. Интеграция в страницу

---

## 📝 Заметки

### Зависимости между задачами
- `StrategySpeedtest` зависит от `StrategyRaceItem`
- `TroubleshootWizard` зависит от всех Step компонентов
- `ChainBuilder` зависит от `ChainBlock` и `ChainConnection`
- `ProxyCardGrid` зависит от `ProxyCountryCard`

### Риски
- **Chain Builder** — сложная drag & drop логика, может потребовать библиотеку
- **AI Pilot** — требует надежный background monitoring
- **Country detection** — нужен GeoIP сервис или база

### Переиспользование
- `StrategyRaceItem` можно использовать в Testing странице
- `CountryFlag` можно использовать везде где есть прокси
- `ImportZone` можно использовать в Subscriptions

---

*Документ создан для планирования редизайна UI Isolate*
