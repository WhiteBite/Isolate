/**
 * Utility functions for generating proxy share URLs
 */

import type { ProxyConfig } from '$lib/api';

/**
 * Generate a shareable URL for a proxy configuration
 */
export function generateProxyUrl(proxy: ProxyConfig): string {
  switch (proxy.protocol) {
    case 'vless':
      return generateVlessUrl(proxy);
    case 'vmess':
      return generateVmessUrl(proxy);
    case 'shadowsocks':
      return generateShadowsocksUrl(proxy);
    case 'trojan':
      return generateTrojanUrl(proxy);
    case 'socks5':
      return generateSocks5Url(proxy);
    case 'http':
    case 'https':
      return generateHttpUrl(proxy);
    default:
      return `${proxy.server}:${proxy.port}`;
  }
}

function generateVlessUrl(proxy: ProxyConfig): string {
  const uuid = proxy.uuid || '';
  const params = new URLSearchParams();
  
  if (proxy.tls) {
    params.set('security', 'tls');
    if (proxy.sni) params.set('sni', proxy.sni);
  } else {
    params.set('security', 'none');
  }
  
  if (proxy.transport && proxy.transport !== 'tcp') {
    params.set('type', proxy.transport);
  }
  
  // Add custom fields
  for (const [key, value] of Object.entries(proxy.custom_fields || {})) {
    if (value) params.set(key, value);
  }
  
  const paramStr = params.toString();
  const fragment = encodeURIComponent(proxy.name);
  
  return `vless://${uuid}@${proxy.server}:${proxy.port}${paramStr ? '?' + paramStr : ''}#${fragment}`;
}

function generateVmessUrl(proxy: ProxyConfig): string {
  // VMess uses base64-encoded JSON
  const config = {
    v: '2',
    ps: proxy.name,
    add: proxy.server,
    port: proxy.port,
    id: proxy.uuid || '',
    aid: '0',
    scy: 'auto',
    net: proxy.transport || 'tcp',
    type: 'none',
    host: proxy.sni || '',
    path: '',
    tls: proxy.tls ? 'tls' : '',
    sni: proxy.sni || '',
    ...proxy.custom_fields
  };
  
  const encoded = btoa(JSON.stringify(config));
  return `vmess://${encoded}`;
}

function generateShadowsocksUrl(proxy: ProxyConfig): string {
  const method = proxy.custom_fields?.method || 'aes-256-gcm';
  const password = proxy.password || '';
  
  // SS URL format: ss://base64(method:password)@server:port#name
  const userInfo = btoa(`${method}:${password}`);
  const fragment = encodeURIComponent(proxy.name);
  
  return `ss://${userInfo}@${proxy.server}:${proxy.port}#${fragment}`;
}

function generateTrojanUrl(proxy: ProxyConfig): string {
  const password = proxy.password || '';
  const params = new URLSearchParams();
  
  if (proxy.sni) params.set('sni', proxy.sni);
  if (proxy.transport && proxy.transport !== 'tcp') {
    params.set('type', proxy.transport);
  }
  
  const paramStr = params.toString();
  const fragment = encodeURIComponent(proxy.name);
  
  return `trojan://${password}@${proxy.server}:${proxy.port}${paramStr ? '?' + paramStr : ''}#${fragment}`;
}

function generateSocks5Url(proxy: ProxyConfig): string {
  const auth = proxy.username && proxy.password 
    ? `${encodeURIComponent(proxy.username)}:${encodeURIComponent(proxy.password)}@` 
    : '';
  
  return `socks5://${auth}${proxy.server}:${proxy.port}`;
}

function generateHttpUrl(proxy: ProxyConfig): string {
  const auth = proxy.username && proxy.password 
    ? `${encodeURIComponent(proxy.username)}:${encodeURIComponent(proxy.password)}@` 
    : '';
  const scheme = proxy.tls ? 'https' : 'http';
  
  return `${scheme}://${auth}${proxy.server}:${proxy.port}`;
}
