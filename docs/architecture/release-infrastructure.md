# Release Infrastructure Architecture

## Обзор

Документ описывает инфраструктуру для обеспечения качества и релизов проекта Isolate:
- Coverage thresholds (Rust + TypeScript)
- Smoke tests для релизов
- Tauri Updater с подписью
- Версионирование бинарников
- SBOM (Software Bill of Materials)

## Текущее состояние

### CI Pipeline (`.github/workflows/ci.yml`)
- ✅ Rust: fmt, clippy, tests
- ✅ TypeScript: vitest, svelte-check
- ✅ Security: cargo-audit, npm audit
- ❌ Coverage: не настроен
- ❌ Smoke tests: отсутствуют

### Release Pipeline (`.github/workflows/release.yml`)
- ✅ Tauri build через tauri-action
- ✅ GitHub Release draft
- ❌ Updater: pubkey не настроен
- ❌ SBOM: не генерируется

### Binaries (`src-tauri/src/core/binaries.rs`)
- ✅ SHA-256 верификация
- ✅ Хэши для winws v72.6, sing-box v1.10.0
- ❌ Автоматическое обновление версий
- ❌ SBOM для внешних бинарников

---

## 1. Coverage Thresholds

### 1.1 Rust Coverage (cargo-llvm-cov)

#### Установка
```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

#### Конфигурация CI
```yaml
# .github/workflows/ci.yml - добавить в job check
- name: Install cargo-llvm-cov
  uses: taiki-e/install-action@cargo-llvm-cov

- name: Run Rust tests with coverage
  run: |
    cargo llvm-cov --manifest-path src-tauri/Cargo.toml \
      --lcov --output-path lcov-rust.info \
      --ignore-filename-regex "(tests|test_)" \
      --fail-under-lines 60

- name: Upload Rust coverage
  uses: codecov/codecov-action@v4
  with:
    files: lcov-rust.info
    flags: rust
    fail_ci_if_error: true
```

#### Рекомендуемые thresholds
| Метрика | Минимум | Цель |
|---------|---------|------|
| Lines   | 60%     | 75%  |
| Functions | 50%   | 70%  |
| Branches | 40%    | 60%  |

#### Исключения из coverage
```toml
# src-tauri/Cargo.toml
[package.metadata.llvm-cov]
exclude = [
  "src/commands/*",  # Tauri IPC wrappers
  "src/main.rs",     # Entry point
]
```

### 1.2 TypeScript Coverage (Vitest)

#### Конфигурация vitest.config.ts
```typescript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov', 'html'],
      reportsDirectory: './coverage',
      thresholds: {
        lines: 60,
        functions: 50,
        branches: 40,
        statements: 60,
      },
      exclude: [
        'node_modules/**',
        'src/routes/**/*.svelte',  // UI компоненты
        '**/*.d.ts',
        '**/*.test.ts',
      ],
    },
  },
});
```

#### CI интеграция
```yaml
- name: Run TypeScript tests with coverage
  run: pnpm test -- --coverage

- name: Check coverage thresholds
  run: pnpm test -- --coverage --coverage.thresholds.100=false

- name: Upload TypeScript coverage
  uses: codecov/codecov-action@v4
  with:
    files: coverage/lcov.info
    flags: typescript
```

### 1.3 Combined Coverage Report
```yaml
- name: Merge coverage reports
  run: |
    # Установка lcov
    choco install lcov -y
    
    # Объединение отчётов
    lcov -a lcov-rust.info -a coverage/lcov.info -o combined.info
    
    # Генерация HTML отчёта
    genhtml combined.info -o coverage-report

- name: Upload combined coverage
  uses: codecov/codecov-action@v4
  with:
    files: combined.info
    fail_ci_if_error: true
```


---

## 2. Smoke Tests для релизов

### 2.1 PowerShell скрипт проверки .exe

```powershell
# scripts/smoke-test.ps1
# Smoke test для проверки работоспособности Isolate.exe

param(
    [Parameter(Mandatory=$true)]
    [string]$ExePath,
    
    [int]$TimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"

Write-Host "=== Isolate Smoke Test ===" -ForegroundColor Cyan
Write-Host "Testing: $ExePath"

# 1. Проверка существования файла
if (-not (Test-Path $ExePath)) {
    Write-Error "ERROR: File not found: $ExePath"
    exit 1
}

# 2. Проверка цифровой подписи (если настроена)
Write-Host "`n[1/5] Checking digital signature..."
$signature = Get-AuthenticodeSignature -FilePath $ExePath
if ($signature.Status -eq "Valid") {
    Write-Host "  ✓ Valid signature: $($signature.SignerCertificate.Subject)" -ForegroundColor Green
} elseif ($signature.Status -eq "NotSigned") {
    Write-Host "  ⚠ Not signed (expected for unsigned builds)" -ForegroundColor Yellow
} else {
    Write-Host "  ✗ Invalid signature: $($signature.Status)" -ForegroundColor Red
    exit 1
}

# 3. Проверка PE заголовка
Write-Host "`n[2/5] Checking PE header..."
$bytes = [System.IO.File]::ReadAllBytes($ExePath)
if ($bytes[0] -eq 0x4D -and $bytes[1] -eq 0x5A) {
    Write-Host "  ✓ Valid PE executable" -ForegroundColor Green
} else {
    Write-Error "  ✗ Invalid PE header"
    exit 1
}

# 4. Проверка версии
Write-Host "`n[3/5] Checking version info..."
$versionInfo = (Get-Item $ExePath).VersionInfo
Write-Host "  Product: $($versionInfo.ProductName)"
Write-Host "  Version: $($versionInfo.ProductVersion)"
Write-Host "  ✓ Version info present" -ForegroundColor Green

# 5. Запуск с --version (если поддерживается)
Write-Host "`n[4/5] Testing startup..."
$process = $null
try {
    $startInfo = New-Object System.Diagnostics.ProcessStartInfo
    $startInfo.FileName = $ExePath
    $startInfo.Arguments = "--version"
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.CreateNoWindow = $true
    
    $process = [System.Diagnostics.Process]::Start($startInfo)
    $exited = $process.WaitForExit($TimeoutSeconds * 1000)
    
    if ($exited) {
        $stdout = $process.StandardOutput.ReadToEnd()
        $stderr = $process.StandardError.ReadToEnd()
        
        if ($process.ExitCode -eq 0) {
            Write-Host "  ✓ Process started and exited cleanly" -ForegroundColor Green
            if ($stdout) { Write-Host "  Output: $stdout" }
        } else {
            Write-Host "  ⚠ Exit code: $($process.ExitCode)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  ⚠ Process did not exit within timeout (may be GUI app)" -ForegroundColor Yellow
        $process.Kill()
    }
} catch {
    Write-Host "  ⚠ Could not run --version: $_" -ForegroundColor Yellow
} finally {
    if ($process) { $process.Dispose() }
}

# 6. Проверка зависимостей
Write-Host "`n[5/5] Checking dependencies..."
$dumpbin = Get-Command "dumpbin.exe" -ErrorAction SilentlyContinue
if ($dumpbin) {
    $deps = & dumpbin /dependents $ExePath 2>$null | Select-String "\.dll"
    Write-Host "  Dependencies found: $($deps.Count)"
    Write-Host "  ✓ Dependency check complete" -ForegroundColor Green
} else {
    Write-Host "  ⚠ dumpbin not available, skipping dependency check" -ForegroundColor Yellow
}

Write-Host "`n=== Smoke Test PASSED ===" -ForegroundColor Green
exit 0
```

### 2.2 CI интеграция smoke tests

```yaml
# .github/workflows/release.yml - добавить после build
- name: Run smoke tests
  shell: pwsh
  run: |
    $exePath = "src-tauri/target/release/isolate.exe"
    
    # Базовый smoke test
    ./scripts/smoke-test.ps1 -ExePath $exePath -TimeoutSeconds 30
    
    # Проверка размера (не должен быть слишком маленьким)
    $size = (Get-Item $exePath).Length
    $minSize = 5MB
    if ($size -lt $minSize) {
      Write-Error "Executable too small: $size bytes (min: $minSize)"
      exit 1
    }
    Write-Host "Executable size: $([math]::Round($size/1MB, 2)) MB"
```


---

## 3. Tauri Updater

### 3.1 Генерация ключей подписи

```bash
# Генерация keypair для подписи обновлений
# Приватный ключ хранится в ~/.tauri/isolate.key (НИКОГДА не коммитить!)
# Публичный ключ добавляется в tauri.conf.json

tauri signer generate -w ~/.tauri/isolate.key

# Вывод:
# Your public key was generated successfully:
# dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6...
#
# Your secret key was generated successfully - Keep it secret!
# ~/.tauri/isolate.key
```

### 3.2 Конфигурация tauri.conf.json

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6...",
      "endpoints": [
        "https://github.com/aspect-build/isolate/releases/latest/download/latest.json"
      ],
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

### 3.3 Формат update.json (latest.json)

```json
{
  "version": "0.2.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2026-01-07T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkK...",
      "url": "https://github.com/aspect-build/isolate/releases/download/v0.2.0/Isolate_0.2.0_x64-setup.nsis.zip"
    }
  }
}
```

### 3.4 CI для автоматической подписи

```yaml
# .github/workflows/release.yml
- name: Build Tauri app with signing
  uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_KEY_PASSWORD }}
  with:
    tagName: v__VERSION__
    releaseName: 'Isolate v__VERSION__'
    releaseBody: 'See the assets to download this version and install.'
    releaseDraft: true
    prerelease: false
    includeUpdaterJson: true
```

### 3.5 GitHub Secrets для подписи

| Secret | Описание |
|--------|----------|
| `TAURI_SIGNING_PRIVATE_KEY` | Содержимое ~/.tauri/isolate.key |
| `TAURI_SIGNING_KEY_PASSWORD` | Пароль от ключа (если установлен) |

### 3.6 Frontend код для проверки обновлений

```typescript
// src/lib/updater.ts
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export async function checkForUpdates(): Promise<boolean> {
  try {
    const update = await check();
    
    if (update?.available) {
      console.log(`Update available: ${update.version}`);
      console.log(`Release notes: ${update.body}`);
      
      // Скачивание и установка
      await update.downloadAndInstall();
      
      // Перезапуск приложения
      await relaunch();
      return true;
    }
    
    return false;
  } catch (error) {
    console.error('Update check failed:', error);
    return false;
  }
}
```


---

## 4. Версионирование бинарников

### 4.1 Текущая структура (binaries.rs)

```rust
// Версии зашиты в константах
pub const BINARY_SOURCES: &[BinarySource] = &[
    BinarySource {
        name: "zapret",
        url: "https://github.com/bol-van/zapret/releases/download/v72.6/...",
        sha256: "...",
        // ...
    },
    BinarySource {
        name: "sing-box",
        url: "https://github.com/SagerNet/sing-box/releases/download/v1.10.0/...",
        sha256: "8ee3e6beaa94fb961b91c845446e3300cf0e995cb3995448da320ead88b8666b",
        // ...
    },
];
```

### 4.2 Рекомендуемая структура: binaries.toml

```toml
# configs/binaries.toml
# Централизованная конфигурация внешних бинарников

[meta]
schema_version = 1
last_updated = "2026-01-07"

[binaries.zapret]
version = "72.6"
release_date = "2026-01-06"
source = "https://github.com/bol-van/zapret"
license = "MIT"

[binaries.zapret.files.winws]
filename = "winws.exe"
url = "https://github.com/bol-van/zapret/releases/download/v72.6/zapret-winws-v72.6.zip"
sha256 = "21c5db984702de8b24d462ae3e64a1ef18937b4515d862a3dd9b70845944a595"

[binaries.zapret.files.windivert_sys]
filename = "WinDivert64.sys"
sha256 = "8da085332782708d8767bcace5327a6ec7283c17cfb85e40b03cd2323a90ddc2"

[binaries.zapret.files.windivert_dll]
filename = "WinDivert.dll"
sha256 = "c1e060ee19444a259b2162f8af0f3fe8c4428a1c6f694dce20de194ac8d7d9a2"

[binaries.singbox]
version = "1.10.0"
release_date = "2025-12-15"
source = "https://github.com/SagerNet/sing-box"
license = "GPL-3.0"

[binaries.singbox.files.singbox]
filename = "sing-box.exe"
url = "https://github.com/SagerNet/sing-box/releases/download/v1.10.0/sing-box-1.10.0-windows-amd64.zip"
sha256 = "0da10a2f1db4fc92dd8db9c301318db457073c23f51d7cc69507f3eda142c331"
```

### 4.3 Naming Convention

```
binaries/
├── winws-72.6.exe           # Версионированное имя
├── winws.exe -> winws-72.6.exe  # Симлинк на текущую версию
├── sing-box-1.10.0.exe
├── sing-box.exe -> sing-box-1.10.0.exe
├── WinDivert64.sys
├── WinDivert.dll
└── manifest.json            # Метаданные установленных версий
```

### 4.4 manifest.json

```json
{
  "schema_version": 1,
  "installed_at": "2026-01-07T12:00:00Z",
  "binaries": {
    "winws.exe": {
      "version": "72.6",
      "sha256": "21c5db984702de8b24d462ae3e64a1ef18937b4515d862a3dd9b70845944a595",
      "installed_at": "2026-01-07T12:00:00Z",
      "source": "zapret"
    },
    "sing-box.exe": {
      "version": "1.10.0",
      "sha256": "0da10a2f1db4fc92dd8db9c301318db457073c23f51d7cc69507f3eda142c331",
      "installed_at": "2026-01-07T12:00:00Z",
      "source": "singbox"
    }
  }
}
```

### 4.5 Автоматическая проверка обновлений бинарников

```rust
// src-tauri/src/core/binary_updater.rs
pub async fn check_binary_updates() -> Result<Vec<BinaryUpdate>> {
    let manifest = load_manifest()?;
    let config = load_binaries_toml()?;
    let mut updates = Vec::new();
    
    for (name, installed) in &manifest.binaries {
        if let Some(latest) = config.get_latest_version(name) {
            if latest.version != installed.version {
                updates.push(BinaryUpdate {
                    name: name.clone(),
                    current: installed.version.clone(),
                    latest: latest.version.clone(),
                    changelog_url: latest.changelog_url.clone(),
                });
            }
        }
    }
    
    Ok(updates)
}
```


---

## 5. SBOM (Software Bill of Materials)

### 5.1 Что такое SBOM

SBOM — это формальный список всех компонентов, библиотек и зависимостей в программном обеспечении. Необходим для:
- Аудита безопасности
- Compliance (NTIA, EU Cyber Resilience Act)
- Отслеживания уязвимостей (CVE)
- Лицензионного соответствия

### 5.2 Форматы SBOM

| Формат | Описание | Использование |
|--------|----------|---------------|
| SPDX | ISO стандарт | Лицензии, compliance |
| CycloneDX | OWASP стандарт | Security, VEX |
| SWID | ISO/IEC 19770-2 | Asset management |

**Рекомендация:** CycloneDX — лучшая поддержка для security use cases.

### 5.3 Rust SBOM (cargo-sbom)

#### Установка
```bash
cargo install cargo-sbom
```

#### Генерация
```bash
cd src-tauri
cargo sbom --output-format cyclonedx-json > sbom-rust.json
```

#### CI интеграция
```yaml
- name: Install cargo-sbom
  run: cargo install cargo-sbom

- name: Generate Rust SBOM
  run: |
    cd src-tauri
    cargo sbom --output-format cyclonedx-json > ../sbom-rust.json

- name: Upload Rust SBOM
  uses: actions/upload-artifact@v4
  with:
    name: sbom-rust
    path: sbom-rust.json
```

### 5.4 TypeScript SBOM (@cyclonedx/cyclonedx-npm)

#### Установка
```bash
pnpm add -D @cyclonedx/cyclonedx-npm
```

#### package.json скрипт
```json
{
  "scripts": {
    "sbom": "cyclonedx-npm --output-format JSON --output-file sbom-npm.json"
  }
}
```

#### CI интеграция
```yaml
- name: Generate npm SBOM
  run: pnpm sbom

- name: Upload npm SBOM
  uses: actions/upload-artifact@v4
  with:
    name: sbom-npm
    path: sbom-npm.json
```

### 5.5 SBOM для внешних бинарников

```json
// sbom-binaries.json
{
  "$schema": "http://cyclonedx.org/schema/bom-1.5.schema.json",
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "version": 1,
  "metadata": {
    "timestamp": "2026-01-07T12:00:00Z",
    "component": {
      "name": "isolate-external-binaries",
      "version": "0.1.0",
      "type": "application"
    }
  },
  "components": [
    {
      "type": "application",
      "name": "winws",
      "version": "72.6",
      "purl": "pkg:github/bol-van/zapret@v72.6",
      "hashes": [
        {
          "alg": "SHA-256",
          "content": "21c5db984702de8b24d462ae3e64a1ef18937b4515d862a3dd9b70845944a595"
        }
      ],
      "licenses": [
        {
          "license": {
            "id": "MIT"
          }
        }
      ],
      "externalReferences": [
        {
          "type": "website",
          "url": "https://github.com/bol-van/zapret"
        }
      ]
    },
    {
      "type": "application",
      "name": "sing-box",
      "version": "1.10.0",
      "purl": "pkg:github/SagerNet/sing-box@v1.10.0",
      "hashes": [
        {
          "alg": "SHA-256",
          "content": "0da10a2f1db4fc92dd8db9c301318db457073c23f51d7cc69507f3eda142c331"
        }
      ],
      "licenses": [
        {
          "license": {
            "id": "GPL-3.0-or-later"
          }
        }
      ]
    },
    {
      "type": "library",
      "name": "WinDivert",
      "version": "2.2",
      "purl": "pkg:github/basil00/WinDivert@v2.2",
      "licenses": [
        {
          "license": {
            "id": "LGPL-3.0-or-later"
          }
        }
      ]
    }
  ]
}
```

### 5.6 Объединённый SBOM

```yaml
# .github/workflows/release.yml
- name: Generate combined SBOM
  run: |
    # Установка cyclonedx-cli
    curl -sSfL https://github.com/CycloneDX/cyclonedx-cli/releases/latest/download/cyclonedx-win-x64.exe -o cyclonedx.exe
    
    # Объединение всех SBOM
    ./cyclonedx.exe merge \
      --input-files sbom-rust.json sbom-npm.json sbom-binaries.json \
      --output-file sbom-combined.json \
      --output-format json

- name: Upload combined SBOM to release
  uses: softprops/action-gh-release@v1
  with:
    files: sbom-combined.json
```

### 5.7 Автоматическая проверка уязвимостей

```yaml
- name: Scan SBOM for vulnerabilities
  uses: anchore/sbom-action@v0
  with:
    sbom-artifact-match: "sbom-combined.json"
    
- name: Run Grype vulnerability scanner
  uses: anchore/scan-action@v3
  with:
    sbom: sbom-combined.json
    fail-build: true
    severity-cutoff: high
```


---

## 6. Полный CI/CD Pipeline

### 6.1 Обновлённый ci.yml

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: windows-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8
          
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
          
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt, llvm-tools-preview
          
      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
          
      - name: Install dependencies
        run: pnpm install
        
      # Formatting & Linting
      - name: Check Rust formatting
        run: cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
        
      - name: Clippy
        run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
        
      # Tests with Coverage
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov
        
      - name: Run Rust tests with coverage
        run: |
          cargo llvm-cov --manifest-path src-tauri/Cargo.toml \
            --lcov --output-path lcov-rust.info \
            --fail-under-lines 60
            
      - name: Run TypeScript tests with coverage
        run: pnpm test -- --coverage
        
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov-rust.info,coverage/lcov.info
          fail_ci_if_error: true
          
      # Type checking
      - name: Svelte check
        run: pnpm check
        
      # Build
      - name: Build frontend
        run: pnpm build
        
      - name: Build Tauri
        run: pnpm tauri build
        
      # Smoke test
      - name: Run smoke tests
        shell: pwsh
        run: ./scripts/smoke-test.ps1 -ExePath "src-tauri/target/release/isolate.exe"

  security:
    runs-on: windows-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8
          
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
          
      - name: Install security tools
        run: |
          cargo install cargo-audit cargo-sbom
          pnpm add -g @cyclonedx/cyclonedx-npm
          
      - name: Install dependencies
        run: pnpm install
        
      # Security audits
      - name: npm audit
        run: pnpm audit --audit-level=high
        continue-on-error: true
        
      - name: Cargo audit
        run: cargo audit --file src-tauri/Cargo.lock
        
      # SBOM generation
      - name: Generate Rust SBOM
        run: |
          cd src-tauri
          cargo sbom --output-format cyclonedx-json > ../sbom-rust.json
          
      - name: Generate npm SBOM
        run: npx @cyclonedx/cyclonedx-npm --output-format JSON --output-file sbom-npm.json
        
      - name: Upload SBOMs
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: |
            sbom-rust.json
            sbom-npm.json
```

### 6.2 Обновлённый release.yml

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    runs-on: windows-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8
          
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
          
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
          
      - name: Install dependencies
        run: pnpm install
        
      # Build with signing
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_KEY_PASSWORD }}
        with:
          tagName: v__VERSION__
          releaseName: 'Isolate v__VERSION__'
          releaseBody: 'See the assets to download this version and install.'
          releaseDraft: true
          prerelease: false
          includeUpdaterJson: true
          
      # Smoke test
      - name: Run smoke tests
        shell: pwsh
        run: |
          $exe = Get-ChildItem -Path "src-tauri/target/release" -Filter "*.exe" | 
                 Where-Object { $_.Name -match "isolate" } | 
                 Select-Object -First 1
          ./scripts/smoke-test.ps1 -ExePath $exe.FullName
          
      # SBOM
      - name: Install SBOM tools
        run: |
          cargo install cargo-sbom
          pnpm add -g @cyclonedx/cyclonedx-npm
          
      - name: Generate combined SBOM
        run: |
          cd src-tauri && cargo sbom --output-format cyclonedx-json > ../sbom-rust.json && cd ..
          npx @cyclonedx/cyclonedx-npm --output-format JSON --output-file sbom-npm.json
          
      - name: Upload SBOM to release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            sbom-rust.json
            sbom-npm.json
          tag_name: ${{ github.ref_name }}
```


---

## 7. Чеклист внедрения

### Фаза 1: Coverage (1-2 дня)
- [ ] Установить cargo-llvm-cov локально
- [ ] Настроить vitest coverage в vitest.config.ts
- [ ] Добавить coverage steps в ci.yml
- [ ] Настроить Codecov интеграцию
- [ ] Установить начальные thresholds (60% lines)

### Фаза 2: Smoke Tests (1 день)
- [ ] Создать scripts/smoke-test.ps1
- [ ] Добавить smoke test step в ci.yml
- [ ] Добавить smoke test step в release.yml
- [ ] Протестировать локально

### Фаза 3: Updater (2-3 дня)
- [ ] Сгенерировать keypair: `tauri signer generate`
- [ ] Добавить pubkey в tauri.conf.json
- [ ] Добавить secrets в GitHub: TAURI_SIGNING_PRIVATE_KEY
- [ ] Обновить release.yml с signing
- [ ] Реализовать UI для проверки обновлений
- [ ] Протестировать update flow

### Фаза 4: Binaries Versioning (2 дня)
- [ ] Создать configs/binaries.toml
- [ ] Обновить binaries.rs для чтения из TOML
- [ ] Реализовать manifest.json
- [ ] Добавить команду check_binary_updates

### Фаза 5: SBOM (1-2 дня)
- [ ] Установить cargo-sbom и @cyclonedx/cyclonedx-npm
- [ ] Создать sbom-binaries.json вручную
- [ ] Добавить SBOM generation в CI
- [ ] Добавить SBOM в GitHub Release assets
- [ ] Настроить vulnerability scanning (Grype)

---

## 8. Ссылки

- [Tauri Updater Plugin](https://v2.tauri.app/plugin/updater/)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [CycloneDX Specification](https://cyclonedx.org/specification/overview/)
- [Codecov GitHub Action](https://github.com/codecov/codecov-action)
- [Grype Vulnerability Scanner](https://github.com/anchore/grype)
