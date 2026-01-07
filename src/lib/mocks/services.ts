/**
 * Mock data for services
 * Used for browser preview and development
 */

export interface MockService {
  id: string;
  name: string;
  icon: string;
  enabled: boolean;
  status: 'working' | 'blocked' | 'unknown';
  ping?: number;
}

export interface MockServiceItem {
  id: string;
  name: string;
  icon: string;
  description: string;
}

export interface MockServiceHealth {
  id: string;
  name: string;
  status: 'healthy' | 'degraded' | 'down' | 'unknown';
  ping?: number;
}

export interface MockServiceBasic {
  id: string;
  name: string;
  critical: boolean;
}

export interface MockServiceInfo {
  id: string;
  name: string;
  icon: string;
}

/**
 * Dashboard services mock
 */
export const mockDashboardServices: MockService[] = [
  { id: 'youtube', name: 'YouTube', icon: '📺', enabled: true, status: 'working', ping: 45 },
  { id: 'discord', name: 'Discord', icon: '💬', enabled: true, status: 'working', ping: 32 },
  { id: 'telegram', name: 'Telegram', icon: '✈️', enabled: true, status: 'unknown', ping: 120 },
  { id: 'twitch', name: 'Twitch', icon: '🎮', enabled: true, status: 'blocked' }
];

/**
 * Onboarding services mock
 */
export const mockOnboardingServices: MockServiceItem[] = [
  { id: 'youtube', name: 'YouTube', icon: '📺', description: 'Video & streams' },
  { id: 'discord', name: 'Discord', icon: '💬', description: 'Voice & chat' },
  { id: 'telegram', name: 'Telegram', icon: '✈️', description: 'Messenger' },
  { id: 'twitch', name: 'Twitch', icon: '🎮', description: 'Streaming' },
  { id: 'spotify', name: 'Spotify', icon: '🎵', description: 'Music' },
  { id: 'instagram', name: 'Instagram', icon: '📷', description: 'Photos & stories' },
];

/**
 * Health monitor services mock
 */
export const mockHealthServices: MockServiceHealth[] = [
  { id: 'youtube', name: 'YouTube', status: 'healthy', ping: 45 },
  { id: 'discord', name: 'Discord', status: 'healthy', ping: 32 },
  { id: 'telegram', name: 'Telegram', status: 'degraded', ping: 120 },
];

/**
 * Testing page services mock
 */
export const mockTestingServices: MockServiceBasic[] = [
  { id: 'discord', name: 'Discord', critical: true },
  { id: 'youtube', name: 'YouTube', critical: true },
  { id: 'telegram', name: 'Telegram', critical: false },
  { id: 'twitch', name: 'Twitch', critical: false },
  { id: 'spotify', name: 'Spotify', critical: false }
];

/**
 * Orchestra page services mock
 */
export const mockOrchestraServices: MockServiceInfo[] = [
  { id: 'discord', name: 'Discord', icon: '💬' },
  { id: 'youtube', name: 'YouTube', icon: '📺' },
  { id: 'telegram', name: 'Telegram', icon: '✈️' },
  { id: 'twitch', name: 'Twitch', icon: '🎮' },
  { id: 'spotify', name: 'Spotify', icon: '🎵' },
];
