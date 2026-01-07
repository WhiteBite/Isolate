// ============================================================================
// API Module Index
// Re-exports all API functions and types for backward compatibility
// Usage: import { getStatus, Strategy, ... } from '$lib/api'
// ============================================================================

// Types - export all types
export type {
    // Core types
    AppStatus,
    ServiceStatus,
    Strategy,
    Service,
    OptimizationProgress,
    OptimizationResult,
    StrategyScore,
    DiagnosticResult,
    // VLESS types
    VlessConfig,
    SingboxStatus,
    SingboxInstance,
    // Proxy types
    ProxyProtocol,
    ProxyTestResult,
    ProxyConfig,
    // Routing types
    DomainRoute,
    AppRoute,
    InstalledApp,
    RoutingRuleSource,
    RoutingRuleAction,
    RoutingRule,
    // Testing types
    TestProgress,
    TestResult,
    // Settings types
    AppSettings,
    // Log types
    LogEntry,
    LogFilter,
    // Tray types
    TrayState,
    // TUN types
    TunStatus,
    TunConfig,
    TunInstance,
    // Monitor types
    HealthCheckResult,
    DegradationEvent,
    RecoveryEvent,
    // Config types
    ConfigUpdate,
    ConfigUpdateResult,
    // DPI types
    DpiTestResult,
    // DNS types
    DnsServer,
    DnsSettings,
    // WinDivert types
    WindivertMode,
    // Event types
    UnlistenFn,
} from './types';

// Core API
export {
    getStatus,
    getStrategies,
    getServices,
    applyStrategy,
    stopStrategy,
    diagnose,
    panicReset,
    enableQuicBlock,
    disableQuicBlock,
    isQuicBlocked,
    isAdmin,
} from './core';

// Optimization API
export {
    runOptimization,
    cancelOptimization,
    isOptimizationRunning,
    // Domain Monitor
    startDomainMonitor,
    stopDomainMonitor,
    isDomainMonitorRunning,
    getDomainStatus,
    getAllDomainStatuses,
    // Strategy Manager
    getBlockedStrategies,
    blockStrategy,
    unblockStrategy,
    getLockedStrategy,
    lockStrategy,
    unlockStrategy,
    invalidateStrategyCache,
    // Event listeners
    onOptimizationProgress,
    onOptimizationComplete,
    onOptimizationError,
    onOptimizationFailed,
    onDomainLocked,
    onDomainUnlocked,
    onMonitorStarted,
    onMonitorStopped,
    // Types
    type MonitorConfig,
    type DomainStatus,
} from './optimization';

// VLESS API
export {
    importVless,
    getVlessConfigs,
    deleteVlessConfig,
    toggleVlessConfig,
    startVlessProxy,
    stopVlessProxy,
    stopAllVlessProxies,
    getVlessStatus,
    getAllVlessStatus,
    healthCheckVless,
    testVlessConnectivity,
    isSingboxAvailable,
    getSingboxVersion,
} from './vless';

// Proxy API
export {
    getProxies,
    addProxy,
    updateProxy,
    deleteProxy,
    applyProxy,
    deactivateProxy,
    testProxy,
    importProxyUrl,
    importSubscription,
    exportProxyUrl,
    setSystemProxy,
    clearSystemProxy,
    isSystemProxySet,
} from './proxy';

// Routing API
export {
    getDomainRoutes,
    addDomainRoute,
    removeDomainRoute,
    getAppRoutes,
    addAppRoute,
    removeAppRoute,
    getInstalledApps,
    getRoutingRules,
    addRoutingRule,
    updateRoutingRule,
    deleteRoutingRule,
    reorderRoutingRules,
    toggleRoutingRule,
} from './routing';

// Testing API
export {
    runTests,
    cancelTests,
    onTestProgress,
    onTestResult,
    onTestComplete,
} from './testing';

// Settings API
export {
    getAppSettings,
    saveAppSettings,
} from './settings';

// Logs API
export {
    getLogs,
    clearLogs,
    exportLogs,
    onLogEntry,
} from './logs';

// Tray API
export {
    updateTray,
    setTrayOptimizing,
    setTrayError,
    getTrayState,
    onTrayOptimize,
    onTrayToggle,
    onTrayStop,
    onTrayPanicReset,
    onTrayNavigate,
    onTrayQuit,
    onTrayQuicBlock,
} from './tray';

// TUN API
export {
    startTun,
    stopTun,
    isTunRunning,
    getTunStatus,
    getTunConfig,
    setTunConfig,
    isTunAvailable,
    restartTun,
} from './tun';

// Monitor API
export {
    startMonitor,
    stopMonitor,
    isMonitorRunning,
    isStrategyDegraded,
    checkStrategyHealth,
    setMonitorUrls,
    setMonitorAutoRestart,
} from './monitor';

// Telemetry API
export {
    setTelemetryEnabled,
    isTelemetryEnabled,
    getTelemetryPendingCount,
    flushTelemetry,
    clearTelemetry,
    reportOptimizationTelemetry,
    reportStrategyUsageTelemetry,
} from './telemetry';

// DNS API
export {
    getDnsSettings,
    setDnsServer,
    applyDnsToSystem,
    restoreSystemDns,
    getWindivertMode,
    setWindivertMode,
} from './dns';

// Config Updater API
export {
    checkConfigUpdates,
    downloadConfigUpdates,
} from './config';

// DPI Testing API
export {
    testStrategyWithDpi,
} from './dpi';

// A/B Testing API
export {
    startABTest,
    getABTestStatus,
    getABTestProgress,
    getABTestResults,
    cancelABTest,
    getActiveABTests,
    onABTestStarted,
    onABTestCompleted,
    onABTestError,
    type ABTestStatus,
    type ABTestStrategyResult,
    type ABTest,
    type ABTestResult,
    type ABTestProgress,
} from './ab-testing';

// Failover API
export {
    getFailoverStatus,
    setFailoverEnabled,
    getFailoverConfig,
    setFailoverConfig,
    triggerManualFailover,
    getLearnedStrategies,
    resetFailoverState,
    isFailoverEnabled,
    getFailoverProgress,
    formatCooldown,
    getFailoverStatusColor,
    type FailoverConfig,
    type FailoverStatus,
} from './failover';

// Crash Reporting API
export {
    setCrashReportingEnabled,
    isCrashReportingEnabled,
    reportCrashError,
    getCrashReportingInfo,
    initFrontendErrorTracking,
    reportError,
    type CrashReportingInfo,
} from './crashReporting';
