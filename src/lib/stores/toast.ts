import { writable, derived } from 'svelte/store';

/** Максимальное количество одновременно отображаемых toast */
export const MAX_VISIBLE_TOASTS = 5;

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info' | 'progress';
  message: string;
  duration: number;
  /** Счётчик дубликатов для группировки */
  count: number;
  /** Прогресс 0-100 для progress toast */
  progress?: number;
  /** Ключ для дедупликации (по умолчанию = message) */
  dedupeKey?: string;
  /** Timestamp создания для сортировки */
  createdAt: number;
  /** Приоритет (error > warning > progress > info > success) */
  priority: number;
}

/** Опции для создания toast */
export interface ToastOptions {
  duration?: number;
  /** Ключ для дедупликации. Если не указан, используется message */
  dedupeKey?: string;
  /** Отключить дедупликацию для этого toast */
  noDedupe?: boolean;
}

/** Приоритеты типов toast (выше = важнее) */
const TYPE_PRIORITY: Record<Toast['type'], number> = {
  error: 5,
  warning: 4,
  progress: 3,
  info: 2,
  success: 1
};

function createToastStore() {
  const { subscribe, update, set } = writable<Toast[]>([]);
  let nextId = 0;
  
  // Map для быстрого поиска по dedupeKey
  const dedupeMap = new Map<string, string>(); // dedupeKey -> toast id
  const timeoutMap = new Map<string, ReturnType<typeof setTimeout>>(); // toast id -> timeout

  /**
   * Генерирует уникальный ID
   */
  function generateId(): string {
    return `toast-${++nextId}-${Date.now()}`;
  }

  /**
   * Форматирует сообщение с учётом счётчика
   */
  function formatMessage(message: string, count: number): string {
    return count > 1 ? `${message} (×${count})` : message;
  }

  /**
   * Логирует toast в консоль
   */
  function logToast(type: Toast['type'], message: string, count: number) {
    const logPrefix = '[Toast]';
    const displayMsg = formatMessage(message, count);
    
    switch (type) {
      case 'success':
        console.log(`${logPrefix} ✓ ${displayMsg}`);
        break;
      case 'error':
        console.error(`${logPrefix} ✗ ${displayMsg}`);
        break;
      case 'warning':
        console.warn(`${logPrefix} ⚠ ${displayMsg}`);
        break;
      case 'info':
      case 'progress':
        console.info(`${logPrefix} ℹ ${displayMsg}`);
        break;
    }
  }

  /**
   * Очищает таймаут для toast
   */
  function clearToastTimeout(id: string) {
    const timeout = timeoutMap.get(id);
    if (timeout) {
      clearTimeout(timeout);
      timeoutMap.delete(id);
    }
  }

  /**
   * Устанавливает таймаут для автоудаления toast
   */
  function setToastTimeout(id: string, duration: number, dismiss: (id: string) => void) {
    if (duration > 0) {
      clearToastTimeout(id);
      const timeout = setTimeout(() => {
        dismiss(id);
      }, duration);
      timeoutMap.set(id, timeout);
    }
  }

  /**
   * Сортирует toasts по приоритету и времени создания
   * Более важные и новые — выше
   */
  function sortToasts(toasts: Toast[]): Toast[] {
    return [...toasts].sort((a, b) => {
      // Сначала по приоритету (выше = важнее)
      if (a.priority !== b.priority) {
        return b.priority - a.priority;
      }
      // Затем по времени (новее = выше)
      return b.createdAt - a.createdAt;
    });
  }

  /**
   * Применяет лимит на количество toast
   * Удаляет наименее важные/старые toast если превышен лимит
   */
  function applyLimit(toasts: Toast[], dismiss: (id: string) => void): Toast[] {
    if (toasts.length <= MAX_VISIBLE_TOASTS) {
      return toasts;
    }

    const sorted = sortToasts(toasts);
    const toKeep = sorted.slice(0, MAX_VISIBLE_TOASTS);
    const toRemove = sorted.slice(MAX_VISIBLE_TOASTS);

    // Очищаем таймауты и dedupeMap для удаляемых
    for (const toast of toRemove) {
      clearToastTimeout(toast.id);
      if (toast.dedupeKey) {
        dedupeMap.delete(toast.dedupeKey);
      }
    }

    return toKeep;
  }

  const store = {
    subscribe,
    
    /**
     * Показать toast с дедупликацией
     * Если toast с таким же message (или dedupeKey) уже показан, увеличивает счётчик
     */
    show(type: Toast['type'], message: string, options: ToastOptions = {}): string {
      const { duration = 5000, dedupeKey, noDedupe = false } = options;
      const key = dedupeKey ?? message;
      
      // Проверяем дедупликацию
      if (!noDedupe) {
        const existingId = dedupeMap.get(key);
        if (existingId) {
          // Увеличиваем счётчик существующего toast
          let newCount = 1;
          update(toasts => {
            return toasts.map(t => {
              if (t.id === existingId) {
                newCount = t.count + 1;
                return { ...t, count: newCount, createdAt: Date.now() };
              }
              return t;
            });
          });
          
          // Сбрасываем таймер
          setToastTimeout(existingId, duration, this.dismiss.bind(this));
          
          logToast(type, message, newCount);
          return existingId;
        }
      }
      
      // Создаём новый toast
      const id = generateId();
      const toast: Toast = {
        id,
        type,
        message,
        duration,
        count: 1,
        dedupeKey: key,
        createdAt: Date.now(),
        priority: TYPE_PRIORITY[type]
      };
      
      if (!noDedupe) {
        dedupeMap.set(key, id);
      }
      
      logToast(type, message, 1);
      
      update(toasts => {
        const newToasts = [...toasts, toast];
        return applyLimit(newToasts, this.dismiss.bind(this));
      });
      
      // Auto-remove after duration
      setToastTimeout(id, duration, this.dismiss.bind(this));
      
      return id;
    },
    
    /**
     * Показать success toast
     */
    success(message: string, options: ToastOptions | number = {}): string {
      const opts = typeof options === 'number' ? { duration: options } : options;
      return this.show('success', message, { duration: 4000, ...opts });
    },
    
    /**
     * Показать error toast (дольше отображается)
     */
    error(message: string, options: ToastOptions | number = {}): string {
      const opts = typeof options === 'number' ? { duration: options } : options;
      return this.show('error', message, { duration: 10000, ...opts });
    },
    
    /**
     * Показать warning toast
     */
    warning(message: string, options: ToastOptions | number = {}): string {
      const opts = typeof options === 'number' ? { duration: options } : options;
      return this.show('warning', message, { duration: 6000, ...opts });
    },
    
    /**
     * Показать info toast
     */
    info(message: string, options: ToastOptions | number = {}): string {
      const opts = typeof options === 'number' ? { duration: options } : options;
      return this.show('info', message, { duration: 5000, ...opts });
    },
    
    /**
     * Обновить существующий toast
     */
    updateToast(id: string, updates: Partial<Omit<Toast, 'id'>>): boolean {
      let found = false;
      
      update(toasts => {
        return toasts.map(t => {
          if (t.id === id) {
            found = true;
            const updated = { ...t, ...updates };
            
            // Если изменился dedupeKey, обновляем map
            if (updates.dedupeKey && t.dedupeKey !== updates.dedupeKey) {
              if (t.dedupeKey) {
                dedupeMap.delete(t.dedupeKey);
              }
              dedupeMap.set(updates.dedupeKey, id);
            }
            
            // Если изменился duration, обновляем таймер
            if (updates.duration !== undefined) {
              setToastTimeout(id, updates.duration, this.dismiss.bind(this));
            }
            
            return updated;
          }
          return t;
        });
      });
      
      return found;
    },
    
    /**
     * Создать или обновить progress toast
     * @param id - ID существующего toast или null для создания нового
     * @param message - Сообщение
     * @param progress - Прогресс 0-100
     * @returns ID toast
     */
    progress(id: string | null, message: string, progress: number): string {
      const clampedProgress = Math.max(0, Math.min(100, progress));
      
      // Если есть существующий toast, обновляем его
      if (id) {
        const updated = this.updateToast(id, {
          message,
          progress: clampedProgress,
          // Если прогресс 100%, автоматически закрываем через 2 сек
          duration: clampedProgress >= 100 ? 2000 : 0
        });
        
        if (updated) {
          if (clampedProgress >= 100) {
            setToastTimeout(id, 2000, this.dismiss.bind(this));
          }
          return id;
        }
      }
      
      // Создаём новый progress toast
      const newId = generateId();
      const toast: Toast = {
        id: newId,
        type: 'progress',
        message,
        duration: 0, // Progress toast не закрывается автоматически
        count: 1,
        progress: clampedProgress,
        dedupeKey: `progress-${newId}`, // Уникальный ключ, без дедупликации
        createdAt: Date.now(),
        priority: TYPE_PRIORITY['progress']
      };
      
      console.info(`[Toast] ⏳ ${message} (${clampedProgress}%)`);
      
      update(toasts => {
        const newToasts = [...toasts, toast];
        return applyLimit(newToasts, this.dismiss.bind(this));
      });
      
      return newId;
    },
    
    /**
     * Алиас для progress (обратная совместимость)
     * @deprecated Используйте progress() вместо progressToast()
     */
    progressToast(id: string | null, message: string, progress: number): string {
      return this.progress(id, message, progress);
    },
    
    /**
     * Завершить progress toast с успехом
     */
    completeProgress(id: string, message?: string): void {
      this.updateToast(id, {
        type: 'success',
        message: message ?? undefined,
        progress: 100,
        duration: 3000
      });
      setToastTimeout(id, 3000, this.dismiss.bind(this));
    },
    
    /**
     * Завершить progress toast с ошибкой
     */
    failProgress(id: string, message?: string): void {
      this.updateToast(id, {
        type: 'error',
        message: message ?? undefined,
        progress: undefined,
        duration: 8000
      });
      setToastTimeout(id, 8000, this.dismiss.bind(this));
    },
    
    /**
     * Закрыть toast
     */
    dismiss(id: string): void {
      clearToastTimeout(id);
      
      update(toasts => {
        const toast = toasts.find(t => t.id === id);
        if (toast?.dedupeKey) {
          dedupeMap.delete(toast.dedupeKey);
        }
        return toasts.filter(t => t.id !== id);
      });
    },
    
    /**
     * Закрыть все toasts
     */
    clear(): void {
      // Очищаем все таймауты
      timeoutMap.forEach(timeout => clearTimeout(timeout));
      timeoutMap.clear();
      dedupeMap.clear();
      
      set([]);
    },
    
    /**
     * Получить отформатированное сообщение для отображения
     */
    getDisplayMessage(toast: Toast): string {
      return formatMessage(toast.message, toast.count);
    }
  };

  return store;
}

export const toasts = createToastStore();

// Экспортируем хелпер для форматирования
export function formatToastMessage(message: string, count: number): string {
  return count > 1 ? `${message} (×${count})` : message;
}
