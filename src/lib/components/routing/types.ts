// Routing types shared across components

export interface RoutingRule {
  id: string;
  name: string;
  enabled: boolean;
  source: 'all' | 'app' | 'domain';
  sourceValue?: string;
  action: 'direct' | 'proxy' | 'block';
  proxyId?: string;
}

export interface ProxyConfig {
  id: string;
  name: string;
  protocol: string;
}
