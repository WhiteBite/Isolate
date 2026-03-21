# Isolate

<p align="center">
  <img src="src-tauri/icons/icon.png" width="128" alt="Isolate Logo">
</p>

<p align="center">
  <strong>Интеллектуальный инструмент для обхода DPI-блокировок</strong>
</p>

<p align="center">
  <a href="#особенности">Особенности</a> •
  <a href="#установка">Установка</a> •
  <a href="#использование">Использование</a> •
  <a href="#разработка">Разработка</a>
</p>

<p align="center">
  <a href="https://github.com/WhiteBite/Isolate/actions/workflows/nightly.yml">
    <img src="https://github.com/WhiteBite/Isolate/actions/workflows/nightly.yml/badge.svg" alt="Nightly Build">
  </a>
  <a href="https://github.com/WhiteBite/Isolate/releases">
    <img src="https://img.shields.io/github/v/release/WhiteBite/Isolate?label=release" alt="Latest Release">
  </a>
</p>

---

## Что это?

Isolate — десктопное Windows-приложение, которое **автоматически** подбирает оптимальную стратегию обхода интернет-блокировок. Никаких батников, ручной настройки параметров или технических знаний — просто запустил и работает.

## Особенности

🔍 **Автоподбор стратегий**  
Два режима: Turbo (10 сек) для быстрого старта и Deep (2-3 мин) для полного перебора

🛡️ **Диагностика блокировок**  
Определяет тип блокировки (Timeout/RST/MITM) и подбирает подходящий метод обхода

⚡ **Block QUIC**  
Принудительный откат браузеров на TCP для корректной работы DPI-обхода

🔗 **Импорт VLESS**  
Поддержка vless:// ссылок для прокси-конфигураций

🚨 **Panic Button**  
Аварийный сброс всех сетевых настроек в один клик

📋 **Автообновление**  
Списки доменов обновляются без обновления программы

## Технологии

| Слой | Стек |
|------|------|
| Backend | Rust, Tauri 2.0, Tokio |
| Frontend | SvelteKit, TypeScript, Tailwind CSS |
| DPI-обход | Zapret (winws), WinDivert |
| Прокси | Sing-box |

## Установка

### Готовые сборки

Скачайте последний релиз из [Releases](https://github.com/WhiteBite/Isolate/releases).

### Сборка из исходников

```bash
# Клонирование
git clone https://github.com/WhiteBite/Isolate.git
cd Isolate

# Установка зависимостей
pnpm install

# Запуск в dev-режиме
pnpm tauri dev

# Сборка
pnpm tauri build
```

**Требования:**
- Windows 10/11 (x64)
- Node.js 18+
- Rust 1.70+
- pnpm

## Использование

1. **Запустите** Isolate от имени администратора
2. **Выберите сервис** (YouTube, Discord, etc.) или добавьте свой домен
3. **Нажмите "Найти стратегию"** — приложение автоматически протестирует варианты
4. **Примените** найденную стратегию

Для продвинутых пользователей доступен импорт VLESS-прокси и ручная настройка параметров.

## Roadmap

Key development directions:

- 🎨 **Dashboard & UX** — Real-time traffic stats, improved onboarding, accessibility (WCAG AA)
- 🔌 **Plugin Ecosystem** — Marketplace, permissions system, third-party strategy packs
- 🎮 **Game Mode** — Auto-detection of games, profile switching, optimized latency
- 🤖 **AI Pilot** — Background optimization, automatic strategy selection
- 🛡️ **Advanced DPI** — New bypass techniques, dynamic fingerprints, combo modes

See [TODO.md](TODO.md) for the full backlog and detailed tasks.

## Разработка

```
src/                  # Frontend (SvelteKit)
src-tauri/src/        # Backend (Rust)
configs/              # Стратегии и списки доменов
docs/                 # Документация
```

Подробнее в [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Статус

🚧 **В активной разработке**

## Лицензия

MIT

---

<p align="center">
  Made with ❤️ for free internet
</p>
