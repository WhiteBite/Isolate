import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

// Mock crypto.randomUUID
vi.stubGlobal('crypto', {
  randomUUID: () => `test-uuid-${Math.random().toString(36).substring(7)}`
});

// Import after mocking
import { logs, logFilters, filteredLogs, logSources, type LogEntry, type LogLevel } from '../stores/logs';

describe('Logs Store', () => {
  beforeEach(() => {
    logs.clear();
    logFilters.set({
      level: 'all',
      source: 'all',
      search: ''
    });
  });

  describe('logs store', () => {
    it('should start empty', () => {
      expect(get(logs)).toEqual([]);
    });

    it('should add log entry with auto-generated id and timestamp', () => {
      logs.add({ level: 'info', source: 'test', message: 'Test message' });
      
      const entries = get(logs);
      expect(entries).toHaveLength(1);
      expect(entries[0].id).toBeDefined();
      expect(entries[0].timestamp).toBeInstanceOf(Date);
      expect(entries[0].level).toBe('info');
      expect(entries[0].source).toBe('test');
      expect(entries[0].message).toBe('Test message');
    });

    it('should add error log', () => {
      logs.error('error-source', 'Error message');
      
      const entries = get(logs);
      expect(entries).toHaveLength(1);
      expect(entries[0].level).toBe('error');
      expect(entries[0].source).toBe('error-source');
      expect(entries[0].message).toBe('Error message');
    });

    it('should add warn log', () => {
      logs.warn('warn-source', 'Warning message');
      
      const entries = get(logs);
      expect(entries).toHaveLength(1);
      expect(entries[0].level).toBe('warn');
    });

    it('should add info log', () => {
      logs.info('info-source', 'Info message');
      
      const entries = get(logs);
      expect(entries).toHaveLength(1);
      expect(entries[0].level).toBe('info');
    });

    it('should add debug log', () => {
      logs.debug('debug-source', 'Debug message');
      
      const entries = get(logs);
      expect(entries).toHaveLength(1);
      expect(entries[0].level).toBe('debug');
    });

    it('should add success log', () => {
      logs.success('success-source', 'Success message');
      
      const entries = get(logs);
      expect(entries).toHaveLength(1);
      expect(entries[0].level).toBe('success');
    });

    it('should clear all logs', () => {
      logs.info('test', 'Message 1');
      logs.info('test', 'Message 2');
      logs.info('test', 'Message 3');
      
      expect(get(logs)).toHaveLength(3);
      
      logs.clear();
      
      expect(get(logs)).toHaveLength(0);
    });

    it('should maintain order of logs', () => {
      logs.info('test', 'First');
      logs.info('test', 'Second');
      logs.info('test', 'Third');
      
      const entries = get(logs);
      expect(entries[0].message).toBe('First');
      expect(entries[1].message).toBe('Second');
      expect(entries[2].message).toBe('Third');
    });

    it('should limit logs to MAX_LOGS (500)', () => {
      // Add more than 500 logs
      for (let i = 0; i < 550; i++) {
        logs.info('test', `Message ${i}`);
      }
      
      const entries = get(logs);
      expect(entries).toHaveLength(500);
      // Should keep the latest logs
      expect(entries[0].message).toBe('Message 50');
      expect(entries[499].message).toBe('Message 549');
    });
  });

  describe('filteredLogs derived store', () => {
    beforeEach(() => {
      // Add test logs
      logs.error('strategy', 'Strategy error');
      logs.warn('network', 'Network warning');
      logs.info('strategy', 'Strategy info');
      logs.debug('system', 'System debug');
      logs.success('network', 'Network success');
    });

    it('should return all logs when no filters applied', () => {
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(5);
    });

    it('should filter by level', () => {
      logFilters.update(f => ({ ...f, level: 'error' }));
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].level).toBe('error');
    });

    it('should filter by source', () => {
      logFilters.update(f => ({ ...f, source: 'strategy' }));
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(2);
      filtered.forEach(log => {
        expect(log.source).toBe('strategy');
      });
    });

    it('should filter by search text in message', () => {
      logFilters.update(f => ({ ...f, search: 'warning' }));
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].message).toContain('warning');
    });

    it('should filter by search text in source', () => {
      logFilters.update(f => ({ ...f, search: 'network' }));
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(2);
      filtered.forEach(log => {
        expect(log.source).toBe('network');
      });
    });

    it('should be case-insensitive in search', () => {
      logFilters.update(f => ({ ...f, search: 'STRATEGY' }));
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(2);
    });

    it('should combine multiple filters', () => {
      logFilters.update(f => ({ 
        ...f, 
        level: 'info',
        source: 'strategy'
      }));
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(1);
      expect(filtered[0].level).toBe('info');
      expect(filtered[0].source).toBe('strategy');
    });

    it('should return empty array when no matches', () => {
      logFilters.update(f => ({ ...f, search: 'nonexistent' }));
      
      const filtered = get(filteredLogs);
      expect(filtered).toHaveLength(0);
    });
  });

  describe('logSources derived store', () => {
    it('should return empty array when no logs', () => {
      expect(get(logSources)).toEqual([]);
    });

    it('should return unique sources sorted alphabetically', () => {
      logs.info('zebra', 'Message');
      logs.info('alpha', 'Message');
      logs.info('beta', 'Message');
      logs.info('alpha', 'Another message'); // duplicate source
      
      const sources = get(logSources);
      expect(sources).toEqual(['alpha', 'beta', 'zebra']);
    });

    it('should update when logs change', () => {
      logs.info('source1', 'Message');
      expect(get(logSources)).toEqual(['source1']);
      
      logs.info('source2', 'Message');
      expect(get(logSources)).toEqual(['source1', 'source2']);
      
      logs.clear();
      expect(get(logSources)).toEqual([]);
    });
  });
});

describe('LogLevel type', () => {
  it('should accept valid log levels', () => {
    const levels: LogLevel[] = ['error', 'warn', 'info', 'debug', 'success'];
    expect(levels).toHaveLength(5);
  });
});

describe('LogEntry interface', () => {
  it('should have correct structure', () => {
    const entry: LogEntry = {
      id: 'test-id',
      timestamp: new Date(),
      level: 'info',
      source: 'test',
      message: 'Test message'
    };
    
    expect(entry.id).toBe('test-id');
    expect(entry.timestamp).toBeInstanceOf(Date);
    expect(entry.level).toBe('info');
    expect(entry.source).toBe('test');
    expect(entry.message).toBe('Test message');
  });
});
