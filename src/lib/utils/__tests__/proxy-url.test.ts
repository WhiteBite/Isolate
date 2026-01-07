import { describe, it, expect } from 'vitest';
import { generateProxyUrl } from '../proxy-url';
import type { ProxyConfig } from '$lib/api';

// ============================================================================
// Test Helpers
// ============================================================================

function createProxy(overrides: Partial<ProxyConfig>): ProxyConfig {
  return {
    id: 'test-proxy',
    name: 'Test Proxy',
    protocol: 'vless',
    server: 'example.com',
    port: 443,
    tls: true,
    custom_fields: {},
    active: false,
    ...overrides
  };
}

// ============================================================================
// VLESS URL Tests
// ============================================================================

describe('generateProxyUrl - VLESS', () => {
  it('should generate valid VLESS URL with basic config', () => {
    const proxy = createProxy({
      protocol: 'vless',
      uuid: '12345678-1234-1234-1234-123456789abc',
      server: 'test.example.com',
      port: 443,
      name: 'My VLESS'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toMatch(/^vless:\/\//);
    expect(url).toContain('12345678-1234-1234-1234-123456789abc@test.example.com:443');
    expect(url).toContain('#My%20VLESS');
  });

  it('should include TLS parameters when enabled', () => {
    const proxy = createProxy({
      protocol: 'vless',
      uuid: 'test-uuid',
      tls: true,
      sni: 'sni.example.com'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toContain('security=tls');
    expect(url).toContain('sni=sni.example.com');
  });

  it('should set security=none when TLS is disabled', () => {
    const proxy = createProxy({
      protocol: 'vless',
      uuid: 'test-uuid',
      tls: false
    });

    const url = generateProxyUrl(proxy);

    expect(url).toContain('security=none');
  });

  it('should include transport type when not TCP', () => {
    const proxy = createProxy({
      protocol: 'vless',
      uuid: 'test-uuid',
      transport: 'ws'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toContain('type=ws');
  });

  it('should include custom fields in URL params', () => {
    const proxy = createProxy({
      protocol: 'vless',
      uuid: 'test-uuid',
      custom_fields: {
        flow: 'xtls-rprx-vision',
        fp: 'chrome'
      }
    });

    const url = generateProxyUrl(proxy);

    expect(url).toContain('flow=xtls-rprx-vision');
    expect(url).toContain('fp=chrome');
  });
});

// ============================================================================
// VMess URL Tests
// ============================================================================

describe('generateProxyUrl - VMess', () => {
  it('should generate valid VMess URL with base64 encoding', () => {
    const proxy = createProxy({
      protocol: 'vmess',
      uuid: 'test-uuid',
      server: 'vmess.example.com',
      port: 443,
      name: 'My VMess'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toMatch(/^vmess:\/\//);
    // VMess URL should be base64 encoded
    const base64Part = url.replace('vmess://', '');
    expect(() => atob(base64Part)).not.toThrow();
  });

  it('should include all config fields in VMess JSON', () => {
    const proxy = createProxy({
      protocol: 'vmess',
      uuid: 'test-uuid',
      server: 'vmess.example.com',
      port: 8443,
      name: 'Test VMess',
      tls: true,
      sni: 'sni.example.com',
      transport: 'ws'
    });

    const url = generateProxyUrl(proxy);
    const base64Part = url.replace('vmess://', '');
    const decoded = JSON.parse(atob(base64Part));

    expect(decoded.ps).toBe('Test VMess');
    expect(decoded.add).toBe('vmess.example.com');
    expect(decoded.port).toBe(8443);
    expect(decoded.id).toBe('test-uuid');
    expect(decoded.tls).toBe('tls');
    expect(decoded.net).toBe('ws');
  });

  it('should handle VMess without TLS', () => {
    const proxy = createProxy({
      protocol: 'vmess',
      uuid: 'test-uuid',
      tls: false
    });

    const url = generateProxyUrl(proxy);
    const base64Part = url.replace('vmess://', '');
    const decoded = JSON.parse(atob(base64Part));

    expect(decoded.tls).toBe('');
  });
});

// ============================================================================
// Shadowsocks URL Tests
// ============================================================================

describe('generateProxyUrl - Shadowsocks', () => {
  it('should generate valid Shadowsocks URL', () => {
    const proxy = createProxy({
      protocol: 'shadowsocks',
      server: 'ss.example.com',
      port: 8388,
      password: 'mypassword',
      name: 'My SS',
      custom_fields: { method: 'aes-256-gcm' }
    });

    const url = generateProxyUrl(proxy);

    expect(url).toMatch(/^ss:\/\//);
    expect(url).toContain('@ss.example.com:8388');
    expect(url).toContain('#My%20SS');
  });

  it('should use default method if not specified', () => {
    const proxy = createProxy({
      protocol: 'shadowsocks',
      password: 'test',
      custom_fields: {}
    });

    const url = generateProxyUrl(proxy);
    // Should contain base64 encoded method:password
    expect(url).toMatch(/^ss:\/\/[A-Za-z0-9+/=]+@/);
  });

  it('should base64 encode userinfo correctly', () => {
    const proxy = createProxy({
      protocol: 'shadowsocks',
      password: 'testpass',
      custom_fields: { method: 'chacha20-ietf-poly1305' }
    });

    const url = generateProxyUrl(proxy);
    const userInfo = url.match(/ss:\/\/([^@]+)@/)?.[1];
    expect(userInfo).toBeDefined();
    
    const decoded = atob(userInfo!);
    expect(decoded).toBe('chacha20-ietf-poly1305:testpass');
  });
});

// ============================================================================
// Trojan URL Tests
// ============================================================================

describe('generateProxyUrl - Trojan', () => {
  it('should generate valid Trojan URL', () => {
    const proxy = createProxy({
      protocol: 'trojan',
      server: 'trojan.example.com',
      port: 443,
      password: 'trojan-password',
      name: 'My Trojan'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toMatch(/^trojan:\/\//);
    expect(url).toContain('trojan-password@trojan.example.com:443');
    expect(url).toContain('#My%20Trojan');
  });

  it('should include SNI parameter', () => {
    const proxy = createProxy({
      protocol: 'trojan',
      password: 'pass',
      sni: 'sni.example.com'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toContain('sni=sni.example.com');
  });

  it('should include transport type when not TCP', () => {
    const proxy = createProxy({
      protocol: 'trojan',
      password: 'pass',
      transport: 'grpc'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toContain('type=grpc');
  });
});

// ============================================================================
// SOCKS5 URL Tests
// ============================================================================

describe('generateProxyUrl - SOCKS5', () => {
  it('should generate valid SOCKS5 URL without auth', () => {
    const proxy = createProxy({
      protocol: 'socks5',
      server: 'socks.example.com',
      port: 1080
    });

    const url = generateProxyUrl(proxy);

    expect(url).toBe('socks5://socks.example.com:1080');
  });

  it('should include auth credentials when provided', () => {
    const proxy = createProxy({
      protocol: 'socks5',
      server: 'socks.example.com',
      port: 1080,
      username: 'user',
      password: 'pass'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toBe('socks5://user:pass@socks.example.com:1080');
  });

  it('should URL-encode special characters in credentials', () => {
    const proxy = createProxy({
      protocol: 'socks5',
      server: 'socks.example.com',
      port: 1080,
      username: 'user@domain',
      password: 'p@ss:word'
    });

    const url = generateProxyUrl(proxy);

    expect(url).toContain('user%40domain');
    expect(url).toContain('p%40ss%3Aword');
  });
});

// ============================================================================
// HTTP/HTTPS URL Tests
// ============================================================================

describe('generateProxyUrl - HTTP/HTTPS', () => {
  it('should generate HTTP URL without TLS', () => {
    const proxy = createProxy({
      protocol: 'http',
      server: 'http.example.com',
      port: 8080,
      tls: false
    });

    const url = generateProxyUrl(proxy);

    expect(url).toBe('http://http.example.com:8080');
  });

  it('should generate HTTPS URL with TLS', () => {
    const proxy = createProxy({
      protocol: 'https',
      server: 'https.example.com',
      port: 8443,
      tls: true
    });

    const url = generateProxyUrl(proxy);

    expect(url).toBe('https://https.example.com:8443');
  });

  it('should include auth for HTTP proxy', () => {
    const proxy = createProxy({
      protocol: 'http',
      server: 'http.example.com',
      port: 8080,
      username: 'admin',
      password: 'secret',
      tls: false
    });

    const url = generateProxyUrl(proxy);

    expect(url).toBe('http://admin:secret@http.example.com:8080');
  });
});

// ============================================================================
// Default/Unknown Protocol Tests
// ============================================================================

describe('generateProxyUrl - Unknown Protocol', () => {
  it('should return server:port for unknown protocols', () => {
    const proxy = createProxy({
      protocol: 'unknown' as any,
      server: 'unknown.example.com',
      port: 12345
    });

    const url = generateProxyUrl(proxy);

    expect(url).toBe('unknown.example.com:12345');
  });
});

// ============================================================================
// Edge Cases
// ============================================================================

describe('generateProxyUrl - Edge Cases', () => {
  it('should handle empty name', () => {
    const proxy = createProxy({
      protocol: 'vless',
      uuid: 'test-uuid',
      name: ''
    });

    const url = generateProxyUrl(proxy);

    expect(url).toMatch(/^vless:\/\//);
    expect(url).toContain('#');
  });

  it('should handle special characters in name', () => {
    const proxy = createProxy({
      protocol: 'vless',
      uuid: 'test-uuid',
      name: 'Test & Proxy <special>'
    });

    const url = generateProxyUrl(proxy);

    // Name should be URL encoded
    expect(url).toContain(encodeURIComponent('Test & Proxy <special>'));
  });

  it('should handle IPv6 server addresses', () => {
    const proxy = createProxy({
      protocol: 'socks5',
      server: '::1',
      port: 1080
    });

    const url = generateProxyUrl(proxy);

    expect(url).toContain('::1:1080');
  });

  it('should handle missing optional fields', () => {
    const proxy: ProxyConfig = {
      id: 'minimal',
      name: 'Minimal',
      protocol: 'vless',
      server: 'test.com',
      port: 443,
      tls: false,
      custom_fields: {},
      active: false
    };

    const url = generateProxyUrl(proxy);

    expect(url).toMatch(/^vless:\/\//);
  });
});
