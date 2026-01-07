import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { logs, logFilters, filteredLogs, logSources, createSourceLogsStore, getLogsBySource, type LogEntry } from './logs';

// Mock crypto.randomUUID
vi.stubGlobal('crypto', {
  randomUUID: () => `test-uuid-${Math.random().toString(36).substring(7)}`
});

describe('logs store', () => {
  beforeEach(() => {
    logs.clear();
    logFilters.set({ level: 'all', source: 'all', search: '' });
  });

  describe('logging methods', () => {
    it('logs.info() creates info log', () => {
      logs.info('TestSource', 'Info message');
      
      const currentLogs = get(logs);
      expect(currentLogs).toHaveLength(1);
      expect(currentLogs[0].level).toBe('info');
      expect(currentLogs[0].source).toBe('TestSource');
      expect(currentLogs[0].message).toBe('Info message');
      expect(currentLogs[0].id).toBeDefined();
      expect(currentLogs[0].timestamp).toBeInstanceOf(Date);
    });

    it('logs.warn() creates warn log', () => {
      logs.warn('TestSource', 'Warning message');
      
      const currentLogs = get(logs);
      expect(currentLogs).toHaveLength(1);
      expect(currentLogs[0].level).toBe('warn');
      expect(currentLogs[0].source).toBe('TestSource');
      expect(currentLogs[0].message).toBe('Warning message');
    });

    it('logs.error() creates error log', () => {
      logs.error('TestSource', 'Error message');
      
      const currentLogs = get(logs);
      expect(currentLogs).toHaveLength(1);
      expect(currentLogs[0].level).toBe('error');
      expect(currentLogs[0].source).toBe('TestSource');
      expect(currentLogs[0].message).toBe('Error message');
    });

    it('logs.debug() creates debug log', () => {
      logs.debug('TestSource', 'Debug message');
      
      const currentLogs = get(logs);
      expect(currentLogs).toHaveLength(1);
      expect(currentLogs[0].level).toBe('debug');
      expect(currentLogs[0].source).toBe('TestSource');
      expect(currentLogs[0].message).toBe('Debug message');
    });

    it('logs.success() creates success log', () => {
      logs.success('TestSource', 'Success message');
      
      const currentLogs = get(logs);
      expect(currentLogs).toHaveLength(1);
      expect(currentLogs[0].level).toBe('success');
      expect(currentLogs[0].source).toBe('TestSource');
      expect(currentLogs[0].message).toBe('Success message');
    });

    it('logs.add() creates log with custom level', () => {
      logs.add({ level: 'info', source: 'CustomSource', message: 'Custom message' });
      
      const currentLogs = get(logs);
      expect(currentLogs).toHaveLength(1);
      expect(currentLogs[0].level).toBe('info');
      expect(currentLogs[0].source).toBe('CustomSource');
    });
  });

  describe('logs.clear()', () => {
    it('clears all logs', () => {
      logs.info('Source1', 'Message 1');
      logs.warn('Source2', 'Message 2');
      logs.error('Source3', 'Message 3');
      
      expect(get(logs)).toHaveLength(3);
      
      logs.clear();
      
      expect(get(logs)).toHaveLength(0);
    });
  });

  describe('MAX_LOGS limit', () => {
    it('keeps only last 500 logs', () => {
      // Add 510 logs
      for (let i = 0; i < 510; i++) {
        logs.info('TestSource', `Message ${i}`);
      }
      
      const currentLogs = get(logs);
      expect(currentLogs).toHaveLength(500);
      // First log should be Message 10 (0-9 were removed)
      expect(currentLogs[0].message).toBe('Message 10');
      // Last log should be Message 509
      expect(currentLogs[499].message).toBe('Message 509');
    });
  });

  describe('filteredLogs - filter by level', () => {
    beforeEach(() => {
      logs.info('Source', 'Info message');
      logs.warn('Source', 'Warn message');
      logs.error('Source', 'Error message');
      logs.debug('Source', 'Debug message');
      logs.success('Source', 'Success message');
    });

    it('filters by level "all" returns all logs', () => {
      logFilters.set({ level: 'all', source: 'all', search: '' });
      
      expect(get(filteredLogs)).toHaveLength(5);
    });

    it('filters by level "info"', () => {
      logFilters.set({ level: 'info', source: 'all', search: '' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].level).toBe('info');
    });

    it('filters by level "warn"', () => {
      logFilters.set({ level: 'warn', source: 'all', search: '' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].level).toBe('warn');
    });

    it('filters by level "error"', () => {
      logFilters.set({ level: 'error', source: 'all', search: '' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].level).toBe('error');
    });

    it('filters by level "debug"', () => {
      logFilters.set({ level: 'debug', source: 'all', search: '' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].level).toBe('debug');
    });

    it('filters by level "success"', () => {
      logFilters.set({ level: 'success', source: 'all', search: '' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].level).toBe('success');
    });
  });

  describe('filteredLogs - filter by source', () => {
    beforeEach(() => {
      logs.info('SourceA', 'Message from A');
      logs.info('SourceB', 'Message from B');
      logs.warn('SourceA', 'Warning from A');
      logs.error('SourceC', 'Error from C');
    });

    it('filters by source "all" returns all logs', () => {
      logFilters.set({ level: 'all', source: 'all', search: '' });
      
      expect(get(filteredLogs)).toHaveLength(4);
    });

    it('filters by specific source', () => {
      logFilters.set({ level: 'all', source: 'SourceA', search: '' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(2);
      expect(filtered.every(log => log.source === 'SourceA')).toBe(true);
    });

    it('filters by source with no matches returns empty', () => {
      logFilters.set({ level: 'all', source: 'NonExistent', search: '' });
      
      expect(get(filteredLogs)).toHaveLength(0);
    });
  });

  describe('filteredLogs - filter by search', () => {
    beforeEach(() => {
      logs.info('SourceA', 'Hello world');
      logs.info('SourceB', 'Goodbye world');
      logs.warn('SourceA', 'Hello again');
      logs.error('HelloSource', 'Error message');
    });

    it('filters by search in message (case insensitive)', () => {
      logFilters.set({ level: 'all', source: 'all', search: 'hello' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(3); // 2 messages + 1 source match
    });

    it('filters by search in source (case insensitive)', () => {
      logFilters.set({ level: 'all', source: 'all', search: 'sourcea' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(2);
    });

    it('empty search returns all logs', () => {
      logFilters.set({ level: 'all', source: 'all', search: '' });
      
      expect(get(filteredLogs)).toHaveLength(4);
    });
  });

  describe('filteredLogs - combined filters', () => {
    beforeEach(() => {
      logs.info('SourceA', 'Hello info');
      logs.warn('SourceA', 'Hello warn');
      logs.info('SourceB', 'Hello info B');
      logs.error('SourceA', 'Goodbye error');
    });

    it('filters by level AND source', () => {
      logFilters.set({ level: 'info', source: 'SourceA', search: '' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].message).toBe('Hello info');
    });

    it('filters by level AND source AND search', () => {
      logFilters.set({ level: 'info', source: 'all', search: 'hello' });
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(2);
    });
  });

  describe('logSources derived store', () => {
    it('returns empty array when no logs', () => {
      expect(get(logSources)).toEqual([]);
    });

    it('returns unique sources sorted alphabetically', () => {
      logs.info('Zebra', 'Message');
      logs.info('Alpha', 'Message');
      logs.info('Beta', 'Message');
      logs.info('Alpha', 'Another message'); // duplicate
      
      expect(get(logSources)).toEqual(['Alpha', 'Beta', 'Zebra']);
    });
  });

  describe('getLogsBySource helper', () => {
    it('returns logs filtered by source', () => {
      logs.info('SourceA', 'Message 1');
      logs.info('SourceB', 'Message 2');
      logs.info('SourceA', 'Message 3');
      
      const sourceALogs = getLogsBySource('SourceA');
      expect(sourceALogs).toHaveLength(2);
      expect(sourceALogs.every(log => log.source === 'SourceA')).toBe(true);
    });

    it('returns empty array for non-existent source', () => {
      logs.info('SourceA', 'Message');
      
      expect(getLogsBySource('NonExistent')).toEqual([]);
    });
  });

  describe('createSourceLogsStore', () => {
    it('creates derived store for specific source', () => {
      const sourceAStore = createSourceLogsStore('SourceA');
      
      logs.info('SourceA', 'Message 1');
      logs.info('SourceB', 'Message 2');
      logs.info('SourceA', 'Message 3');
      
      const sourceALogs = get(sourceAStore);
      expect(sourceALogs).toHaveLength(2);
      expect(sourceALogs.every(log => log.source === 'SourceA')).toBe(true);
    });

    it('updates when new logs are added', () => {
      const sourceAStore = createSourceLogsStore('SourceA');
      
      expect(get(sourceAStore)).toHaveLength(0);
      
      logs.info('SourceA', 'Message 1');
      expect(get(sourceAStore)).toHaveLength(1);
      
      logs.info('SourceA', 'Message 2');
      expect(get(sourceAStore)).toHaveLength(2);
    });
  });

  describe('log entry structure', () => {
    it('generates unique IDs for each log', () => {
      logs.info('Source', 'Message 1');
      logs.info('Source', 'Message 2');
      
      const currentLogs = get(logs);
      expect(currentLogs[0].id).not.toBe(currentLogs[1].id);
    });

    it('sets timestamp to current date', () => {
      const before = new Date();
      logs.info('Source', 'Message');
      const after = new Date();
      
      const currentLogs = get(logs);
      expect(currentLogs[0].timestamp.getTime()).toBeGreaterThanOrEqual(before.getTime());
      expect(currentLogs[0].timestamp.getTime()).toBeLessThanOrEqual(after.getTime());
    });
  });
});
