/**
 * Plugin Type Definitions for Isolate
 * 
 * Supports three levels of plugins:
 * - Level 1: Declarative (JSON manifests)
 * - Level 2: UI Plugins (Svelte components)
 * - Level 3: Script Plugins (Lua scripts)
 */

// ============================================================================
// Widget Definitions
// ============================================================================

export type PluginSlotLocation = 'dashboard' | 'sidebar' | 'settings' | 'toolbar';

export interface WidgetSize {
  cols: 1 | 2 | 3 | 4;
  rows: 1 | 2;
}

export interface WidgetDefinition {
  /** Unique widget ID within the plugin */
  id: string;
  /** Display name */
  name: string;
  /** Where to render the widget */
  slot: PluginSlotLocation;
  /** Path to .svelte file relative to plugin root */
  component: string;
  /** Default size for dashboard widgets */
  defaultSize?: WidgetSize;
  /** Widget order in slot (lower = first) */
  order?: number;
  /** Icon (emoji or SVG path) */
  icon?: string;
}

// ============================================================================
// Settings Panel Definitions
// ============================================================================

export interface SettingsPanelDefinition {
  /** Unique settings panel ID */
  id: string;
  /** Display name in settings */
  name: string;
  /** Path to .svelte file */
  component: string;
  /** Icon for settings menu */
  icon?: string;
  /** Order in settings list */
  order?: number;
}

// ============================================================================
// Plugin Contributions
// ============================================================================

// Service Definitions (matches Rust ServiceDefinition)
export interface ServiceEndpoint {
  id: string;
  name: string;
  url: string;
  method?: 'GET' | 'HEAD' | 'POST'; // Default: GET
}

export interface ServiceDefinition {
  id: string;
  name: string;
  icon: string;
  category: string;
  description?: string;
  endpoints: ServiceEndpoint[];
}

// Hostlist Definitions (matches Rust HostlistDefinition)
export type HostlistFormat = 'plain' | 'wildcard' | 'regex';

export interface HostlistDefinition {
  /** Unique identifier for the hostlist */
  id: string;
  /** Human-readable name */
  name: string;
  /** Optional description */
  description?: string;
  /** Format of domain entries (default: plain) */
  format?: HostlistFormat;
  /** Inline domain list (for small lists) */
  domains?: string[];
  /** External file path (relative to plugin directory) */
  file?: string;
  /** Remote URL for updates */
  update_url?: string;
  /** Update interval in seconds (0 = no auto-update) */
  update_interval?: number;
  /** Category for grouping (e.g., "social", "video", "gaming") */
  category?: string;
  /** Tags for filtering */
  tags?: string[];
}

// Strategy Definitions (matches Rust StrategyDefinition - legacy simple version)
export interface StrategyDefinition {
  id: string;
  name: string;
  family: string;
  config_file: string;
}

// Full Strategy Definition (matches Rust PluginStrategyDefinition)
export type StrategyFamily = 'zapret' | 'vless' | 'shadowsocks' | 'custom';

export interface StrategyPorts {
  tcp?: string;
  udp?: string;
}

export interface StrategyProfile {
  filter: string;
  hostlist?: string;
  hostlist_exclude?: string;
  hostlist_domains?: string;
  desync: string;
  repeats?: number;
  split_seqovl?: number;
  split_pos?: string;
  fooling?: string;
  fake_tls?: string;
  fake_quic?: string;
  ttl?: number;
  autottl?: string;
  extra_args?: string[];
}

export interface PluginStrategyConfig {
  args?: string[];
  hostlist?: string;
  ports?: StrategyPorts;
  profiles?: StrategyProfile[];
}

export interface PluginStrategyDefinition {
  id: string;
  name: string;
  description?: string;
  family?: StrategyFamily;
  engine?: string; // Default: winws
  target_services?: string[];
  priority?: number;
  config: PluginStrategyConfig;
  author?: string;
  label?: string;
  source_plugin?: string;
}

// Plugin Contributions (matches Rust PluginContributes)
// This is the main contract for what a plugin can contribute
export interface PluginContributes {
  /** Service definitions for service-checker plugins */
  services?: ServiceDefinition[];
  /** Hostlist definitions */
  hostlists?: HostlistDefinition[];
  /** Strategy definitions (legacy simple version) */
  strategies?: StrategyDefinition[];
  /** Dashboard/sidebar/toolbar widgets (UI plugins only) */
  widgets?: WidgetDefinition[];
  /** Settings panels (UI plugins only) */
  settings?: SettingsPanelDefinition[];
}

// ============================================================================
// Plugin Permissions (matches Rust Permissions)
// ============================================================================

export interface PluginPermissions {
  /** Allowed HTTP domains (for network requests) */
  http?: string[];
  /** Access to filesystem */
  filesystem?: boolean;
  /** Access to process spawning */
  process?: boolean;
  /** Access to plugin-scoped storage (UI plugins) */
  storage?: boolean;
  /** Allowed events to emit/listen (UI plugins) */
  events?: string[];
  /** Max execution time in ms (script plugins) */
  timeout?: number;
  /** Max memory in bytes (script plugins) */
  memory?: number;
}

// ============================================================================
// Plugin Manifest
// ============================================================================

export type PluginType = 'service-checker' | 'strategy-provider' | 'hostlist-provider' | 'ui-widget' | 'script-plugin';

// Plugin Manifest (matches Rust PluginManifest)
export interface PluginManifest {
  /** Unique plugin ID (kebab-case) */
  id: string;
  /** Display name */
  name: string;
  /** Semantic version */
  version: string;
  /** Author name or organization */
  author: string;
  /** Short description */
  description?: string;
  /** Plugin type */
  type: PluginType;
  /** Legacy: Service definition (for service-checker type) - use contributes.services instead */
  service?: ServiceDefinition;
  /** Legacy: Strategy definition (for strategy-provider type) - use contributes.strategies instead */
  strategy?: StrategyDefinition;
  /** Legacy: Hostlist definition (for hostlist-provider type) - use contributes.hostlists instead */
  hostlist?: HostlistDefinition;
  /** What the plugin contributes (unified system) */
  contributes?: PluginContributes;
  /** Required permissions */
  permissions?: PluginPermissions;
  /** Icon (emoji or path to image) */
  icon?: string;
  /** Minimum Isolate version required */
  minAppVersion?: string;
  /** Plugin homepage URL */
  homepage?: string;
  /** Plugin repository URL */
  repository?: string;
}

// ============================================================================
// Plugin Context (passed to UI components)
// ============================================================================

export interface PluginStorage {
  /** Get a value from plugin-scoped storage */
  get<T = unknown>(key: string): Promise<T | null>;
  /** Set a value in plugin-scoped storage */
  set<T = unknown>(key: string, value: T): Promise<void>;
  /** Delete a value from storage */
  delete(key: string): Promise<void>;
  /** Get all keys */
  keys(): Promise<string[]>;
}

export interface PluginEvents {
  /** Emit an event */
  emit(event: string, data?: unknown): void;
  /** Subscribe to an event, returns unsubscribe function */
  on(event: string, handler: (data: unknown) => void): () => void;
  /** Subscribe to an event once */
  once(event: string, handler: (data: unknown) => void): () => void;
}

export interface PluginContext {
  /** Plugin ID */
  pluginId: string;
  /** Plugin version */
  pluginVersion: string;
  /** Plugin-scoped storage */
  storage: PluginStorage;
  /** Event system */
  events: PluginEvents;
  /** Invoke Tauri commands (sandboxed) */
  invoke: <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
  /** Plugin manifest */
  manifest: PluginManifest;
}

// ============================================================================
// Loaded Plugin State
// ============================================================================

export interface LoadedUIPlugin {
  manifest: PluginManifest;
  /** Plugin root path */
  path: string;
  /** Whether plugin is enabled */
  enabled: boolean;
  /** Loaded Svelte components by path */
  components: Map<string, unknown>;
  /** Error if loading failed */
  error?: string;
}

// ============================================================================
// Component Props
// ============================================================================

export interface PluginComponentProps {
  /** Plugin context with APIs */
  context: PluginContext;
  /** Additional props passed from slot */
  [key: string]: unknown;
}
