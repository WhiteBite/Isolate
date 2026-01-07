/**
 * Mock data for diagnostics page
 * Used for browser preview and development
 */

export type ComponentStatus = 'healthy' | 'warning' | 'error' | 'unknown' | 'checking';

export type ConflictSeverity = 'critical' | 'high' | 'medium' | 'low';
export type ConflictCategory = 'network_filter' | 'vpn' | 'network_optimization' | 'security' | 'windivert';

export interface MockSystemComponent {
  id: string;
  name: string;
  description: string;
  status: ComponentStatus;
  details: string;
  icon: string;
}

export interface MockSystemInfo {
  os: string;
  osVersion: string;
  arch: string;
  memory: string;
  adminRights: boolean;
}

export interface MockConflictInfo {
  name: string;
  category: ConflictCategory;
  severity: ConflictSeverity;
  description: string;
  recommendation: string;
  detected_processes: string[];
  detected_services: string[];
}

/**
 * Diagnostics page components mock
 */
export const mockDiagnosticsComponents: MockSystemComponent[] = [
  { id: 'windivert', name: 'WinDivert', description: 'Kernel-level packet filter driver', status: 'unknown', details: 'Not checked', icon: '🔧' },
  { id: 'singbox', name: 'Sing-box', description: 'Universal proxy platform', status: 'unknown', details: 'Not checked', icon: '📦' },
  { id: 'winws', name: 'WinWS', description: 'DPI bypass tool (Zapret)', status: 'unknown', details: 'Not checked', icon: '⚡' },
  { id: 'network', name: 'Network', description: 'Internet connectivity', status: 'unknown', details: 'Not checked', icon: '🌐' },
  { id: 'dns', name: 'DNS', description: 'Domain name resolution', status: 'unknown', details: 'Not checked', icon: '🔍' },
  { id: 'firewall', name: 'Firewall', description: 'Windows Firewall status', status: 'unknown', details: 'Not checked', icon: '🛡️' },
  { id: 'tcp_timestamps', name: 'TCP Timestamps', description: 'RFC 1323 timestamps for DPI bypass', status: 'unknown', details: 'Not checked', icon: '⏱️' },
];

/**
 * Diagnostics page system info mock (browser preview)
 */
export const mockSystemInfo: MockSystemInfo = {
  os: 'Windows',
  osVersion: '11 Pro (22H2)',
  arch: 'x64',
  memory: '16 GB',
  adminRights: true
};

/**
 * Simulated diagnostics check results
 */
export const mockDiagnosticsResults = [
  { id: 'network', delay: 300, status: 'healthy' as ComponentStatus, details: 'Connected (45ms latency)' },
  { id: 'dns', delay: 400, status: 'healthy' as ComponentStatus, details: 'Resolving correctly' },
  { id: 'windivert', delay: 600, status: 'healthy' as ComponentStatus, details: 'Driver loaded (v2.2)' },
  { id: 'winws', delay: 500, status: 'healthy' as ComponentStatus, details: 'Binary found' },
  { id: 'singbox', delay: 700, status: 'warning' as ComponentStatus, details: 'Not configured' },
  { id: 'firewall', delay: 400, status: 'healthy' as ComponentStatus, details: 'Rules configured' },
  { id: 'tcp_timestamps', delay: 350, status: 'warning' as ComponentStatus, details: 'Disabled (enable for better DPI bypass)' },
];

/**
 * Mock conflict data for demo mode
 */
export const mockConflicts: MockConflictInfo[] = [
  {
    name: 'NordVPN',
    category: 'vpn',
    severity: 'medium',
    description: 'NordVPN tunnels traffic, which may bypass winws',
    recommendation: 'Use split tunneling in NordVPN or disable it for target services',
    detected_processes: ['NordVPN.exe'],
    detected_services: ['nordvpn-service'],
  },
];
