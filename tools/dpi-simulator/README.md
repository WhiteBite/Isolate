# DPI Simulator

Симулятор DPI-блокировок для тестирования Isolate. Работает в Hyper-V VM.

## Быстрый старт

```powershell
# 1. Запустить Hyper-V daemon (в админ PowerShell)
python D:\Sources\StartUp\Isolate\tools\mcp-hyperv\hyperv_daemon.py

# 2. Запустить VM
Get-VM "DPI-Simulator" | Start-VM

# 3. SSH туннель для API (если используете TUN/VPN)
ssh -f -N -L 8888:localhost:8888 dpi@192.168.100.10

# 4. Проверить статус
curl http://localhost:8888/status
```

## Архитектура

```
┌─────────────────┐     ┌──────────────────────────────────┐
│  Windows Host   │     │  DPI-Simulator VM (Ubuntu 22.04) │
│                 │     │                                  │
│  192.168.100.1  │◄───►│  192.168.100.10                  │
│  (DPI-Internal) │     │  eth0: DPI-Internal              │
│                 │     │  eth1: Default Switch (NAT)      │
│                 │     │                                  │
│  SSH tunnel     │     │  ┌─────────────────────────┐     │
│  localhost:8888 │◄────┼──│ dpi_simulator.py        │     │
│                 │     │  │ - NetfilterQueue        │     │
│                 │     │  │ - API :8888             │     │
│                 │     │  └─────────────────────────┘     │
└─────────────────┘     └──────────────────────────────────┘
```

## API

| Endpoint | Method | Описание |
|----------|--------|----------|
| `/status` | GET | Статус симулятора |
| `/stats` | GET | Детальная статистика |
| `/domains` | GET | Список блокируемых доменов |
| `/mode` | POST | Изменить режим `{"mode": "rst"}` |
| `/reset-stats` | POST | Сбросить статистику |
| `/stop` | POST | Остановить симулятор |

## Режимы блокировки

| Режим | Описание | Использование |
|-------|----------|---------------|
| `drop` | Дропает пакеты (timeout) | По умолчанию |
| `rst` | TCP RST (мгновенный сброс) | Имитация активного DPI |
| `throttle` | Задержка 2 сек | Имитация замедления |

```powershell
# Сменить режим на RST
Invoke-RestMethod -Uri "http://localhost:8888/mode" -Method POST -Body '{"mode":"rst"}' -ContentType "application/json"
```

## Блокируемые домены

Файл: `/opt/dpi-simulator/blocked_domains.txt`

- youtube.com, googlevideo.com, ytimg.com
- discord.com, discordapp.com
- twitch.tv
- instagram.com
- twitter.com, x.com

## Тестирование

```powershell
# 1. Сбросить статистику
Invoke-RestMethod -Uri "http://localhost:8888/reset-stats" -Method POST

# 2. Запустить тест изнутри VM
ssh dpi@192.168.100.10 "curl -s --connect-timeout 3 https://youtube.com"

# 3. Проверить статистику
Invoke-RestMethod -Uri "http://localhost:8888/stats"
```

## Управление VM

```powershell
# Статус
Get-VM "DPI-Simulator"

# Запуск/Остановка
Start-VM "DPI-Simulator"
Stop-VM "DPI-Simulator"

# Сервис внутри VM
ssh dpi@192.168.100.10 "sudo systemctl status dpi-simulator"
ssh dpi@192.168.100.10 "sudo systemctl restart dpi-simulator"
```

## Логи

```powershell
# Логи сервиса
ssh dpi@192.168.100.10 "sudo journalctl -u dpi-simulator -f"
```

## Интеграция с Isolate

Rust модуль `src-tauri/src/core/strategy_tester.rs` предоставляет API для тестирования стратегий:

```rust
use crate::core::strategy_tester::{StrategyTester, DpiSimulatorConfig};

// Создать тестер
let tester = StrategyTester::new();

// Проверить доступность DPI-симулятора
let available = tester.check_availability().await?;

// Получить статистику
let stats = tester.get_stats().await?;

// Тестировать стратегию
let result = tester.test_strategy(&strategy).await?;
```

## Credentials

- VM: `dpi` / `dpi`
- SSH: ключ `~/.ssh/id_rsa` (passwordless)
