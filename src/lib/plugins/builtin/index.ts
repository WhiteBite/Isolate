/**
 * Builtin Plugins Registry
 * 
 * These plugins are bundled with Isolate and cannot be uninstalled.
 * They provide core functionality like status widgets, health monitoring, etc.
 */

import type { PluginManifest } from '$lib/types/plugin';
import { registerUIPlugin } from '../loader';

// ============================================================================
// Builtin Plugin Manifests
// ============================================================================

/**
 * Status Widget - Shows current connection status
 */
export const statusWidgetManifest: PluginManifest = {
  id: 'builtin-status',
  name: 'Status Widget',
  version: '1.0.0',
  type: 'ui-widget',
  author: 'Isolate',
  description: 'Shows current connection and protection status',
  icon: '🛡️',
  contributes: {
    widgets: [
      {
        id: 'status-indicator',
        name: 'Status',
        slot: 'dashboard',
        component: 'ui/StatusIndicator.svelte',
        defaultSize: { cols: 1, rows: 1 },
        order: 0
      }
    ]
  }
};

/**
 * Health Widget - Shows system health metrics
 */
export const healthWidgetManifest: PluginManifest = {
  id: 'builtin-health',
  name: 'Health Monitor',
  version: '1.0.0',
  type: 'ui-widget',
  author: 'Isolate',
  description: 'Monitors system and connection health',
  icon: '💚',
  contributes: {
    widgets: [
      {
        id: 'health-monitor',
        name: 'Health',
        slot: 'dashboard',
        component: 'ui/HealthMonitor.svelte',
        defaultSize: { cols: 1, rows: 1 },
        order: 1
      }
    ]
  }
};

// ============================================================================
// Registration
// ============================================================================

/** All builtin plugin manifests */
export const builtinManifests: PluginManifest[] = [
  statusWidgetManifest,
  healthWidgetManifest,
  // Add more builtin plugins here
];

/**
 * Register all builtin plugins
 */
export async function registerBuiltinPlugins(): Promise<void> {
  console.log('[Builtin] Registering builtin plugins...');
  
  for (const manifest of builtinManifests) {
    try {
      await registerUIPlugin(manifest, `builtin/${manifest.id}`);
      console.log(`[Builtin] Registered: ${manifest.id}`);
    } catch (error) {
      console.error(`[Builtin] Failed to register ${manifest.id}:`, error);
    }
  }
  
  console.log(`[Builtin] Registered ${builtinManifests.length} builtin plugins`);
}
