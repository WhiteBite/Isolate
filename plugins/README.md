# Isolate Plugins

Плагины для расширения функциональности Isolate.

## Структура плагина

```
my-plugin/
  Cargo.toml      # Rust проект
  src/lib.rs      # Код плагина
  manifest.toml   # Метаданные и capabilities
```

## Разработка плагина

1. Скопируйте discord-checker как шаблон
2. Измените manifest.toml
3. Реализуйте логику в src/lib.rs
4. Соберите: `cargo build --release --target wasm32-wasi`
5. Скопируйте .wasm в plugins/

## API

Плагины имеют доступ к:
- `http_get(url)` — HTTP GET запрос
- `http_post(url, body)` — HTTP POST запрос
- `log_info(msg)` — Логирование
- `config_get(key)` — Чтение конфигурации
