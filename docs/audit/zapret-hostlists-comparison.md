# Сравнение Hostlists: Isolate vs zapret-discord-youtube

**Дата анализа:** Автоматически сгенерировано  
**Источник:** `thirdparty/zapret-discord-youtube/`

## 📊 Сводная таблица

| Файл | zapret-discord-youtube | Isolate | Разница |
|------|------------------------|---------|---------|
| **list-general.txt** (Discord + Cloudflare) | 45 доменов | 27 (discord.txt) + 17 (cloudflare в general.txt) = 44 | ≈ паритет |
| **list-google.txt** (YouTube) | 17 доменов | 17 (youtube.txt) | ✅ Полное совпадение |
| **list-exclude.txt** (Whitelist) | 25 доменов | 127 доменов | +102 у нас |
| **ipset-all.txt** (IP для bypass) | 1 IP | 0 | -1 |
| **ipset-exclude.txt** (Private IP) | 11 диапазонов | 0 | -11 |
| **hosts** (Discord media) | 200 записей | 0 | -200 |

## 🔍 Детальный анализ

### 1. Discord домены (list-general.txt)

#### ✅ Полностью совпадают с нашим discord.txt:
```
dis.gd, discord.app, discord.co, discord.com, discord.design, discord.dev,
discord.gift, discord.gifts, discord.gg, discord.media, discord.new,
discord.store, discord.status, discord-activities.com, discordactivities.com,
discordapp.com, discordapp.net, discordcdn.com, discordmerch.com,
discordpartygames.com, discordsays.com, discordsez.com, discordstatus.com,
discord-attachments-uploads-prd.storage.googleapis.com, stable.dl2.discordapp.net
```

#### ✅ Расширения Discord (есть у нас):
```
frankerfacez.com, ffzap.com, betterttv.net, 7tv.app, 7tv.io
```

### 2. Cloudflare ECH домены

#### ✅ Полностью совпадают:
```
cloudflare-ech.com, encryptedsni.com, cloudflareaccess.com, cloudflareapps.com,
cloudflarebolt.com, cloudflareclient.com, cloudflareinsights.com, cloudflareok.com,
cloudflarepartners.com, cloudflareportal.com, cloudflarepreview.com,
cloudflareresolve.com, cloudflaressl.com, cloudflarestatus.com,
cloudflarestorage.com, cloudflarestream.com, cloudflaretest.com
```

### 3. YouTube/Google домены (list-google.txt)

#### ✅ Полностью совпадают с нашим youtube.txt:
```
yt3.ggpht.com, yt4.ggpht.com, yt3.googleusercontent.com, googlevideo.com,
jnn-pa.googleapis.com, stable.dl2.discordapp.net, wide-youtube.l.google.com,
youtube-nocookie.com, youtube-ui.l.google.com, youtube.com,
youtubeembeddedplayer.googleapis.com, youtubekids.com, youtubei.googleapis.com,
youtu.be, yt-video-upload.l.google.com, ytimg.com, ytimg.l.google.com
```

### 4. Exclude List (Whitelist)

#### У них есть, у нас тоже:
```
pusher.com, live-video.net, ttvnw.net, twitch.tv, mail.ru, citilink.ru,
yandex.com, nvidia.com, donationalerts.com, vk.com, yandex.kz, mts.ru,
multimc.org, ya.ru, dns-shop.ru, habr.com, 3dnews.ru, sberbank.ru,
ozon.ru, wildberries.ru, microsoft.com, msi.com, akamaitechnologies.com,
2ip.ru, yandex.ru
```

#### ✅ У нас значительно больше:
- Российские банки: 25+ доменов
- Госуслуги: 15+ доменов
- Яндекс сервисы: 20+ доменов
- Mail.ru/VK: 10+ доменов
- Маркетплейсы: 20+ доменов
- Медиа: 25+ доменов
- Телеком: 10+ доменов

## 🚨 Чего НЕТ у нас

### 1. IP диапазоны (ipset-all.txt)
```
203.0.113.113/32  # Тестовый IP (RFC 5737 TEST-NET-3)
```
**Рекомендация:** Не нужен — это тестовый диапазон.

### 2. Private IP Exclude (ipset-exclude.txt)
```
0.0.0.0/8         # "This" network
10.0.0.0/8        # Private (Class A)
127.0.0.0/8       # Loopback
172.16.0.0/12     # Private (Class B)
192.168.0.0/16    # Private (Class C)
169.254.0.0/16    # Link-local
224.0.0.0/4       # Multicast
100.64.0.0/10     # Carrier-grade NAT
::1               # IPv6 loopback
fc00::/7          # IPv6 unique local
fe80::/10         # IPv6 link-local
```
**Рекомендация:** ⚠️ **ДОБАВИТЬ** — важно для корректной работы WinDivert, чтобы не перехватывать локальный трафик.

### 3. Discord Media Hosts (.service/hosts)
200 записей вида:
```
104.25.158.178 finland10000.discord.media
104.25.158.178 finland10001.discord.media
...
104.25.158.178 finland10199.discord.media
```
**Назначение:** Хардкод IP для Discord voice серверов в Финляндии (обход DNS-блокировки).

**Рекомендация:** ⚠️ **НЕ ДОБАВЛЯТЬ напрямую** — это workaround для DNS-блокировки. Лучше использовать DoH/DoT или ECH.

## 📋 Рекомендации

### ✅ Высокий приоритет

1. **Создать `configs/hostlists/ipset-exclude.txt`** с private IP диапазонами:
   ```
   # Private and reserved IP ranges to exclude from DPI bypass
   0.0.0.0/8
   10.0.0.0/8
   127.0.0.0/8
   172.16.0.0/12
   192.168.0.0/16
   169.254.0.0/16
   224.0.0.0/4
   100.64.0.0/10
   ::1
   fc00::/7
   fe80::/10
   ```

### ⚡ Средний приоритет

2. **Добавить поддержку IP-based exclude** в strategy_engine — сейчас работаем только с доменами.

### ℹ️ Низкий приоритет

3. **Discord media hosts** — не добавлять, это устаревший подход. ECH и DoH решают проблему лучше.

## 📈 Выводы

| Аспект | Статус |
|--------|--------|
| Discord домены | ✅ Полное покрытие |
| YouTube домены | ✅ Полное покрытие |
| Cloudflare ECH | ✅ Полное покрытие |
| Exclude list | ✅ У нас больше (127 vs 25) |
| IP exclude | ⚠️ Нужно добавить |
| Discord hosts hack | ❌ Не нужен |

**Общий вывод:** Наши hostlists полностью покрывают zapret-discord-youtube и даже превосходят по exclude list. Единственное улучшение — добавить IP exclude для private диапазонов.
