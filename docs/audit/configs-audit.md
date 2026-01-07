# Аудит конфигураций Isolate

**Дата:** 2025-01-XX  
**Версия:** 1.0  
**Проанализировано:**
- 30 стратегий в `configs/strategies/`
- 7 сервисов в `configs/services/`
- 14 hostlists в `configs/hostlists/`
- 1 шаблон sing-box в `configs/singbox/`

---

## 🔴 Критичные проблемы

### 1. Ошибки в параметрах winws

**Файлы:** `twitter_multisplit.yaml`, `ai_multisplit.yaml`, `meta_multisplit.yaml`

```yaml
# ОШИБКА: hostlist используется как pattern (бинарный файл)
--dpi-desync-split-seqovl-pattern=hostlists/twitter.txt  # ❌ Неверно!
```

**Проблема:** Параметр `--dpi-desync-split-seqovl-pattern` ожидает бинарный файл (TLS ClientHello), а не текстовый hostlist.

**Решение:** Заменить на корректный бинарный файл:
```yaml
--dpi-desync-split-seqovl-pattern=binaries/tls_clienthello_www_google_com.bin
```

### 2. Отсутствует сервис Google

**Проблема:** В `configs/services/` нет `google.yaml`, хотя:
- Есть `hostlists/google.txt` с 100+ доменами
- Стратегии `youtube_google.yaml`, `youtube_split.yaml` ссылаются на сервис `google`
- Google часто блокируется вместе с YouTube

**Решение:** Создать `configs/services/google.yaml`

### 3. Несоответствие services в стратегиях

| Стратегия | Указанные services | Проблема |
|-----------|-------------------|----------|
| `gaming_multisplit.yaml` | steam, epic_games, riot_games, blizzard, ea_origin, ubisoft, xbox, playstation, nintendo, wargaming, gaijin, twitch | Нет соответствующих service файлов |
| `streaming_*.yaml` | spotify, netflix, twitch, tiktok, soundcloud, deezer, apple_music | Только spotify.yaml существует |
| `ai_multisplit.yaml` | chatgpt | Сервис называется `chatgpt`, но файл `chatgpt.yaml` |

---

## 🟠 Важные улучшения

### 1. Дублирование доменов в hostlists

**Проблема:** Домены дублируются между файлами:

| Домен | Файлы |
|-------|-------|
| `youtube.com` | youtube.txt, google.txt, general.txt, all.txt |
| `googlevideo.com` | youtube.txt, google.txt, general.txt |
| `discord.com` | discord.txt, all.txt |

**Рекомендация:** 
- `all.txt` должен генерироваться автоматически из других файлов
- `general.txt` не должен дублировать специфичные hostlists
- Добавить скрипт для проверки дубликатов

### 2. Неконсистентные weight_hint

**Текущее распределение:**
```
weight_hint: 15 - general_simple_fake, universal_zapret
weight_hint: 13 - youtube_google
weight_hint: 12 - general_multisplit
weight_hint: 11 - general_fake_tls, telegram_fake, general_cutoff_n3
weight_hint: 10 - большинство стратегий
weight_hint: 5-9 - альтернативные стратегии
```

**Проблема:** Нет документации что означает weight_hint и как он используется при автовыборе.

**Рекомендация:** Добавить комментарий в модель:
```rust
/// Weight hint for strategy selection (higher = preferred)
/// 15+ = simple/fast strategies for weak DPI
/// 10-14 = recommended strategies
/// 5-9 = alternative/experimental strategies
pub weight_hint: i32,
```

### 3. Отсутствует версионирование стратегий

**Проблема:** Нет способа отследить изменения в стратегиях между версиями приложения.

**Рекомендация:** Добавить поле `version` в стратегии:
```yaml
id: "zapret_general_multisplit"
version: "1.2.0"  # Добавить
changelog:
  - "1.2.0: Добавлен cutoff для оптимизации"
  - "1.1.0: Обновлены split-seqovl параметры"
```

### 4. Неполные тесты сервисов

**Проблема:** Некоторые сервисы имеют минимальные тесты:

| Сервис | Количество тестов | Рекомендация |
|--------|-------------------|--------------|
| telegram.yaml | 4 | Добавить тесты для t.me, web.telegram.org |
| spotify.yaml | 7 | Добавить тест для spclient.wg.spotify.com |

### 5. Отсутствует hostlist для Twitch

**Проблема:** В `streaming.txt` есть домены Twitch, но нет отдельного `twitch.txt` для специализированных стратегий.

---

## 🟡 Рекомендации

### 1. Структура hostlists

**Текущая структура:**
```
hostlists/
├── ai.txt           # AI сервисы
├── all.txt          # Все домены (дубликаты!)
├── discord.txt      # Discord
├── exclude.txt      # Исключения
├── gaming.txt       # Игры (неполный)
├── general.txt      # Общие + дубликаты YouTube/Google
├── google.txt       # Google сервисы
├── ipset-all.txt    # IP диапазоны
├── ipset-exclude.txt # Исключённые IP
├── meta.txt         # Meta (Instagram, Facebook, WhatsApp)
├── streaming.txt    # Стриминг (Spotify, Netflix, Twitch, TikTok)
├── telegram.txt     # Telegram
├── twitter.txt      # Twitter/X
└── youtube.txt      # YouTube
```

**Рекомендуемая структура:**
```
hostlists/
├── services/        # По сервисам
│   ├── discord.txt
│   ├── youtube.txt
│   ├── telegram.txt
│   └── ...
├── categories/      # По категориям
│   ├── gaming.txt
│   ├── streaming.txt
│   └── ai.txt
├── generated/       # Автогенерируемые
│   └── all.txt
├── system/          # Системные
│   ├── exclude.txt
│   ├── ipset-all.txt
│   └── ipset-exclude.txt
└── README.md        # Документация формата
```

### 2. Добавить метаданные в hostlists

```txt
# @name: Discord
# @version: 2024.01.15
# @source: zapret-discord-youtube
# @domains: 30
# @last_updated: 2024-01-15

discord.com
...
```

### 3. Унифицировать naming conventions

**Текущее:**
- `zapret_discord_fake` vs `zapret_general_alt2`
- `youtube_google` vs `youtube_split` vs `youtube_zapret`

**Рекомендуемое:**
```
{engine}_{target}_{method}
zapret_discord_fake
zapret_youtube_multisplit
zapret_general_multisplit_alt2
vless_universal_proxy
```

### 4. Добавить JSON Schema для валидации

Создать `configs/schemas/strategy.schema.json`:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["id", "name", "family", "engine"],
  "properties": {
    "id": { "type": "string", "pattern": "^[a-z0-9_]+$" },
    "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
    ...
  }
}
```

### 5. Документировать параметры winws

Добавить комментарии к сложным параметрам:
```yaml
args:
  # split-seqovl: размер перекрытия TCP sequence numbers
  # 568 - оптимально для большинства DPI
  # 681 - для Google/YouTube (больший ClientHello)
  - "--dpi-desync-split-seqovl=568"
```

---

## 🟢 Идеи нового функционала

### 1. Новые сервисы для добавления

| Сервис | Приоритет | Причина |
|--------|-----------|---------|
| **Google** | Высокий | Часто блокируется с YouTube |
| **Twitch** | Высокий | Популярный стриминг |
| **TikTok** | Высокий | Массовый сервис |
| **Netflix** | Средний | Стриминг |
| **Steam** | Средний | Игровая платформа |
| **GitHub** | Средний | Иногда блокируется |
| **LinkedIn** | Низкий | Заблокирован в РФ |
| **Notion** | Низкий | Иногда проблемы |

### 2. Новые стратегии

#### 2.1 Стратегия для Twitch
```yaml
id: "zapret_twitch_multisplit"
name: "Twitch Multisplit"
description: "Стратегия для Twitch стриминга с поддержкой HLS"
services: [twitch]
```

#### 2.2 Стратегия для GitHub
```yaml
id: "zapret_github_fake"
name: "GitHub Fake"
description: "Обход для GitHub (git clone, API)"
services: [github]
```

#### 2.3 Комбинированная стратегия Discord + YouTube
```yaml
id: "zapret_discord_youtube_optimized"
name: "Discord + YouTube Optimized"
description: "Оптимизированная стратегия для Discord и YouTube"
services: [discord, youtube]
```

### 3. Автоматическое обновление hostlists

```rust
// Добавить в core/
pub struct HostlistUpdater {
    sources: Vec<HostlistSource>,
    update_interval: Duration,
}

impl HostlistUpdater {
    pub async fn update_from_github(&self, repo: &str) -> Result<()>;
    pub async fn merge_hostlists(&self) -> Result<()>;
    pub async fn validate_domains(&self) -> Result<ValidationReport>;
}
```

### 4. Профили провайдеров

```yaml
# configs/providers/rostelecom.yaml
id: "rostelecom"
name: "Ростелеком"
recommended_strategies:
  - zapret_general_multisplit
  - zapret_general_alt3
dpi_characteristics:
  - blocks_quic: true
  - blocks_sni: true
  - deep_packet_inspection: medium
```

### 5. A/B тестирование стратегий

```rust
pub struct StrategyABTest {
    pub strategy_a: String,
    pub strategy_b: String,
    pub test_domains: Vec<String>,
    pub metrics: ABTestMetrics,
}
```

### 6. Экспорт/импорт конфигураций

- Экспорт работающей стратегии в файл для шаринга
- Импорт стратегий от сообщества
- QR-код для быстрого шаринга настроек

---

## Статистика

### Стратегии по типам

| Тип | Количество | Примеры |
|-----|------------|---------|
| General | 15 | general_multisplit, general_alt2-8, general_fake_tls_* |
| YouTube | 4 | youtube_split, youtube_google, youtube_zapret |
| Discord | 3 | discord_fake, discord_zapret, universal_zapret |
| Telegram | 2 | telegram_multisplit, telegram_fake |
| Streaming | 2 | streaming_multisplit, streaming_fake |
| Gaming | 1 | gaming_multisplit |
| Meta | 1 | meta_multisplit |
| Twitter | 1 | twitter_multisplit |
| AI | 1 | ai_multisplit |
| VLESS | 1 | vless_proxy |

### Hostlists по размеру

| Файл | Доменов | Комментарий |
|------|---------|-------------|
| google.txt | ~100 | Полный |
| streaming.txt | ~150 | Полный |
| all.txt | ~200 | Дубликаты |
| ai.txt | ~70 | Полный |
| meta.txt | ~40 | Полный |
| discord.txt | ~30 | Полный |
| telegram.txt | ~30 | Полный |
| twitter.txt | ~20 | Полный |
| youtube.txt | ~15 | Минимальный (основное в google.txt) |
| gaming.txt | ~25 | Неполный |
| exclude.txt | ~150 | Полный |

### Покрытие сервисов

| Сервис | Service файл | Hostlist | Стратегии |
|--------|--------------|----------|-----------|
| YouTube | ✅ | ✅ | ✅ (4) |
| Discord | ✅ | ✅ | ✅ (3) |
| Telegram | ✅ | ✅ | ✅ (2) |
| Twitter | ✅ | ✅ | ✅ (1) |
| Meta | ✅ | ✅ | ✅ (1) |
| ChatGPT | ✅ | ✅ (ai.txt) | ✅ (1) |
| Spotify | ✅ | ✅ (streaming.txt) | ✅ (2) |
| **Google** | ❌ | ✅ | ⚠️ (в youtube_*) |
| **Twitch** | ❌ | ⚠️ (в streaming.txt) | ⚠️ (в streaming_*) |
| **Netflix** | ❌ | ✅ (streaming.txt) | ⚠️ (в streaming_*) |
| **TikTok** | ❌ | ✅ (streaming.txt) | ⚠️ (в streaming_*) |
| **Steam** | ❌ | ⚠️ (gaming.txt) | ⚠️ (в gaming_*) |

---

## План действий

### Немедленно (P0)
1. [ ] Исправить ошибки с `--dpi-desync-split-seqovl-pattern` в twitter/ai/meta стратегиях
2. [ ] Создать `configs/services/google.yaml`

### Краткосрочно (P1)
3. [ ] Убрать дубликаты из `general.txt`
4. [ ] Автогенерировать `all.txt` из других hostlists
5. [ ] Добавить сервисы: twitch, netflix, tiktok, steam

### Среднесрочно (P2)
6. [ ] Добавить версионирование стратегий
7. [ ] Создать JSON Schema для валидации
8. [ ] Документировать weight_hint
9. [ ] Реорганизовать структуру hostlists

### Долгосрочно (P3)
10. [ ] Автообновление hostlists
11. [ ] Профили провайдеров
12. [ ] A/B тестирование стратегий
