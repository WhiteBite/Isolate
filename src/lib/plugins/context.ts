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

// Commands that plugins are allowed to call
const ALLOWED_COMMANDS = new Set([
  // Read-only status commands
  'is_backend_ready',
  'get_services',
  'get_service_status',
  'get_strategies',
  'get_current_strategy',
  'get_app_status',
  // Plugin-specific commands
  'plugin_http_request',
  'plugin_storage_get',
  'plugin_storage_set',
  // Ping/check commands
  'ping_service',
  'check_service',
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
