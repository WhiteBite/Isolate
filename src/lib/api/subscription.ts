/**
 * Subscription URL support for proxy management.
 * Supports base64-encoded lists and JSON subscription formats.
 */

import type { ProxyConfig, ProxyProtocol } from './types';

// ============================================================================
// Types
// ============================================================================

export interface Subscription {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  autoUpdate: boolean;
  updateInterval: number; // minutes
  lastUpdated: string | null;
  proxyCount: number;
  proxyIds: string[];
  error: string | null;
}

export interface SubscriptionParseResult {
  success: boolean;
  proxies: ProxyConfig[];
  error?: string;
}

export interface JsonSubscription {
  proxies?: JsonProxy[];
  servers?: JsonProxy[];
  outbounds?: JsonProxy[];
}

interface JsonProxy {
  name?: string;
  tag?: string;
  type?: string;
  protocol?: string;
  server?: string;
  address?: string;
  port?: number;
  uuid?: string;
  id?: string;
  password?: string;
  method?: string;
  cipher?: string;
  tls?: boolean | string;
  sni?: string;
  network?: string;
  transport?: string;
  flow?: string;
  [key: string]: unknown;
}

// ============================================================================
// Storage Keys
// ============================================================================

const STORAGE_KEY = 'isolate_subscriptions';

// ============================================================================
// Subscription Management
// ============================================================================

/**
 * Get all subscriptions from localStorage.
 */
export function getSubscriptions(): Subscription[] {
  if (typeof localStorage === 'undefined') return [];
  const data = localStorage.getItem(STORAGE_KEY);
  if (!data) return [];
  try {
    return JSON.parse(data);
  } catch {
    return [];
  }
}

/**
 * Save subscriptions to localStorage.
 */
export function saveSubscriptions(subscriptions: Subscription[]): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(subscriptions));
}

/**
 * Add a new subscription.
 */
export function addSubscription(name: string, url: string, autoUpdate = true, updateInterval = 60): Subscription {
  const subscription: Subscription = {
    id: `sub-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    name,
    url,
    enabled: true,
    autoUpdate,
    updateInterval,
    lastUpdated: null,
    proxyCount: 0,
    proxyIds: [],
    error: null
  };
  
  const subscriptions = getSubscriptions();
  subscriptions.push(subscription);
  saveSubscriptions(subscriptions);
  
  return subscription;
}

/**
 * Update a subscription.
 */
export function updateSubscription(id: string, updates: Partial<Subscription>): Subscription | null {
  const subscriptions = getSubscriptions();
  const index = subscriptions.findIndex(s => s.id === id);
  if (index === -1) return null;
  
  subscriptions[index] = { ...subscriptions[index], ...updates };
  saveSubscriptions(subscriptions);
  
  return subscriptions[index];
}

/**
 * Remove a subscription.
 */
export function removeSubscription(id: string): boolean {
  const subscriptions = getSubscriptions();
  const filtered = subscriptions.filter(s => s.id !== id);
  if (filtered.length === subscriptions.length) return false;
  
  saveSubscriptions(filtered);
  return true;
}

// ============================================================================
// Subscription Fetching & Parsing
// ============================================================================

/**
 * Fetch and parse subscription URL.
 */
export async function fetchSubscription(url: string): Promise<SubscriptionParseResult> {
  try {
    // Check if running in Tauri
    const isTauri = typeof window !== 'undefined' && 
      ('__TAURI__' in window || '__TAURI_INTERNALS__' in window);
    
    let content: string;
    
    if (isTauri) {
      // Use Tauri's HTTP client for CORS-free requests
      const { fetch: tauriFetch } = await import('@tauri-apps/plugin-http');
      const response = await tauriFetch(url, {
        method: 'GET',
        headers: {
          'User-Agent': 'Isolate/1.0'
        }
      });
      
      if (!response.ok) {
        return { success: false, proxies: [], error: `HTTP ${response.status}: ${response.statusText}` };
      }
      
      content = await response.text();
    } else {
      // Browser mode - try direct fetch (may fail due to CORS)
      const response = await fetch(url, {
        headers: {
          'User-Agent': 'Isolate/1.0'
        }
      });
      
      if (!response.ok) {
        return { success: false, proxies: [], error: `HTTP ${response.status}: ${response.statusText}` };
      }
      
      content = await response.text();
    }
    
    return parseSubscriptionContent(content);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { success: false, proxies: [], error: message };
  }
}

/**
 * Parse subscription content (base64 or JSON).
 */
export function parseSubscriptionContent(content: string): SubscriptionParseResult {
  const trimmed = content.trim();
  
  // Try JSON first
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    return parseJsonSubscription(trimmed);
  }
  
  // Try base64 decode
  try {
    const decoded = atob(trimmed);
    // Check if decoded content looks like proxy URLs
    if (decoded.includes('://')) {
      return parseBase64Subscription(decoded);
    }
  } catch {
    // Not valid base64, try as plain text
  }
  
  // Try as plain text (line-separated URLs)
  if (trimmed.includes('://')) {
    return parseBase64Subscription(trimmed);
  }
  
  return { success: false, proxies: [], error: 'Unknown subscription format' };
}

/**
 * Parse base64-encoded subscription (line-separated proxy URLs).
 */
function parseBase64Subscription(content: string): SubscriptionParseResult {
  const lines = content.split('\n')
    .map(l => l.trim())
    .filter(l => l && l.includes('://'));
  
  if (lines.length === 0) {
    return { success: false, proxies: [], error: 'No valid proxy URLs found' };
  }
  
  const proxies: ProxyConfig[] = [];
  const errors: string[] = [];
  
  for (const line of lines) {
    try {
      const proxy = parseProxyUrl(line);
      if (proxy) {
        proxies.push(proxy);
      }
    } catch (e) {
      errors.push(`Failed to parse: ${line.slice(0, 50)}...`);
    }
  }
  
  if (proxies.length === 0) {
    return { success: false, proxies: [], error: errors.join('; ') || 'No valid proxies parsed' };
  }
  
  return { success: true, proxies };
}

/**
 * Parse JSON subscription format.
 */
function parseJsonSubscription(content: string): SubscriptionParseResult {
  try {
    const data = JSON.parse(content) as JsonSubscription | JsonProxy[];
    
    let proxyList: JsonProxy[] = [];
    
    if (Array.isArray(data)) {
      proxyList = data;
    } else if (data.proxies) {
      proxyList = data.proxies;
    } else if (data.servers) {
      proxyList = data.servers;
    } else if (data.outbounds) {
      proxyList = data.outbounds;
    }
    
    if (proxyList.length === 0) {
      return { success: false, proxies: [], error: 'No proxies found in JSON' };
    }
    
    const proxies: ProxyConfig[] = [];
    
    for (const item of proxyList) {
      const proxy = parseJsonProxy(item);
      if (proxy) {
        proxies.push(proxy);
      }
    }
    
    if (proxies.length === 0) {
      return { success: false, proxies: [], error: 'Failed to parse any proxies from JSON' };
    }
    
    return { success: true, proxies };
  } catch (e) {
    return { success: false, proxies: [], error: 'Invalid JSON format' };
  }
}

/**
 * Parse a single JSON proxy object.
 */
function parseJsonProxy(item: JsonProxy): ProxyConfig | null {
  const type = (item.type || item.protocol || '').toLowerCase();
  const server = item.server || item.address || '';
  const port = item.port || 443;
  const name = item.name || item.tag || `${type}@${server}:${port}`;
  
  if (!server) return null;
  
  const protocolMap: Record<string, ProxyProtocol> = {
    'vless': 'vless',
    'vmess': 'vmess',
    'ss': 'shadowsocks',
    'shadowsocks': 'shadowsocks',
    'trojan': 'trojan',
    'hysteria': 'hysteria',
    'hysteria2': 'hysteria2',
    'hy2': 'hysteria2',
    'socks': 'socks5',
    'socks5': 'socks5',
    'http': 'http',
    'https': 'https'
  };
  
  const protocol = protocolMap[type];
  if (!protocol) return null;
  
  const tls = item.tls === true || item.tls === 'true' || item.tls === 'tls';
  
  return {
    id: `proxy-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    name,
    protocol,
    server,
    port,
    uuid: item.uuid || item.id,
    password: item.password,
    tls,
    sni: item.sni,
    transport: item.network || item.transport,
    custom_fields: {},
    active: false
  };
}

// ============================================================================
// Proxy URL Parsing
// ============================================================================

/**
 * Parse a proxy URL (vless://, vmess://, ss://, trojan://, etc.)
 */
export function parseProxyUrl(url: string): ProxyConfig | null {
  const trimmed = url.trim();
  
  if (trimmed.startsWith('vless://')) {
    return parseVlessUrl(trimmed);
  } else if (trimmed.startsWith('vmess://')) {
    return parseVmessUrl(trimmed);
  } else if (trimmed.startsWith('ss://')) {
    return parseShadowsocksUrl(trimmed);
  } else if (trimmed.startsWith('trojan://')) {
    return parseTrojanUrl(trimmed);
  } else if (trimmed.startsWith('hysteria://') || trimmed.startsWith('hysteria2://') || trimmed.startsWith('hy2://')) {
    return parseHysteriaUrl(trimmed);
  }
  
  return null;
}

/**
 * Parse VLESS URL.
 * Format: vless://uuid@server:port?params#name
 */
function parseVlessUrl(url: string): ProxyConfig | null {
  try {
    const parsed = new URL(url);
    const uuid = parsed.username;
    const server = parsed.hostname;
    const port = parseInt(parsed.port) || 443;
    const name = decodeURIComponent(parsed.hash.slice(1)) || `VLESS@${server}`;
    
    const params = new URLSearchParams(parsed.search);
    const security = params.get('security') || 'none';
    const sni = params.get('sni') || params.get('host') || server;
    const transport = params.get('type') || 'tcp';
    const flow = params.get('flow') || undefined;
    
    return {
      id: `proxy-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      name,
      protocol: 'vless',
      server,
      port,
      uuid,
      tls: security === 'tls' || security === 'reality',
      sni,
      transport,
      custom_fields: {
        ...(flow && { flow }),
        ...(params.get('fp') && { fingerprint: params.get('fp')! }),
        ...(params.get('pbk') && { publicKey: params.get('pbk')! }),
        ...(params.get('sid') && { shortId: params.get('sid')! })
      },
      active: false
    };
  } catch {
    return null;
  }
}

/**
 * Parse VMess URL.
 * Format: vmess://base64(json)
 */
function parseVmessUrl(url: string): ProxyConfig | null {
  try {
    const base64 = url.replace('vmess://', '');
    const json = JSON.parse(atob(base64));
    
    return {
      id: `proxy-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      name: json.ps || json.name || `VMess@${json.add}`,
      protocol: 'vmess',
      server: json.add || json.address,
      port: parseInt(json.port) || 443,
      uuid: json.id || json.uuid,
      tls: json.tls === 'tls' || json.tls === true,
      sni: json.sni || json.host,
      transport: json.net || json.network || 'tcp',
      custom_fields: {
        ...(json.aid && { alterId: String(json.aid) }),
        ...(json.path && { path: json.path }),
        ...(json.host && { host: json.host })
      },
      active: false
    };
  } catch {
    return null;
  }
}

/**
 * Parse Shadowsocks URL.
 * Format: ss://base64(method:password)@server:port#name
 * Or: ss://base64(method:password@server:port)#name
 */
function parseShadowsocksUrl(url: string): ProxyConfig | null {
  try {
    // Remove ss:// prefix
    let rest = url.slice(5);
    
    // Extract name from hash
    let name = '';
    const hashIndex = rest.indexOf('#');
    if (hashIndex !== -1) {
      name = decodeURIComponent(rest.slice(hashIndex + 1));
      rest = rest.slice(0, hashIndex);
    }
    
    // Try SIP002 format: ss://base64@server:port
    const atIndex = rest.lastIndexOf('@');
    if (atIndex !== -1) {
      const userInfo = rest.slice(0, atIndex);
      const serverPart = rest.slice(atIndex + 1);
      
      // Decode userinfo
      let method: string, password: string;
      try {
        const decoded = atob(userInfo);
        const colonIndex = decoded.indexOf(':');
        method = decoded.slice(0, colonIndex);
        password = decoded.slice(colonIndex + 1);
      } catch {
        // Maybe it's already decoded
        const colonIndex = userInfo.indexOf(':');
        method = userInfo.slice(0, colonIndex);
        password = userInfo.slice(colonIndex + 1);
      }
      
      // Parse server:port
      const colonIndex = serverPart.lastIndexOf(':');
      const server = serverPart.slice(0, colonIndex);
      const port = parseInt(serverPart.slice(colonIndex + 1)) || 443;
      
      return {
        id: `proxy-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        name: name || `SS@${server}`,
        protocol: 'shadowsocks',
        server,
        port,
        password,
        tls: false,
        custom_fields: { method },
        active: false
      };
    }
    
    // Try legacy format: ss://base64(method:password@server:port)
    const decoded = atob(rest);
    const match = decoded.match(/^(.+?):(.+?)@(.+?):(\d+)$/);
    if (match) {
      const [, method, password, server, portStr] = match;
      return {
        id: `proxy-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        name: name || `SS@${server}`,
        protocol: 'shadowsocks',
        server,
        port: parseInt(portStr) || 443,
        password,
        tls: false,
        custom_fields: { method },
        active: false
      };
    }
    
    return null;
  } catch {
    return null;
  }
}

/**
 * Parse Trojan URL.
 * Format: trojan://password@server:port?params#name
 */
function parseTrojanUrl(url: string): ProxyConfig | null {
  try {
    const parsed = new URL(url);
    const password = parsed.username;
    const server = parsed.hostname;
    const port = parseInt(parsed.port) || 443;
    const name = decodeURIComponent(parsed.hash.slice(1)) || `Trojan@${server}`;
    
    const params = new URLSearchParams(parsed.search);
    const sni = params.get('sni') || params.get('host') || server;
    
    return {
      id: `proxy-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      name,
      protocol: 'trojan',
      server,
      port,
      password,
      tls: true,
      sni,
      custom_fields: {},
      active: false
    };
  } catch {
    return null;
  }
}

/**
 * Parse Hysteria/Hysteria2 URL.
 */
function parseHysteriaUrl(url: string): ProxyConfig | null {
  try {
    const isHy2 = url.startsWith('hysteria2://') || url.startsWith('hy2://');
    const parsed = new URL(url);
    const password = parsed.username || parsed.searchParams.get('auth');
    const server = parsed.hostname;
    const port = parseInt(parsed.port) || 443;
    const name = decodeURIComponent(parsed.hash.slice(1)) || `Hysteria${isHy2 ? '2' : ''}@${server}`;
    
    const params = new URLSearchParams(parsed.search);
    const sni = params.get('sni') || server;
    
    return {
      id: `proxy-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      name,
      protocol: isHy2 ? 'hysteria2' : 'hysteria',
      server,
      port,
      password: password || undefined,
      tls: true,
      sni,
      custom_fields: {
        ...(params.get('obfs') && { obfs: params.get('obfs')! }),
        ...(params.get('obfs-password') && { obfsPassword: params.get('obfs-password')! })
      },
      active: false
    };
  } catch {
    return null;
  }
}
