import type { UnlistenFn } from '@tauri-apps/api/event';

// Re-export UnlistenFn for convenience
export type { UnlistenFn };

// ============================================================================
// Core Types
// ============================================================================

export interface AppStatus {
    is_active: boolean;
    current_strategy: string | null;
    services_status: Record<string, ServiceStatus>;
}

export interface ServiceStatus {
    name: string;
    is_available: boolean;
    latency_ms: number | null;
}

export interface Strategy {
    id: string;
    name: string;
    description: string;
    family: string;
    engine: string;
}

export interface Service {
    id: string;
    name: string;
    critical: boolean;
}

export interface OptimizationProgress {
    stage: string;
    percent: number;
    message: string;
    current_strategy: string | null;
    tested_count: number;
    total_count: number;
    best_score: number | null;
}

export interface OptimizationResult {
    strategy_id: string;
    strategy_name: string;
    score: number;
    from_cache: boolean;
    all_scores: StrategyScore[];
}

export interface StrategyScore {
    strategy_id: string;
    score: number;
    success_rate: number;
    critical_success_rate: number;
    latency_avg: number;
    latency_jitter: number;
}

export interface DiagnosticResult {
    profile: {
        kind: string;
        details: string | null;
        candidate_families: string[];
    };
    tested_services: string[];
    blocked_services: string[];
}

// ============================================================================
// VLESS Types
// ============================================================================

export interface VlessConfig {
    id: string;
    name: string;
    server: string;
    port: number;
    uuid: string;
    flow: string | null;
    security: string;
    sni: string | null;
    active: boolean;
}

export type SingboxStatus =
    | 'starting'
    | 'running'
    | 'stopping'
    | 'stopped'
    | 'failed'
    | 'health_check_failed';

export interface SingboxInstance {
    config_id: string;
    config_name: string;
    socks_port: number;
    status: SingboxStatus;
    pid: number | null;
    started_at: number | null;
    last_health_check: number | null;
    health_check_failures: number;
}

// ============================================================================
// Proxy Types
// ============================================================================

export type ProxyProtocol =
    | 'socks5'
    | 'http'
    | 'https'
    | 'shadowsocks'
    | 'trojan'
    | 'vmess'
    | 'vless'
    | 'tuic'
    | 'hysteria'
    | 'hysteria2'
    | 'wireguard'
    | 'ssh';

export interface ProxyTestResult {
    success: boolean;
    latency?: number;
    error?: string;
}

export interface ProxyConfig {
    id: string;
    name: string;
    protocol: ProxyProtocol;
    server: string;
    port: number;
    username?: string;
    password?: string;
    uuid?: string;
    tls: boolean;
    sni?: string;
    transport?: string;
    custom_fields: Record<string, string>;
    active: boolean;
    country?: string | null;
    ping?: number;
}

// ============================================================================
// Routing Types
// ============================================================================

export interface DomainRoute {
    domain: string;
    proxy_id: string;
}

export interface AppRoute {
    app_name: string;
    app_path: string;
    proxy_id: string;
}

export interface InstalledApp {
    name: string;
    path: string;
    icon?: string;
}

export type RoutingRuleSource = 'domain' | 'app' | 'ip' | 'all';
export type RoutingRuleAction = 'direct' | 'proxy' | 'block' | 'dpi-bypass';

export interface RoutingRule {
    id: string;
    name: string;
    enabled: boolean;
    source: RoutingRuleSource;
    sourceValue?: string;
    action: RoutingRuleAction;
    proxyId?: string;
    priority: number;
}

// ============================================================================
// Testing Types
// ============================================================================

export interface TestProgress {
    current_item: string;
    current_type: 'proxy' | 'strategy';
    tested_count: number;
    total_count: number;
    percent: number;
}

export interface TestResult {
    id: string;
    name: string;
    type: 'proxy' | 'strategy';
    success_rate: number;
    latency_ms: number;
    score: number;
    services_tested: string[];
    services_passed: string[];
}

// ============================================================================
// Settings Types
// ============================================================================

export interface AppSettings {
    auto_start: boolean;
    auto_apply: boolean;
    minimize_to_tray: boolean;
    block_quic: boolean;
    default_mode: 'turbo' | 'deep';
    system_proxy: boolean;
    tun_mode: boolean;
    per_domain_routing: boolean;
    per_app_routing: boolean;
    test_timeout: number;
    test_services: string[];
    language: 'ru' | 'en';
    telemetry_enabled: boolean;
}

// ============================================================================
// Log Types
// ============================================================================

export interface LogEntry {
    timestamp: string;
    level: string;
    module: string;
    message: string;
}

export interface LogFilter {
    level?: 'error' | 'warn' | 'info' | 'debug' | 'trace';
    module?: string;
    search?: string;
}

// ============================================================================
// Tray Types
// ============================================================================

export type TrayState = 'inactive' | 'active' | 'optimizing' | 'error';

// ============================================================================
// TUN Types
// ============================================================================

export type TunStatus = 'stopped' | 'starting' | 'running' | 'stopping' | 'failed';

export interface TunConfig {
    interface_name: string;
    mtu: number;
    address_v4: string;
    address_v6?: string;
    strict_route: boolean;
    auto_route: boolean;
    stack: string;
}

export interface TunInstance {
    status: TunStatus;
    socks_port: number;
    pid?: number;
    started_at?: number;
    config: TunConfig;
}

// ============================================================================
// Monitor Types
// ============================================================================

export interface HealthCheckResult {
    strategy_id: string;
    is_healthy: boolean;
    success_rate: number;
    consecutive_failures: number;
    timestamp: string;
}

export interface DegradationEvent {
    strategy_id: string;
    consecutive_failures: number;
    last_success_rate: number;
    timestamp: string;
}

export interface RecoveryEvent {
    strategy_id: string;
    timestamp: string;
}

// ============================================================================
// Config Updater Types
// ============================================================================

export interface ConfigUpdate {
    name: string;
    path: string;
    sha: string;
    is_new: boolean;
}

export interface ConfigUpdateResult {
    updated_count: number;
    new_count: number;
    files: string[];
}

// ============================================================================
// DPI Testing Types
// ============================================================================

export interface DpiTestResult {
    strategy_id: string;
    success: boolean;
    blocked_before: number;
    blocked_after: number;
    passed_after: number;
    latency_ms?: number;
    error?: string;
}

// ============================================================================
// DNS Types
// ============================================================================

export type DnsServer = 'system' | 'cloudflare' | 'google' | 'custom';

export interface DnsSettings {
    server: DnsServer;
    customAddress?: string;
}

// ============================================================================
// WinDivert Types
// ============================================================================

export type WindivertMode = 'normal' | 'autottl' | 'autohostlist';

// ============================================================================
// Conflict Detection Types
// ============================================================================

export type ConflictSeverity = 'critical' | 'high' | 'medium' | 'low';

export type ConflictCategory = 
    | 'network_filter' 
    | 'vpn' 
    | 'network_optimization' 
    | 'security' 
    | 'windivert';

export interface ConflictInfo {
    name: string;
    category: ConflictCategory;
    severity: ConflictSeverity;
    description: string;
    recommendation: string;
    detected_processes: string[];
    detected_services: string[];
}


// ============================================================================
// Provider Types
// ============================================================================

export interface ProviderSummary {
    id: string;
    name: string;
    description: string;
    dpi_type: string;
    strategy_count: number;
}

export interface ProviderRecommendations {
    provider_id: string;
    provider_name: string;
    strategies: string[];
    notes: string;
}
