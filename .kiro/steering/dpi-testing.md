---
inclusion: manual
---

# DPI Testing Guide

## Архитектура тестирования

```
Host (Isolate + MCP) ──SSH──► WindhawkTest (192.168.100.20)
                                    │ winws.exe
                                    │ gateway .10
                                    ▼
                              DPI-Simulator (192.168.100.10)
                                    │ NFQUEUE blocking
                                    ▼
                                Internet
```

## Доступные MCP инструменты

### Управление VM
- `hyperv_vm_list` — список VM и их статус
- `hyperv_vm_start` / `hyperv_vm_stop` — запуск/остановка VM

### SSH команды
- `powershell_exec` с SSH: `ssh VM-test@192.168.100.20 "команда"`

### DPI тестирование
- `dpi_test_domain` — проверить блокировку домена (http_code: 000 = blocked)
- `winws_deploy` — развернуть winws на VM
- `winws_start` — запустить winws с аргументами стратегии
- `winws_stop` — остановить winws
- `dpi_full_test` — полный цикл: блокировка → обход → проверка

## Типичный workflow тестирования стратегии

1. Проверить что VM запущены: `hyperv_vm_list`
2. Проверить блокировку: `dpi_test_domain` domain=youtube.com
3. Запустить стратегию: `winws_start` args="--wf-tcp=80,443 --dpi-desync=fake"
4. Проверить обход: `dpi_test_domain` domain=youtube.com
5. Остановить: `winws_stop`

## Параметры winws для тестирования

Базовые:
- `--wf-tcp=80,443` — фильтр TCP портов
- `--wf-udp=443` — фильтр UDP (QUIC)

DPI обход:
- `--dpi-desync=fake` — fake пакет
- `--dpi-desync=split2` — разбиение пакета
- `--dpi-desync-ttl=3` — TTL для fake

## Заблокированные домены в DPI-Simulator
- youtube.com
- googlevideo.com
- ytimg.com

## Важно
- winws требует admin прав на VM
- Только ОДИН winws процесс одновременно (иначе BSOD)
- После теста ВСЕГДА останавливать winws
