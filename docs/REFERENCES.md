# Референсные проекты

Проекты для изучения и заимствования идей.

## 1. zapret-discord-youtube

**URL:** https://github.com/Flowseal/zapret-discord-youtube

**Что изучить:**
- Готовые батники и конфиги для Discord/YouTube
- Параметры winws для разных провайдеров
- Списки доменов (hostlist)
- Структура аргументов командной строки

**Полезные файлы:**
- `*.bat` — примеры запуска winws
- `list-*.txt` — списки доменов для фильтрации

---

## 2. Throne

**URL:** https://github.com/throneproj/Throne

**Что изучить:**
- UI/UX десктопного приложения для DPI-обхода
- Архитектура Electron/Tauri приложения
- Логика автоподбора стратегий
- Работа с system tray
- Настройки пользователя

**Полезные файлы:**
- Структура проекта
- Компоненты UI
- Логика переключения режимов

---

## 3. zapret (оригинал)

**URL:** https://github.com/bol-van/zapret

**Что изучить:**
- Полная документация по параметрам winws
- Все доступные методы DPI-обхода
- Работа с WinDivert
- Примеры конфигураций для разных провайдеров
- Диагностика типов блокировок

**Полезные файлы:**
- `docs/` — документация
- `binaries/` — скомпилированные winws
- `ipset/` — списки IP
- Примеры скриптов

---

## Как использовать

1. Клонировать в `thirdparty/`:
```bash
git clone https://github.com/Flowseal/zapret-discord-youtube thirdparty/zapret-discord-youtube
git clone https://github.com/throneproj/Throne thirdparty/throne
git clone https://github.com/bol-van/zapret thirdparty/zapret
```

2. Изучить структуру и подходы
3. Адаптировать конфиги стратегий для Isolate
4. Взять бинарники winws из zapret

---

## Заметки по интеграции

### Из zapret-discord-youtube:
- Скопировать списки доменов в `configs/hostlists/`
- Адаптировать параметры winws в `configs/strategies/`

### Из Throne:
- Изучить UX паттерны
- Посмотреть как реализован tray
- Как показывается статус подключения

### Из zapret:
- Взять `winws.exe` и `WinDivert64.sys` в `src-tauri/binaries/`
- Изучить все параметры `--dpi-desync-*`
- Понять логику `--wf-tcp` и `--wf-udp`
