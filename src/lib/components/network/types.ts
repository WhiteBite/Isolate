export interface NetworkRule {
  id: string;
  name: string;
  enabled: boolean;
  source: 'domain' | 'app' | 'ip';
  sourceValue: string; // "youtube.com", "chrome.exe", "192.168.1.0/24"
  action: 'direct' | 'proxy' | 'block' | 'dpi-bypass';
  proxyId?: string; // ID gateway если action === 'proxy'
  priority: number;
}

export type CaptureMode = 'system' | 'tun';

export interface ProxyConfig {
  id: string;
  name: string;
  protocol: string;
}
