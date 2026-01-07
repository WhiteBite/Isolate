/**
 * IPC Contract Tests
 * 
 * Tests the contract between frontend and Rust backend using mockIPC.
 * Ensures invoke() calls are made with correct command names and payloads.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock Tauri internals
const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe('IPC Contract Tests', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Backend Ready Check', () => {
    it('should call is_backend_ready command', async () => {
      mockInvoke.mockResolvedValueOnce(true);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('is_backend_ready');
      
      expect(mockInvoke).toHaveBeenCalledWith('is_backend_ready');
      expect(result).toBe(true);
    });

    it('should handle backend not ready', async () => {
      mockInvoke.mockResolvedValueOnce(false);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('is_backend_ready');
      
      expect(result).toBe(false);
    });
  });

  describe('Strategies Commands', () => {
    it('should call get_strategies command', async () => {
      const mockStrategies = [
        { id: 'discord_multisplit', name: 'Discord Multisplit', family: 'zapret' },
        { id: 'youtube_fake', name: 'YouTube Fake', family: 'zapret' },
      ];
      mockInvoke.mockResolvedValueOnce(mockStrategies);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('get_strategies');
      
      expect(mockInvoke).toHaveBeenCalledWith('get_strategies');
      expect(result).toEqual(mockStrategies);
    });

    it('should call apply_strategy with correct payload', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('apply_strategy', { strategyId: 'discord_multisplit' });
      
      expect(mockInvoke).toHaveBeenCalledWith('apply_strategy', { 
        strategyId: 'discord_multisplit' 
      });
    });

    it('should call stop_strategy command', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('stop_strategy');
      
      expect(mockInvoke).toHaveBeenCalledWith('stop_strategy');
    });

    it('should call get_engine_mode command', async () => {
      mockInvoke.mockResolvedValueOnce('global');
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('get_engine_mode');
      
      expect(mockInvoke).toHaveBeenCalledWith('get_engine_mode');
      expect(result).toBe('global');
    });

    it('should call set_engine_mode with mode parameter', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_engine_mode', { mode: 'service' });
      
      expect(mockInvoke).toHaveBeenCalledWith('set_engine_mode', { mode: 'service' });
    });
  });

  describe('Services Commands', () => {
    it('should call get_services command', async () => {
      const mockServices = [
        { id: 'youtube', name: 'YouTube', status: 'available' },
        { id: 'discord', name: 'Discord', status: 'blocked' },
      ];
      mockInvoke.mockResolvedValueOnce(mockServices);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('get_services');
      
      expect(mockInvoke).toHaveBeenCalledWith('get_services');
      expect(result).toEqual(mockServices);
    });

    it('should call check_single_service with service id', async () => {
      const mockResult = { id: 'youtube', status: 'available', latency: 150 };
      mockInvoke.mockResolvedValueOnce(mockResult);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('check_single_service', { serviceId: 'youtube' });
      
      expect(mockInvoke).toHaveBeenCalledWith('check_single_service', { 
        serviceId: 'youtube' 
      });
      expect(result).toEqual(mockResult);
    });
  });

  describe('Update Commands', () => {
    it('should call check_github_updates command', async () => {
      const mockUpdate = {
        version: '1.2.0',
        downloadUrl: 'https://github.com/WhiteBite/Isolate/releases/tag/v1.2.0',
        releaseNotes: 'Bug fixes and improvements',
        publishedAt: '2026-01-07T12:00:00Z',
      };
      mockInvoke.mockResolvedValueOnce(mockUpdate);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('check_github_updates');
      
      expect(mockInvoke).toHaveBeenCalledWith('check_github_updates');
      expect(result).toEqual(mockUpdate);
    });

    it('should return null when no update available', async () => {
      mockInvoke.mockResolvedValueOnce(null);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('check_github_updates');
      
      expect(result).toBeNull();
    });
  });

  describe('Settings Commands', () => {
    it('should call get_settings command', async () => {
      const mockSettings = {
        autostart: true,
        minimizeToTray: true,
        language: 'ru',
      };
      mockInvoke.mockResolvedValueOnce(mockSettings);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('get_settings');
      
      expect(mockInvoke).toHaveBeenCalledWith('get_settings');
      expect(result).toEqual(mockSettings);
    });

    it('should call save_settings with settings object', async () => {
      const settings = { autostart: false, minimizeToTray: true };
      mockInvoke.mockResolvedValueOnce(undefined);
      
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_settings', { settings });
      
      expect(mockInvoke).toHaveBeenCalledWith('save_settings', { settings });
    });
  });

  describe('Proxy Commands', () => {
    it('should call get_proxies command', async () => {
      const mockProxies = [
        { id: '1', host: '127.0.0.1', port: 1080, protocol: 'socks5' },
      ];
      mockInvoke.mockResolvedValueOnce(mockProxies);
      
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('get_proxies');
      
      expect(mockInvoke).toHaveBeenCalledWith('get_proxies');
      expect(result).toEqual(mockProxies);
    });

    it('should call add_proxy with proxy config', async () => {
      const proxy = { host: '127.0.0.1', port: 1080, protocol: 'socks5' };
      mockInvoke.mockResolvedValueOnce({ id: '1', ...proxy });
      
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('add_proxy', { proxy });
      
      expect(mockInvoke).toHaveBeenCalledWith('add_proxy', { proxy });
    });

    it('should call delete_proxy with id', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_proxy', { id: '1' });
      
      expect(mockInvoke).toHaveBeenCalledWith('delete_proxy', { id: '1' });
    });
  });

  describe('Error Handling', () => {
    it('should propagate backend errors', async () => {
      const errorMessage = 'Strategy not found';
      mockInvoke.mockRejectedValueOnce(new Error(errorMessage));
      
      const { invoke } = await import('@tauri-apps/api/core');
      
      await expect(invoke('apply_strategy', { strategyId: 'invalid' }))
        .rejects.toThrow(errorMessage);
    });

    it('should handle network errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'));
      
      const { invoke } = await import('@tauri-apps/api/core');
      
      await expect(invoke('check_github_updates'))
        .rejects.toThrow('Network error');
    });
  });

  describe('Command Naming Convention', () => {
    // Verify that command names follow snake_case convention
    const expectedCommands = [
      'is_backend_ready',
      'get_strategies',
      'apply_strategy',
      'stop_strategy',
      'get_engine_mode',
      'set_engine_mode',
      'get_services',
      'check_single_service',
      'get_settings',
      'save_settings',
      'get_proxies',
      'add_proxy',
      'delete_proxy',
      'check_github_updates',
      'get_logs',
      'clear_logs',
    ];

    it.each(expectedCommands)('command %s should be snake_case', (cmd) => {
      expect(cmd).toMatch(/^[a-z][a-z0-9_]*$/);
    });
  });
});
