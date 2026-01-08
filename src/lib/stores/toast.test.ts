import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { toasts, formatToastMessage, type Toast } from './toast';

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
        duration: 4000,
        count: 1
      });
    });

    it('использует кастомный duration', () => {
      toasts.success('Test', 10000);

      const currentToasts = get(toasts);
      expect(currentToasts[0].duration).toBe(10000);
    });

    it('использует кастомный duration через options', () => {
      toasts.success('Test', { duration: 15000 });

      const currentToasts = get(toasts);
      expect(currentToasts[0].duration).toBe(15000);
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
        duration: 8000,
        count: 1
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
        duration: 5000,
        count: 1
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
        duration: 4000,
        count: 1
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

      toasts.dismiss('non-existent-id');

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
      toasts.success('Short', { duration: 1000, noDedupe: true });
      toasts.error('Long', { duration: 5000, noDedupe: true });

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
      const id1 = toasts.success('Toast 1', { noDedupe: true });
      const id2 = toasts.success('Toast 2', { noDedupe: true });
      const id3 = toasts.success('Toast 3', { noDedupe: true });

      expect(id1).not.toBe(id2);
      expect(id2).not.toBe(id3);
      expect(id1).not.toBe(id3);
    });

    it('id является строкой', () => {
      const id = toasts.success('Test');
      expect(typeof id).toBe('string');
      expect(id).toMatch(/^toast-\d+-\d+$/);
    });
  });

  describe('show()', () => {
    it('возвращает id созданного toast', () => {
      const id = toasts.success('Test');
      expect(typeof id).toBe('string');
      expect(id.length).toBeGreaterThan(0);
    });

    it('добавляет toast в конец списка', () => {
      toasts.success('First', { noDedupe: true });
      toasts.error('Second', { noDedupe: true });
      toasts.info('Third', { noDedupe: true });

      const currentToasts = get(toasts);
      expect(currentToasts[0].message).toBe('First');
      expect(currentToasts[1].message).toBe('Second');
      expect(currentToasts[2].message).toBe('Third');
    });
  });

  describe('дедупликация', () => {
    it('увеличивает счётчик при повторном toast с тем же сообщением', () => {
      const id1 = toasts.error('Ошибка подключения');
      const id2 = toasts.error('Ошибка подключения');
      const id3 = toasts.error('Ошибка подключения');

      // Должен быть только один toast
      expect(get(toasts)).toHaveLength(1);
      
      // ID должен быть одинаковым
      expect(id1).toBe(id2);
      expect(id2).toBe(id3);

      // Счётчик должен быть 3
      const currentToasts = get(toasts);
      expect(currentToasts[0].count).toBe(3);
    });

    it('логирует с учётом счётчика', () => {
      toasts.error('Ошибка');
      toasts.error('Ошибка');
      toasts.error('Ошибка');

      expect(console.error).toHaveBeenCalledWith('[Toast] ✗ Ошибка');
      expect(console.error).toHaveBeenCalledWith('[Toast] ✗ Ошибка (×2)');
      expect(console.error).toHaveBeenCalledWith('[Toast] ✗ Ошибка (×3)');
    });

    it('разные сообщения создают разные toasts', () => {
      toasts.error('Ошибка 1');
      toasts.error('Ошибка 2');
      toasts.error('Ошибка 1');

      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(2);
      expect(currentToasts[0].count).toBe(2);
      expect(currentToasts[1].count).toBe(1);
    });

    it('noDedupe отключает дедупликацию', () => {
      toasts.error('Ошибка', { noDedupe: true });
      toasts.error('Ошибка', { noDedupe: true });
      toasts.error('Ошибка', { noDedupe: true });

      expect(get(toasts)).toHaveLength(3);
    });

    it('dedupeKey позволяет группировать разные сообщения', () => {
      toasts.error('Ошибка сети: timeout', { dedupeKey: 'network-error' });
      toasts.error('Ошибка сети: connection refused', { dedupeKey: 'network-error' });

      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(1);
      expect(currentToasts[0].count).toBe(2);
      // Сообщение остаётся от первого toast
      expect(currentToasts[0].message).toBe('Ошибка сети: timeout');
    });

    it('сбрасывает таймер при дедупликации', () => {
      toasts.error('Ошибка', { duration: 5000 });
      
      vi.advanceTimersByTime(4000);
      expect(get(toasts)).toHaveLength(1);

      // Повторный toast сбрасывает таймер
      toasts.error('Ошибка', { duration: 5000 });
      
      vi.advanceTimersByTime(4000);
      expect(get(toasts)).toHaveLength(1); // Ещё не удалён

      vi.advanceTimersByTime(1000);
      expect(get(toasts)).toHaveLength(0); // Теперь удалён
    });

    it('dismiss очищает dedupeKey', () => {
      const id = toasts.error('Ошибка');
      expect(get(toasts)).toHaveLength(1);

      toasts.dismiss(id);
      expect(get(toasts)).toHaveLength(0);

      // Новый toast с тем же сообщением создаётся заново
      toasts.error('Ошибка');
      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(1);
      expect(currentToasts[0].count).toBe(1);
    });
  });

  describe('updateToast()', () => {
    it('обновляет существующий toast', () => {
      const id = toasts.info('Загрузка...');
      
      const updated = toasts.updateToast(id, { message: 'Загрузка завершена' });
      
      expect(updated).toBe(true);
      expect(get(toasts)[0].message).toBe('Загрузка завершена');
    });

    it('возвращает false для несуществующего toast', () => {
      const updated = toasts.updateToast('non-existent', { message: 'Test' });
      expect(updated).toBe(false);
    });

    it('обновляет несколько полей', () => {
      const id = toasts.info('Начало');
      
      toasts.updateToast(id, { 
        message: 'Конец', 
        type: 'success',
        duration: 10000
      });
      
      const toast = get(toasts)[0];
      expect(toast.message).toBe('Конец');
      expect(toast.type).toBe('success');
      expect(toast.duration).toBe(10000);
    });
  });

  describe('progressToast()', () => {
    it('создаёт новый progress toast', () => {
      const id = toasts.progressToast(null, 'Загрузка файла', 0);
      
      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(1);
      expect(currentToasts[0]).toMatchObject({
        id,
        type: 'progress',
        message: 'Загрузка файла',
        progress: 0,
        duration: 0
      });
    });

    it('обновляет существующий progress toast', () => {
      const id = toasts.progressToast(null, 'Загрузка', 0);
      
      toasts.progressToast(id, 'Загрузка', 50);
      
      const currentToasts = get(toasts);
      expect(currentToasts).toHaveLength(1);
      expect(currentToasts[0].progress).toBe(50);
    });

    it('ограничивает progress в диапазоне 0-100', () => {
      const id = toasts.progressToast(null, 'Test', -10);
      expect(get(toasts)[0].progress).toBe(0);

      toasts.progressToast(id, 'Test', 150);
      expect(get(toasts)[0].progress).toBe(100);
    });

    it('автоматически закрывается при 100%', () => {
      const id = toasts.progressToast(null, 'Загрузка', 0);
      
      toasts.progressToast(id, 'Загрузка завершена', 100);
      
      expect(get(toasts)).toHaveLength(1);
      
      vi.advanceTimersByTime(2000);
      expect(get(toasts)).toHaveLength(0);
    });

    it('логирует прогресс', () => {
      toasts.progressToast(null, 'Загрузка', 50);
      expect(console.info).toHaveBeenCalledWith('[Toast] ⏳ Загрузка (50%)');
    });
  });

  describe('completeProgress()', () => {
    it('завершает progress toast успехом', () => {
      const id = toasts.progressToast(null, 'Загрузка', 50);
      
      toasts.completeProgress(id, 'Загрузка завершена!');
      
      const toast = get(toasts)[0];
      expect(toast.type).toBe('success');
      expect(toast.message).toBe('Загрузка завершена!');
      expect(toast.progress).toBe(100);
    });

    it('автоматически закрывается через 3 секунды', () => {
      const id = toasts.progressToast(null, 'Загрузка', 50);
      toasts.completeProgress(id);
      
      expect(get(toasts)).toHaveLength(1);
      
      vi.advanceTimersByTime(3000);
      expect(get(toasts)).toHaveLength(0);
    });
  });

  describe('failProgress()', () => {
    it('завершает progress toast ошибкой', () => {
      const id = toasts.progressToast(null, 'Загрузка', 50);
      
      toasts.failProgress(id, 'Ошибка загрузки');
      
      const toast = get(toasts)[0];
      expect(toast.type).toBe('error');
      expect(toast.message).toBe('Ошибка загрузки');
      expect(toast.progress).toBeUndefined();
    });

    it('автоматически закрывается через 8 секунд', () => {
      const id = toasts.progressToast(null, 'Загрузка', 50);
      toasts.failProgress(id);
      
      expect(get(toasts)).toHaveLength(1);
      
      vi.advanceTimersByTime(8000);
      expect(get(toasts)).toHaveLength(0);
    });
  });

  describe('getDisplayMessage()', () => {
    it('возвращает сообщение без счётчика для count=1', () => {
      const id = toasts.success('Test');
      const toast = get(toasts)[0];
      
      expect(toasts.getDisplayMessage(toast)).toBe('Test');
    });

    it('возвращает сообщение со счётчиком для count>1', () => {
      toasts.error('Ошибка');
      toasts.error('Ошибка');
      toasts.error('Ошибка');
      
      const toast = get(toasts)[0];
      expect(toasts.getDisplayMessage(toast)).toBe('Ошибка (×3)');
    });
  });

  describe('formatToastMessage()', () => {
    it('форматирует сообщение без счётчика для count=1', () => {
      expect(formatToastMessage('Test', 1)).toBe('Test');
    });

    it('форматирует сообщение со счётчиком для count>1', () => {
      expect(formatToastMessage('Error', 5)).toBe('Error (×5)');
    });
  });
});
