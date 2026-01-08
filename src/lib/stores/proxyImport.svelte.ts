/**
 * Proxy Import Store
 * Handles parsing and validation of proxy URLs
 */

export interface ParsedProxy {
  protocol: 'vless' | 'vmess' | 'shadowsocks' | 'trojan' | 'ss';
  name: string;
  server: string;
  port: number;
  uuid?: string;
  password?: string;
  method?: string;
  security?: string;
  sni?: string;
  flow?: string;
  network?: string;
  raw: string;
}

class ProxyImportStore {
  rawInput = $state('');
  parsedProxy = $state<ParsedProxy | null>(null);
  isValid = $state(false);
  error = $state<string | null>(null);
  
  parseInput(input: string) {
    this.rawInput = input;
    this.error = null;
    this.parsedProxy = null;
    this.isValid = false;
    
    if (!input.trim()) return;
    
    const trimmed = input.trim();
    
    try {
      if (trimmed.startsWith('vless://')) {
        this.parsedProxy = this.parseVless(trimmed);
      } else if (trimmed.startsWith('vmess://')) {
        this.parsedProxy = this.parseVmess(trimmed);
      } else if (trimmed.startsWith('ss://')) {
        this.parsedProxy = this.parseShadowsocks(trimmed);
      } else if (trimmed.startsWith('trojan://')) {
        this.parsedProxy = this.parseTrojan(trimmed);
      } else {
        this.error = 'Unsupported format. Use vless://, vmess://, ss://, or trojan://';
        return;
      }
      this.isValid = true;
    } catch (e) {
      this.error = e instanceof Error ? e.message : 'Failed to parse proxy URL';
    }
  }

  private parseVless(url: string): ParsedProxy {
    // vless://uuid@server:port?params#name
    const match = url.match(/^vless:\/\/([^@]+)@([^:]+):(\d+)(\?[^#]*)?(#.*)?$/);
    if (!match) throw new Error('Invalid VLESS URL format');
    
    const [, uuid, server, port, queryStr, fragment] = match;
    const params = new URLSearchParams(queryStr?.slice(1) || '');
    const name = fragment ? decodeURIComponent(fragment.slice(1)) : server;
    
    return {
      protocol: 'vless',
      name,
      server,
      port: parseInt(port, 10),
      uuid,
      security: params.get('security') || 'none',
      sni: params.get('sni') || undefined,
      flow: params.get('flow') || undefined,
      network: params.get('type') || 'tcp',
      raw: url
    };
  }
  
  private parseVmess(url: string): ParsedProxy {
    // vmess://base64
    const base64 = url.slice(8);
    try {
      const decoded = atob(base64);
      const config = JSON.parse(decoded);
      
      return {
        protocol: 'vmess',
        name: config.ps || config.add || 'VMess Server',
        server: config.add,
        port: parseInt(config.port, 10),
        uuid: config.id,
        security: config.scy || 'auto',
        network: config.net || 'tcp',
        sni: config.sni || config.host || undefined,
        raw: url
      };
    } catch {
      throw new Error('Invalid VMess URL: failed to decode');
    }
  }
  
  private parseShadowsocks(url: string): ParsedProxy {
    // ss://base64@server:port#name or ss://method:password@server:port#name
    let server: string, port: number, method: string, password: string, name: string;
    
    // Try SIP002 format first (base64 encoded method:password)
    const sip002Match = url.match(/^ss:\/\/([A-Za-z0-9+/=]+)@([^:]+):(\d+)(#.*)?$/);
    if (sip002Match) {
      const [, encoded, srv, prt, fragment] = sip002Match;
      const decoded = atob(encoded);
      const [m, ...pwdParts] = decoded.split(':');
      method = m;
      password = pwdParts.join(':');
      server = srv;
      port = parseInt(prt, 10);
      name = fragment ? decodeURIComponent(fragment.slice(1)) : server;
    } else {
      // Legacy format
      const legacyMatch = url.match(/^ss:\/\/([^:]+):([^@]+)@([^:]+):(\d+)(#.*)?$/);
      if (!legacyMatch) throw new Error('Invalid Shadowsocks URL format');
      
      const [, m, pwd, srv, prt, fragment] = legacyMatch;
      method = m;
      password = pwd;
      server = srv;
      port = parseInt(prt, 10);
      name = fragment ? decodeURIComponent(fragment.slice(1)) : server;
    }
    
    return {
      protocol: 'shadowsocks',
      name,
      server,
      port,
      method,
      password,
      raw: url
    };
  }
  
  private parseTrojan(url: string): ParsedProxy {
    // trojan://password@server:port?params#name
    const match = url.match(/^trojan:\/\/([^@]+)@([^:]+):(\d+)(\?[^#]*)?(#.*)?$/);
    if (!match) throw new Error('Invalid Trojan URL format');
    
    const [, password, server, port, queryStr, fragment] = match;
    const params = new URLSearchParams(queryStr?.slice(1) || '');
    const name = fragment ? decodeURIComponent(fragment.slice(1)) : server;
    
    return {
      protocol: 'trojan',
      name,
      server,
      port: parseInt(port, 10),
      password,
      sni: params.get('sni') || params.get('peer') || undefined,
      security: params.get('security') || 'tls',
      raw: url
    };
  }
  
  clear() {
    this.rawInput = '';
    this.parsedProxy = null;
    this.isValid = false;
    this.error = null;
  }
  
  updateName(name: string) {
    if (this.parsedProxy) {
      this.parsedProxy = { ...this.parsedProxy, name };
    }
  }
}

export const proxyImportStore = new ProxyImportStore();
