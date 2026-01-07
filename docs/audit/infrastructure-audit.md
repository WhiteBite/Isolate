# Infrastructure & DevOps Audit

**Дата:** 2026-01-07  
**Версия проекта:** 0.1.0  
**Аудитор:** Kiro AI

---

## Содержание

1. [Критичные проблемы (🔴)](#критичные-проблемы-)
2. [Важные улучшения (🟠)](#важные-улучшения-)
3. [Рекомендации (🟡)](#рекомендации-)
4. [Идеи нового функционала (🟢)](#идеи-нового-функционала-)

---

## Критичные проблемы (🔴)

### 🔴 1. Updater не настроен — автообновления не работают

**Файл:** `src-tauri/tauri.conf.json`

```json
"updater": {
    "_comment": "TODO: Generate real minisign keypair...",
    "pubkey": "",  // ← ПУСТОЙ КЛЮЧ
    "endpoints": [
        "https://github.com/aspect-build/isolate/releases/latest/download/latest.json"
    ]
}
```

**Проблема:**
- Публичный ключ пустой — updater не будет работать
- URL указывает на `aspect-build/isolate` — вероятно неверный репозиторий
- Нет `latest.json` в релизах

**Решение:**
```bash
# 1. Сгенерировать ключи
tauri signer generate -w ~/.tauri/isolate.key

# 2. Добавить pubkey в tauri.conf.json
# 3. Добавить TAURI_SIGNING_PRIVATE_KEY в GitHub Secrets
# 4. Создать workflow для генерации latest.json
```

---

### 🔴 2. Security audit в CI игнорирует уязвимости

**Файл:** `.github/workflows/ci.yml`

```yaml
- name: npm audit
  run: pnpm audit --audit-level=high
  continue-on-error: true  # ← ИГНОРИРУЕТ ОШИБКИ
  
- name: Cargo audit
  run: cargo audit --file src-tauri/Cargo.lock
  continue-on-error: true  # ← ИГНОРИРУЕТ ОШИБКИ
```

**Проблема:** Уязвимости в зависимостях не блокируют CI — можно смержить код с критическими CVE.

**Решение:**
```yaml
- name: npm audit
  run: pnpm audit --audit-level=critical
  # Убрать continue-on-error или сделать отдельный job

- name: Cargo audit
  run: cargo audit --deny warnings --file src-tauri/Cargo.lock
```

---

### 🔴 3. E2E Hyper-V workflow использует небезопасную передачу credentials

**Файл:** `.github/workflows/e2e-hyperv.yml`

```yaml
Invoke-Command -ComputerName $vmIp -Credential ${{ secrets.VM_CREDENTIALS }}
```

**Проблема:**
- `VM_CREDENTIALS` передаётся напрямую в PowerShell — может логироваться
- Нет проверки что VM_CREDENTIALS существует
- Self-hosted runner с Hyper-V требует особой защиты

**Решение:**
```yaml
- name: Configure DPI mode on VM
  shell: pwsh
  env:
    VM_CRED: ${{ secrets.VM_CREDENTIALS }}
  run: |
    $securePassword = ConvertTo-SecureString $env:VM_CRED -AsPlainText -Force
    $credential = New-Object PSCredential("VM-test", $securePassword)
    # ...
```

---

### 🔴 4. Хэши бинарников не проверяются

**Файл:** `scripts/download-binaries.ps1`

```powershell
$EXPECTED_HASHES = @{
    "winws.exe"       = "SKIP"  # ← НЕ ПРОВЕРЯЕТСЯ
    "cygwin1.dll"     = "SKIP"
    "sing-box.exe"    = "SKIP"
    "WinDivert.dll"   = "SKIP"
    "WinDivert64.sys" = "SKIP"
}
```

**Проблема:** Скачиваемые бинарники не верифицируются — supply chain attack vector.

**Решение:**
```powershell
$EXPECTED_HASHES = @{
    "winws.exe"       = "abc123..."  # Реальный SHA256
    "sing-box.exe"    = "def456..."
    # ...
}

function Verify-Hash {
    param([string]$Path, [string]$Expected)
    $actual = (Get-FileHash $Path -Algorithm SHA256).Hash
    if ($actual -ne $Expected) {
        throw "Hash mismatch for $Path"
    }
}
```

---

## Важные улучшения (🟠)

### 🟠 1. Нет coverage отчётов в CI

**Текущее состояние:**
- `vitest.config.ts` настроен на coverage с порогом 50%
- CI не запускает coverage и не публикует отчёты

**Решение:**
```yaml
# .github/workflows/ci.yml
- name: Run TypeScript tests with coverage
  run: pnpm test -- --coverage

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: ./coverage/lcov.info
    fail_ci_if_error: true
```

---

### 🟠 2. Rust тесты не запускаются с coverage

**Проблема:** `cargo test` запускается, но без coverage.

**Решение:**
```yaml
- name: Install cargo-llvm-cov
  run: cargo install cargo-llvm-cov

- name: Run Rust tests with coverage
  run: cargo llvm-cov --manifest-path src-tauri/Cargo.toml --lcov --output-path lcov.info
```

---

### 🟠 3. Дублирование capabilities для shell:allow-execute и shell:allow-spawn

**Файл:** `src-tauri/capabilities/default.json`

Одинаковые валидаторы для `winws` и `singbox` дублируются в `shell:allow-execute` и `shell:allow-spawn` (~200 строк дублирования).

**Решение:** Вынести в отдельные capability файлы:
```
src-tauri/capabilities/
├── default.json
├── winws.json      # shell permissions для winws
└── singbox.json    # shell permissions для sing-box
```

---

### 🟠 4. Release workflow создаёт только draft релизы

**Файл:** `.github/workflows/release.yml`

```yaml
- name: Create GitHub Release
  uses: softprops/action-gh-release@v2
  with:
    draft: true  # ← Всегда draft
```

**Проблема:** Требуется ручная публикация каждого релиза.

**Решение:** Добавить input для выбора:
```yaml
workflow_dispatch:
  inputs:
    draft:
      description: 'Create as draft'
      type: boolean
      default: true
```

---

### 🟠 5. Нет кэширования pnpm store в release workflow

**Файл:** `.github/workflows/release.yml`

В `ci.yml` есть кэш pnpm, в `release.yml` — нет.

**Решение:**
```yaml
- name: pnpm cache
  uses: actions/cache@v4
  with:
    path: ~/.pnpm-store
    key: ${{ runner.os }}-pnpm-${{ hashFiles('**/pnpm-lock.yaml') }}
```

---

### 🟠 6. Версия pnpm не синхронизирована между workflows

| Workflow | pnpm version |
|----------|--------------|
| ci.yml | 8 |
| e2e-hyperv.yml | 8 |
| release.yml | **9** |

**Решение:** Использовать одну версию везде или указать в `package.json`:
```json
{
  "packageManager": "pnpm@9.0.0"
}
```

---

### 🟠 7. E2E тесты не запускаются в CI

**Текущее состояние:**
- 11 E2E тестов в `tests/e2e/`
- `playwright.config.ts` настроен
- CI запускает только unit тесты (`pnpm test`)

**Проблема:** E2E тесты требуют запущенное Tauri приложение, что сложно в CI.

**Решение:** Добавить отдельный job для E2E на self-hosted runner:
```yaml
e2e:
  runs-on: [self-hosted, windows]
  steps:
    - name: Build app
      run: pnpm tauri build --debug
    - name: Run E2E tests
      run: pnpm test:e2e
```

---

## Рекомендации (🟡)

### 🟡 1. Добавить pre-commit hooks

**Текущее состояние:** Нет pre-commit hooks.

**Решение:**
```bash
pnpm add -D husky lint-staged
```

```json
// package.json
{
  "lint-staged": {
    "*.ts": ["eslint --fix"],
    "*.svelte": ["eslint --fix"],
    "*.rs": ["cargo fmt --"]
  }
}
```

---

### 🟡 2. Добавить ESLint

**Текущее состояние:** Только `svelte-check` для типов, нет линтера.

**Решение:**
```bash
pnpm add -D eslint @typescript-eslint/eslint-plugin eslint-plugin-svelte
```

---

### 🟡 3. Оптимизировать время сборки в CI

**Текущее время:** ~10-15 минут (оценка)

**Оптимизации:**
1. Использовать `sccache` для Rust:
```yaml
- name: Setup sccache
  uses: mozilla-actions/sccache-action@v0.0.4
  
- name: Build
  env:
    RUSTC_WRAPPER: sccache
```

2. Параллельные jobs для check и security:
```yaml
jobs:
  check:
    # ...
  security:
    # ... (уже параллельно)
  sbom:
    # ... (уже параллельно)
```

---

### 🟡 4. Добавить dependabot auto-merge для patch updates

**Файл:** `.github/dependabot.yml`

```yaml
# Добавить
- package-ecosystem: "npm"
  # ...
  automerged_updates:
    - match:
        dependency_type: "development"
        update_type: "semver:patch"
```

---

### 🟡 5. Улучшить dev-admin.ps1

**Файл:** `scripts/dev-admin.ps1`

```powershell
# Хардкод путей
$env:CARGO_HOME = "D:\SDKs\Rust\cargo"
$env:RUSTUP_HOME = "D:\SDKs\Rust\rustup"
```

**Проблема:** Пути захардкожены для конкретной машины.

**Решение:**
```powershell
# Использовать стандартные пути или переменные окружения
if (-not $env:CARGO_HOME) {
    $env:CARGO_HOME = "$env:USERPROFILE\.cargo"
}
```

---

### 🟡 6. Добавить .nvmrc в CI

**Файл:** `.nvmrc` существует, но CI использует `node-version: '20'` напрямую.

**Решение:**
```yaml
- name: Setup Node.js
  uses: actions/setup-node@v4
  with:
    node-version-file: '.nvmrc'
```

---

### 🟡 7. Добавить CODEOWNERS

**Решение:** Создать `.github/CODEOWNERS`:
```
# Default owners
* @WhiteBite

# Rust backend
/src-tauri/ @WhiteBite

# Frontend
/src/ @WhiteBite

# CI/CD
/.github/ @WhiteBite
```

---

### 🟡 8. Улучшить структуру тестов

**Текущее состояние:**
- Unit тесты разбросаны: `*.test.ts` рядом с кодом + `__tests__/`
- Нет чёткой конвенции

**Рекомендация:** Выбрать один подход:
```
src/lib/
├── stores/
│   ├── theme.ts
│   └── __tests__/
│       └── theme.test.ts
```

---

## Идеи нового функционала (🟢)

### 🟢 1. Nightly builds

Автоматические ночные сборки для тестирования:
```yaml
name: Nightly Build

on:
  schedule:
    - cron: '0 2 * * *'

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - run: pnpm tauri build
      - uses: actions/upload-artifact@v4
        with:
          name: nightly-${{ github.sha }}
          path: src-tauri/target/release/bundle/
          retention-days: 7
```

---

### 🟢 2. Performance benchmarks в CI

```yaml
- name: Run benchmarks
  run: cargo bench --manifest-path src-tauri/Cargo.toml -- --save-baseline main

- name: Compare benchmarks
  run: cargo bench --manifest-path src-tauri/Cargo.toml -- --baseline main
```

---

### 🟢 3. Автоматическое обновление CHANGELOG

Использовать conventional commits + auto-changelog:
```yaml
- name: Generate changelog
  uses: TriPSs/conventional-changelog-action@v5
  with:
    github-token: ${{ secrets.GITHUB_TOKEN }}
```

---

### 🟢 4. Smoke tests после релиза

```yaml
post-release:
  needs: build
  runs-on: windows-latest
  steps:
    - name: Download installer
      run: |
        Invoke-WebRequest -Uri "${{ needs.build.outputs.installer_url }}" -OutFile installer.exe
    
    - name: Install
      run: Start-Process installer.exe -ArgumentList "/S" -Wait
    
    - name: Verify installation
      run: |
        $app = Get-Process -Name "Isolate" -ErrorAction SilentlyContinue
        if (-not $app) { exit 1 }
```

---

### 🟢 5. Интеграция с Sentry для crash reporting

```rust
// src-tauri/src/main.rs
let _guard = sentry::init(("DSN", sentry::ClientOptions {
    release: sentry::release_name!(),
    ..Default::default()
}));
```

---

### 🟢 6. Docker-based DPI simulation для CI

Вместо Hyper-V можно использовать Docker с iptables для симуляции DPI:
```dockerfile
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y iptables
COPY dpi-simulator.sh /
ENTRYPOINT ["/dpi-simulator.sh"]
```

---

### 🟢 7. Telemetry dashboard

Собирать анонимную статистику использования:
- Какие стратегии работают лучше
- Какие сервисы чаще блокируются
- Crash reports

---

## Сводка

| Категория | Количество |
|-----------|------------|
| 🔴 Критичные | 4 |
| 🟠 Важные | 7 |
| 🟡 Рекомендации | 8 |
| 🟢 Идеи | 7 |

### Приоритеты исправления

1. **Немедленно:** Настроить updater, исправить security audit
2. **Эта неделя:** Добавить проверку хэшей бинарников, coverage в CI
3. **Этот месяц:** Pre-commit hooks, ESLint, оптимизация CI
4. **Бэклог:** Nightly builds, benchmarks, Sentry

---

*Аудит выполнен на основе анализа:*
- `.github/workflows/*.yml`
- `package.json`, `src-tauri/Cargo.toml`
- `vitest.config.ts`, `playwright.config.ts`
- `src-tauri/tauri.conf.json`, `src-tauri/capabilities/`
- `scripts/*.ps1`
- `.kiro/settings/mcp.json`
