import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// Types
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

export interface DiagnosticResult {
    profile: {
        kind: string;
        details: string | null;
        candidate_families: string[];
    };
    tested_services: string[];
    blocked_services: string[];
}

// API Functions
export async function getStatus(): Promise<AppStatus> {
    return invoke('get_status');
}

export async function getStrategies(): Promise<Strategy[]> {
    return invoke('get_strategies');
}

export async function getServices(): Promise<Service[]> {
    return invoke('get_services');
}

export async function runOptimization(mode: 'turbo' | 'deep'): Promise<string> {
    return invoke('run_optimization', { mode });
}

export async function cancelOptimization(): Promise<void> {
    return invoke('cancel_optimization');
}

export async function applyStrategy(strategyId: string): Promise<void> {
    return invoke('apply_strategy', { strategyId });
}

export async function stopStrategy(): Promise<void> {
    return invoke('stop_strategy');
}

export async function diagnose(): Promise<DiagnosticResult> {
    return invoke('diagnose');
}

export async function panicReset(): Promise<void> {
    return invoke('panic_reset');
}

// Event Listeners
export function onOptimizationProgress(
    callback: (progress: OptimizationProgress) => void
): Promise<UnlistenFn> {
    return listen('optimization:progress', (event) => {
        callback(event.payload as OptimizationProgress);
    });
}

export function onOptimizationComplete(
    callback: (result: { strategy_id: string; score: number }) => void
): Promise<UnlistenFn> {
    return listen('optimization:complete', (event) => {
        callback(event.payload as { strategy_id: string; score: number });
    });
}

export function onOptimizationFailed(
    callback: (error: string) => void
): Promise<UnlistenFn> {
    return listen('optimization:failed', (event) => {
        callback(event.payload as string);
    });
}

export function onStrategyDegraded(
    callback: () => void
): Promise<UnlistenFn> {
    return listen('strategy:degraded', () => {
        callback();
    });
}

// VLESS Types
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

// Sing-box instance status
export type SingboxStatus =
    | 'starting'
    | 'running'
    | 'stopping'
    | 'stopped'
    | 'failed'
    | 'health_check_failed';

// Sing-box instance info
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

// VLESS API Functions
export async function importVless(url: string): Promise<VlessConfig> {
    return invoke('import_vless', { url });
}

export async function getVlessConfigs(): Promise<VlessConfig[]> {
    return invoke('get_vless_configs');
}

export async function deleteVlessConfig(id: string): Promise<void> {
    return invoke('delete_vless_config', { id });
}

export async function toggleVlessConfig(id: string, active: boolean): Promise<void> {
    return invoke('toggle_vless_config', { id, active });
}

// ============================================================================
// VLESS Proxy Control API
// ============================================================================

/**
 * Start VLESS proxy for a specific config.
 * @param configId - The ID of the VLESS config to start
 * @param socksPort - Optional SOCKS port (auto-allocated if not provided)
 * @returns The running instance info
 */
export async function startVlessProxy(
    configId: string,
    socksPort?: number
): Promise<SingboxInstance> {
    return invoke('start_vless_proxy', { configId, socksPort });
}

/**
 * Stop VLESS proxy for a specific config.
 * @param configId - The ID of the VLESS config to stop
 */
export async function stopVlessProxy(configId: string): Promise<void> {
    return invoke('stop_vless_proxy', { configId });
}

/**
 * Stop all running VLESS proxies.
 */
export async function stopAllVlessProxies(): Promise<void> {
    return invoke('stop_all_vless_proxies');
}

/**
 * Get status of a specific VLESS proxy.
 * @param configId - The ID of the VLESS config
 * @returns The instance info or null if not running
 */
export async function getVlessStatus(configId: string): Promise<SingboxInstance | null> {
    return invoke('get_vless_status', { configId });
}

/**
 * Get status of all running VLESS proxies.
 * @returns Array of all running instances
 */
export async function getAllVlessStatus(): Promise<SingboxInstance[]> {
    return invoke('get_all_vless_status');
}

/**
 * Perform health check on a running VLESS proxy.
 * @param configId - The ID of the VLESS config
 * @returns true if healthy
 */
export async function healthCheckVless(configId: string): Promise<boolean> {
    return invoke('health_check_vless', { configId });
}

/**
 * Test VLESS proxy connectivity by making a test request.
 * @param configId - The ID of the VLESS config
 * @param testUrl - Optional URL to test (defaults to google.com)
 * @returns Latency in milliseconds
 */
export async function testVlessConnectivity(
    configId: string,
    testUrl?: string
): Promise<number> {
    return invoke('test_vless_connectivity', { configId, testUrl });
}

/**
 * Check if sing-box binary is available.
 * @returns true if sing-box is installed
 */
export async function isSingboxAvailable(): Promise<boolean> {
    return invoke('is_singbox_available');
}

/**
 * Get sing-box version.
 * @returns Version string
 */
export async function getSingboxVersion(): Promise<string> {
    return invoke('get_singbox_version');
}

// ============================================================================
// QUIC Blocking API
// ============================================================================

/**
 * Enable QUIC blocking via Windows Firewall.
 * Blocks UDP port 443 to force browsers to use TCP/TLS.
 * Requires administrator privileges.
 */
export async function enableQuicBlock(): Promise<void> {
    return invoke('enable_quic_block');
}

/**
 * Disable QUIC blocking.
 * Removes the firewall rule that blocks QUIC protocol.
 * Requires administrator privileges.
 */
export async function disableQuicBlock(): Promise<void> {
    return invoke('disable_quic_block');
}

/**
 * Check if QUIC is currently blocked.
 * @returns true if the QUIC blocking firewall rule exists
 */
export async function isQuicBlocked(): Promise<boolean> {
    return invoke('is_quic_blocked');
}

/**
 * Check if the application is running with administrator privileges.
 * @returns true if running as admin
 */
export async function isAdmin(): Promise<boolean> {
    return invoke('is_admin');
}

// ============================================================================
// Proxy Types
// ============================================================================

/** Supported proxy protocols */
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

/** Proxy configuration */
export interface ProxyConfig {
    /** Unique identifier */
    id: string;
    /** Display name */
    name: string;
    /** Protocol type */
    protocol: ProxyProtocol;
    /** Server address */
    server: string;
    /** Server port */
    port: number;
    /** Username for authentication */
    username?: string;
    /** Password for authentication */
    password?: string;
    /** UUID for VLESS/VMess protocols */
    uuid?: string;
    /** TLS enabled */
    tls: boolean;
    /** Server Name Indication */
    sni?: string;
    /** Transport type (ws, grpc, etc.) */
    transport?: string;
    /** Additional protocol-specific fields */
    custom_fields: Record<string, string>;
    /** Whether this proxy is active */
    active: boolean;
}

// ============================================================================
// Routing Types
// ============================================================================

/** Domain-based routing rule */
export interface DomainRoute {
    /** Domain pattern (e.g., "youtube.com", "*.google.com") */
    domain: string;
    /** ID of the proxy to use for this domain */
    proxy_id: string;
}

/** Application-based routing rule */
export interface AppRoute {
    /** Application display name */
    app_name: string;
    /** Full path to the application executable */
    app_path: string;
    /** ID of the proxy to use for this application */
    proxy_id: string;
}

/** Installed application info */
export interface InstalledApp {
    /** Application display name */
    name: string;
    /** Full path to the executable */
    path: string;
    /** Base64 encoded icon (optional) */
    icon?: string;
}

// ============================================================================
// Testing Types
// ============================================================================

/** Progress information during testing */
export interface TestProgress {
    /** Current item being tested */
    current_item: string;
    /** Type of current item */
    current_type: 'proxy' | 'strategy';
    /** Number of items tested so far */
    tested_count: number;
    /** Total number of items to test */
    total_count: number;
    /** Progress percentage (0-100) */
    percent: number;
}

/** Result of a single test */
export interface TestResult {
    /** ID of the tested item */
    id: string;
    /** Name of the tested item */
    name: string;
    /** Type of the tested item */
    type: 'proxy' | 'strategy';
    /** Success rate (0-100) */
    success_rate: number;
    /** Average latency in milliseconds */
    latency_ms: number;
    /** Calculated score */
    score: number;
    /** List of services that were tested */
    services_tested: string[];
    /** List of services that passed */
    services_passed: string[];
}

// ============================================================================
// Proxy API Functions
// ============================================================================

/**
 * Get all configured proxies.
 * @returns Array of proxy configurations
 */
export async function getProxies(): Promise<ProxyConfig[]> {
    return invoke('get_proxies');
}

/**
 * Add a new proxy configuration.
 * @param proxy - Partial proxy configuration (id will be generated)
 * @returns The created proxy configuration
 */
export async function addProxy(proxy: Partial<ProxyConfig>): Promise<ProxyConfig> {
    return invoke('add_proxy', { proxy });
}

/**
 * Update an existing proxy configuration.
 * @param proxy - Full proxy configuration with id
 */
export async function updateProxy(proxy: ProxyConfig): Promise<void> {
    return invoke('update_proxy', { proxy });
}

/**
 * Delete a proxy configuration.
 * @param id - Proxy ID to delete
 */
export async function deleteProxy(id: string): Promise<void> {
    return invoke('delete_proxy', { id });
}

/**
 * Apply (activate) a proxy.
 * @param id - Proxy ID to apply
 */
export async function applyProxy(id: string): Promise<void> {
    return invoke('apply_proxy', { id });
}

/**
 * Test proxy connectivity.
 * @param id - Proxy ID to test
 * @returns Latency in milliseconds
 */
export async function testProxy(id: string): Promise<number> {
    return invoke('test_proxy', { id });
}

/**
 * Import proxy from URL (ss://, vmess://, vless://, trojan://, etc.)
 * @param url - Proxy URL to import
 * @returns Parsed proxy configuration
 */
export async function importProxyUrl(url: string): Promise<ProxyConfig> {
    return invoke('import_proxy_url', { url });
}

/**
 * Import proxies from subscription URL.
 * @param url - Subscription URL
 * @returns Array of parsed proxy configurations
 */
export async function importSubscription(url: string): Promise<ProxyConfig[]> {
    return invoke('import_subscription', { url });
}

// ============================================================================
// Routing API Functions
// ============================================================================

/**
 * Get all domain routing rules.
 * @returns Array of domain routes
 */
export async function getDomainRoutes(): Promise<DomainRoute[]> {
    return invoke('get_domain_routes');
}

/**
 * Add a domain routing rule.
 * @param domain - Domain pattern
 * @param proxyId - Proxy ID to route through
 */
export async function addDomainRoute(domain: string, proxyId: string): Promise<void> {
    return invoke('add_domain_route', { domain, proxyId });
}

/**
 * Remove a domain routing rule.
 * @param domain - Domain pattern to remove
 */
export async function removeDomainRoute(domain: string): Promise<void> {
    return invoke('remove_domain_route', { domain });
}

/**
 * Get all application routing rules.
 * @returns Array of app routes
 */
export async function getAppRoutes(): Promise<AppRoute[]> {
    return invoke('get_app_routes');
}

/**
 * Add an application routing rule.
 * @param appName - Application display name
 * @param appPath - Full path to executable
 * @param proxyId - Proxy ID to route through
 */
export async function addAppRoute(appName: string, appPath: string, proxyId: string): Promise<void> {
    return invoke('add_app_route', { appName, appPath, proxyId });
}

/**
 * Remove an application routing rule.
 * @param appPath - Application path to remove
 */
export async function removeAppRoute(appPath: string): Promise<void> {
    return invoke('remove_app_route', { appPath });
}

/**
 * Get list of installed applications.
 * @returns Array of installed apps with names, paths, and icons
 */
export async function getInstalledApps(): Promise<InstalledApp[]> {
    return invoke('get_installed_apps');
}

// ============================================================================
// Testing API Functions
// ============================================================================

/**
 * Run tests on proxies and/or strategies.
 * @param proxyIds - Array of proxy IDs to test
 * @param strategyIds - Array of strategy IDs to test
 * @param serviceIds - Array of service IDs to test against
 * @param mode - Testing mode: 'turbo' (fast) or 'deep' (thorough)
 */
export async function runTests(
    proxyIds: string[],
    strategyIds: string[],
    serviceIds: string[],
    mode: 'turbo' | 'deep'
): Promise<void> {
    return invoke('run_tests', { proxyIds, strategyIds, serviceIds, mode });
}

/**
 * Cancel running tests.
 */
export async function cancelTests(): Promise<void> {
    return invoke('cancel_tests');
}

/**
 * Subscribe to test progress events.
 * @param callback - Function to call on progress update
 * @returns Unsubscribe function
 */
export function onTestProgress(callback: (progress: TestProgress) => void): Promise<UnlistenFn> {
    return listen('test:progress', (event) => {
        callback(event.payload as TestProgress);
    });
}

/**
 * Subscribe to individual test result events.
 * @param callback - Function to call when a test completes
 * @returns Unsubscribe function
 */
export function onTestResult(callback: (result: TestResult) => void): Promise<UnlistenFn> {
    return listen('test:result', (event) => {
        callback(event.payload as TestResult);
    });
}

/**
 * Subscribe to test completion event.
 * @param callback - Function to call when all tests complete
 * @returns Unsubscribe function
 */
export function onTestComplete(callback: (results: TestResult[]) => void): Promise<UnlistenFn> {
    return listen('test:complete', (event) => {
        callback(event.payload as TestResult[]);
    });
}

// ============================================================================
// Settings Types
// ============================================================================

/** Application settings */
export interface AppSettings {
    /** Start application on system boot */
    auto_start: boolean;
    /** Automatically apply last strategy on start */
    auto_apply: boolean;
    /** Minimize to system tray instead of closing */
    minimize_to_tray: boolean;
    /** Block QUIC protocol (UDP 443) */
    block_quic: boolean;
    /** Default testing mode */
    default_mode: 'turbo' | 'deep';
    /** Use system proxy settings */
    system_proxy: boolean;
    /** Enable TUN mode for full traffic capture */
    tun_mode: boolean;
    /** Enable per-domain routing */
    per_domain_routing: boolean;
    /** Enable per-application routing */
    per_app_routing: boolean;
    /** Test timeout in seconds */
    test_timeout: number;
    /** Services to test by default */
    test_services: string[];
    /** UI language */
    language: 'ru' | 'en';
    /** Enable anonymous telemetry */
    telemetry_enabled: boolean;
}

/**
 * Get application settings.
 * @returns Current settings
 */
export async function getAppSettings(): Promise<AppSettings> {
    return invoke('get_settings');
}

/**
 * Save application settings.
 * @param settings - Settings to save
 */
export async function saveAppSettings(settings: AppSettings): Promise<void> {
    return invoke('save_settings', { settings });
}

// ============================================================================
// Log Types
// ============================================================================

/** Log entry */
export interface LogEntry {
    /** Timestamp in ISO format */
    timestamp: string;
    /** Log level (error, warn, info, debug, trace) */
    level: string;
    /** Module that generated the log */
    module: string;
    /** Log message */
    message: string;
}

/** Log filter parameters */
export interface LogFilter {
    /** Filter by log level (shows this level and above) */
    level?: 'error' | 'warn' | 'info' | 'debug' | 'trace';
    /** Filter by module name (partial match) */
    module?: string;
    /** Search text in message or module */
    search?: string;
}

// ============================================================================
// Log API Functions
// ============================================================================

/**
 * Get logs with optional filtering.
 * @param filter - Optional filter parameters
 * @returns Array of log entries
 */
export async function getLogs(filter?: LogFilter): Promise<LogEntry[]> {
    return invoke('get_logs', { filter });
}

/**
 * Clear all logs from memory.
 */
export async function clearLogs(): Promise<void> {
    return invoke('clear_logs');
}

/**
 * Export logs to a file.
 * @returns Path to the exported file
 */
export async function exportLogs(): Promise<string> {
    return invoke('export_logs');
}

/**
 * Subscribe to real-time log entries.
 * @param callback - Function to call when a new log entry is received
 * @returns Unsubscribe function
 */
export function onLogEntry(callback: (entry: LogEntry) => void): Promise<UnlistenFn> {
    return listen('log:entry', (event) => {
        callback(event.payload as LogEntry);
    });
}

// ============================================================================
// Tray Types
// ============================================================================

/** Tray icon states */
export type TrayState = 'inactive' | 'active' | 'optimizing' | 'error';

// ============================================================================
// Tray API Functions
// ============================================================================

/**
 * Update tray status and icon.
 * @param state - New tray state
 * @param strategyName - Optional strategy name to display
 */
export async function updateTray(state: TrayState, strategyName?: string): Promise<void> {
    return invoke('update_tray', { state, strategyName });
}

/**
 * Set tray to optimizing state.
 */
export async function setTrayOptimizing(): Promise<void> {
    return invoke('set_tray_optimizing');
}

/**
 * Set tray to error state.
 * @param errorMsg - Error message to display in tooltip
 */
export async function setTrayError(errorMsg: string): Promise<void> {
    return invoke('set_tray_error', { errorMsg });
}

/**
 * Get current tray state.
 * @returns Current tray state
 */
export async function getTrayState(): Promise<TrayState> {
    return invoke('get_tray_state');
}

// ============================================================================
// Tray Event Listeners
// ============================================================================

/**
 * Subscribe to tray optimize events.
 * @param callback - Function to call when optimize is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayOptimize(callback: (mode: 'turbo' | 'deep') => void): Promise<UnlistenFn> {
    return listen('tray:optimize', (event) => {
        callback(event.payload as 'turbo' | 'deep');
    });
}

/**
 * Subscribe to tray toggle events.
 * @param callback - Function to call when toggle is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayToggle(callback: () => void): Promise<UnlistenFn> {
    return listen('tray:toggle', () => {
        callback();
    });
}

/**
 * Subscribe to tray stop events.
 * @param callback - Function to call when stop is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayStop(callback: () => void): Promise<UnlistenFn> {
    return listen('tray:stop', () => {
        callback();
    });
}

/**
 * Subscribe to tray panic reset events.
 * @param callback - Function to call when panic reset is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayPanicReset(callback: () => void): Promise<UnlistenFn> {
    return listen('tray:panic_reset', () => {
        callback();
    });
}

/**
 * Subscribe to tray navigation events.
 * @param callback - Function to call with the route to navigate to
 * @returns Unsubscribe function
 */
export function onTrayNavigate(callback: (route: string) => void): Promise<UnlistenFn> {
    return listen('tray:navigate', (event) => {
        callback(event.payload as string);
    });
}

/**
 * Subscribe to tray quit events.
 * @param callback - Function to call when quit is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayQuit(callback: () => void): Promise<UnlistenFn> {
    return listen('tray:quit', () => {
        callback();
    });
}

/**
 * Subscribe to tray QUIC block events.
 * @param callback - Function to call with block state (true = block, false = unblock)
 * @returns Unsubscribe function
 */
export function onTrayQuicBlock(callback: (block: boolean) => void): Promise<UnlistenFn> {
    return listen('tray:quic_block', (event) => {
        callback(event.payload as boolean);
    });
}

// ============================================================================
// TUN Types
// ============================================================================

/** TUN status */
export type TunStatus = 'stopped' | 'starting' | 'running' | 'stopping' | 'failed';

/** TUN configuration */
export interface TunConfig {
    /** TUN interface name */
    interface_name: string;
    /** MTU size */
    mtu: number;
    /** IPv4 address for TUN */
    address_v4: string;
    /** IPv6 address for TUN (optional) */
    address_v6?: string;
    /** Enable strict routing */
    strict_route: boolean;
    /** Auto route */
    auto_route: boolean;
    /** Stack implementation */
    stack: string;
}

/** TUN instance information */
export interface TunInstance {
    status: TunStatus;
    socks_port: number;
    pid?: number;
    started_at?: number;
    config: TunConfig;
}

// ============================================================================
// TUN API Functions
// ============================================================================

/**
 * Start TUN mode.
 * Routes all system traffic through the specified SOCKS proxy.
 * Requires administrator privileges.
 * @param socksPort - SOCKS5 proxy port to route traffic through
 * @returns TUN instance information
 */
export async function startTun(socksPort: number): Promise<TunInstance> {
    return invoke('start_tun', { socksPort });
}

/**
 * Stop TUN mode.
 */
export async function stopTun(): Promise<void> {
    return invoke('stop_tun');
}

/**
 * Check if TUN is running.
 * @returns true if TUN is running
 */
export async function isTunRunning(): Promise<boolean> {
    return invoke('is_tun_running');
}

/**
 * Get TUN status.
 * @returns TUN instance information
 */
export async function getTunStatus(): Promise<TunInstance> {
    return invoke('get_tun_status');
}

/**
 * Get TUN configuration.
 * @returns Current TUN configuration
 */
export async function getTunConfig(): Promise<TunConfig> {
    return invoke('get_tun_config');
}

/**
 * Update TUN configuration.
 * Note: Changes take effect on next TUN start.
 * @param config - New TUN configuration
 */
export async function setTunConfig(config: TunConfig): Promise<void> {
    return invoke('set_tun_config', { config });
}

/**
 * Check if TUN mode is available.
 * Returns true if sing-box exists and running with admin privileges.
 * @returns true if TUN is available
 */
export async function isTunAvailable(): Promise<boolean> {
    return invoke('is_tun_available');
}

/**
 * Restart TUN with optional new SOCKS port.
 * @param socksPort - Optional new SOCKS port
 * @returns TUN instance information
 */
export async function restartTun(socksPort?: number): Promise<TunInstance> {
    return invoke('restart_tun', { socksPort });
}

// ============================================================================
// System Proxy API Functions
// ============================================================================

/**
 * Set system proxy.
 * Configures Windows system proxy settings.
 * @param host - Proxy host
 * @param port - Proxy port
 * @param scheme - Proxy scheme ('socks5', 'http')
 */
export async function setSystemProxy(host: string, port: number, scheme: string): Promise<void> {
    return invoke('set_system_proxy', { host, port, scheme });
}

/**
 * Clear system proxy settings.
 */
export async function clearSystemProxy(): Promise<void> {
    return invoke('clear_system_proxy');
}

/**
 * Check if system proxy is currently set.
 * @returns true if system proxy is enabled
 */
export async function isSystemProxySet(): Promise<boolean> {
    return invoke('is_system_proxy_set');
}

// ============================================================================
// Monitor API
// ============================================================================

/** Start strategy health monitoring */
export async function startMonitor(): Promise<void> {
    return invoke('start_monitor');
}

/** Stop strategy health monitoring */
export async function stopMonitor(): Promise<void> {
    return invoke('stop_monitor');
}

/** Check if monitor is running */
export async function isMonitorRunning(): Promise<boolean> {
    return invoke('is_monitor_running');
}

/** Check if strategy is degraded */
export async function isStrategyDegraded(): Promise<boolean> {
    return invoke('is_strategy_degraded');
}

/** Perform manual health check */
export async function checkStrategyHealth(): Promise<boolean> {
    return invoke('check_strategy_health');
}

/** Set monitor test URLs */
export async function setMonitorUrls(urls: string[]): Promise<void> {
    return invoke('set_monitor_urls', { urls });
}

/** Enable/disable auto-restart on degradation */
export async function setMonitorAutoRestart(enabled: boolean): Promise<void> {
    return invoke('set_monitor_auto_restart', { enabled });
}

// ============================================================================
// Telemetry API
// ============================================================================

/** Enable or disable telemetry (opt-in) */
export async function setTelemetryEnabled(enabled: boolean): Promise<void> {
    return invoke('set_telemetry_enabled', { enabled });
}

/** Check if telemetry is enabled */
export async function isTelemetryEnabled(): Promise<boolean> {
    return invoke('is_telemetry_enabled');
}

/** Get number of pending telemetry events */
export async function getTelemetryPendingCount(): Promise<number> {
    return invoke('get_telemetry_pending_count');
}

/** Manually flush telemetry events */
export async function flushTelemetry(): Promise<void> {
    return invoke('flush_telemetry');
}

/** Clear pending telemetry events without sending */
export async function clearTelemetry(): Promise<void> {
    return invoke('clear_telemetry');
}

/** Report optimization result to telemetry */
export async function reportOptimizationTelemetry(
    strategyId: string,
    score: number,
    success: boolean
): Promise<void> {
    return invoke('report_optimization_telemetry', { strategyId, score, success });
}

/** Report strategy usage to telemetry */
export async function reportStrategyUsageTelemetry(
    strategyId: string,
    durationSecs: number
): Promise<void> {
    return invoke('report_strategy_usage_telemetry', { strategyId, durationSecs });
}

// ============================================================================
// Monitor Event Types
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
// Config Updater API
// ============================================================================

/** Config update info */
export interface ConfigUpdate {
    name: string;
    path: string;
    sha: string;
    is_new: boolean;
}

/** Config update result */
export interface ConfigUpdateResult {
    updated_count: number;
    new_count: number;
    files: string[];
}

/** Check for config updates from remote repository */
export async function checkConfigUpdates(): Promise<ConfigUpdate[]> {
    return invoke('check_config_updates');
}

/** Download and apply config updates */
export async function downloadConfigUpdates(): Promise<ConfigUpdateResult> {
    return invoke('download_config_updates');
}
