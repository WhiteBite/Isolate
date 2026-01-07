# E2E Testing Guide

Isolate использует два подхода к E2E тестированию:

1. **Playwright** — тестирование веб-части через dev server
2. **WebdriverIO + tauri-driver** — тестирование реального Tauri приложения

## Быстрый старт

### Playwright (веб-тесты)

```bash
# Запуск всех E2E тестов
pnpm test:e2e

# С UI интерфейсом
pnpm test:e2e:ui

# В headed режиме (видно браузер)
pnpm test:e2e:headed

# Отладка
pnpm test:e2e:debug

# Просмотр отчёта
pnpm test:e2e:report
```

### WebdriverIO + tauri-driver (Tauri тесты)

```bash
# 1. Установить tauri-driver (один раз)
cargo install tauri-driver

# 2. Собрать приложение
pnpm tauri build

# 3. Запустить тесты
pnpm test:e2e:tauri
```

## Структура тестов

```
tests/
├── e2e/                    # Playwright тесты (веб)
│   ├── app.spec.ts         # Базовые тесты приложения
│   ├── navigation-flow.spec.ts
│   ├── strategies.spec.ts
│   └── ...
│
└── e2e-tauri/              # WebdriverIO тесты (Tauri)
    └── smoke.spec.ts       # Smoke тесты реального приложения
```

## Когда использовать какой подход

### Playwright (рекомендуется для большинства случаев)

✅ **Используй для:**
- Тестирования UI компонентов
- Навигации и роутинга
- Визуальных регрессий
- Быстрой итерации (не нужен build)

❌ **Не подходит для:**
- Тестирования Tauri IPC команд
- Нативных функций (файловая система, системный трей)
- Тестирования production сборки

### WebdriverIO + tauri-driver

✅ **Используй для:**
- Smoke тестов production сборки
- Тестирования Tauri-специфичных функций
- Интеграционных тестов с бэкендом
- CI/CD pipeline (финальная проверка)

❌ **Не подходит для:**
- Быстрой разработки (требует build)
- Детального тестирования UI

## Конфигурация

### Playwright (`playwright.config.ts`)

```typescript
export default defineConfig({
  testDir: './tests/e2e',
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:1420',
  },
  // ...
});
```

### WebdriverIO (`wdio.conf.ts`)

```typescript
export const config: Options.Testrunner = {
  specs: ['./tests/e2e-tauri/**/*.spec.ts'],
  capabilities: [{
    'tauri:options': {
      application: 'src-tauri/target/release/Isolate.exe',
    },
  }],
  services: [['tauri', {}]],
  // ...
};
```

## Написание тестов

### Playwright пример

```typescript
import { test, expect } from '@playwright/test';

test('navigates to strategies page', async ({ page }) => {
  await page.goto('/');
  await page.click('a:has-text("Strategies")');
  await expect(page).toHaveURL('/strategies');
});
```

### WebdriverIO пример

```typescript
describe('Smoke Tests', () => {
  it('should launch the application', async () => {
    const title = await browser.getTitle();
    expect(title).toBe('Isolate');
  });

  it('should display sidebar', async () => {
    const sidebar = await $('aside');
    await expect(sidebar).toBeDisplayed();
  });
});
```

## Отладка

### Playwright

```bash
# Запуск с отладчиком
pnpm test:e2e:debug

# Запуск конкретного теста
pnpm test:e2e -- tests/e2e/app.spec.ts

# Генерация тестов через UI
pnpm exec playwright codegen http://localhost:1420
```

### WebdriverIO

```bash
# Подробный вывод
pnpm test:e2e:tauri -- --logLevel=debug

# Запуск конкретного файла
pnpm test:e2e:tauri -- --spec tests/e2e-tauri/smoke.spec.ts
```

## CI/CD интеграция

### GitHub Actions пример

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e-playwright:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - run: pnpm install
      - run: pnpm exec playwright install --with-deps
      - run: pnpm test:e2e

  e2e-tauri:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install tauri-driver
      - run: pnpm install
      - run: pnpm tauri build
      - run: pnpm test:e2e:tauri
```

## Troubleshooting

### "Tauri application not found"

Убедитесь что приложение собрано:
```bash
pnpm tauri build
```

### "tauri-driver not found"

Установите tauri-driver:
```bash
cargo install tauri-driver
```

### Тесты зависают

1. Проверьте что нет других запущенных экземпляров приложения
2. Увеличьте таймауты в конфигурации
3. Проверьте логи в `./wdio-report/`

### WebView2 ошибки

На Windows убедитесь что установлен WebView2 Runtime:
- Скачайте с [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Best Practices

1. **Изолируйте тесты** — каждый тест должен быть независимым
2. **Используйте data-testid** — для стабильных селекторов
3. **Избегайте sleep** — используйте явные ожидания
4. **Группируйте по функциональности** — один файл = одна фича
5. **Запускайте в CI** — автоматизируйте проверки

## Полезные ссылки

- [Playwright Documentation](https://playwright.dev/)
- [WebdriverIO Documentation](https://webdriver.io/)
- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/webdriver/)
- [tauri-driver](https://crates.io/crates/tauri-driver)
