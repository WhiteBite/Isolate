# Архитектура: Импорт прокси из буфера обмена

## Статус: ✅ Уже реализовано

Функциональность импорта прокси-конфигураций из буфера обмена **уже полностью реализована** в проекте.

---

## 1. Диаграмма потока данных

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (Svelte)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────┐    ┌─────────────────────┐    ┌───────────────────┐   │
│  │ "Import from     │───▶│ navigator.clipboard │───▶│ Валидация         │   │
│  │  Clipboard" btn  │    │ .readText()         │    │ префикса URL      │   │
│  └──────────────────┘    └─────────────────────┘    └─────────┬─────────┘   │
│                                                                │             │
│                                                                ▼             │
│                                                    ┌───────────────────┐    │
│                                                    │ importProxyUrl()  │    │
│                                                    │ (Tauri invoke)    │    │
│                                                    └─────────┬─────────┘    │
└──────────────────────────────────────────────────────────────┼──────────────┘
                                                               │
                                                               ▼ IPC
┌─────────────────────────────────────────────────────────────────────────────┐
│                              BACKEND (Rust)                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────┐    ┌─────────────────────┐    ┌───────────────────┐   │
│  │ import_proxy_url │───▶│ proxy_parser::      │───▶│ Определение       │   │
│  │ (Tauri command)  │    │ parse_proxy_url()   │    │ протокола         │   │
│  └──────────────────┘    └─────────────────────┘    └─────────┬─────────┘   │
│                                                                │             │
│                          ┌─────────────────────────────────────┼─────────┐   │
│                          │                                     ▼         │   │
│                          │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌──────┐│   │
│                          │  │ VLESS   │ │ VMess   │ │ SS      │ │Trojan││   │
│                          │  │ parser  │ │ parser  │ │ parser  │ │parser││   │
│                          │  └────┬────┘ └────┬────┘ └────┬────┘ └──┬───┘│   │
│                          │       │           │           │         │    │   │
│                          │       └───────────┴───────────┴─────────┘    │   │
│                          │                       │                       │   │
│                          │                       ▼                       │   │
│                          │              ┌───────────────┐                │   │
│                          │              │ ProxyConfig   │                │   │
│                          │              │ (unified)     │                │   │
│                          │              └───────┬───────┘                │   │
│                          │                      │                        │   │
│                          └──────────────────────┼────────────────────────┘   │
│                                                 │                            │
│                                                 ▼                            │
│                                    ┌────────────────────┐                    │
│                                    │ storage.save_proxy │                    │
│                                    │ (SQLite/JSON)      │                    │
│                                    └────────────────────┘                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Существующие файлы

### Backend (Rust)

| Файл | Назначение |
|------|------------|
| `src-tauri/src/core/proxy_parser.rs` | Парсеры всех протоколов |
| `src-tauri/src/commands/proxies.rs` | Tauri команды для прокси |
| `src-tauri/src/core/models/proxy.rs` | Модели данных |

### Frontend (Svelte)

| Файл | Назначение |
|------|------------|
| `src/routes/network/+page.svelte` | Страница Network с кнопкой импорта |
| `src/lib/api.ts` | API обёртки для Tauri команд |
| `src/lib/components/network/GatewayList.svelte` | Компонент списка гейтвеев |

---

## 3. API контракты (Tauri Commands)

### `import_proxy_url`

```rust
#[tauri::command]
pub async fn import_proxy_url(
    state: State<'_, Arc<AppState>>,
    url: String,
) -> Result<ProxyConfig, IsolateError>
```

**Вход:** URL строка (например `vless://uuid@server:443#name`)

**Выход:** `ProxyConfig` — унифицированная структура прокси

**Ошибки:**
- `IsolateError::Validation` — пустой URL
- `IsolateError::Config` — неподдерживаемый протокол или невалидный формат

### `parse_proxy_url` (preview без сохранения)

```rust
#[tauri::command]
pub async fn parse_proxy_url(url: String) -> Result<ProxyConfig, IsolateError>
```

---

## 4. Поддерживаемые форматы URL

### VLESS
```
vless://uuid@server:port?security=tls&sni=example.com&flow=xtls-rprx-vision#name
```
**Параметры:**
- `security`: tls, reality, none
- `sni`: Server Name Indication
- `flow`: xtls-rprx-vision, xtls-rprx-direct
- `type`: tcp, ws, grpc, http
- `path`: WebSocket path
- `host`: WebSocket host
- `pbk`: Reality public key
- `sid`: Reality short ID
- `fp`: Fingerprint (chrome, firefox, safari)

### VMess
```
vmess://base64({
  "v": "2",
  "ps": "name",
  "add": "server",
  "port": 443,
  "id": "uuid",
  "aid": 0,
  "net": "ws",
  "tls": "tls"
})
```

### Shadowsocks (SIP002)
```
ss://base64(method:password)@server:port#name
```
**Методы:** aes-256-gcm, aes-128-gcm, chacha20-ietf-poly1305

### Trojan
```
trojan://password@server:port?sni=example.com#name
```

### Hysteria2
```
hysteria2://auth@server:port?sni=example.com&obfs=salamander&obfs-password=xxx#name
hy2://auth@server:port#name
```

### TUIC
```
tuic://uuid:password@server:port?congestion_control=bbr&alpn=h3#name
```

### SOCKS5 / HTTP
```
socks5://user:pass@server:port
http://user:pass@server:port
```

---

## 5. Обработка ошибок

### Frontend (Svelte)

```typescript
async function handleImportFromClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    
    // Валидация на клиенте (быстрый фейл)
    const supportedPrefixes = ['vless://', 'vmess://', 'ss://', 'trojan://', 'hysteria://', 'hysteria2://'];
    const isSupported = supportedPrefixes.some(prefix => trimmed.toLowerCase().startsWith(prefix));
    
    if (!isSupported) {
      toasts.error('Unknown URL format. Supported: vless://, vmess://, ss://, trojan://');
      return;
    }
    
    const config = await importProxyUrl(trimmed);
    gateways = [...gateways, config];
    toasts.success(`Imported: ${config.name}`);
    
  } catch (e) {
    toasts.error(`Import failed: ${e.message}`);
  }
}
```

### Backend (Rust)

```rust
pub fn parse_proxy_url(url: &str) -> Result<ProxyConfig> {
    let url = url.trim();

    if url.starts_with("vless://") {
        parse_vless_url(url)
    } else if url.starts_with("vmess://") {
        parse_vmess_url(url)
    // ... другие протоколы
    } else {
        Err(anyhow!("Unsupported proxy protocol: {}", 
            url.split("://").next().unwrap_or("unknown")))
    }
}
```

### Типы ошибок

| Ошибка | Причина | Сообщение пользователю |
|--------|---------|------------------------|
| Empty clipboard | Буфер пуст | "Clipboard is empty" |
| Unknown protocol | Неизвестный префикс | "Unknown URL format. Supported: ..." |
| Invalid URL format | Невалидный синтаксис | "Failed to parse proxy URL: ..." |
| Missing required field | Нет UUID/server/port | "VLESS URL missing UUID" |
| Invalid base64 | Битый VMess/SS | "Failed to decode base64: ..." |

---

## 6. Модель данных

### ProxyConfig (унифицированная)

```rust
pub struct ProxyConfig {
    pub id: String,              // Уникальный ID (генерируется)
    pub name: String,            // Имя из URL fragment или server:port
    pub protocol: ProxyProtocol, // vless, vmess, ss, trojan, etc.
    pub server: String,          // Адрес сервера
    pub port: u16,               // Порт
    pub username: Option<String>,// Для SOCKS5/HTTP
    pub password: Option<String>,// Для SS/Trojan/SOCKS5
    pub uuid: Option<String>,    // Для VLESS/VMess/TUIC
    pub tls: bool,               // TLS включён
    pub sni: Option<String>,     // Server Name Indication
    pub transport: Option<String>,// tcp, ws, grpc, http
    pub custom_fields: HashMap<String, String>, // Дополнительные параметры
    pub active: bool,            // Активен ли прокси
}
```

### ProxyProtocol (enum)

```rust
pub enum ProxyProtocol {
    Socks5,
    Http,
    Https,
    Shadowsocks,
    Trojan,
    Vmess,
    Vless,
    Tuic,
    Hysteria,
    Hysteria2,
    Wireguard,
    Ssh,
}
```

---

## 7. UI компонент

Кнопка импорта находится в `GatewayList.svelte`:

```svelte
<button
  onclick={onimport}
  disabled={importing}
  class="..."
  title="Import from Clipboard"
>
  {#if importing}
    <Spinner size="sm" />
  {:else}
    <ClipboardIcon />
  {/if}
</button>
```

---

## 8. Возможные улучшения (не реализованы)

### 8.1 Batch Import
Импорт нескольких URL за раз (разделённых переносом строки):

```typescript
const urls = text.split('\n').filter(line => line.trim());
for (const url of urls) {
  await importProxyUrl(url);
}
```

### 8.2 JSON Config Import
Поддержка JSON конфигов sing-box/clash:

```json
{
  "outbounds": [
    { "type": "vless", "server": "...", "uuid": "..." }
  ]
}
```

### 8.3 Subscription Import
Импорт подписки по URL (уже есть команда `import_subscription`):

```rust
#[tauri::command]
pub async fn import_subscription(url: String) -> Result<Vec<ProxyConfig>>
```

---

## 9. Тестирование

### Unit тесты (Rust)

```rust
#[test]
fn test_parse_vless_url() {
    let url = "vless://uuid@server.com:443?security=tls#Test";
    let config = parse_proxy_url(url).unwrap();
    assert_eq!(config.protocol, ProxyProtocol::Vless);
    assert_eq!(config.server, "server.com");
}
```

### E2E тесты (Playwright)

```typescript
test('import from clipboard', async ({ page }) => {
  await page.evaluate(() => {
    navigator.clipboard.writeText('vless://uuid@server:443#Test');
  });
  
  await page.click('[title="Import from Clipboard"]');
  await expect(page.locator('text=Imported: Test')).toBeVisible();
});
```

---

## Заключение

Функциональность **полностью реализована**. Для использования:

1. Скопировать URL прокси в буфер обмена
2. Открыть страницу Network
3. Нажать кнопку "Import from Clipboard" в секции Gateways
4. Прокси добавится в список

Поддерживаются все основные протоколы: VLESS, VMess, Shadowsocks, Trojan, Hysteria2, TUIC, SOCKS5, HTTP.
