import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { logs, logFilters, filteredLogs, logSources, type LogEntry, type LogFilters } from './logs';

describe('logs store', () => {
  beforeEach(() => {
    logs.clear();
  });

  describe('initial state', () => {
    it('starts with empty array', () => {
      expect(get(logs)).toEqual([]);
    });
  });

  describe('add method', () => {
    it('adds log entry with generated id and timestamp', () => {
      logs.add({ level: 'info', source: 'test', message: 'Test message' });
      
      const entries = get(logs);
      expect(entries).toHaveLength(1);
      expect(entries[0]).toMatchObject({
        level: 'info',
        source: 'test',
        message: 'Test message'
      });
      expect(entries[0].id).toBeDefined();
      expect(entries[0].timestamp).toBeInstanceOf(Date);
    });

    it('adds multiple entries in order', () => {
      logs.add({ level: 'info', source: 'test', message: 'First' });
      logs.add({ level: 'warn', source: 'test', message: 'Second' });
      logs.add({ level: 'error', source: 'test', message: 'Third' });
      
      const entries = get(logs);
      expect(entries).toHaveLength(3);
      expect(entries[0].message).toBe('First');
      expect(entries[1].message).toBe('Second');
      expect(entries[2].message).toBe('Third');
    });
  });

  describe('level-specific methods', () => {
    it('error() adds error level log', () => {
      logs.error('source', 'Error message');
      
      const entries = get(logs);
      expect(entries[0].level).toBe('error');
      expect(entries[0].source).toBe('source');
      expect(entries[0].message).toBe('Error message');
    });

    it('warn() adds warn level log', () => {
      logs.warn('source', 'Warning message');
      
      const entries = get(logs);
      expect(entries[0].level).toBe('warn');
    });

    it('info() adds info level log', () => {
      logs.info('source', 'Info message');
      
      const entries = get(logs);
      expect(entries[0].level).toBe('info');
    });

    it('debug() adds debug level log', () => {
      logs.debug('source', 'Debug message');
      
      const entries = get(logs);
      expect(entries[0].level).toBe('debug');
    });

    it('success() adds success level log', () => {
      logs.success('source', 'Success message');
      
      const entries = get(logs);
      expect(entries[0].level).toBe('success');
    });
  });

  describe('clear method', () => {
    it('removes all entries', () => {
      logs.info('test', 'Message 1');
      logs.info('test', 'Message 2');
      logs.info('test', 'Message 3');
      
      expect(get(logs)).toHaveLength(3);
      
      logs.clear();
      
      expect(get(logs)).toEqual([]);
    });
  });

  describe('MAX_LOGS limit', () => {
    it('keeps only last 500 entries', () => {
      // Add 510 entries
      for (let i = 0; i < 510; i++) {
        logs.info('test', `Message ${i}`);
      }
      
      const entries = get(logs);
      expect(entries).toHaveLength(500);
      // First entry should be Message 10 (0-9 were removed)
      expect(entries[0].message).toBe('Message 10');
      expect(entries[499].message).toBe('Message 509');
    });
  });

  describe('subscription', () => {
    it('notifies subscribers on changes', () => {
      const callback = vi.fn();
      const unsubscribe = logs.subscribe(callback);
      
      // Initial call
      expect(callback).toHaveBeenCalledTimes(1);
      expect(callback).toHaveBeenLastCalledWith([]);
      
      logs.info('test', 'New message');
      
      expect(callback).toHaveBeenCalledTimes(2);
      expect(callback.mock.calls[1][0]).toHaveLength(1);
      
      unsubscribe();
    });
  });
});

describe('logFilters store', () => {
  beforeEach(() => {
    logFilters.set({
      level: 'all',
      source: 'all',
      search: ''
    });
  });

  describe('initial state', () => {
    it('has correct default values', () => {
      const filters = get(logFilters);
      expect(filters).toEqual({
        level: 'all',
        source: 'all',
        search: ''
      });
    });
  });

  describe('set method', () => {
    it('updates all filter values', () => {
      logFilters.set({
        level: 'error',
        source: 'system',
        search: 'test'
      });
      
      expect(get(logFilters)).toEqual({
        level: 'error',
        source: 'system',
        search: 'test'
      });
    });
  });

  describe('update method', () => {
    it('updates specific filter values', () => {
      logFilters.update(f => ({ ...f, level: 'warn' }));
      
      const filters = get(logFilters);
      expect(filters.level).toBe('warn');
      expect(filters.source).toBe('all');
      expect(filters.search).toBe('');
    });
  });
});

describe('filteredLogs derived store', () => {
  beforeEach(() => {
    logs.clear();
    logFilters.set({
      level: 'all',
      source: 'all',
      search: ''
    });
  });

  it('returns all logs when no filters applied', () => {
    logs.info('source1', 'Message 1');
    logs.error('source2', 'Message 2');
    
    expect(get(filteredLogs)).toHaveLength(2);
  });

  it('filters by level', () => {
    logs.info('test', 'Info message');
    logs.error('test', 'Error message');
    logs.warn('test', 'Warning message');
    
    logFilters.update(f => ({ ...f, level: 'error' }));
    
    const filtered = get(filteredLogs);
    expect(filtered).toHaveLength(1);
    expect(filtered[0].level).toBe('error');
  });

  it('filters by source', () => {
    logs.info('source1', 'Message 1');
    logs.info('source2', 'Message 2');
    logs.info('source1', 'Message 3');
    
    logFilters.update(f => ({ ...f, source: 'source1' }));
    
    const filtered = get(filteredLogs);
    expect(filtered).toHaveLength(2);
    expect(filtered.every(l => l.source === 'source1')).toBe(true);
  });

  it('filters by search text in message', () => {
    logs.info('test', 'Hello world');
    logs.info('test', 'Goodbye world');
    logs.info('test', 'Hello there');
    
    logFilters.update(f => ({ ...f, search: 'hello' }));
    
    const filtered = get(filteredLogs);
    expect(filtered).toHaveLength(2);
  });

  it('filters by search text in source', () => {
    logs.info('system', 'Message 1');
    logs.info('network', 'Message 2');
    logs.info('system-core', 'Message 3');
    
    logFilters.update(f => ({ ...f, search: 'system' }));
    
    const filtered = get(filteredLogs);
    expect(filtered).toHaveLength(2);
  });

  it('combines multiple filters', () => {
    logs.info('system', 'Hello world');
    logs.error('system', 'Hello error');
    logs.info('network', 'Hello network');
    logs.error('network', 'Goodbye error');
    
    logFilters.set({
      level: 'error',
      source: 'all',
      search: 'hello'
    });
    
    const filtered = get(filteredLogs);
    expect(filtered).toHaveLength(1);
    expect(filtered[0].message).toBe('Hello error');
  });
});

describe('logSources derived store', () => {
  beforeEach(() => {
    logs.clear();
  });

  it('returns empty array when no logs', () => {
    expect(get(logSources)).toEqual([]);
  });

  it('returns unique sources sorted alphabetically', () => {
    logs.info('zebra', 'Message');
    logs.info('alpha', 'Message');
    logs.info('beta', 'Message');
    logs.info('alpha', 'Another message');
    
    expect(get(logSources)).toEqual(['alpha', 'beta', 'zebra']);
  });
});
