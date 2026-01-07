/**
 * Centralized logger for Isolate
 * Only logs in development mode (except warn/error which always log)
 */

const isDev = import.meta.env.DEV;

export const logger = {
  debug: (prefix: string, ...args: unknown[]) => {
    if (isDev) console.debug(`[${prefix}]`, ...args);
  },
  log: (prefix: string, ...args: unknown[]) => {
    if (isDev) console.log(`[${prefix}]`, ...args);
  },
  warn: (prefix: string, ...args: unknown[]) => {
    console.warn(`[${prefix}]`, ...args);
  },
  error: (prefix: string, ...args: unknown[]) => {
    console.error(`[${prefix}]`, ...args);
  }
};

export default logger;
