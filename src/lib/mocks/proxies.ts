/**
 * Mock data for proxies/gateways
 * Used for browser preview and development
 */

import type { ProxyConfig } from '$lib/api';

/**
 * Network page gateways mock
 */
export const mockGateways: ProxyConfig[] = [
  { 
    id: 'vless-1', 
    name: 'VLESS Germany', 
    protocol: 'vless', 
    server: '185.232.205.172', 
    port: 443, 
    tls: true, 
    active: false, 
    custom_fields: {}, 
    country: 'DE' 
  },
  { 
    id: 'vless-2', 
    name: 'VLESS Netherlands', 
    protocol: 'vless', 
    server: '45.89.55.12', 
    port: 443, 
    tls: true, 
    active: false, 
    custom_fields: {}, 
    country: 'NL' 
  },
  { 
    id: 'ss-1', 
    name: 'Shadowsocks US', 
    protocol: 'shadowsocks', 
    server: '104.21.45.67', 
    port: 8388, 
    tls: false, 
    active: false, 
    custom_fields: {}, 
    country: 'US' 
  },
];
