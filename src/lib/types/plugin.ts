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

export interface ServiceEndpoint {
  id?: string;
  url: string;
  method: 'GET' | 'HEAD' | 'POST';
  expectedStatus?: number;
  timeout?: number;
}

export interface ServiceDefinition {
  id: string;
  name: string;
  icon: string;
  category?: string;
  endpoints: ServiceEndpoint[];
}

export interface UIPluginContributes {
  /** Dashboard/sidebar/toolbar widgets */
  widgets?: WidgetDefinition[];
  /** Settings panels */
  settings?: SettingsPanelDefinition[];
  /** Services for checking */
  services?: ServiceDefinition[];
}

// ============================================================================
// Plugin Permissions
// ============================================================================

export interface PluginPermissions {
  /** Allowed HTTP domains (supports wildcards like *.discord.com) */
  http?: string[];
  /** Access to plugin-scoped storage */
  storage?: boolean;
  /** Allowed events to emit/listen */
  events?: string[];
  /** Max execution time in ms */
  timeout?: number;
  /** Max memory in bytes */
  memory?: number;
}

// ============================================================================
// Plugin Manifest
// ============================================================================

export type PluginType = 'service-checker' | 'ui-plugin' | 'script-plugin' | 'hostlist-provider' | 'strategy-config';

export interface PluginManifest {
  /** Unique plugin ID (kebab-case) */
  id: string;
  /** Display name */
  name: string;
  /** Semantic version */
  version: string;
  /** Plugin type */
  type: PluginType;
  /** Author name or organization */
  author?: string;
  /** Short description */
  description?: string;
  /** Icon (emoji or path to image) */
  icon?: string;
  /** What the plugin contributes */
  contributes: UIPluginContributes;
  /** Required permissions */
  permissions?: PluginPermissions;
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
