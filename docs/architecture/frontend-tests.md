# Frontend Unit Tests Architecture

## Обзор

Isolate использует **Vitest** + **@testing-library/svelte** для unit-тестирования frontend-кода.
Тесты размещаются **рядом с тестируемыми файлами** (co-location pattern).

## Структура тестов

```
src/lib/
├── stores/
│   ├── layout.ts
│   ├── layout.test.ts          # ✅ Тест рядом с файлом
│   ├── logs.ts
│   ├── logs.test.ts
│   ├── plugins.ts
│   ├── plugins.test.ts
│   └── ...
├── utils/
│   ├── countries.ts
│   ├── countries.test.ts
│   └── ...
├── components/
│   ├── ContextMenu.svelte
│   ├── ContextMenu.test.ts
│   └── ...
├── plugins/
│   ├── context.ts
│   ├── loader.ts
│   ├── plugins.test.ts         # Общий тест для модуля
│   └── ...
├── __mocks__/                   # Моки для SvelteKit
│   ├── app-environment.ts
│   ├── app-navigation.ts
│   └── app-stores.ts
└── __tests__/                   # Интеграционные тесты
    ├── api.test.ts
    └── stores-logs.test.ts
```

### Правила размещения

| Тип файла | Расположение теста |
|-----------|-------------------|
| Store (`*.ts`) | `*.test.ts` рядом |
| Utility (`*.ts`) | `*.test.ts` рядом |
| Component (`*.svelte`) | `*.test.ts` рядом |
| API types | `__tests__/api.test.ts` |
| Интеграционные | `__tests__/*.test.ts` |

## Конфигурация Vitest

### vitest.config.ts

```typescript
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig({
    plugins: [
        svelte(),
        svelteTesting()
    ],
    test: {
        include: ['src/**/*.{test,spec}.{js,ts}'],
        globals: true,
        environment: 'happy-dom',
        alias: {
            $lib: '/src/lib',
            '$app/environment': '/src/lib/__mocks__/app-environment.ts',
            '$app/stores': '/src/lib/__mocks__/app-stores.ts',
            '$app/navigation': '/src/lib/__mocks__/app-navigation.ts'
        },
        coverage: {
            provider: 'v8',
            reporter: ['text', 'json', 'html'],
            include: ['src/lib/**/*.ts'],
            exclude: [
                'src/lib/**/*.test.ts',
                'src/lib/**/*.spec.ts',
                'src/lib/__mocks__/**',
                'src/lib/__tests__/**'
            ]
        }
    },
    resolve: {
        alias: {
            $lib: '/src/lib'
        }
    }
});
```

### Ключевые настройки

- **environment: 'happy-dom'** — легковесный DOM для тестов
- **globals: true** — `describe`, `it`, `expect` без импорта
- **alias** — моки для SvelteKit модулей

## Mocking Strategy

### 1. SvelteKit Modules

Моки в `src/lib/__mocks__/`:

**app-environment.ts**
```typescript
export const browser = true;
export const dev = true;
export const building = false;
export const version = 'test';
```

**app-navigation.ts**
```typescript
export const goto = async (url: string) => {};
export const invalidate = async (url: string) => {};
export const invalidateAll = async () => {};
export const preloadData = async (url: string) => ({ type: 'loaded' as const, status: 200, data: {} });
export const preloadCode = async (...urls: string[]) => {};
export const beforeNavigate = (callback: (navigation: any) => void) => {};
export const afterNavigate = (callback: (navigation: any) => void) => {};
export const onNavigate = (callback: (navigation: any) => void) => {};
export const disableScrollHandling = () => {};
export const pushState = (url: string, state: any) => {};
export const replaceState = (url: string, state: any) => {};
```

**app-stores.ts**
```typescript
import { writable, readable } from 'svelte/store';

export const page = readable({
  url: new URL('http://localhost/'),
  params: {},
  route: { id: '/' },
  status: 200,
  error: null,
  data: {},
  form: null,
  state: {}
});

export const navigating = readable(null);
export const updated = {
  subscribe: writable(false).subscribe,
  check: async () => false
};
```

### 2. Tauri API Mocking

```typescript
// В начале тестового файла
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

import { invoke } from '@tauri-apps/api/core';

const mockedInvoke = vi.mocked(invoke);

// В тесте
mockedInvoke.mockResolvedValueOnce(true);  // is_backend_ready
mockedInvoke.mockResolvedValueOnce([]);    // get_services
```

### 3. localStorage Mocking

```typescript
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { store = {}; }),
    get _store() { return store; }  // Для инспекции в тестах
  };
})();

Object.defineProperty(globalThis, 'localStorage', {
  value: localStorageMock,
  writable: true
});
```

### 4. Browser APIs

```typescript
// crypto.randomUUID
vi.stubGlobal('crypto', {
  randomUUID: () => `test-uuid-${Math.random().toString(36).substring(7)}`
});

// matchMedia
const matchMediaMock = vi.fn((query: string) => ({
  matches: query.includes('dark'),
  media: query,
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn()
}));

Object.defineProperty(globalThis, 'matchMedia', { value: matchMediaMock });

// window dimensions
Object.defineProperty(window, 'innerWidth', { value: 1920, writable: true });
Object.defineProperty(window, 'innerHeight', { value: 1080, writable: true });
```

## Примеры тестов

### Store Test (Svelte Store)

```typescript
// src/lib/stores/toast.test.ts
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { toasts } from './toast';

describe('toast store', () => {
  beforeEach(() => {
    toasts.clear();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('creates success toast', () => {
    const id = toasts.success('Operation completed');
    
    const currentToasts = get(toasts);
    expect(currentToasts).toHaveLength(1);
    expect(currentToasts[0]).toMatchObject({
      id,
      type: 'success',
      message: 'Operation completed'
    });
  });

  it('auto-dismisses after timeout', () => {
    toasts.success('Test', 3000);
    expect(get(toasts)).toHaveLength(1);
    
    vi.advanceTimersByTime(3000);
    expect(get(toasts)).toHaveLength(0);
  });
});
```

### Utility Test

```typescript
// src/lib/utils/countries.test.ts
import { describe, it, expect } from 'vitest';
import { getCountryFlag, getCountryName, detectCountryFromServer } from './countries';

describe('getCountryFlag', () => {
  it('returns correct flag for valid country code', () => {
    expect(getCountryFlag('US')).toBe('🇺🇸');
    expect(getCountryFlag('DE')).toBe('🇩🇪');
  });

  it('is case-insensitive', () => {
    expect(getCountryFlag('us')).toBe('🇺🇸');
    expect(getCountryFlag('Us')).toBe('🇺🇸');
  });

  it('returns globe for unknown codes', () => {
    expect(getCountryFlag('XX')).toBe('🌐');
    expect(getCountryFlag(null)).toBe('🌐');
  });
});

describe('detectCountryFromServer', () => {
  it('detects country from TLD', () => {
    expect(detectCountryFromServer('example.ru')).toBe('RU');
    expect(detectCountryFromServer('server.de')).toBe('DE');
  });

  it('returns null for IP addresses', () => {
    expect(detectCountryFromServer('192.168.1.1')).toBeNull();
  });
});
```

### Component Test (Svelte 5)

```typescript
// src/lib/components/ContextMenu.test.ts
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import { tick } from 'svelte';
import ContextMenu from './ContextMenu.svelte';

beforeEach(() => {
  Object.defineProperty(window, 'innerWidth', { value: 1920, writable: true });
  vi.spyOn(window, 'requestAnimationFrame').mockImplementation((cb) => {
    cb(0);
    return 0;
  });
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe('ContextMenu', () => {
  it('should not render when not visible', () => {
    render(ContextMenu);
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('should render when show() is called', async () => {
    const { component } = render(ContextMenu);
    const mockEvent = new MouseEvent('contextmenu', {
      clientX: 100,
      clientY: 200,
      bubbles: true
    });
    
    component.show(mockEvent);
    await tick();
    
    await waitFor(() => {
      expect(screen.getByRole('menu')).toBeTruthy();
    });
  });

  it('should hide on Escape key', async () => {
    const { component } = render(ContextMenu);
    component.show(new MouseEvent('contextmenu', { clientX: 100, clientY: 100 }));
    await tick();
    
    await fireEvent.keyDown(window, { key: 'Escape' });
    await tick();
    
    await waitFor(() => {
      expect(screen.queryByRole('menu')).toBeNull();
    });
  });
});
```

### Store with Tauri API

```typescript
// src/lib/stores/plugins.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

import { invoke } from '@tauri-apps/api/core';
import { installedPlugins, loadPluginsFromBackend } from './plugins';

const mockedInvoke = vi.mocked(invoke);

describe('loadPluginsFromBackend', () => {
  beforeEach(() => {
    installedPlugins.set([]);
    vi.clearAllMocks();
  });

  it('returns empty array when backend not ready', async () => {
    mockedInvoke.mockResolvedValueOnce(false);
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toEqual([]);
    expect(mockedInvoke).toHaveBeenCalledWith('is_backend_ready');
  });

  it('maps backend plugins to frontend format', async () => {
    mockedInvoke
      .mockResolvedValueOnce(true)  // is_backend_ready
      .mockResolvedValueOnce([{     // get_all_plugins_cmd
        manifest: {
          id: 'test-plugin',
          name: 'Test Plugin',
          version: '1.0.0',
          type: 'ui-plugin'
        },
        enabled: true,
        path: '/plugins/test'
      }]);
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe('test-plugin');
  });
});
```

### Type-only Tests (API Types)

```typescript
// src/lib/__tests__/api.test.ts
import { describe, it, expect } from 'vitest';
import type { Strategy, Service, AppSettings } from '../api';

describe('API Types', () => {
  describe('Strategy interface', () => {
    it('should have correct structure', () => {
      const strategy: Strategy = {
        id: 'test-strategy',
        name: 'Test Strategy',
        description: 'A test strategy',
        family: 'zapret',
        engine: 'winws'
      };

      expect(strategy.id).toBe('test-strategy');
      expect(strategy).toHaveProperty('family');
      expect(strategy).toHaveProperty('engine');
    });
  });

  describe('Type compatibility with Backend', () => {
    it('Strategy fields should match Rust struct', () => {
      const requiredFields = ['id', 'name', 'description', 'family', 'engine'];
      const strategy: Strategy = {
        id: 'test',
        name: 'Test',
        description: 'Test',
        family: 'zapret',
        engine: 'winws'
      };

      requiredFields.forEach(field => {
        expect(strategy).toHaveProperty(field);
      });
    });
  });
});
```

## Coverage Configuration

### Запуск с coverage

```bash
# Запуск тестов с coverage
pnpm vitest run --coverage

# Watch mode с coverage
pnpm vitest --coverage
```

### Coverage отчёты

- **text** — в консоль
- **json** — `coverage/coverage-final.json`
- **html** — `coverage/index.html`

### Целевые метрики

| Метрика | Цель | Текущее |
|---------|------|---------|
| Statements | 70% | TBD |
| Branches | 60% | TBD |
| Functions | 70% | TBD |
| Lines | 70% | TBD |

### Исключения из coverage

```typescript
coverage: {
  include: ['src/lib/**/*.ts'],
  exclude: [
    'src/lib/**/*.test.ts',
    'src/lib/**/*.spec.ts',
    'src/lib/__mocks__/**',
    'src/lib/__tests__/**',
    'src/lib/types/**',        // Только типы
    'src/lib/mocks/**'         // Моки для dev
  ]
}
```

## Best Practices

### 1. Изоляция тестов

```typescript
beforeEach(() => {
  // Сброс состояния store
  myStore.set(initialState);
  // Очистка моков
  vi.clearAllMocks();
  // Очистка localStorage
  localStorageMock.clear();
});

afterEach(() => {
  cleanup();  // Для компонентов
  vi.restoreAllMocks();
});
```

### 2. Async/Await с Svelte

```typescript
// Используй tick() для ожидания обновления DOM
import { tick } from 'svelte';

it('updates after state change', async () => {
  const { component } = render(MyComponent);
  component.updateState();
  await tick();
  expect(screen.getByText('Updated')).toBeTruthy();
});

// Используй waitFor для асинхронных операций
await waitFor(() => {
  expect(screen.getByRole('menu')).toBeTruthy();
});
```

### 3. Тестирование Svelte 5 Runes

```typescript
// Компоненты с $state, $derived, $effect тестируются так же
// Важно: используй tick() после изменений

it('derived value updates', async () => {
  const { component } = render(MyComponent);
  // Изменяем props или вызываем методы
  await tick();
  // Проверяем результат
});
```

### 4. Fake Timers

```typescript
beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

it('handles timeout', () => {
  startTimer(5000);
  vi.advanceTimersByTime(5000);
  expect(timerCompleted).toBe(true);
});
```

### 5. Snapshot Testing (осторожно)

```typescript
// Используй только для стабильных структур данных
it('matches snapshot', () => {
  const result = transformData(input);
  expect(result).toMatchSnapshot();
});
```

## Команды

```bash
# Запуск всех тестов
pnpm test

# Watch mode
pnpm test:watch

# С coverage
pnpm vitest run --coverage

# Конкретный файл
pnpm vitest run src/lib/stores/toast.test.ts

# По паттерну
pnpm vitest run --grep "toast"
```

## Что тестировать

### ✅ Обязательно

- **Stores** — вся логика состояния
- **Utils** — чистые функции
- **API types** — совместимость с backend
- **Критичные компоненты** — модалки, формы

### ⚠️ По необходимости

- **UI компоненты** — только поведение, не стили
- **Интеграции** — взаимодействие модулей

### ❌ Не тестировать

- Стили (Tailwind классы)
- Статический контент
- Внешние библиотеки
- Tauri backend (отдельные тесты на Rust)

## Troubleshooting

### "Cannot find module '$app/environment'"

Проверь alias в `vitest.config.ts`:
```typescript
alias: {
  '$app/environment': '/src/lib/__mocks__/app-environment.ts'
}
```

### "Component not updating in test"

Используй `await tick()` после изменений:
```typescript
component.updateState();
await tick();
```

### "Tauri invoke not mocked"

Мок должен быть ДО импорта модуля:
```typescript
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
import { myFunction } from './module';  // После мока
```

### "localStorage is not defined"

Добавь мок в начало файла:
```typescript
Object.defineProperty(globalThis, 'localStorage', {
  value: localStorageMock,
  writable: true
});
```
