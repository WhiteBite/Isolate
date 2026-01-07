import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { toasts, type Toast } from './toast';

describe('toast store', () => {
  beforeEach(() => {
    // Очищаем toasts перед каждым тестом
    toasts.clear();
    // Используем fake timers
    vi.useFakeTimers();
    // Мокаем console методы
    vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'info').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  describe('success()', () => {
    it('создаёт success toast с правильным типом и сообщением', () => {
      const message = 'Операция выполнена успешно';
      const id = toasts.success(message);

      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(1);
      expect(currentToasts[0]).toMatchObject({
        id,
        type: 'success',
        message,
        duration: 4000
      });
    });

    it('использует кастомный duration', () => {
      toasts.success('Test', 10000);

      const currentToasts = get(toasts);
      expect(currentToasts[0].duration).toBe(10000);
    });

    it('логирует в console.log', () => {
      toasts.success('Success message');
      expect(console.log).toHaveBeenCalledWith('[Toast] ✓ Success message');
    });
  });

  describe('error()', () => {
    it('создаёт error toast с правильным типом и сообщением', () => {
      const message = 'Произошла ошибка';
      const id = toasts.error(message);

      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(1);
      expect(currentToasts[0]).toMatchObject({
        id,
        type: 'error',
        message,
        duration: 8000
      });
    });

    it('логирует в console.error', () => {
      toasts.error('Error message');
      expect(console.error).toHaveBeenCalledWith('[Toast] ✗ Error message');
    });
  });

  describe('warning()', () => {
    it('создаёт warning toast с правильным типом и сообщением', () => {
      const message = 'Внимание!';
      const id = toasts.warning(message);

      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(1);
      expect(currentToasts[0]).toMatchObject({
        id,
        type: 'warning',
        message,
        duration: 5000
      });
    });

    it('логирует в console.warn', () => {
      toasts.warning('Warning message');
      expect(console.warn).toHaveBeenCalledWith('[Toast] ⚠ Warning message');
    });
  });

  describe('info()', () => {
    it('создаёт info toast с правильным типом и сообщением', () => {
      const message = 'Информация';
      const id = toasts.info(message);

      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(1);
      expect(currentToasts[0]).toMatchObject({
        id,
        type: 'info',
        message,
        duration: 4000
      });
    });

    it('логирует в console.info', () => {
      toasts.info('Info message');
      expect(console.info).toHaveBeenCalledWith('[Toast] ℹ Info message');
    });
  });

  describe('dismiss()', () => {
    it('удаляет toast по id', () => {
      const id1 = toasts.success('Toast 1');
      const id2 = toasts.error('Toast 2');
      const id3 = toasts.info('Toast 3');

      expect(get(toasts)).toHaveLength(3);

      toasts.dismiss(id2);

      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(2);
      expect(currentToasts.find(t => t.id === id2)).toBeUndefined();
      expect(currentToasts.find(t => t.id === id1)).toBeDefined();
      expect(currentToasts.find(t => t.id === id3)).toBeDefined();
    });

    it('ничего не делает при несуществующем id', () => {
      toasts.success('Toast 1');
      toasts.error('Toast 2');

      expect(get(toasts)).toHaveLength(2);

      toasts.dismiss(99999);

      expect(get(toasts)).toHaveLength(2);
    });
  });

  describe('clear()', () => {
    it('удаляет все toasts', () => {
      toasts.success('Toast 1');
      toasts.error('Toast 2');
      toasts.warning('Toast 3');
      toasts.info('Toast 4');

      expect(get(toasts)).toHaveLength(4);

      toasts.clear();

      expect(get(toasts)).toHaveLength(0);
    });

    it('работает на пустом store', () => {
      expect(get(toasts)).toHaveLength(0);
      toasts.clear();
      expect(get(toasts)).toHaveLength(0);
    });
  });

  describe('auto-dismiss', () => {
    it('автоматически удаляет toast после timeout', () => {
      toasts.success('Auto dismiss test', 3000);

      expect(get(toasts)).toHaveLength(1);

      // Прошло 2999ms — toast ещё есть
      vi.advanceTimersByTime(2999);
      expect(get(toasts)).toHaveLength(1);

      // Прошло ещё 1ms (всего 3000ms) — toast удалён
      vi.advanceTimersByTime(1);
      expect(get(toasts)).toHaveLength(0);
    });

    it('не удаляет toast если duration = 0', () => {
      toasts.success('Persistent toast', 0);

      expect(get(toasts)).toHaveLength(1);

      vi.advanceTimersByTime(100000);

      expect(get(toasts)).toHaveLength(1);
    });

    it('каждый toast удаляется по своему timeout', () => {
      toasts.success('Short', 1000);
      toasts.error('Long', 5000);

      expect(get(toasts)).toHaveLength(2);

      vi.advanceTimersByTime(1000);
      expect(get(toasts)).toHaveLength(1);
      expect(get(toasts)[0].type).toBe('error');

      vi.advanceTimersByTime(4000);
      expect(get(toasts)).toHaveLength(0);
    });
  });

  describe('уникальные id', () => {
    it('каждый toast получает уникальный id', () => {
      const id1 = toasts.success('Toast 1');
      const id2 = toasts.success('Toast 2');
      const id3 = toasts.success('Toast 3');

      expect(id1).not.toBe(id2);
      expect(id2).not.toBe(id3);
      expect(id1).not.toBe(id3);
    });

    it('id инкрементируется', () => {
      const id1 = toasts.success('Toast 1');
      const id2 = toasts.success('Toast 2');

      expect(id2).toBe(id1 + 1);
    });
  });

  describe('show()', () => {
    it('возвращает id созданного toast', () => {
      const id = toasts.success('Test');
      expect(typeof id).toBe('number');
      expect(id).toBeGreaterThan(0);
    });

    it('добавляет toast в конец списка', () => {
      toasts.success('First');
      toasts.error('Second');
      toasts.info('Third');

      const currentToasts = get(toasts);
      expect(currentToasts[0].message).toBe('First');
      expect(currentToasts[1].message).toBe('Second');
      expect(currentToasts[2].message).toBe('Third');
    });
  });
});
