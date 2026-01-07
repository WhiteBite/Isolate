/**
 * Unit tests for countries.ts utilities
 * @vitest-environment happy-dom
 */

import { describe, it, expect } from 'vitest';
import {
  getCountryFlag,
  getCountryName,
  detectCountryFromServer,
  getProxyFlag,
  getProxyCountryName,
  countryFlags,
  countryNames
} from './countries';

describe('getCountryFlag', () => {
  it('should return correct flag for valid country code', () => {
    expect(getCountryFlag('US')).toBe('🇺🇸');
    expect(getCountryFlag('DE')).toBe('🇩🇪');
    expect(getCountryFlag('JP')).toBe('🇯🇵');
    expect(getCountryFlag('RU')).toBe('🇷🇺');
  });

  it('should be case-insensitive', () => {
    expect(getCountryFlag('us')).toBe('🇺🇸');
    expect(getCountryFlag('Us')).toBe('🇺🇸');
    expect(getCountryFlag('uS')).toBe('🇺🇸');
  });

  it('should return globe emoji for null/undefined', () => {
    expect(getCountryFlag(null)).toBe('🌐');
    expect(getCountryFlag(undefined)).toBe('🌐');
  });

  it('should return globe emoji for unknown country code', () => {
    expect(getCountryFlag('XX')).toBe('🌐');
    expect(getCountryFlag('ZZ')).toBe('🌐');
    expect(getCountryFlag('')).toBe('🌐');
  });
});

describe('getCountryName', () => {
  it('should return correct name for valid country code', () => {
    expect(getCountryName('US')).toBe('United States');
    expect(getCountryName('DE')).toBe('Germany');
    expect(getCountryName('JP')).toBe('Japan');
    expect(getCountryName('RU')).toBe('Russia');
  });

  it('should be case-insensitive', () => {
    expect(getCountryName('us')).toBe('United States');
    expect(getCountryName('de')).toBe('Germany');
  });

  it('should return "Unknown" for null/undefined', () => {
    expect(getCountryName(null)).toBe('Unknown');
    expect(getCountryName(undefined)).toBe('Unknown');
  });

  it('should return uppercase code for unknown country', () => {
    expect(getCountryName('xx')).toBe('XX');
    expect(getCountryName('zz')).toBe('ZZ');
  });
});

describe('detectCountryFromServer', () => {
  it('should return null for null/undefined input', () => {
    expect(detectCountryFromServer(null)).toBeNull();
    expect(detectCountryFromServer(undefined)).toBeNull();
  });

  it('should return null for IP addresses', () => {
    expect(detectCountryFromServer('192.168.1.1')).toBeNull();
    expect(detectCountryFromServer('8.8.8.8')).toBeNull();
    expect(detectCountryFromServer('10.0.0.1')).toBeNull();
  });

  it('should detect country from TLD', () => {
    expect(detectCountryFromServer('example.ru')).toBe('RU');
    expect(detectCountryFromServer('server.de')).toBe('DE');
    expect(detectCountryFromServer('proxy.jp')).toBe('JP');
    expect(detectCountryFromServer('host.nl')).toBe('NL');
  });

  it('should detect country from known providers', () => {
    expect(detectCountryFromServer('server.hetzner.com')).toBe('DE');
    expect(detectCountryFromServer('vps.digitalocean.com')).toBe('US');
    expect(detectCountryFromServer('host.ovh.net')).toBe('FR');
    expect(detectCountryFromServer('selectel.ru')).toBe('RU');
  });

  it('should detect country from city keywords', () => {
    expect(detectCountryFromServer('moscow-server.example.com')).toBe('RU');
    expect(detectCountryFromServer('frankfurt.proxy.net')).toBe('DE');
    expect(detectCountryFromServer('tokyo1.vpn.org')).toBe('JP');
    expect(detectCountryFromServer('amsterdam-01.host.com')).toBe('NL');
  });

  it('should detect country from airport codes', () => {
    expect(detectCountryFromServer('fra-01.example.com')).toBe('DE');
    expect(detectCountryFromServer('ams-proxy.net')).toBe('NL');
    expect(detectCountryFromServer('sgp-server.org')).toBe('SG');
  });

  it('should return null for generic domains', () => {
    expect(detectCountryFromServer('example.com')).toBeNull();
    expect(detectCountryFromServer('server.org')).toBeNull();
    expect(detectCountryFromServer('proxy.net')).toBeNull();
  });
});

describe('getProxyFlag', () => {
  it('should return flag for explicit country code', () => {
    expect(getProxyFlag('US')).toBe('🇺🇸');
    expect(getProxyFlag('DE')).toBe('🇩🇪');
  });

  it('should fallback to server detection when country is null', () => {
    expect(getProxyFlag(null, 'server.ru')).toBe('🇷🇺');
    expect(getProxyFlag(undefined, 'frankfurt.example.com')).toBe('🇩🇪');
  });

  it('should prefer explicit country over server detection', () => {
    expect(getProxyFlag('US', 'server.ru')).toBe('🇺🇸');
    expect(getProxyFlag('JP', 'frankfurt.example.com')).toBe('🇯🇵');
  });

  it('should return globe when both country and server are unknown', () => {
    expect(getProxyFlag(null, 'unknown.com')).toBe('🌐');
    expect(getProxyFlag(null)).toBe('🌐');
  });
});

describe('getProxyCountryName', () => {
  it('should return name for explicit country code', () => {
    expect(getProxyCountryName('US')).toBe('United States');
    expect(getProxyCountryName('DE')).toBe('Germany');
  });

  it('should fallback to server detection when country is null', () => {
    expect(getProxyCountryName(null, 'server.ru')).toBe('Russia');
    expect(getProxyCountryName(undefined, 'tokyo.example.com')).toBe('Japan');
  });

  it('should return "Unknown" when both country and server are unknown', () => {
    expect(getProxyCountryName(null, 'unknown.com')).toBe('Unknown');
    expect(getProxyCountryName(null)).toBe('Unknown');
  });
});

describe('countryFlags mapping', () => {
  it('should have consistent keys between flags and names', () => {
    const flagKeys = Object.keys(countryFlags);
    const nameKeys = Object.keys(countryNames);
    
    // All flag keys should have corresponding names
    for (const key of flagKeys) {
      expect(countryNames).toHaveProperty(key);
    }
    
    // All name keys should have corresponding flags
    for (const key of nameKeys) {
      expect(countryFlags).toHaveProperty(key);
    }
  });

  it('should have valid emoji flags', () => {
    for (const [code, flag] of Object.entries(countryFlags)) {
      // Flag emojis are typically 4 bytes (2 regional indicator symbols)
      expect(flag.length).toBeGreaterThanOrEqual(2);
      expect(flag).not.toBe('');
    }
  });
});
