import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  parseSubscriptionContent,
  parseProxyUrl,
  addSubscription,
  updateSubscription,
  removeSubscription,
  getSubscriptions,
  saveSubscriptions,
  type Subscription
} from '../subscription';

// ============================================================================
// Mock localStorage
// ============================================================================

const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { store = {}; })
  };
})();

Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock });

beforeEach(() => {
  localStorageMock.clear();
  vi.clearAllMocks();
});

// ============================================================================
// parseProxyUrl Tests
// ============================================================================

describe('parseProxyUrl', () => {
  describe('VLESS URLs', () => {
    it('should parse valid VLESS URL', () => {
      const url = 'vless://uuid-1234@example.com:443?security=tls&sni=sni.example.com#MyVLESS';
      const result = parseProxyUrl(url);

      expect(result).not.toBeNull();
      expect(result?.protocol).toBe('vless');
      expect(result?.server).toBe('example.com');
      expect(result?.port).toBe(443);
      expect(result?.uuid).toBe('uuid-1234');
      expect(result?.name).toBe('MyVLESS');
      expect(result?.tls).toBe(true);
    });

    it('should parse VLESS URL with reality security', () => {
      const url = 'vless://uuid@example.com:443?security=reality&sni=sni.com&pbk=publickey&sid=shortid#Test';
      const result = parseProxyUrl(url);

      expect(result?.tls).toBe(true);
      expect(result?.custom_fields?.publicKey).toBe('publickey');
      expect(result?.custom_fields?.shortId).toBe('shortid');
    });

    it('should handle VLESS URL without TLS', () => {
      const url = 'vless://uuid@example.com:80?security=none#NoTLS';
      const result = parseProxyUrl(url);

      expect(result?.tls).toBe(false);
    });

    it('should parse transport type', () => {
      const url = 'vless://uuid@example.com:443?type=ws&security=tls#WS';
      const result = parseProxyUrl(url);

      expect(result?.transport).toBe('ws');
    });
  });

  describe('VMess URLs', () => {
    it('should parse valid VMess URL', () => {
      const config = {
        v: '2',
        ps: 'MyVMess',
        add: 'vmess.example.com',
        port: 443,
        id: 'vmess-uuid',
        net: 'ws',
        tls: 'tls',
        sni: 'sni.example.com'
      };
      const url = `vmess://${btoa(JSON.stringify(config))}`;
      const result = parseProxyUrl(url);

      expect(result).not.toBeNull();
      expect(result?.protocol).toBe('vmess');
      expect(result?.server).toBe('vmess.example.com');
      expect(result?.port).toBe(443);
      expect(result?.uuid).toBe('vmess-uuid');
      expect(result?.name).toBe('MyVMess');
      expect(result?.tls).toBe(true);
      expect(result?.transport).toBe('ws');
    });

    it('should handle VMess without TLS', () => {
      const config = {
        v: '2',
        ps: 'NoTLS',
        add: 'example.com',
        port: 80,
        id: 'uuid',
        tls: ''
      };
      const url = `vmess://${btoa(JSON.stringify(config))}`;
      const result = parseProxyUrl(url);

      expect(result?.tls).toBe(false);
    });

    it('should return null for invalid VMess base64', () => {
      const url = 'vmess://invalid-base64!!!';
      const result = parseProxyUrl(url);

      expect(result).toBeNull();
    });
  });

  describe('Shadowsocks URLs', () => {
    it('should parse SIP002 format SS URL', () => {
      const userInfo = btoa('aes-256-gcm:mypassword');
      const url = `ss://${userInfo}@ss.example.com:8388#MySS`;
      const result = parseProxyUrl(url);

      expect(result).not.toBeNull();
      expect(result?.protocol).toBe('shadowsocks');
      expect(result?.server).toBe('ss.example.com');
      expect(result?.port).toBe(8388);
      expect(result?.password).toBe('mypassword');
      expect(result?.name).toBe('MySS');
      expect(result?.custom_fields?.method).toBe('aes-256-gcm');
    });

    it('should parse legacy SS URL format', () => {
      const encoded = btoa('chacha20-ietf-poly1305:password@example.com:443');
      const url = `ss://${encoded}#LegacySS`;
      const result = parseProxyUrl(url);

      expect(result?.protocol).toBe('shadowsocks');
      expect(result?.password).toBe('password');
    });
  });

  describe('Trojan URLs', () => {
    it('should parse valid Trojan URL', () => {
      const url = 'trojan://mypassword@trojan.example.com:443?sni=sni.example.com#MyTrojan';
      const result = parseProxyUrl(url);

      expect(result).not.toBeNull();
      expect(result?.protocol).toBe('trojan');
      expect(result?.server).toBe('trojan.example.com');
      expect(result?.port).toBe(443);
      expect(result?.password).toBe('mypassword');
      expect(result?.name).toBe('MyTrojan');
      expect(result?.tls).toBe(true);
      expect(result?.sni).toBe('sni.example.com');
    });
  });

  describe('Hysteria URLs', () => {
    it('should parse Hysteria2 URL', () => {
      const url = 'hysteria2://authpassword@hy2.example.com:443?sni=sni.com#MyHy2';
      const result = parseProxyUrl(url);

      expect(result).not.toBeNull();
      expect(result?.protocol).toBe('hysteria2');
      expect(result?.server).toBe('hy2.example.com');
      expect(result?.password).toBe('authpassword');
    });

    it('should parse hy2:// shorthand URL', () => {
      const url = 'hy2://password@example.com:443#ShortHy2';
      const result = parseProxyUrl(url);

      expect(result?.protocol).toBe('hysteria2');
    });
  });

  describe('Unknown URLs', () => {
    it('should return null for unsupported protocols', () => {
      expect(parseProxyUrl('http://example.com')).toBeNull();
      expect(parseProxyUrl('unknown://test')).toBeNull();
      expect(parseProxyUrl('not-a-url')).toBeNull();
    });
  });
});

// ============================================================================
// parseSubscriptionContent Tests
// ============================================================================

describe('parseSubscriptionContent', () => {
  describe('Base64 encoded content', () => {
    it('should parse base64 encoded proxy list', () => {
      const proxyUrls = [
        'vless://uuid1@server1.com:443?security=tls#Proxy1',
        'vless://uuid2@server2.com:443?security=tls#Proxy2'
      ].join('\n');
      const encoded = btoa(proxyUrls);

      const result = parseSubscriptionContent(encoded);

      expect(result.success).toBe(true);
      expect(result.proxies).toHaveLength(2);
      expect(result.proxies[0].name).toBe('Proxy1');
      expect(result.proxies[1].name).toBe('Proxy2');
    });

    it('should handle plain text proxy list', () => {
      const content = [
        'vless://uuid@server.com:443?security=tls#Test',
        'trojan://pass@trojan.com:443#Trojan'
      ].join('\n');

      const result = parseSubscriptionContent(content);

      expect(result.success).toBe(true);
      expect(result.proxies).toHaveLength(2);
    });

    it('should skip invalid lines', () => {
      const content = [
        'vless://uuid@server.com:443#Valid',
        'invalid-line',
        'another-invalid',
        'trojan://pass@trojan.com:443#AlsoValid'
      ].join('\n');

      const result = parseSubscriptionContent(content);

      expect(result.success).toBe(true);
      expect(result.proxies).toHaveLength(2);
    });

    it('should return error when no valid proxies found', () => {
      const content = 'no-valid-urls-here\njust-text';

      const result = parseSubscriptionContent(content);

      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
    });
  });

  describe('JSON content', () => {
    it('should parse JSON with proxies array', () => {
      const json = {
        proxies: [
          { type: 'vless', server: 'server1.com', port: 443, uuid: 'uuid1', name: 'Proxy1' },
          { type: 'trojan', server: 'server2.com', port: 443, password: 'pass', name: 'Proxy2' }
        ]
      };

      const result = parseSubscriptionContent(JSON.stringify(json));

      expect(result.success).toBe(true);
      expect(result.proxies).toHaveLength(2);
    });

    it('should parse JSON with servers array', () => {
      const json = {
        servers: [
          { type: 'vmess', address: 'vmess.com', port: 443, id: 'uuid', tag: 'VMess1' }
        ]
      };

      const result = parseSubscriptionContent(JSON.stringify(json));

      expect(result.success).toBe(true);
      expect(result.proxies[0].protocol).toBe('vmess');
    });

    it('should parse JSON with outbounds array', () => {
      const json = {
        outbounds: [
          { type: 'shadowsocks', server: 'ss.com', port: 8388, password: 'pass', method: 'aes-256-gcm' }
        ]
      };

      const result = parseSubscriptionContent(JSON.stringify(json));

      expect(result.success).toBe(true);
      expect(result.proxies[0].protocol).toBe('shadowsocks');
    });

    it('should parse JSON array directly', () => {
      const json = [
        { type: 'vless', server: 'server.com', port: 443, uuid: 'uuid' }
      ];

      const result = parseSubscriptionContent(JSON.stringify(json));

      expect(result.success).toBe(true);
      expect(result.proxies).toHaveLength(1);
    });

    it('should return error for invalid JSON', () => {
      const result = parseSubscriptionContent('{invalid json}');

      expect(result.success).toBe(false);
      expect(result.error).toContain('Invalid JSON');
    });

    it('should return error for empty proxies array', () => {
      const json = { proxies: [] };

      const result = parseSubscriptionContent(JSON.stringify(json));

      expect(result.success).toBe(false);
    });
  });

  describe('Unknown format', () => {
    it('should return error for unknown format', () => {
      const result = parseSubscriptionContent('random text without urls');

      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
    });
  });
});

// ============================================================================
// Subscription Management Tests
// ============================================================================

describe('Subscription Management', () => {
  describe('addSubscription', () => {
    it('should create subscription with unique ID', () => {
      const sub = addSubscription('Test Sub', 'https://example.com/sub');

      expect(sub.id).toMatch(/^sub-\d+-[a-z0-9]+$/);
      expect(sub.name).toBe('Test Sub');
      expect(sub.url).toBe('https://example.com/sub');
      expect(sub.enabled).toBe(true);
      expect(sub.autoUpdate).toBe(true);
      expect(sub.updateInterval).toBe(60);
      expect(sub.lastUpdated).toBeNull();
      expect(sub.proxyCount).toBe(0);
      expect(sub.proxyIds).toEqual([]);
      expect(sub.error).toBeNull();
    });

    it('should use custom autoUpdate and interval', () => {
      const sub = addSubscription('Test', 'https://test.com', false, 120);

      expect(sub.autoUpdate).toBe(false);
      expect(sub.updateInterval).toBe(120);
    });

    it('should persist to localStorage', () => {
      addSubscription('Test', 'https://test.com');

      expect(localStorageMock.setItem).toHaveBeenCalled();
      const stored = getSubscriptions();
      expect(stored).toHaveLength(1);
    });
  });

  describe('getSubscriptions', () => {
    it('should return empty array when no subscriptions', () => {
      const subs = getSubscriptions();
      expect(subs).toEqual([]);
    });

    it('should return stored subscriptions', () => {
      addSubscription('Sub1', 'https://sub1.com');
      addSubscription('Sub2', 'https://sub2.com');

      const subs = getSubscriptions();
      expect(subs).toHaveLength(2);
    });

    it('should handle corrupted localStorage data', () => {
      localStorageMock.getItem.mockReturnValueOnce('invalid-json');

      const subs = getSubscriptions();
      expect(subs).toEqual([]);
    });
  });

  describe('updateSubscription', () => {
    it('should update existing subscription', () => {
      const sub = addSubscription('Original', 'https://original.com');

      const updated = updateSubscription(sub.id, { 
        name: 'Updated', 
        proxyCount: 5 
      });

      expect(updated?.name).toBe('Updated');
      expect(updated?.proxyCount).toBe(5);
      expect(updated?.url).toBe('https://original.com');
    });

    it('should return null for non-existent subscription', () => {
      const result = updateSubscription('non-existent-id', { name: 'Test' });
      expect(result).toBeNull();
    });

    it('should persist changes', () => {
      const sub = addSubscription('Test', 'https://test.com');
      updateSubscription(sub.id, { enabled: false });

      const subs = getSubscriptions();
      expect(subs[0].enabled).toBe(false);
    });
  });

  describe('removeSubscription', () => {
    it('should remove existing subscription', () => {
      const sub = addSubscription('ToRemove', 'https://remove.com');

      const result = removeSubscription(sub.id);

      expect(result).toBe(true);
      expect(getSubscriptions()).toHaveLength(0);
    });

    it('should return false for non-existent subscription', () => {
      const result = removeSubscription('non-existent-id');
      expect(result).toBe(false);
    });

    it('should not affect other subscriptions', () => {
      const sub1 = addSubscription('Keep', 'https://keep.com');
      const sub2 = addSubscription('Remove', 'https://remove.com');

      removeSubscription(sub2.id);

      const subs = getSubscriptions();
      expect(subs).toHaveLength(1);
      expect(subs[0].id).toBe(sub1.id);
    });
  });

  describe('saveSubscriptions', () => {
    it('should save subscriptions to localStorage', () => {
      const subs: Subscription[] = [{
        id: 'test-id',
        name: 'Test',
        url: 'https://test.com',
        enabled: true,
        autoUpdate: true,
        updateInterval: 60,
        lastUpdated: null,
        proxyCount: 0,
        proxyIds: [],
        error: null
      }];

      saveSubscriptions(subs);

      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        'isolate_subscriptions',
        JSON.stringify(subs)
      );
    });
  });
});
