/**
 * Mock data for network rules
 * Used for browser preview and development
 */

import type { NetworkRule } from '$lib/components/network';

/**
 * Network page routing rules mock
 */
export const mockNetworkRules: NetworkRule[] = [
  { 
    id: '1', 
    name: 'YouTube DPI', 
    enabled: true, 
    source: 'domain', 
    sourceValue: 'youtube.com', 
    action: 'dpi-bypass', 
    priority: 1 
  },
  { 
    id: '2', 
    name: 'Discord Proxy', 
    enabled: true, 
    source: 'domain', 
    sourceValue: 'discord.com', 
    action: 'proxy', 
    proxyId: 'vless-1', 
    priority: 2 
  },
  { 
    id: '3', 
    name: 'Telegram', 
    enabled: true, 
    source: 'domain', 
    sourceValue: 'telegram.org', 
    action: 'proxy', 
    proxyId: 'vless-2', 
    priority: 3 
  },
  { 
    id: '4', 
    name: 'Block Ads', 
    enabled: false, 
    source: 'domain', 
    sourceValue: '*.doubleclick.net', 
    action: 'block', 
    priority: 4 
  },
];
