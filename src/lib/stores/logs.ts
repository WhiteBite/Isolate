import { writable, derived, get } from 'svelte/store';

export type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'success';

export interface LogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  source: string;
  message: string;
}

export interface LogFilters {
  level: LogLevel | 'all';
  source: string | 'all';
  search: string;
}

const MAX_LOGS = 500;

function createLogsStore() {
  const { subscribe, update, set } = writable<LogEntry[]>([]);

  function addLog(entry: Omit<LogEntry, 'id' | 'timestamp'>) {
    update(logs => {
      const newLog: LogEntry = {
        ...entry,
        id: crypto.randomUUID(),
        timestamp: new Date()
      };
      // Keep only last MAX_LOGS entries
      const newLogs = [...logs, newLog];
      return newLogs.slice(-MAX_LOGS);
    });
  }

  return {
    subscribe,
    
    add: addLog,
    
    clear: () => set([]),
    
    error: (source: string, message: string) => {
      addLog({ level: 'error', source, message });
    },
    
    warn: (source: string, message: string) => {
      addLog({ level: 'warn', source, message });
    },
    
    info: (source: string, message: string) => {
      addLog({ level: 'info', source, message });
    },
    
    debug: (source: string, message: string) => {
      addLog({ level: 'debug', source, message });
    },
    
    success: (source: string, message: string) => {
      addLog({ level: 'success', source, message });
    },
  };
}

export const logs = createLogsStore();

// Filters store
export const logFilters = writable<LogFilters>({
  level: 'all',
  source: 'all',
  search: ''
});

// Derived store for filtered logs
export const filteredLogs = derived(
  [logs, logFilters],
  ([$logs, $filters]) => {
    return $logs.filter(log => {
      // Filter by level
      if ($filters.level !== 'all' && log.level !== $filters.level) {
        return false;
      }
      
      // Filter by source
      if ($filters.source !== 'all' && log.source !== $filters.source) {
        return false;
      }
      
      // Filter by search text
      if ($filters.search) {
        const searchLower = $filters.search.toLowerCase();
        return (
          log.message.toLowerCase().includes(searchLower) ||
          log.source.toLowerCase().includes(searchLower)
        );
      }
      
      return true;
    });
  }
);

// Derived store for unique sources
export const logSources = derived(logs, ($logs) => {
  const sources = new Set($logs.map(log => log.source));
  return Array.from(sources).sort();
});

// Helper function to get logs by source (for use outside of Svelte components)
export function getLogsBySource(source: string): LogEntry[] {
  return get(logs).filter(log => log.source === source);
}

// Create a derived store for a specific source
export function createSourceLogsStore(source: string) {
  return derived(logs, ($logs) => {
    return $logs.filter(log => log.source === source);
  });
}
