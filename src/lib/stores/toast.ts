import { writable } from 'svelte/store';

export interface Toast {
  id: number;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
  duration: number;
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);
  let nextId = 0;

  return {
    subscribe,
    
    /**
     * Show a toast notification and log it
     */
    show(type: Toast['type'], message: string, duration = 5000) {
      const id = ++nextId;
      
      // Log to console with appropriate level
      const logPrefix = '[Toast]';
      switch (type) {
        case 'success':
          console.log(`${logPrefix} ✓ ${message}`);
          break;
        case 'error':
          console.error(`${logPrefix} ✗ ${message}`);
          break;
        case 'warning':
          console.warn(`${logPrefix} ⚠ ${message}`);
          break;
        case 'info':
          console.info(`${logPrefix} ℹ ${message}`);
          break;
      }
      
      update(toasts => [...toasts, { id, type, message, duration }]);
      
      // Auto-remove after duration
      if (duration > 0) {
        setTimeout(() => {
          this.dismiss(id);
        }, duration);
      }
      
      return id;
    },
    
    success(message: string, duration = 4000) {
      return this.show('success', message, duration);
    },
    
    error(message: string, duration = 8000) {
      return this.show('error', message, duration);
    },
    
    warning(message: string, duration = 5000) {
      return this.show('warning', message, duration);
    },
    
    info(message: string, duration = 4000) {
      return this.show('info', message, duration);
    },
    
    dismiss(id: number) {
      update(toasts => toasts.filter(t => t.id !== id));
    },
    
    clear() {
      update(() => []);
    }
  };
}

export const toasts = createToastStore();
