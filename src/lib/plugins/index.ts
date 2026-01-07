/**
 * Plugin System Exports
 * 
 * Level 1: Declarative plugins (JSON manifests)
 * Level 2: UI plugins (Svelte components)
 * Level 3: Script plugins (Lua - future)
 */

// Context API
export { 
  createPluginContext, 
  getPluginContext, 
  destroyPluginContext,
  clearAllContexts 
} from './context';

// Component Loader
export {
  loadPluginComponent,
  loadPluginComponents,
  registerUIPlugin,
  unregisterUIPlugin,
  setPluginEnabled,
  getLoadedPlugin,
  getWidgetComponent,
  clearAllPlugins,
  initializePluginLoader,
  uiPlugins,
  pluginErrors,
  widgetsBySlot,
  type LoadedComponent,
  type PluginLoadError
} from './loader';

// Scanner
export {
  discoverPlugins,
  loadDiscoveredPlugins,
  reloadPlugin,
  initializePlugins
} from './scanner';

// Builtin plugins
export { registerBuiltinPlugins, builtinManifests } from './builtin';

// Re-export types
export type {
  PluginManifest,
  PluginType,
  PluginContext,
  PluginStorage,
  PluginEvents,
  PluginPermissions,
  WidgetDefinition,
  WidgetSize,
  SettingsPanelDefinition,
  UIPluginContributes,
  ServiceDefinition,
  ServiceEndpoint,
  LoadedUIPlugin,
  PluginSlotLocation,
  PluginComponentProps
} from '$lib/types/plugin';
