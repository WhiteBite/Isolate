# Plugin Security Policy

## Overview

Isolate's plugin system runs in a **sandboxed environment** with restricted access to Tauri commands. This document defines the security policy for plugin command execution.

## Security Principles

### 1. **READ-ONLY Access**
Plugins can only read data, never modify application state. This prevents malicious plugins from:
- Changing user settings
- Applying or stopping strategies
- Modifying network configuration
- Altering system settings

### 2. **NO EXECUTION**
Plugins cannot:
- Spawn external processes
- Execute system commands
- Run arbitrary scripts
- Launch binaries (winws, sing-box, etc.)

### 3. **NO FILE ACCESS**
Plugins cannot:
- Read arbitrary files from disk
- Write files to disk
- Delete files
- Modify configuration files

### 4. **NO NETWORK CHANGES**
Plugins cannot:
- Apply or stop strategies
- Change proxy settings
- Modify DNS configuration
- Enable/disable services
- Change routing rules

### 5. **ISOLATED STORAGE**
Each plugin has its own isolated storage namespace:
- Storage key: `isolate:plugin:{pluginId}`
- No access to other plugins' storage
- No access to application storage
- Stored in browser localStorage

## Allowed Commands

### Backend Status (Read-Only)
```typescript
'is_backend_ready'    // Check if backend is initialized
'get_app_version'     // Get application version
'get_app_status'      // Get overall app status
'is_portable_mode'    // Check if running in portable mode
'is_silent_mode'      // Check if started in silent mode
```

**Why allowed:** These commands provide basic application information without exposing sensitive data or allowing modifications.

### Services (Read-Only)
```typescript
'get_services'               // Get all services (legacy)
'get_registry_services'      // Get services from new registry
'get_service_status'         // Get status of a specific service
'get_services_by_category'   // Get services filtered by category
'get_protected_services'     // Get services with protection enabled
'get_top_services'           // Get most used services
```

**Why allowed:** Plugins need to know which services exist to provide relevant functionality. These commands only read service metadata, not modify them.

### Service Health (Read-Only)
```typescript
'check_single_service'       // Check if a service is accessible
'check_all_registry_services'// Check all services health
'get_service_health_history' // Get health check history
'get_service_health_stats'   // Get health statistics
```

**Why allowed:** Health checks are read-only operations that help plugins understand service availability.

### Strategies (Read-Only)
```typescript
'get_strategies'             // Get all strategies (legacy)
'get_strategies_unified'     // Get unified strategies list
'get_current_strategy'       // Get currently active strategy
'get_all_registered_strategies' // Get all registered strategies
'get_strategies_for_service' // Get strategies for a service
'get_strategies_by_family'   // Get strategies by family
```

**Why allowed:** Plugins need to know available strategies to provide recommendations or display information. No modification allowed.

### Strategy Statistics (Read-Only)
```typescript
'get_strategy_history'       // Get strategy usage history
'get_strategy_statistics'    // Get statistics for a strategy
'get_strategy_metrics'       // Get real-time metrics
'get_metrics_snapshots'      // Get metrics snapshots
```

**Why allowed:** Statistics are historical data that cannot be modified. Useful for analytics plugins.

### Diagnostics (Read-Only)
```typescript
'run_diagnostics'            // Run system diagnostics
'diagnose'                   // Run DPI diagnostics
'check_ipv6'                 // Check IPv6 availability
'check_conflicts'            // Check for software conflicts
```

**Why allowed:** Diagnostic commands only read system state and don't modify anything.

### Settings (Read-Only)
```typescript
'get_settings'               // Get all settings
'get_setting'                // Get specific setting
'get_windivert_mode'         // Get WinDivert mode
'get_dns_settings'           // Get DNS settings
```

**Why allowed:** Plugins can read settings to adapt their behavior, but cannot modify them.

### Plugin-Specific (Isolated)
```typescript
'get_plugins_dir'            // Get plugins directory
'get_all_plugins_cmd'        // Get all plugins
'get_plugin_services'        // Get services provided by plugins
'get_plugin_strategies'      // Get strategies provided by plugins
'get_plugin_storage'         // Get plugin storage (isolated)
'get_plugin_settings'        // Get plugin settings
```

**Why allowed:** These commands operate on plugin-specific data with proper isolation.

## Forbidden Commands

### Strategy Control (FORBIDDEN)
```typescript
❌ 'apply_strategy'          // Apply a strategy
❌ 'stop_strategy'           // Stop active strategy
❌ 'start_strategy'          // Start a strategy
❌ 'test_strategy'           // Test a strategy
❌ 'prewarm_strategy'        // Prewarm a strategy
❌ 'enable_strategy'         // Enable a strategy
❌ 'disable_strategy'        // Disable a strategy
```

**Why forbidden:** These commands modify the active protection state and could be used maliciously to disable protection or apply harmful strategies.

### Settings Modification (FORBIDDEN)
```typescript
❌ 'save_settings'           // Save settings
❌ 'set_setting'             // Set a setting
❌ 'set_windivert_mode'      // Change WinDivert mode
❌ 'set_game_filter_mode'    // Change game filter mode
❌ 'toggle_service'          // Enable/disable service
```

**Why forbidden:** Settings control application behavior. Malicious plugins could disable security features or change critical settings.

### Process Execution (FORBIDDEN)
```typescript
❌ 'execute_plugin_script'   // Execute plugin script
❌ 'execute_plugin_raw'      // Execute raw plugin code
❌ 'start_vless_proxy'       // Start VLESS proxy
❌ 'stop_vless_proxy'        // Stop VLESS proxy
❌ 'start_tun'               // Start TUN mode
❌ 'stop_tun'                // Stop TUN mode
```

**Why forbidden:** Process execution could be used to run arbitrary code or malicious binaries.

### File Operations (FORBIDDEN)
```typescript
❌ 'export_config'           // Export configuration
❌ 'import_config'           // Import configuration
❌ 'export_logs'             // Export logs
❌ 'clear_logs'              // Clear logs
❌ 'download_binaries'       // Download binaries
```

**Why forbidden:** File operations could be used to exfiltrate data or inject malicious files.

### Network Configuration (FORBIDDEN)
```typescript
❌ 'set_system_proxy'        // Set system proxy
❌ 'clear_system_proxy'      // Clear system proxy
❌ 'apply_proxy'             // Apply proxy
❌ 'save_dns_settings'       // Save DNS settings
❌ 'apply_dns_to_system'     // Apply DNS to system
```

**Why forbidden:** Network configuration changes could redirect traffic to malicious servers.

### Automation Control (FORBIDDEN)
```typescript
❌ 'run_optimization_v2'     // Run optimization
❌ 'cancel_optimization_v2'  // Cancel optimization
❌ 'start_domain_monitor'    // Start domain monitor
❌ 'stop_domain_monitor'     // Stop domain monitor
❌ 'start_ai_pilot'          // Start AI Pilot
❌ 'stop_ai_pilot'           // Stop AI Pilot
```

**Why forbidden:** Automation features control application behavior and could be abused.

### Data Modification (FORBIDDEN)
```typescript
❌ 'add_hostlist_domain'     // Add domain to hostlist
❌ 'remove_hostlist_domain'  // Remove domain from hostlist
❌ 'create_hostlist'         // Create hostlist
❌ 'delete_hostlist'         // Delete hostlist
❌ 'clear_strategy_history'  // Clear strategy history
❌ 'clear_telemetry'         // Clear telemetry
```

**Why forbidden:** Data modification could corrupt application state or hide malicious activity.

## Command Validation

### Frontend Validation
The `createSandboxedInvoke` function in `src/lib/plugins/context.ts` validates all commands:

```typescript
if (!ALLOWED_COMMANDS.has(cmd)) {
  throw new Error(`[Plugin ${pluginId}] Command "${cmd}" is not allowed`);
}
```

### Backend Validation
**TODO:** Backend should also validate that commands come from plugins and enforce the same whitelist.

Recommended implementation in `src-tauri/src/commands/mod.rs`:
```rust
pub fn is_plugin_command_allowed(cmd: &str) -> bool {
    matches!(cmd,
        "is_backend_ready" |
        "get_services" |
        "get_strategies" |
        // ... all allowed commands
    )
}
```

## Timeout Policy

All plugin commands have a configurable timeout (default: 10 seconds):

```typescript
const timeout = permissions?.timeout ?? 10000;
```

This prevents:
- Infinite loops in backend commands
- Resource exhaustion
- Denial of service attacks

## Event System Security

### Event Permissions
Plugins can specify allowed events in their manifest:

```json
{
  "permissions": {
    "events": [
      "strategy:*",
      "service:status",
      "plugin:myPlugin:*"
    ]
  }
}
```

### Event Validation
The `createPluginEvents` function validates events:

```typescript
function isEventAllowed(event: string): boolean {
  if (!allowedEvents) return true; // No restrictions
  
  return allowedEvents.some(pattern => {
    if (pattern.endsWith('*')) {
      return event.startsWith(pattern.slice(0, -1));
    }
    return event === pattern;
  });
}
```

### Event Namespacing
All plugin events are prefixed with `plugin:{pluginId}:` to prevent conflicts:

```typescript
const eventPrefix = `plugin:${pluginId}:`;
window.dispatchEvent(
  new CustomEvent(eventPrefix + event, { detail: data })
);
```

## Storage Security

### Isolation
Each plugin has isolated storage:
- Key: `isolate:plugin:{pluginId}`
- Stored in browser localStorage
- No access to other plugins' data
- No access to application data

### Storage API
```typescript
interface PluginStorage {
  get<T>(key: string): Promise<T | null>;
  set<T>(key: string, value: T): Promise<void>;
  delete(key: string): Promise<void>;
  keys(): Promise<string[]>;
}
```

### Size Limits
localStorage has browser-imposed limits (typically 5-10MB per origin). Plugins should:
- Store only essential data
- Clean up old data
- Handle quota exceeded errors

## Best Practices for Plugin Developers

### 1. Request Minimal Permissions
Only request access to events you actually need:

```json
{
  "permissions": {
    "events": ["service:status"],
    "timeout": 5000
  }
}
```

### 2. Handle Command Errors
Always handle command failures gracefully:

```typescript
try {
  const services = await context.invoke('get_services');
} catch (error) {
  console.error('Failed to get services:', error);
  // Fallback behavior
}
```

### 3. Respect Timeouts
Keep command execution fast. If you need long-running operations, break them into smaller chunks.

### 4. Use Storage Wisely
- Store only necessary data
- Clean up old data
- Handle quota errors

### 5. Validate Data
Always validate data received from commands:

```typescript
const services = await context.invoke<Service[]>('get_services');
if (!Array.isArray(services)) {
  throw new Error('Invalid services data');
}
```

## Security Audit Checklist

When reviewing plugin security:

- [ ] All commands in ALLOWED_COMMANDS are read-only
- [ ] No commands allow process execution
- [ ] No commands allow file access
- [ ] No commands allow network configuration changes
- [ ] No commands allow settings modification
- [ ] Event permissions are properly validated
- [ ] Storage is properly isolated
- [ ] Timeouts are enforced
- [ ] Error messages don't leak sensitive information
- [ ] Backend validates plugin commands (TODO)

## Threat Model

### Threats Mitigated
1. **Malicious Strategy Application**: Plugins cannot apply harmful strategies
2. **Settings Tampering**: Plugins cannot modify security settings
3. **Process Execution**: Plugins cannot run arbitrary code
4. **Data Exfiltration**: Plugins cannot access files or export data
5. **Network Hijacking**: Plugins cannot change proxy/DNS settings
6. **Cross-Plugin Attacks**: Storage isolation prevents data access

### Remaining Risks
1. **Information Disclosure**: Plugins can read service/strategy information
2. **Resource Exhaustion**: Plugins could make many read commands (mitigated by timeout)
3. **UI Spoofing**: Plugins could display misleading information (user responsibility)
4. **Backend Bypass**: No backend validation yet (TODO)

## Future Improvements

### 1. Backend Command Validation
Implement command whitelist validation in Rust backend to prevent bypass.

### 2. Rate Limiting
Add rate limiting to prevent plugins from spamming commands:
```typescript
const rateLimiter = new RateLimiter(100, 1000); // 100 calls per second
```

### 3. Permission Levels
Introduce permission levels (Level 1: read-only, Level 2: limited write, Level 3: full access):
```json
{
  "permissions": {
    "level": 1,
    "commands": ["get_services", "get_strategies"]
  }
}
```

### 4. Audit Logging
Log all plugin command invocations for security auditing:
```typescript
logger.audit(`Plugin ${pluginId} invoked ${cmd} with args ${args}`);
```

### 5. Content Security Policy
Implement CSP for plugin iframes to prevent XSS:
```html
<iframe sandbox="allow-scripts" csp="default-src 'self'"></iframe>
```

## References

- [Tauri Security Best Practices](https://tauri.app/v1/guides/security/)
- [OWASP Plugin Security](https://owasp.org/www-community/vulnerabilities/Insecure_Plugin_Architecture)
- [Chrome Extension Security](https://developer.chrome.com/docs/extensions/mv3/security/)

## Changelog

### 2024-01-XX - Initial Policy
- Defined read-only command whitelist
- Implemented event permission system
- Added storage isolation
- Documented security principles
