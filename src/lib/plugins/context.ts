/**
 * Plugin Context Factory
 * 
 * Creates isolated context for each plugin with:
 * - Scoped storage
 * - Event system
 * - Sandboxed Tauri invoke
 */

import { invoke } from '@tauri-apps/api/core';
import type { 
  PluginContext, 
  PluginStorage, 
  PluginEvents, 
  PluginManifest,
  PluginPermissions 
} from '$lib/types/plugin';

// ============================================================================
// Storage Implementation
// ============================================================================

function createPluginStorage(pluginId: string): PluginStorage {
  const storageKey = `isolate:plugin:${pluginId}`;
  
  // Get all plugin data from localStorage
  function getPluginData(): Record<string, unknown> {
    try {
      const data = localStorage.getItem(storageKey);
      return data ? JSON.parse(data) : {};
    } catch {
      return {};
    }
  }
  
  // Save all plugin data to localStorage
  function savePluginData(data: Record<string, unknown>): void {
    localStorage.setItem(storageKey, JSON.stringify(data));
  }
  
  return {
    async get<T = unknown>(key: string): Promise<T | null> {
      const data = getPluginData();
      return (data[key] as T) ?? null;
    },
    
    async set<T = unknown>(key: string, value: T): Promise<void> {
      const data = getPluginData();
      data[key] = value;
      savePluginData(data);
    },
    
    async delete(key: string): Promise<void> {
      const data = getPluginData();
      delete data[key];
      savePluginData(data);
    },
    
    async keys(): Promise<string[]> {
      const data = getPluginData();
      return Object.keys(data);
    }
  };
}

// ============================================================================
// Events Implementation
// ============================================================================

function createPluginEvents(pluginId: string, permissions?: PluginPermissions): PluginEvents {
  const eventPrefix = `plugin:${pluginId}:`;
  const allowedEvents = permissions?.events;
  
  // Check if event is allowed
  function isEventAllowed(event: string): boolean {
    if (!allowedEvents) return true; // No restrictions
    
    return allowedEvents.some(pattern => {
      if (pattern.endsWith('*')) {
        return event.startsWith(pattern.slice(0, -1));
      }
      return event === pattern;
    });
  }
  
  return {
    emit(event: string, data?: unknown): void {
      if (!isEventAllowed(event)) {
        console.warn(`[Plugin ${pluginId}] Event "${event}" not allowed`);
        return;
      }
      
      window.dispatchEvent(
        new CustomEvent(eventPrefix + event, { detail: data })
      );
    },
    
    on(event: string, handler: (data: unknown) => void): () => void {
      if (!isEventAllowed(event)) {
        console.warn(`[Plugin ${pluginId}] Event "${event}" not allowed`);
        return () => {};
      }
      
      const listener = (e: Event) => {
        handler((e as CustomEvent).detail);
      };
      
      window.addEventListener(eventPrefix + event, listener);
      return () => window.removeEventListener(eventPrefix + event, listener);
    },
    
    once(event: string, handler: (data: unknown) => void): () => void {
      if (!isEventAllowed(event)) {
        console.warn(`[Plugin ${pluginId}] Event "${event}" not allowed`);
        return () => {};
      }
      
      const listener = (e: Event) => {
        handler((e as CustomEvent).detail);
        window.removeEventListener(eventPrefix + event, listener);
      };
      
      window.addEventListener(eventPrefix + event, listener);
      return () => window.removeEventListener(eventPrefix + event, listener);
    }
  };
}

// ============================================================================
// Sandboxed Invoke
// ============================================================================

/**
 * Plugin Command Security Policy
 * ============================================================================
 * 
 * Plugins run in a sandboxed environment with RESTRICTED access to Tauri commands.
 * This whitelist defines the ONLY commands that plugins can invoke.
 * 
 * SECURITY PRINCIPLES:
 * ────────────────────
 * 1. READ-ONLY: Only commands that read data, NEVER modify state
 * 2. NO EXECUTION: No process spawning, script execution, or system commands
 * 3. NO FILE ACCESS: No file read/write/delete operations
 * 4. NO NETWORK CHANGES: No strategy application, proxy changes, DNS changes
 * 5. ISOLATED STORAGE: Plugins have their own storage namespace
 * 
 * THREAT MODEL:
 * ─────────────
 * Malicious plugins could attempt to:
 * - Disable DPI bypass protection (apply_strategy, stop_strategy)
 * - Exfiltrate user data (export_config, get_logs with sensitive data)
 * - Execute arbitrary code (execute_plugin_script, execute_plugin_raw)
 * - Modify system settings (save_settings, set_system_proxy)
 * - Cause system instability (start multiple winws processes → BSOD)
 * 
 * RISK LEVELS:
 * ────────────
 * 🟢 LOW RISK: Read-only status checks, metadata queries
 * 🟡 MEDIUM RISK: Health checks (network requests), diagnostics
 * 🔴 HIGH RISK (FORBIDDEN): Any state modification, process execution
 * 
 * REVIEW CHECKLIST:
 * ─────────────────
 * Before adding a new command, verify:
 * □ Command is read-only (no side effects)
 * □ Command doesn't expose sensitive data (passwords, tokens)
 * □ Command doesn't allow process execution
 * □ Command doesn't modify files or settings
 * □ Command doesn't change network configuration
 * 
 * @see docs/plugin-security.md for full documentation
 */

// Commands that plugins are allowed to call
const ALLOWED_COMMANDS = new Set([
  // ============================================================================
  // BACKEND STATUS (Read-Only)
  // ============================================================================
  'is_backend_ready',           // Check if backend is initialized
  'get_app_version',            // Get application version
  'get_app_status',             // Get overall app status
  'is_portable_mode',           // Check if running in portable mode
  'is_silent_mode',             // Check if started in silent mode
  
  // ============================================================================
  // SERVICES (Read-Only)
  // ============================================================================
  'get_services',               // Get all services (legacy)
  'get_registry_services',      // Get services from new registry
  'get_service_status',         // Get status of a specific service
  'get_services_by_category',   // Get services filtered by category
  'get_protected_services',     // Get services with protection enabled
  'get_top_services',           // Get most used services
  
  // ============================================================================
  // SERVICE HEALTH (Read-Only)
  // ============================================================================
  'check_single_service',       // Check if a service is accessible
  'check_all_registry_services',// Check all services health
  'get_service_health_history', // Get health check history
  'get_service_health_stats',   // Get health statistics
  'get_all_services_health_history', // Get all services health history
  
  // ============================================================================
  // STRATEGIES (Read-Only)
  // ============================================================================
  'get_strategies',             // Get all strategies (legacy)
  'get_strategies_unified',     // Get unified strategies list
  'get_current_strategy',       // Get currently active strategy
  'get_strategy_info',          // Get detailed strategy info
  'get_all_registered_strategies', // Get all registered strategies
  'get_strategies_for_service', // Get strategies for a service
  'get_strategies_by_family',   // Get strategies by family (zapret/vless)
  'get_registered_strategy',    // Get specific registered strategy
  'get_strategy_registry_stats',// Get strategy registry statistics
  
  // ============================================================================
  // STRATEGY STATISTICS (Read-Only)
  // ============================================================================
  'get_strategy_history',       // Get strategy usage history
  'get_strategy_statistics',    // Get statistics for a strategy
  'get_all_strategy_statistics',// Get all strategies statistics
  'get_automation_history',     // Get automation history
  'get_strategy_metrics',       // Get real-time metrics
  'get_strategy_metrics_history', // Get metrics history
  'get_metrics_snapshots',      // Get metrics snapshots
  'get_aggregated_metrics',     // Get aggregated metrics
  'is_strategy_prewarmed',      // Check if strategy is prewarmed
  'get_prewarmed_strategies',   // Get list of prewarmed strategies
  
  // ============================================================================
  // COMPOSITION (Read-Only)
  // ============================================================================
  'get_composition_rules',      // Get composition rules
  'get_compositions',           // Get all compositions
  'get_active_composition',     // Get active composition
  'get_strategy_for_service',   // Get strategy for service from composition
  
  // ============================================================================
  // AI PILOT (Read-Only Status)
  // ============================================================================
  'get_ai_pilot_status',        // Get AI Pilot status
  'get_ai_pilot_history',       // Get AI Pilot action history
  
  // ============================================================================
  // AUTOMATION (Read-Only Status)
  // ============================================================================
  'is_optimization_v2_running', // Check if optimization is running
  'is_domain_monitor_running',  // Check if domain monitor is running
  'get_domain_status',          // Get status of a domain
  'get_all_domain_statuses',    // Get all domain statuses
  'get_blocked_strategies',     // Get blocked strategies for domain
  'get_locked_strategy',        // Get locked strategy for domain
  
  // ============================================================================
  // A/B TESTING (Read-Only Status)
  // ============================================================================
  'get_ab_test_status',         // Get A/B test status
  'get_ab_test_progress',       // Get A/B test progress
  'get_ab_test_results',        // Get A/B test results
  'get_active_ab_tests',        // Get active A/B tests
  'list_ab_tests',              // List all A/B tests
  'get_all_ab_test_results',    // Get all A/B test results
  'get_ab_test_history',        // Get A/B test history
  'get_ab_test_statistics',     // Get A/B test statistics
  
  // ============================================================================
  // MONITOR (Read-Only Status)
  // ============================================================================
  'is_monitor_running',         // Check if monitor is running
  'is_strategy_degraded',       // Check if strategy is degraded
  'get_monitor_config',         // Get monitor configuration
  
  // ============================================================================
  // PROXY (Read-Only)
  // ============================================================================
  'get_proxies',                // Get all proxies
  'get_proxy_chains',           // Get proxy chains
  'get_proxy_chain_stats',      // Get proxy chain statistics
  'is_system_proxy_set',        // Check if system proxy is set
  
  // ============================================================================
  // VLESS (Read-Only Status)
  // ============================================================================
  'get_vless_configs',          // Get VLESS configurations
  'get_vless_status',           // Get VLESS proxy status
  'get_all_vless_status',       // Get all VLESS proxies status
  'is_singbox_available',       // Check if sing-box is available
  'get_singbox_version',        // Get sing-box version
  
  // ============================================================================
  // TUN MODE (Read-Only Status)
  // ============================================================================
  'is_tun_running',             // Check if TUN is running
  'get_tun_status',             // Get TUN status
  'get_tun_config',             // Get TUN configuration
  'is_tun_available',           // Check if TUN is available
  
  // ============================================================================
  // HOSTLISTS (Read-Only)
  // ============================================================================
  'get_hostlists',              // Get all hostlists
  'get_hostlist',               // Get specific hostlist
  'get_hostlist_info',          // Get hostlist info
  'get_plugin_hostlists',       // Get plugin hostlists
  'get_plugin_hostlist',        // Get specific plugin hostlist
  'get_plugin_hostlist_domains',// Get domains from plugin hostlist
  'get_all_plugin_domains',     // Get all plugin domains
  'check_domain_in_hostlists',  // Check if domain is in hostlists
  'find_matching_hostlists',    // Find hostlists matching criteria
  'get_hostlists_by_category',  // Get hostlists by category
  'get_hostlists_by_plugin',    // Get hostlists by plugin
  'get_hostlist_registry_stats',// Get hostlist registry stats
  
  // ============================================================================
  // IPSET (Read-Only)
  // ============================================================================
  'get_ipset_info',             // Get ipset information
  'get_ipset_stats',            // Get ipset statistics
  'get_ipset_mode',             // Get ipset mode
  'get_ipset_sources',          // Get ipset sources
  
  // ============================================================================
  // ROUTING (Read-Only)
  // ============================================================================
  'get_routing_rules',          // Get routing rules
  'get_domain_routes',          // Get domain routes
  'get_app_routes',             // Get app routes
  'get_installed_apps',         // Get installed apps
  
  // ============================================================================
  // DIAGNOSTICS (Read-Only)
  // ============================================================================
  'run_diagnostics',            // Run system diagnostics
  'diagnose',                   // Run DPI diagnostics
  'diagnose_dual_stack',        // Run dual-stack diagnostics
  'check_ipv6',                 // Check IPv6 availability
  'check_conflicts',            // Check for software conflicts
  
  // ============================================================================
  // BINARIES (Read-Only)
  // ============================================================================
  'check_binaries',             // Check binaries availability
  'get_binaries_dir',           // Get binaries directory
  'verify_binaries_integrity',  // Verify binaries integrity
  'is_admin',                   // Check if running as admin
  
  // ============================================================================
  // SETTINGS (Read-Only)
  // ============================================================================
  'get_settings',               // Get all settings
  'get_services_settings',      // Get services settings
  'get_setting',                // Get specific setting
  'get_windivert_mode',         // Get WinDivert mode
  'get_game_filter_mode',       // Get game filter mode
  'get_dns_settings',           // Get DNS settings
  'get_tcp_timestamps_status',  // Get TCP timestamps status
  'get_failover_status',        // Get failover status
  'get_failover_config',        // Get failover configuration
  'get_learned_strategies',     // Get learned strategies
  'get_current_profile',        // Get current profile
  'get_all_profiles',           // Get all profiles
  'get_engine_mode',            // Get engine mode
  
  // ============================================================================
  // TELEMETRY (Read-Only Status)
  // ============================================================================
  'is_telemetry_enabled',       // Check if telemetry is enabled
  'get_telemetry_pending_count',// Get pending telemetry count
  'is_crash_reporting_enabled', // Check if crash reporting is enabled
  'get_crash_reporting_info',   // Get crash reporting info
  
  // ============================================================================
  // AUTORUN (Read-Only Status)
  // ============================================================================
  'get_autorun_status',         // Get autorun status
  
  // ============================================================================
  // HOSTS (Read-Only Status)
  // ============================================================================
  'get_hosts_status',           // Get hosts file status
  
  // ============================================================================
  // QUIC (Read-Only Status)
  // ============================================================================
  'is_quic_blocked',            // Check if QUIC is blocked
  
  // ============================================================================
  // LOGS (Read-Only)
  // ============================================================================
  'get_logs',                   // Get application logs
  
  // ============================================================================
  // TRAY (Read-Only Status)
  // ============================================================================
  'get_tray_state',             // Get tray state
  
  // ============================================================================
  // DASHBOARD (Read-Only)
  // ============================================================================
  'get_live_connections',       // Get live connections
  'get_traffic_stats',          // Get traffic statistics
  'get_protection_issues',      // Get protection issues
  
  // ============================================================================
  // PROVIDERS (Read-Only)
  // ============================================================================
  'get_providers',              // Get providers
  'get_provider_recommendations', // Get provider recommendations
  'is_strategy_recommended',    // Check if strategy is recommended
  
  // ============================================================================
  // LIBRARY (Read-Only)
  // ============================================================================
  'get_library_rules',          // Get library rules
  'get_library_rule',           // Get specific library rule
  'get_library_presets',        // Get library presets
  'get_library_preset',         // Get specific library preset
  'get_rules_for_service',      // Get rules for service
  'get_rules_by_method',        // Get rules by method
  
  // ============================================================================
  // PLUGIN-SPECIFIC COMMANDS (Isolated)
  // ============================================================================
  'get_plugins_dir',            // Get plugins directory
  'get_all_plugins_cmd',        // Get all plugins
  'get_plugin_services',        // Get services provided by plugins
  'get_plugin_strategies',      // Get strategies provided by plugins
  'check_plugin_service',       // Check plugin service health
  'check_all_plugin_services',  // Check all plugin services
  'list_plugin_scripts',        // List plugin scripts
  'get_plugin_storage',         // Get plugin storage (isolated)
  'get_plugin_enabled',         // Get plugin enabled state
  'get_all_plugin_states',      // Get all plugin states
  'get_plugin_settings',        // Get plugin settings
  'get_all_plugin_settings',    // Get all plugin settings
  
  // ============================================================================
  // RESOURCE MONITORING (Read-Only)
  // ============================================================================
  'get_process_usage',          // Get process resource usage
  'get_multiple_process_usage', // Get multiple processes usage
  'get_default_resource_limits',// Get default resource limits
  'get_recommended_resource_limits', // Get recommended limits
  
  // ============================================================================
  // CONFIG UPDATES (Read-Only Status)
  // ============================================================================
  'check_config_updates',       // Check for config updates
  'check_github_updates',       // Check for GitHub updates
  'check_hostlist_updates',     // Check for hostlist updates
]);

function createSandboxedInvoke(
  pluginId: string, 
  permissions?: PluginPermissions
): <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T> {
  const timeout = permissions?.timeout ?? 10000;
  
  return async function sandboxedInvoke<T = unknown>(
    cmd: string, 
    args?: Record<string, unknown>
  ): Promise<T> {
    // Check if command is allowed
    if (!ALLOWED_COMMANDS.has(cmd)) {
      throw new Error(`[Plugin ${pluginId}] Command "${cmd}" is not allowed`);
    }
    
    // Add plugin context to args
    const enrichedArgs = {
      ...args,
      __pluginId: pluginId
    };
    
    // Execute with timeout
    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(() => reject(new Error(`Command "${cmd}" timed out`)), timeout);
    });
    
    try {
      return await Promise.race([
        invoke<T>(cmd, enrichedArgs),
        timeoutPromise
      ]);
    } catch (error) {
      console.error(`[Plugin ${pluginId}] Command "${cmd}" failed:`, error);
      throw error;
    }
  };
}

// ============================================================================
// Context Factory
// ============================================================================

/** Cache of created contexts */
const contextCache = new Map<string, PluginContext>();

/**
 * Create a plugin context with isolated APIs
 */
export function createPluginContext(manifest: PluginManifest): PluginContext {
  const { id: pluginId, version: pluginVersion, permissions } = manifest;
  
  // Return cached context if exists
  const cached = contextCache.get(pluginId);
  if (cached) {
    return cached;
  }
  
  const context: PluginContext = {
    pluginId,
    pluginVersion,
    manifest,
    storage: createPluginStorage(pluginId),
    events: createPluginEvents(pluginId, permissions),
    invoke: createSandboxedInvoke(pluginId, permissions)
  };
  
  // Cache the context
  contextCache.set(pluginId, context);
  
  return context;
}

/**
 * Get existing context for a plugin
 */
export function getPluginContext(pluginId: string): PluginContext | undefined {
  return contextCache.get(pluginId);
}

/**
 * Destroy plugin context and cleanup
 */
export function destroyPluginContext(pluginId: string): void {
  contextCache.delete(pluginId);
}

/**
 * Clear all plugin contexts
 */
export function clearAllContexts(): void {
  contextCache.clear();
}
