# E2E Testing Architecture — Isolate

## Обзор

Isolate использует **двухуровневую стратегию E2E тестирования**:

1. **Browser-mode тесты** — Playwright тестирует WebView через dev server (текущая реализация)
2. **Native-mode тесты** — tauri-driver для полноценного тестирования Tauri приложения

## Текущее состояние

### Структура тестов

```
tests/
└── e2e/
    ├── app.spec.ts              # Базовые тесты запуска
    ├── navigation-flow.spec.ts  # Навигация между страницами
    ├── strategies.spec.ts       # Страница стратегий
    ├── plugins.spec.ts          # Marketplace плагинов
    ├── proxies.spec.ts          # Управление прокси
    ├── services.spec.ts         # Страница сервисов
    ├── settings.spec.ts         # Настройки приложения
    ├── routing.spec.ts          # Правила маршрутизации
    ├── keyboard-shortcuts.spec.ts # Горячие клавиши
    ├── command-palette.spec.ts  # Command Palette (Ctrl+K)
    └── ui-elements.spec.ts      # UI компоненты
```

### Текущая конфигурация Playwright

```typescript
// playwright.config.ts
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,      // Tauri тесты последовательно
  workers: 1,                 // Один worker для Tauri
  timeout: 60000,
  
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },

  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
```

---

## Tauri-Driver Setup (Native E2E)

### Зачем нужен tauri-driver?

Browser-mode тесты **не могут** проверить:
- Tauri IPC команды (invoke)
- Системный трей
- Нативные диалоги
- WinDivert/winws интеграцию
- Реальное поведение AppState

### Установка tauri-driver

```bash
# Установка tauri-driver (WebDriver сервер для Tauri)
cargo install tauri-driver --locked

# Для Windows: установка msedgedriver
cargo install --git https://github.com/chippers/msedgedriver-tool
msedgedriver-tool.exe
```

### Архитектура Native E2E

```
┌─────────────────┐     WebDriver Protocol     ┌──────────────────┐
│   Playwright    │ ◄─────────────────────────► │  tauri-driver    │
│   (Test Runner) │        :4444                │  (WebDriver)     │
└─────────────────┘                             └────────┬─────────┘
                                                         │
                                                         ▼
                                                ┌──────────────────┐
                                                │  Isolate.exe     │
                                                │  (Tauri App)     │
                                                └──────────────────┘
```

---

## Конфигурация для Native E2E

### Новый конфиг: `playwright.native.config.ts`

```typescript
import { defineConfig } from '@playwright/test';
import path from 'path';
import os from 'os';

const isCI = !!process.env.CI;
const appPath = path.resolve(
  __dirname,
  'src-tauri/target/debug/isolate.exe'
);

export default defineConfig({
  testDir: './tests/e2e-native',
  fullyParallel: false,
  workers: 1,
  timeout: 120000,  // Больше времени для нативных тестов
  retries: isCI ? 2 : 0,
  
  reporter: [
    ['html', { outputFolder: 'playwright-report-native' }],
    ['json', { outputFile: 'test-results/native-results.json' }],
    ['list'],
  ],

  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    actionTimeout: 15000,
  },

  // Глобальный setup/teardown для tauri-driver
  globalSetup: './tests/e2e-native/global-setup.ts',
  globalTeardown: './tests/e2e-native/global-teardown.ts',

  projects: [
    {
      name: 'tauri-native',
      use: {
        // WebDriver endpoint (tauri-driver)
        connectOptions: {
          wsEndpoint: 'ws://127.0.0.1:4444',
        },
      },
    },
  ],
});
```

### Global Setup: `tests/e2e-native/global-setup.ts`

```typescript
import { spawn, spawnSync, ChildProcess } from 'child_process';
import path from 'path';
import os from 'os';

let tauriDriver: ChildProcess | null = null;

async function globalSetup() {
  console.log('🔧 Building Tauri app...');
  
  // Сборка приложения в debug режиме
  const buildResult = spawnSync('pnpm', ['tauri', 'build', '--debug', '--no-bundle'], {
    cwd: process.cwd(),
    stdio: 'inherit',
    shell: true,
  });

  if (buildResult.status !== 0) {
    throw new Error('Failed to build Tauri app');
  }

  console.log('🚀 Starting tauri-driver...');
  
  // Запуск tauri-driver
  const driverPath = path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver');
  
  tauriDriver = spawn(driverPath, [], {
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  tauriDriver.stdout?.on('data', (data) => {
    console.log(`[tauri-driver] ${data}`);
  });

  tauriDriver.stderr?.on('data', (data) => {
    console.error(`[tauri-driver] ${data}`);
  });

  // Ждём запуска драйвера
  await new Promise((resolve) => setTimeout(resolve, 2000));

  // Сохраняем PID для teardown
  process.env.TAURI_DRIVER_PID = String(tauriDriver.pid);
  
  console.log('✅ tauri-driver started on :4444');
}

export default globalSetup;
```

### Global Teardown: `tests/e2e-native/global-teardown.ts`

```typescript
async function globalTeardown() {
  const pid = process.env.TAURI_DRIVER_PID;
  
  if (pid) {
    console.log('🛑 Stopping tauri-driver...');
    try {
      process.kill(Number(pid));
    } catch (e) {
      // Процесс уже завершён
    }
  }
}

export default globalTeardown;
```

---

## Test Scenarios

### 1. Onboarding Flow

```typescript
// tests/e2e-native/onboarding.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Onboarding Flow', () => {
  test('first launch shows welcome screen', async ({ page }) => {
    // Очистка настроек перед тестом
    // await clearAppData();
    
    await page.goto('/');
    
    // Проверка welcome экрана
    await expect(page.locator('text=Добро пожаловать')).toBeVisible();
  });

  test('can complete initial setup', async ({ page }) => {
    await page.goto('/');
    
    // Шаг 1: Выбор языка
    await page.click('button:has-text("Русский")');
    await page.click('button:has-text("Далее")');
    
    // Шаг 2: Проверка системы
    await expect(page.locator('text=Проверка системы')).toBeVisible();
    await page.waitForSelector('text=WinDivert', { timeout: 10000 });
    
    // Шаг 3: Завершение
    await page.click('button:has-text("Начать")');
    
    // Должны попасть на Dashboard
    await expect(page).toHaveURL('/');
    await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();
  });
});
```

### 2. Proxy Management

```typescript
// tests/e2e-native/proxy-management.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Proxy Management', () => {
  test('can add VLESS proxy via paste', async ({ page }) => {
    await page.goto('/proxies');
    
    // Открыть модал добавления
    await page.click('button:has-text("Add")');
    await expect(page.locator('text=Add Proxy')).toBeVisible();
    
    // Вставить VLESS ссылку
    const vlessUrl = 'vless://uuid@server.com:443?security=tls&sni=server.com#TestProxy';
    await page.fill('textarea', vlessUrl);
    await page.click('button:has-text("Import")');
    
    // Проверить что прокси добавлен
    await expect(page.locator('text=TestProxy')).toBeVisible();
  });

  test('can test proxy connection', async ({ page }) => {
    await page.goto('/proxies');
    
    // Найти прокси и нажать тест
    const proxyCard = page.locator('.transform.transition-all').first();
    await proxyCard.hover();
    await proxyCard.locator('button[title="Test"]').click();
    
    // Ждём результат теста
    await expect(proxyCard.locator('.text-green-500, .text-red-500')).toBeVisible({
      timeout: 30000
    });
  });

  test('can delete proxy', async ({ page }) => {
    await page.goto('/proxies');
    
    const proxyCard = page.locator('.transform.transition-all').first();
    const proxyName = await proxyCard.locator('h3').textContent();
    
    // Удалить прокси
    await proxyCard.hover();
    await proxyCard.locator('button[title="Delete"]').click();
    
    // Подтвердить удаление
    await page.click('button:has-text("Удалить")');
    
    // Проверить что прокси удалён
    await expect(page.locator(`text=${proxyName}`)).not.toBeVisible();
  });
});
```

### 3. Strategy Testing

```typescript
// tests/e2e-native/strategy-testing.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Strategy Testing', () => {
  // ВАЖНО: Эти тесты требуют реальной сети и DPI
  // Запускать только в специальном окружении (Hyper-V)
  
  test.skip('can apply Zapret strategy', async ({ page }) => {
    await page.goto('/strategies');
    
    // Найти YouTube стратегию
    const strategyCard = page.locator('.bg-\\[\\#1a1f3a\\]').filter({
      hasText: 'YouTube'
    }).first();
    
    // Применить стратегию
    await strategyCard.locator('button:has-text("Применить")').click();
    
    // Проверить статус в toolbar
    await expect(page.locator('.h-14 text=Protected')).toBeVisible({
      timeout: 10000
    });
  });

  test('can run Turbo optimization', async ({ page }) => {
    await page.goto('/');
    
    // Нажать Turbo
    await page.click('button:has-text("Turbo")');
    
    // Должен появиться прогресс
    await expect(page.locator('text=Оптимизация')).toBeVisible();
    
    // Ждём завершения (до 2 минут)
    await expect(page.locator('text=Завершено')).toBeVisible({
      timeout: 120000
    });
  });

  test('strategy details modal shows correct info', async ({ page }) => {
    await page.goto('/strategies');
    
    const strategyCard = page.locator('.bg-\\[\\#1a1f3a\\].rounded-xl.p-5').first();
    await strategyCard.locator('button[title="Детали"]').click();
    
    // Проверить содержимое модала
    const modal = page.locator('.fixed.inset-0');
    await expect(modal.locator('text=Описание')).toBeVisible();
    await expect(modal.locator('text=Автор')).toBeVisible();
    await expect(modal.locator('text=Сервисы')).toBeVisible();
    
    // Закрыть модал
    await page.keyboard.press('Escape');
    await expect(modal).not.toBeVisible();
  });
});
```

### 4. Tauri IPC Testing

```typescript
// tests/e2e-native/tauri-ipc.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Tauri IPC Commands', () => {
  test('backend is ready after startup', async ({ page }) => {
    await page.goto('/');
    
    // Выполнить IPC команду через evaluate
    const isReady = await page.evaluate(async () => {
      // @ts-ignore - Tauri API доступен глобально
      const { invoke } = window.__TAURI__.core;
      return await invoke('is_backend_ready');
    });
    
    expect(isReady).toBe(true);
  });

  test('can fetch services via IPC', async ({ page }) => {
    await page.goto('/');
    
    // Ждём готовности бэкенда
    await page.waitForFunction(async () => {
      const { invoke } = (window as any).__TAURI__.core;
      return await invoke('is_backend_ready');
    }, { timeout: 10000 });
    
    // Получить сервисы
    const services = await page.evaluate(async () => {
      const { invoke } = (window as any).__TAURI__.core;
      return await invoke('get_services');
    });
    
    expect(Array.isArray(services)).toBe(true);
    expect(services.length).toBeGreaterThan(0);
  });

  test('can fetch strategies via IPC', async ({ page }) => {
    await page.goto('/');
    
    await page.waitForFunction(async () => {
      const { invoke } = (window as any).__TAURI__.core;
      return await invoke('is_backend_ready');
    }, { timeout: 10000 });
    
    const strategies = await page.evaluate(async () => {
      const { invoke } = (window as any).__TAURI__.core;
      return await invoke('get_strategies');
    });
    
    expect(Array.isArray(strategies)).toBe(true);
  });
});
```

---

## CI Workflow

### GitHub Actions: `.github/workflows/e2e.yml`

```yaml
name: E2E Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  # Browser-mode тесты (быстрые, на каждый PR)
  e2e-browser:
    name: E2E Browser Tests
    runs-on: windows-latest
    timeout-minutes: 20
    
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
          
      - name: Install dependencies
        run: pnpm install --frozen-lockfile
        
      - name: Install Playwright browsers
        run: pnpm exec playwright install chromium
        
      - name: Run E2E tests (browser mode)
        run: pnpm test:e2e
        
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report-browser
          path: playwright-report/
          retention-days: 7
          
      - name: Upload screenshots
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-screenshots
          path: test-results/
          retention-days: 7

  # Native-mode тесты (полные, на main)
  e2e-native:
    name: E2E Native Tests
    runs-on: windows-latest
    timeout-minutes: 45
    if: github.ref == 'refs/heads/main' || github.event_name == 'workflow_dispatch'
    
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

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
          
      - name: Install dependencies
        run: pnpm install --frozen-lockfile
        
      - name: Install Playwright browsers
        run: pnpm exec playwright install chromium
        
      - name: Install tauri-driver
        run: cargo install tauri-driver --locked
        
      - name: Install msedgedriver
        run: |
          cargo install --git https://github.com/chippers/msedgedriver-tool
          & "$HOME/.cargo/bin/msedgedriver-tool.exe"
          $PWD.Path >> $env:GITHUB_PATH
          
      - name: Build Tauri app
        run: pnpm tauri build --debug --no-bundle
        
      - name: Run E2E tests (native mode)
        run: pnpm test:e2e:native
        env:
          TAURI_E2E: true
          
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report-native
          path: playwright-report-native/
          retention-days: 14
          
      - name: Upload videos
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-videos
          path: test-results/
          retention-days: 14
```

---

## Fixtures и Helpers

### Test Fixtures: `tests/e2e/fixtures.ts`

```typescript
import { test as base, expect } from '@playwright/test';

// Расширенные fixtures для Isolate
export const test = base.extend<{
  isolatePage: IsolatePage;
  mockBackend: MockBackend;
}>({
  // Fixture для работы с Isolate UI
  isolatePage: async ({ page }, use) => {
    const isolatePage = new IsolatePage(page);
    await isolatePage.waitForReady();
    await use(isolatePage);
  },
  
  // Fixture для мока бэкенда (browser mode)
  mockBackend: async ({ page }, use) => {
    const mock = new MockBackend(page);
    await mock.setup();
    await use(mock);
    await mock.teardown();
  },
});

export { expect };

// Page Object для Isolate
class IsolatePage {
  constructor(private page: Page) {}
  
  async waitForReady() {
    await this.page.waitForSelector('aside', { timeout: 10000 });
  }
  
  async navigateTo(route: 'dashboard' | 'services' | 'proxies' | 'strategies' | 'settings') {
    const routes = {
      dashboard: '/',
      services: '/services',
      proxies: '/proxies',
      strategies: '/strategies',
      settings: '/settings',
    };
    await this.page.goto(routes[route]);
    await this.waitForReady();
  }
  
  async openCommandPalette() {
    await this.page.keyboard.press('Control+k');
    await expect(this.page.locator('[role="dialog"][aria-label="Command Palette"]')).toBeVisible();
  }
  
  async executeCommand(command: string) {
    await this.openCommandPalette();
    await this.page.fill('input[placeholder*="command"]', command);
    await this.page.keyboard.press('Enter');
  }
  
  async getBackendStatus(): Promise<boolean> {
    if (process.env.TAURI_E2E) {
      return await this.page.evaluate(async () => {
        const { invoke } = (window as any).__TAURI__.core;
        return await invoke('is_backend_ready');
      });
    }
    return true; // В browser mode всегда true
  }
}

// Mock для browser mode тестов
class MockBackend {
  constructor(private page: Page) {}
  
  async setup() {
    // Мокаем Tauri API для browser mode
    await this.page.addInitScript(() => {
      (window as any).__TAURI__ = {
        core: {
          invoke: async (cmd: string, args?: any) => {
            const mocks: Record<string, any> = {
              'is_backend_ready': true,
              'get_services': [
                { id: 'youtube', name: 'YouTube', status: 'unknown' },
                { id: 'discord', name: 'Discord', status: 'unknown' },
              ],
              'get_strategies': [
                { id: 'zapret-youtube', name: 'YouTube Zapret', family: 'zapret' },
              ],
            };
            return mocks[cmd] ?? null;
          },
        },
      };
    });
  }
  
  async teardown() {
    // Cleanup если нужен
  }
}
```


---

## Screenshot Testing

### Visual Regression с Playwright

```typescript
// tests/e2e/visual.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Visual Regression', () => {
  test('dashboard matches snapshot', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('aside', { timeout: 10000 });
    
    // Ждём загрузки всех данных
    await page.waitForTimeout(1000);
    
    await expect(page).toHaveScreenshot('dashboard.png', {
      maxDiffPixels: 100,
      threshold: 0.2,
    });
  });

  test('strategies page matches snapshot', async ({ page }) => {
    await page.goto('/strategies');
    await page.waitForSelector('.grid', { timeout: 10000 });
    
    await expect(page).toHaveScreenshot('strategies.png', {
      maxDiffPixels: 100,
    });
  });

  test('settings page matches snapshot', async ({ page }) => {
    await page.goto('/settings');
    await page.waitForSelector('h1:has-text("Settings")', { timeout: 10000 });
    
    await expect(page).toHaveScreenshot('settings.png');
  });

  test('command palette matches snapshot', async ({ page }) => {
    await page.goto('/');
    await page.keyboard.press('Control+k');
    
    const palette = page.locator('[role="dialog"][aria-label="Command Palette"]');
    await expect(palette).toBeVisible();
    
    await expect(palette).toHaveScreenshot('command-palette.png');
  });
});
```

### Конфигурация скриншотов

```typescript
// playwright.config.ts (дополнение)
export default defineConfig({
  // ...existing config...
  
  expect: {
    toHaveScreenshot: {
      // Директория для baseline скриншотов
      snapshotDir: './tests/e2e/__snapshots__',
      
      // Настройки сравнения
      maxDiffPixels: 50,
      maxDiffPixelRatio: 0.01,
      threshold: 0.2,
      
      // Анимации могут вызывать flaky тесты
      animations: 'disabled',
    },
  },
  
  use: {
    // Фиксированный viewport для консистентных скриншотов
    viewport: { width: 1100, height: 800 },
    
    // Отключить анимации для стабильных скриншотов
    reducedMotion: 'reduce',
  },
});
```


### Обновление baseline скриншотов

```bash
# Обновить все скриншоты
pnpm test:e2e --update-snapshots

# Обновить конкретный тест
pnpm test:e2e visual.spec.ts --update-snapshots

# Интерактивный режим для review
pnpm test:e2e:ui
```

---

## Package.json Scripts

```json
{
  "scripts": {
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:headed": "playwright test --headed",
    "test:e2e:debug": "playwright test --debug",
    "test:e2e:report": "playwright show-report",
    "test:e2e:native": "playwright test --config=playwright.native.config.ts",
    "test:e2e:visual": "playwright test visual.spec.ts",
    "test:e2e:update-snapshots": "playwright test --update-snapshots"
  }
}
```

---

## Best Practices

### 1. Стабильность тестов

```typescript
// ❌ Плохо: жёсткие таймауты
await page.waitForTimeout(5000);

// ✅ Хорошо: ожидание конкретного элемента
await page.waitForSelector('text=Загружено', { timeout: 10000 });

// ✅ Хорошо: ожидание состояния
await expect(page.locator('.loading')).not.toBeVisible();
```

### 2. Изоляция тестов

```typescript
// ❌ Плохо: тесты зависят друг от друга
test('add proxy', async ({ page }) => { /* ... */ });
test('delete proxy', async ({ page }) => { /* зависит от add proxy */ });

// ✅ Хорошо: каждый тест независим
test('can delete proxy', async ({ page }) => {
  // Setup: создать прокси для удаления
  await addTestProxy(page, 'test-proxy');
  
  // Test: удалить прокси
  await deleteProxy(page, 'test-proxy');
  
  // Assert
  await expect(page.locator('text=test-proxy')).not.toBeVisible();
});
```

### 3. Селекторы

```typescript
// ❌ Плохо: хрупкие селекторы
await page.click('.bg-\\[\\#1a1f3a\\].rounded-xl.p-5 > div > button:nth-child(2)');

// ✅ Хорошо: семантические селекторы
await page.click('button[title="Детали"]');
await page.click('button:has-text("Применить")');
await page.getByRole('button', { name: 'Save' }).click();

// ✅ Хорошо: data-testid для сложных случаев
await page.click('[data-testid="strategy-apply-btn"]');
```


### 4. Обработка асинхронности Tauri

```typescript
// ❌ Плохо: не учитывает race condition AppState
await page.goto('/services');
const services = await page.locator('.service-card').count();

// ✅ Хорошо: ждём готовности бэкенда
await page.goto('/services');
await page.waitForFunction(async () => {
  // Проверяем что данные загружены
  const cards = document.querySelectorAll('.service-card');
  return cards.length > 0;
}, { timeout: 10000 });
```

### 5. Группировка тестов

```typescript
// Группировка по функциональности
test.describe('Proxy Management', () => {
  test.describe('Add Proxy', () => {
    test('via paste link', async ({ page }) => { /* ... */ });
    test('via manual form', async ({ page }) => { /* ... */ });
    test('via file import', async ({ page }) => { /* ... */ });
  });
  
  test.describe('Edit Proxy', () => {
    test('can change name', async ({ page }) => { /* ... */ });
    test('can change server', async ({ page }) => { /* ... */ });
  });
});
```

---

## Интеграция с Hyper-V DPI Simulation

Для полноценного тестирования обхода DPI используется отдельный workflow с Hyper-V VM.

См. `.github/workflows/e2e-hyperv.yml` для деталей.

### Ключевые особенности:

1. **Self-hosted runner** с Hyper-V
2. **DPI VM** симулирует блокировки (drop/rst режимы)
3. **Маршрутизация** через VM для реального тестирования
4. **Артефакты**: логи DPI, скриншоты, видео

---

## Troubleshooting

### Тест падает с "Element not found"

```typescript
// Увеличить timeout
await expect(page.locator('text=Loading')).toBeVisible({ timeout: 30000 });

// Проверить что элемент в DOM
const element = await page.locator('text=Loading').elementHandle();
console.log('Element exists:', !!element);
```

### Flaky тесты

```typescript
// Добавить retry на уровне теста
test('flaky test', async ({ page }) => {
  test.info().annotations.push({ type: 'flaky', description: 'Network dependent' });
  // ...
});

// Или в конфиге
export default defineConfig({
  retries: process.env.CI ? 2 : 0,
});
```

### tauri-driver не запускается

```bash
# Проверить что tauri-driver установлен
tauri-driver --version

# Проверить что порт 4444 свободен
netstat -an | findstr 4444

# Запустить вручную для отладки
tauri-driver --port 4444
```

### WebView2 ошибки

```bash
# Обновить WebView2 Runtime
winget install Microsoft.EdgeWebView2Runtime

# Очистить кэш WebView2
rmdir /s /q "%LOCALAPPDATA%\Microsoft\Edge\User Data"
```

---

## Roadmap

- [ ] Добавить `data-testid` атрибуты в UI компоненты
- [ ] Настроить visual regression для всех страниц
- [ ] Интегрировать native E2E в CI
- [ ] Добавить тесты для system tray
- [ ] Добавить тесты для нативных диалогов
- [ ] Performance тесты (startup time, memory usage)
