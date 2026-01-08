/**
 * Simple i18n system for Isolate
 * Uses Svelte 5 runes for reactivity
 */

import { browser } from '$app/environment';
import en from './locales/en.json';
import ru from './locales/ru.json';

// Types
export type Locale = 'en' | 'ru';
export type TranslationKey = string;

interface Translations {
  [key: string]: string | Translations;
}

// Available locales
const locales: Record<Locale, Translations> = { en, ru };

// Storage key
const LOCALE_STORAGE_KEY = 'isolate_locale';

// Default locale
const DEFAULT_LOCALE: Locale = 'ru';

// Reactive state using module-level state
let currentLocale = $state<Locale>(DEFAULT_LOCALE);

/**
 * Initialize locale from localStorage
 * Call this in root layout
 */
export function initLocale(): void {
  if (!browser) return;
  
  const stored = localStorage.getItem(LOCALE_STORAGE_KEY) as Locale | null;
  if (stored && (stored === 'en' || stored === 'ru')) {
    currentLocale = stored;
  }
}

/**
 * Get current locale
 */
export function getLocale(): Locale {
  return currentLocale;
}

/**
 * Set locale and persist to localStorage
 */
export function setLocale(locale: Locale): void {
  if (locale !== 'en' && locale !== 'ru') return;
  
  currentLocale = locale;
  
  if (browser) {
    localStorage.setItem(LOCALE_STORAGE_KEY, locale);
  }
}

/**
 * Get nested value from object by dot-notation path
 */
function getNestedValue(obj: Translations, path: string): string | undefined {
  const keys = path.split('.');
  let current: Translations | string = obj;
  
  for (const key of keys) {
    if (typeof current !== 'object' || current === null) {
      return undefined;
    }
    current = current[key];
  }
  
  return typeof current === 'string' ? current : undefined;
}

/**
 * Translate a key to current locale
 * Supports dot notation: t('settings.general.title')
 * Falls back to English, then to key itself
 */
export function t(key: TranslationKey): string {
  // Try current locale
  const translation = getNestedValue(locales[currentLocale], key);
  if (translation) return translation;
  
  // Fallback to English
  if (currentLocale !== 'en') {
    const fallback = getNestedValue(locales.en, key);
    if (fallback) return fallback;
  }
  
  // Return key as last resort
  return key;
}

/**
 * Reactive translation function for use in components
 * Returns a getter that updates when locale changes
 */
export function useTranslation() {
  return {
    t,
    get locale() {
      return currentLocale;
    },
    setLocale,
    locales: ['en', 'ru'] as const
  };
}

/**
 * Get all available locales with their display names
 */
export function getAvailableLocales(): Array<{ code: Locale; name: string }> {
  return [
    { code: 'en', name: 'English' },
    { code: 'ru', name: 'Русский' }
  ];
}
