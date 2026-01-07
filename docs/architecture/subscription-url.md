# Subscription URL Support — Архитектура

## Обзор

Поддержка subscription URL для автоматического обновления списка прокси. Subscription — это URL, который возвращает список прокси в формате base64-encoded или plain text.

## Текущее состояние

### Уже реализовано:
- **Парсинг subscription** — `src-tauri/src/core/proxy_parser.rs`:
  - `parse_subscription(content)` — парсит base64/plain text контент
  - Поддержка форматов: VLESS, VMess, Shadowsocks, Trojan, TUIC, Hysteria/Hysteria2
  - Автоматическое определение base64 (standard и URL-safe)
  
- **Команда импорта** — `src-tauri/src/commands/proxies.rs`:
  - `import_subscription(url)` — скачивает и парсит subscription
  - Сохраняет все прокси в storage

### Что нужно добавить:
1. Модель данных Subscription (хранение URL, интервал обновления)
2. Background task для периодического обновления
3. UI для управления subscriptions
4. Поддержка Clash формата

---

## 1. Модель данных

### Rust (src-tauri/src/core/models/subscription.rs)

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    /// Unique identifier
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// Subscription URL
    pub url: String,
    
    /// Update interval in seconds (0 = manual only)
    pub update_interval: u64,
    
    /// Last successful update timestamp
    pub last_updated: Option<DateTime<Utc>>,
    
    /// Last update error (if any)
    pub last_error: Option<String>,
    
    /// Number of proxies from this subscription
    pub proxy_count: u32,
    
    /// Whether auto-update is enabled
    pub auto_update: bool,
    
    /// User-Agent header for requests (optional)
    pub user_agent: Option<String>,
    
    /// Subscription format hint
    pub format: SubscriptionFormat,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// Subscription format
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionFormat {
    /// Auto-detect format
    #[default]
    Auto,
    /// V2Ray subscription (base64 encoded proxy URLs)
    V2ray,
    /// Clash YAML format
    Clash,
    /// Plain text (one URL per line)
    Plain,
    /// JSON array of ProxyConfig
    Json,
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            url: String::new(),
            update_interval: 86400, // 24 hours
            last_updated: None,
            last_error: None,
            proxy_count: 0,
            auto_update: true,
            user_agent: None,
            format: SubscriptionFormat::Auto,
            created_at: Utc::now(),
        }
    }
}

/// Result of subscription update
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionUpdateResult {
    pub subscription_id: String,
    pub success: bool,
    pub added_count: u32,
    pub updated_count: u32,
    pub removed_count: u32,
    pub error: Option<String>,
}
```

### TypeScript (src/lib/api/types.ts)

```typescript
export interface Subscription {
  id: string;
  name: string;
  url: string;
  updateInterval: number; // seconds
  lastUpdated: string | null; // ISO timestamp
  lastError: string | null;
  proxyCount: number;
  autoUpdate: boolean;
  userAgent: string | null;
  format: SubscriptionFormat;
  createdAt: string;
}

export type SubscriptionFormat = 'auto' | 'v2ray' | 'clash' | 'plain' | 'json';

export interface SubscriptionUpdateResult {
  subscriptionId: string;
  success: boolean;
  addedCount: number;
  updatedCount: number;
  removedCount: number;
  error: string | null;
}
```

---

## 2. Database Schema

### Миграция (src-tauri/src/core/storage/migrations.rs)

```sql
-- Subscriptions table
CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    update_interval INTEGER NOT NULL DEFAULT 86400,
    last_updated TEXT,
    last_error TEXT,
    proxy_count INTEGER NOT NULL DEFAULT 0,
    auto_update INTEGER NOT NULL DEFAULT 1,
    user_agent TEXT,
    format TEXT NOT NULL DEFAULT 'auto',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Link proxies to subscriptions
ALTER TABLE proxies ADD COLUMN subscription_id TEXT REFERENCES subscriptions(id);

-- Index for subscription lookup
CREATE INDEX IF NOT EXISTS idx_proxies_subscription 
    ON proxies(subscription_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_auto_update 
    ON subscriptions(auto_update, update_interval);
```

---

## 3. Tauri Commands

### CRUD Commands (src-tauri/src/commands/subscriptions.rs)

```rust
/// Get all subscriptions
#[tauri::command]
pub async fn get_subscriptions(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Subscription>, IsolateError>;

/// Add new subscription
#[tauri::command]
pub async fn add_subscription(
    state: State<'_, Arc<AppState>>,
    subscription: Subscription,
) -> Result<Subscription, IsolateError>;

/// Update subscription settings
#[tauri::command]
pub async fn update_subscription(
    state: State<'_, Arc<AppState>>,
    subscription: Subscription,
) -> Result<(), IsolateError>;

/// Delete subscription and optionally its proxies
#[tauri::command]
pub async fn delete_subscription(
    state: State<'_, Arc<AppState>>,
    id: String,
    delete_proxies: bool,
) -> Result<(), IsolateError>;

/// Manually trigger subscription update
#[tauri::command]
pub async fn update_subscription_now(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<SubscriptionUpdateResult, IsolateError>;

/// Update all subscriptions
#[tauri::command]
pub async fn update_all_subscriptions(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<SubscriptionUpdateResult>, IsolateError>;

/// Get proxies for a specific subscription
#[tauri::command]
pub async fn get_subscription_proxies(
    state: State<'_, Arc<AppState>>,
    subscription_id: String,
) -> Result<Vec<ProxyConfig>, IsolateError>;
```

### Регистрация в lib.rs

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::get_subscriptions,
    commands::add_subscription,
    commands::update_subscription,
    commands::delete_subscription,
    commands::update_subscription_now,
    commands::update_all_subscriptions,
    commands::get_subscription_proxies,
])
```

---

## 4. Background Task для периодического обновления

### SubscriptionUpdater (src-tauri/src/core/subscription_updater.rs)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

/// Subscription auto-updater service
pub struct SubscriptionUpdater {
    storage: Arc<Storage>,
    running: Arc<RwLock<bool>>,
    check_interval: Duration,
}

impl SubscriptionUpdater {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            running: Arc::new(RwLock::new(false)),
            check_interval: Duration::from_secs(60), // Check every minute
        }
    }

    /// Start the background updater
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let storage = self.storage.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));
            
            loop {
                ticker.tick().await;
                
                if !*running.read().await {
                    break;
                }

                // Get subscriptions that need update
                match storage.get_subscriptions_needing_update().await {
                    Ok(subscriptions) => {
                        for sub in subscriptions {
                            info!(id = %sub.id, name = %sub.name, "Auto-updating subscription");
                            
                            match Self::update_subscription_internal(&storage, &sub).await {
                                Ok(result) => {
                                    info!(
                                        id = %sub.id, 
                                        added = result.added_count,
                                        updated = result.updated_count,
                                        "Subscription updated"
                                    );
                                }
                                Err(e) => {
                                    warn!(id = %sub.id, error = %e, "Failed to update subscription");
                                    // Save error to subscription
                                    let _ = storage.set_subscription_error(&sub.id, &e.to_string()).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to get subscriptions for update");
                    }
                }
            }
        });
    }

    /// Stop the background updater
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Internal update logic
    async fn update_subscription_internal(
        storage: &Storage,
        subscription: &Subscription,
    ) -> Result<SubscriptionUpdateResult> {
        // Fetch content
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(subscription.user_agent.as_deref().unwrap_or("Isolate/1.0"))
            .build()?;

        let response = client.get(&subscription.url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP {}", response.status()));
        }

        let content = response.text().await?;

        // Parse based on format
        let proxies = match subscription.format {
            SubscriptionFormat::Auto => parse_subscription_auto(&content)?,
            SubscriptionFormat::V2ray => parse_subscription(&content)?,
            SubscriptionFormat::Clash => parse_clash_subscription(&content)?,
            SubscriptionFormat::Plain => parse_subscription_content(&content)?,
            SubscriptionFormat::Json => serde_json::from_str(&content)?,
        };

        // Update proxies in storage
        let result = storage.sync_subscription_proxies(&subscription.id, proxies).await?;

        // Update subscription metadata
        storage.update_subscription_success(&subscription.id, result.added_count + result.updated_count).await?;

        Ok(result)
    }
}
```

### Интеграция в AppState

```rust
pub struct AppState {
    pub storage: Arc<Storage>,
    pub subscription_updater: Arc<SubscriptionUpdater>,
    // ... other fields
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let storage = Arc::new(Storage::new().await?);
        let subscription_updater = Arc::new(SubscriptionUpdater::new(storage.clone()));
        
        // Start background updater
        subscription_updater.start().await;
        
        Ok(Self {
            storage,
            subscription_updater,
            // ...
        })
    }
}
```

---

## 5. Clash Subscription Parser

### Добавить в proxy_parser.rs

```rust
/// Parse Clash YAML subscription
pub fn parse_clash_subscription(content: &str) -> Result<Vec<ProxyConfig>> {
    let yaml: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| anyhow!("Invalid Clash YAML: {}", e))?;

    let proxies = yaml.get("proxies")
        .and_then(|p| p.as_sequence())
        .ok_or_else(|| anyhow!("No 'proxies' array in Clash config"))?;

    let mut configs = Vec::new();

    for proxy in proxies {
        match parse_clash_proxy(proxy) {
            Ok(config) => configs.push(config),
            Err(e) => {
                tracing::warn!("Failed to parse Clash proxy: {}", e);
            }
        }
    }

    if configs.is_empty() {
        Err(anyhow!("No valid proxies found in Clash config"))
    } else {
        Ok(configs)
    }
}

fn parse_clash_proxy(proxy: &serde_yaml::Value) -> Result<ProxyConfig> {
    let proxy_type = proxy.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow!("Missing proxy type"))?;

    let name = proxy.get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Unnamed")
        .to_string();

    let server = proxy.get("server")
        .and_then(|s| s.as_str())
        .ok_or_else(|| anyhow!("Missing server"))?
        .to_string();

    let port = proxy.get("port")
        .and_then(|p| p.as_u64())
        .ok_or_else(|| anyhow!("Missing port"))? as u16;

    match proxy_type {
        "vless" => parse_clash_vless(proxy, name, server, port),
        "vmess" => parse_clash_vmess(proxy, name, server, port),
        "ss" | "shadowsocks" => parse_clash_shadowsocks(proxy, name, server, port),
        "trojan" => parse_clash_trojan(proxy, name, server, port),
        "hysteria2" | "hy2" => parse_clash_hysteria2(proxy, name, server, port),
        "tuic" => parse_clash_tuic(proxy, name, server, port),
        "socks5" => parse_clash_socks5(proxy, name, server, port),
        "http" => parse_clash_http(proxy, name, server, port),
        _ => Err(anyhow!("Unsupported Clash proxy type: {}", proxy_type)),
    }
}

fn parse_clash_vless(
    proxy: &serde_yaml::Value,
    name: String,
    server: String,
    port: u16,
) -> Result<ProxyConfig> {
    let uuid = proxy.get("uuid")
        .and_then(|u| u.as_str())
        .ok_or_else(|| anyhow!("VLESS missing UUID"))?
        .to_string();

    let tls = proxy.get("tls")
        .and_then(|t| t.as_bool())
        .unwrap_or(true);

    let sni = proxy.get("servername")
        .or_else(|| proxy.get("sni"))
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let mut custom_fields = HashMap::new();

    if let Some(flow) = proxy.get("flow").and_then(|f| f.as_str()) {
        custom_fields.insert("flow".to_string(), flow.to_string());
    }

    if let Some(network) = proxy.get("network").and_then(|n| n.as_str()) {
        custom_fields.insert("transport_type".to_string(), network.to_string());
    }

    // Reality params
    if let Some(reality_opts) = proxy.get("reality-opts") {
        if let Some(pbk) = reality_opts.get("public-key").and_then(|p| p.as_str()) {
            custom_fields.insert("public_key".to_string(), pbk.to_string());
        }
        if let Some(sid) = reality_opts.get("short-id").and_then(|s| s.as_str()) {
            custom_fields.insert("short_id".to_string(), sid.to_string());
        }
    }

    Ok(ProxyConfig {
        id: generate_id("vless"),
        name,
        protocol: ProxyProtocol::Vless,
        server,
        port,
        username: None,
        password: None,
        uuid: Some(uuid),
        tls,
        sni,
        transport: proxy.get("network").and_then(|n| n.as_str()).map(|s| s.to_string()),
        custom_fields,
        active: false,
    })
}

// Similar implementations for other protocols...
```

---

## 6. UI Components

### Структура компонентов

```
src/lib/components/subscriptions/
├── index.ts                    # Re-exports
├── SubscriptionList.svelte     # Список subscriptions
├── SubscriptionCard.svelte     # Карточка subscription
├── AddSubscriptionModal.svelte # Модалка добавления
├── EditSubscriptionModal.svelte # Модалка редактирования
└── SubscriptionStatus.svelte   # Статус обновления
```

### SubscriptionList.svelte

```svelte
<script lang="ts">
  import type { Subscription } from '$lib/api';
  
  let subscriptions = $state<Subscription[]>([]);
  let loading = $state(true);
  let updating = $state<string | null>(null);
  
  // Props
  let { 
    onadd,
    onedit,
    ondelete,
    onupdate 
  }: {
    onadd: () => void;
    onedit: (sub: Subscription) => void;
    ondelete: (id: string) => void;
    onupdate: (id: string) => void;
  } = $props();
  
  $effect(() => {
    loadSubscriptions();
  });
  
  async function loadSubscriptions() {
    loading = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      subscriptions = await invoke<Subscription[]>('get_subscriptions');
    } catch (e) {
      console.error('Failed to load subscriptions:', e);
    } finally {
      loading = false;
    }
  }
  
  async function handleUpdate(id: string) {
    updating = id;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_subscription_now', { id });
      await loadSubscriptions();
    } finally {
      updating = null;
    }
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h2 class="text-lg font-semibold text-white">Subscriptions</h2>
    <button
      onclick={onadd}
      class="px-3 py-1.5 bg-blue-500/20 text-blue-400 rounded-lg hover:bg-blue-500/30"
    >
      Add Subscription
    </button>
  </div>
  
  {#if loading}
    <div class="text-zinc-500">Loading...</div>
  {:else if subscriptions.length === 0}
    <div class="text-zinc-500 text-center py-8">
      No subscriptions yet. Add one to get started.
    </div>
  {:else}
    <div class="space-y-3">
      {#each subscriptions as sub}
        <SubscriptionCard
          subscription={sub}
          updating={updating === sub.id}
          onedit={() => onedit(sub)}
          ondelete={() => ondelete(sub.id)}
          onupdate={() => handleUpdate(sub.id)}
        />
      {/each}
    </div>
  {/if}
</div>
```

### AddSubscriptionModal.svelte

```svelte
<script lang="ts">
  import BaseModal from '$lib/components/BaseModal.svelte';
  import type { Subscription, SubscriptionFormat } from '$lib/api';
  
  let { open = $bindable(false), onadd }: {
    open: boolean;
    onadd: (sub: Omit<Subscription, 'id' | 'createdAt' | 'lastUpdated' | 'lastError' | 'proxyCount'>) => void;
  } = $props();
  
  let name = $state('');
  let url = $state('');
  let updateInterval = $state(86400); // 24 hours
  let autoUpdate = $state(true);
  let format = $state<SubscriptionFormat>('auto');
  
  const intervalOptions = [
    { value: 0, label: 'Manual only' },
    { value: 3600, label: 'Every hour' },
    { value: 21600, label: 'Every 6 hours' },
    { value: 43200, label: 'Every 12 hours' },
    { value: 86400, label: 'Every 24 hours' },
    { value: 604800, label: 'Every week' },
  ];
  
  function handleSubmit() {
    if (!name.trim() || !url.trim()) return;
    
    onadd({
      name: name.trim(),
      url: url.trim(),
      updateInterval,
      autoUpdate,
      format,
      userAgent: null,
    });
    
    // Reset form
    name = '';
    url = '';
    updateInterval = 86400;
    autoUpdate = true;
    format = 'auto';
    open = false;
  }
</script>

<BaseModal bind:open class="w-full max-w-md">
  <form onsubmit|preventDefault={handleSubmit} class="p-6 space-y-4">
    <h2 class="text-lg font-semibold text-white">Add Subscription</h2>
    
    <div>
      <label class="block text-sm text-zinc-400 mb-1">Name</label>
      <input
        type="text"
        bind:value={name}
        placeholder="My Subscription"
        class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white"
      />
    </div>
    
    <div>
      <label class="block text-sm text-zinc-400 mb-1">URL</label>
      <input
        type="url"
        bind:value={url}
        placeholder="https://example.com/subscription"
        class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white"
      />
    </div>
    
    <div>
      <label class="block text-sm text-zinc-400 mb-1">Format</label>
      <select
        bind:value={format}
        class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white"
      >
        <option value="auto">Auto-detect</option>
        <option value="v2ray">V2Ray (base64)</option>
        <option value="clash">Clash (YAML)</option>
        <option value="plain">Plain text</option>
        <option value="json">JSON</option>
      </select>
    </div>
    
    <div>
      <label class="block text-sm text-zinc-400 mb-1">Update Interval</label>
      <select
        bind:value={updateInterval}
        class="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-white"
      >
        {#each intervalOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>
    
    <label class="flex items-center gap-2">
      <input type="checkbox" bind:checked={autoUpdate} class="rounded" />
      <span class="text-sm text-zinc-300">Enable auto-update</span>
    </label>
    
    <div class="flex gap-3 pt-4">
      <button
        type="button"
        onclick={() => open = false}
        class="flex-1 px-4 py-2 bg-zinc-800 text-zinc-300 rounded-lg"
      >
        Cancel
      </button>
      <button
        type="submit"
        disabled={!name.trim() || !url.trim()}
        class="flex-1 px-4 py-2 bg-blue-500 text-white rounded-lg disabled:opacity-50"
      >
        Add
      </button>
    </div>
  </form>
</BaseModal>
```

---

## 7. Форматы Subscription

### V2Ray Subscription
```
Base64 encoded content:
vless://uuid@server1:443#Name1
vmess://base64json
ss://base64@server:port#Name
trojan://pass@server:443#Name
```

### Clash Subscription (YAML)
```yaml
proxies:
  - name: "Server 1"
    type: vless
    server: example.com
    port: 443
    uuid: "550e8400-e29b-41d4-a716-446655440000"
    tls: true
    servername: example.com
    flow: xtls-rprx-vision
    
  - name: "Server 2"
    type: vmess
    server: vmess.example.com
    port: 443
    uuid: "uuid-here"
    alterId: 0
    cipher: auto
    tls: true
    
  - name: "SS Server"
    type: ss
    server: ss.example.com
    port: 8388
    cipher: aes-256-gcm
    password: "password"
```

### Plain Text
```
vless://uuid@server1:443#Name1
vless://uuid@server2:443#Name2
# Comments are ignored
vmess://base64json
```

---

## 8. API Functions (Frontend)

### src/lib/api/subscriptions.ts

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Subscription, SubscriptionUpdateResult } from './types';

export async function getSubscriptions(): Promise<Subscription[]> {
  return invoke<Subscription[]>('get_subscriptions');
}

export async function addSubscription(
  subscription: Omit<Subscription, 'id' | 'createdAt' | 'lastUpdated' | 'lastError' | 'proxyCount'>
): Promise<Subscription> {
  return invoke<Subscription>('add_subscription', { subscription });
}

export async function updateSubscription(subscription: Subscription): Promise<void> {
  return invoke('update_subscription', { subscription });
}

export async function deleteSubscription(id: string, deleteProxies = false): Promise<void> {
  return invoke('delete_subscription', { id, deleteProxies });
}

export async function updateSubscriptionNow(id: string): Promise<SubscriptionUpdateResult> {
  return invoke<SubscriptionUpdateResult>('update_subscription_now', { id });
}

export async function updateAllSubscriptions(): Promise<SubscriptionUpdateResult[]> {
  return invoke<SubscriptionUpdateResult[]>('update_all_subscriptions');
}

export async function getSubscriptionProxies(subscriptionId: string): Promise<ProxyConfig[]> {
  return invoke<ProxyConfig[]>('get_subscription_proxies', { subscriptionId });
}
```

---

## 9. План реализации

### Phase 1: Backend (2-3 дня)
1. [ ] Добавить модель `Subscription` в `models/`
2. [ ] Добавить миграцию БД для таблицы `subscriptions`
3. [ ] Реализовать Storage queries для subscriptions
4. [ ] Добавить Clash parser в `proxy_parser.rs`
5. [ ] Создать `subscription_updater.rs` с background task
6. [ ] Добавить Tauri commands в `commands/subscriptions.rs`

### Phase 2: Frontend (2-3 дня)
1. [ ] Создать компоненты в `components/subscriptions/`
2. [ ] Добавить API функции в `lib/api/subscriptions.ts`
3. [ ] Интегрировать в страницу Network
4. [ ] Добавить индикатор обновления в UI

### Phase 3: Polish (1 день)
1. [ ] Тестирование с реальными subscriptions
2. [ ] Обработка ошибок и edge cases
3. [ ] Документация

---

## 10. Зависимости

### Rust
- `serde_yaml` — для парсинга Clash формата (уже есть)
- `chrono` — для работы с датами (уже есть)
- `reqwest` — для HTTP запросов (уже есть)
- `base64` — для декодирования (уже есть)

### Frontend
- Нет новых зависимостей
