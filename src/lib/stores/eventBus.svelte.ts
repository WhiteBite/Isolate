/**
 * Centralized Event Bus for application-wide event communication
 * Uses typed events for type safety
 */

// ============================================================================
// Event Types
// ============================================================================

/**
 * All supported event types in the application
 */
export type EventType =
  // Traffic & Connection events
  | 'traffic:update'
  | 'connection:opened'
  | 'connection:closed'
  // Issue events
  | 'issue:detected'
  | 'issue:resolved'
  // Strategy events
  | 'strategy:changed'
  | 'strategy:applied'
  | 'strategy:stopped'
  // AI Pilot events
  | 'ai_pilot:started'
  | 'ai_pilot:stopped'
  | 'ai_pilot:check_complete'
  | 'ai_pilot:action'
  // Library events
  | 'library:rule_added'
  | 'library:rule_removed'
  | 'library:rule_updated'
  | 'library:method_changed'
  // Proxy & VPN events
  | 'proxy:chain_activated'
  | 'proxy:chain_deactivated'
  | 'proxy:imported'
  | 'proxy:test_complete'
  // Ipset events
  | 'ipset:updated'
  | 'ipset:mode_changed';

// ============================================================================
// Payload Types
// ============================================================================

/**
 * AI Pilot check result type
 */
export type AiPilotCheckResult = 'success' | 'failed' | 'timeout';

/**
 * AI Pilot action types
 */
export type AiPilotActionType = 'switch_strategy' | 'restart' | 'fallback';

/**
 * Issue severity levels
 */
export type IssueSeverity = 'warning' | 'error';

/**
 * Payload types for each event
 */
export interface EventPayload {
  // Traffic & Connection events
  'traffic:update': { 
    download: number; 
    upload: number; 
  };
  'connection:opened': { 
    serviceId: string; 
    method: string; 
  };
  'connection:closed': { 
    serviceId: string; 
  };
  
  // Issue events
  'issue:detected': { 
    id: string; 
    message: string; 
    severity: IssueSeverity; 
  };
  'issue:resolved': { 
    id: string; 
  };
  
  // Strategy events
  'strategy:changed': { 
    serviceId: string; 
    strategyId: string; 
  };
  /** Emitted when a strategy is applied/activated */
  'strategy:applied': {
    strategyId: string;
    serviceId?: string;
    timestamp: number;
  };
  /** Emitted when a strategy is stopped/deactivated */
  'strategy:stopped': {
    strategyId?: string;
    serviceId?: string;
    timestamp: number;
  };
  
  // AI Pilot events
  /** Emitted when AI Pilot monitoring starts */
  'ai_pilot:started': { 
    timestamp: number; 
  };
  /** Emitted when AI Pilot monitoring stops */
  'ai_pilot:stopped': { 
    timestamp: number; 
  };
  /** Emitted when AI Pilot completes a service check */
  'ai_pilot:check_complete': { 
    service_id: string;
    result: AiPilotCheckResult;
    latency?: number;
    strategy_id?: string;
  };
  /** Emitted when AI Pilot takes an action */
  'ai_pilot:action': {
    action_type: AiPilotActionType;
    service_id: string;
    from_strategy?: string;
    to_strategy?: string;
    reason: string;
  };
  
  // Library events
  /** Emitted when a new rule is added to the library */
  'library:rule_added': { 
    rule_id: string; 
    pattern: string; 
    method: string; 
  };
  /** Emitted when a rule is removed from the library */
  'library:rule_removed': { 
    rule_id: string; 
  };
  /** Emitted when a rule is updated */
  'library:rule_updated': { 
    rule_id: string; 
    changes: Record<string, unknown>; 
  };
  /** Emitted when service method is changed */
  'library:method_changed': { 
    service_id: string; 
    old_method: string; 
    new_method: string; 
  };
  
  // Proxy & VPN events
  /** Emitted when a proxy chain is activated */
  'proxy:chain_activated': { 
    chain_id: string; 
    name: string; 
  };
  /** Emitted when a proxy chain is deactivated */
  'proxy:chain_deactivated': { 
    chain_id: string; 
  };
  /** Emitted when proxies are imported */
  'proxy:imported': { 
    count: number; 
    source: string; 
  };
  /** Emitted when proxy test completes */
  'proxy:test_complete': { 
    proxy_id: string; 
    latency: number; 
    success: boolean; 
  };
  
  // Ipset events
  /** Emitted when IP set is updated */
  'ipset:updated': { 
    ip_count: number; 
    source: string; 
  };
  /** Emitted when IP set mode changes */
  'ipset:mode_changed': { 
    old_mode: string; 
    new_mode: string; 
  };
}

// Handler type for event listeners
type EventHandler<T extends EventType> = (payload: EventPayload[T]) => void;

// Generic handler for internal storage
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyHandler = (payload: any) => void;

// ============================================================================
// EventBus Class
// ============================================================================

/**
 * Type-safe Event Bus implementation
 * Supports on, off, emit, and once methods
 */
class EventBus {
  private listeners: Map<EventType, Set<AnyHandler>> = new Map();
  private onceListeners: Map<EventType, Set<AnyHandler>> = new Map();

  /**
   * Subscribe to an event
   * @param event - Event type to listen for
   * @param handler - Callback function to execute when event is emitted
   * @returns Unsubscribe function
   */
  on<T extends EventType>(event: T, handler: EventHandler<T>): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    
    this.listeners.get(event)!.add(handler as AnyHandler);
    
    // Return unsubscribe function
    return () => this.off(event, handler);
  }

  /**
   * Unsubscribe from an event
   * @param event - Event type to unsubscribe from
   * @param handler - Handler function to remove
   */
  off<T extends EventType>(event: T, handler: EventHandler<T>): void {
    const handlers = this.listeners.get(event);
    if (handlers) {
      handlers.delete(handler as AnyHandler);
      if (handlers.size === 0) {
        this.listeners.delete(event);
      }
    }

    // Also remove from once listeners if present
    const onceHandlers = this.onceListeners.get(event);
    if (onceHandlers) {
      onceHandlers.delete(handler as AnyHandler);
      if (onceHandlers.size === 0) {
        this.onceListeners.delete(event);
      }
    }
  }

  /**
   * Emit an event with payload
   * @param event - Event type to emit
   * @param payload - Data to pass to handlers
   */
  emit<T extends EventType>(event: T, payload: EventPayload[T]): void {
    // Call regular listeners
    const handlers = this.listeners.get(event);
    if (handlers) {
      handlers.forEach((handler) => {
        try {
          handler(payload);
        } catch (error) {
          console.error(`[EventBus] Error in handler for "${event}":`, error);
        }
      });
    }

    // Call once listeners and remove them
    const onceHandlers = this.onceListeners.get(event);
    if (onceHandlers) {
      onceHandlers.forEach((handler) => {
        try {
          handler(payload);
        } catch (error) {
          console.error(`[EventBus] Error in once handler for "${event}":`, error);
        }
      });
      this.onceListeners.delete(event);
    }
  }

  /**
   * Subscribe to an event for a single emission
   * @param event - Event type to listen for
   * @param handler - Callback function to execute once
   * @returns Unsubscribe function
   */
  once<T extends EventType>(event: T, handler: EventHandler<T>): () => void {
    if (!this.onceListeners.has(event)) {
      this.onceListeners.set(event, new Set());
    }
    
    this.onceListeners.get(event)!.add(handler as AnyHandler);
    
    // Return unsubscribe function
    return () => this.off(event, handler);
  }

  /**
   * Remove all listeners for a specific event or all events
   * @param event - Optional event type to clear. If not provided, clears all.
   */
  clear(event?: EventType): void {
    if (event) {
      this.listeners.delete(event);
      this.onceListeners.delete(event);
    } else {
      this.listeners.clear();
      this.onceListeners.clear();
    }
  }

  /**
   * Get the number of listeners for an event
   * @param event - Event type to check
   * @returns Number of listeners
   */
  listenerCount(event: EventType): number {
    const regular = this.listeners.get(event)?.size ?? 0;
    const once = this.onceListeners.get(event)?.size ?? 0;
    return regular + once;
  }

  /**
   * Check if there are any listeners for an event
   * @param event - Event type to check
   * @returns True if there are listeners
   */
  hasListeners(event: EventType): boolean {
    return this.listenerCount(event) > 0;
  }

  // ==========================================================================
  // AI Pilot Helper Methods
  // ==========================================================================

  /**
   * Subscribe to AI Pilot started event
   * @param handler - Callback when AI Pilot starts
   * @returns Unsubscribe function
   */
  onAiPilotStarted(handler: EventHandler<'ai_pilot:started'>): () => void {
    return this.on('ai_pilot:started', handler);
  }

  /**
   * Subscribe to AI Pilot stopped event
   * @param handler - Callback when AI Pilot stops
   * @returns Unsubscribe function
   */
  onAiPilotStopped(handler: EventHandler<'ai_pilot:stopped'>): () => void {
    return this.on('ai_pilot:stopped', handler);
  }

  /**
   * Subscribe to AI Pilot check complete event
   * @param handler - Callback when a check completes
   * @returns Unsubscribe function
   */
  onAiPilotCheckComplete(handler: EventHandler<'ai_pilot:check_complete'>): () => void {
    return this.on('ai_pilot:check_complete', handler);
  }

  /**
   * Subscribe to AI Pilot action event
   * @param handler - Callback when AI Pilot takes an action
   * @returns Unsubscribe function
   */
  onAiPilotAction(handler: EventHandler<'ai_pilot:action'>): () => void {
    return this.on('ai_pilot:action', handler);
  }

  // ==========================================================================
  // Library Helper Methods
  // ==========================================================================

  /**
   * Subscribe to library rule added event
   * @param handler - Callback when a rule is added
   * @returns Unsubscribe function
   */
  onLibraryRuleAdded(handler: EventHandler<'library:rule_added'>): () => void {
    return this.on('library:rule_added', handler);
  }

  /**
   * Subscribe to library rule removed event
   * @param handler - Callback when a rule is removed
   * @returns Unsubscribe function
   */
  onLibraryRuleRemoved(handler: EventHandler<'library:rule_removed'>): () => void {
    return this.on('library:rule_removed', handler);
  }

  /**
   * Subscribe to library rule updated event
   * @param handler - Callback when a rule is updated
   * @returns Unsubscribe function
   */
  onLibraryRuleUpdated(handler: EventHandler<'library:rule_updated'>): () => void {
    return this.on('library:rule_updated', handler);
  }

  /**
   * Subscribe to library method changed event
   * @param handler - Callback when service method changes
   * @returns Unsubscribe function
   */
  onLibraryMethodChanged(handler: EventHandler<'library:method_changed'>): () => void {
    return this.on('library:method_changed', handler);
  }

  // ==========================================================================
  // Proxy & VPN Helper Methods
  // ==========================================================================

  /**
   * Subscribe to proxy chain activated event
   * @param handler - Callback when a proxy chain is activated
   * @returns Unsubscribe function
   */
  onProxyChainActivated(handler: EventHandler<'proxy:chain_activated'>): () => void {
    return this.on('proxy:chain_activated', handler);
  }

  /**
   * Subscribe to proxy chain deactivated event
   * @param handler - Callback when a proxy chain is deactivated
   * @returns Unsubscribe function
   */
  onProxyChainDeactivated(handler: EventHandler<'proxy:chain_deactivated'>): () => void {
    return this.on('proxy:chain_deactivated', handler);
  }

  /**
   * Subscribe to proxy imported event
   * @param handler - Callback when proxies are imported
   * @returns Unsubscribe function
   */
  onProxyImported(handler: EventHandler<'proxy:imported'>): () => void {
    return this.on('proxy:imported', handler);
  }

  /**
   * Subscribe to proxy test complete event
   * @param handler - Callback when proxy test completes
   * @returns Unsubscribe function
   */
  onProxyTestComplete(handler: EventHandler<'proxy:test_complete'>): () => void {
    return this.on('proxy:test_complete', handler);
  }

  // ==========================================================================
  // Ipset Helper Methods
  // ==========================================================================

  /**
   * Subscribe to ipset updated event
   * @param handler - Callback when IP set is updated
   * @returns Unsubscribe function
   */
  onIpsetUpdated(handler: EventHandler<'ipset:updated'>): () => void {
    return this.on('ipset:updated', handler);
  }

  /**
   * Subscribe to ipset mode changed event
   * @param handler - Callback when IP set mode changes
   * @returns Unsubscribe function
   */
  onIpsetModeChanged(handler: EventHandler<'ipset:mode_changed'>): () => void {
    return this.on('ipset:mode_changed', handler);
  }

  // ==========================================================================
  // Traffic & Connection Helper Methods
  // ==========================================================================

  /**
   * Subscribe to traffic update event
   * @param handler - Callback when traffic stats update
   * @returns Unsubscribe function
   */
  onTrafficUpdate(handler: EventHandler<'traffic:update'>): () => void {
    return this.on('traffic:update', handler);
  }

  /**
   * Subscribe to connection opened event
   * @param handler - Callback when connection opens
   * @returns Unsubscribe function
   */
  onConnectionOpened(handler: EventHandler<'connection:opened'>): () => void {
    return this.on('connection:opened', handler);
  }

  /**
   * Subscribe to connection closed event
   * @param handler - Callback when connection closes
   * @returns Unsubscribe function
   */
  onConnectionClosed(handler: EventHandler<'connection:closed'>): () => void {
    return this.on('connection:closed', handler);
  }

  // ==========================================================================
  // Issue Helper Methods
  // ==========================================================================

  /**
   * Subscribe to issue detected event
   * @param handler - Callback when an issue is detected
   * @returns Unsubscribe function
   */
  onIssueDetected(handler: EventHandler<'issue:detected'>): () => void {
    return this.on('issue:detected', handler);
  }

  /**
   * Subscribe to issue resolved event
   * @param handler - Callback when an issue is resolved
   * @returns Unsubscribe function
   */
  onIssueResolved(handler: EventHandler<'issue:resolved'>): () => void {
    return this.on('issue:resolved', handler);
  }

  // ==========================================================================
  // Strategy Helper Methods
  // ==========================================================================

  /**
   * Subscribe to strategy changed event
   * @param handler - Callback when strategy changes
   * @returns Unsubscribe function
   */
  onStrategyChanged(handler: EventHandler<'strategy:changed'>): () => void {
    return this.on('strategy:changed', handler);
  }

  /**
   * Subscribe to strategy applied event
   * @param handler - Callback when strategy is applied
   * @returns Unsubscribe function
   */
  onStrategyApplied(handler: EventHandler<'strategy:applied'>): () => void {
    return this.on('strategy:applied', handler);
  }

  /**
   * Subscribe to strategy stopped event
   * @param handler - Callback when strategy is stopped
   * @returns Unsubscribe function
   */
  onStrategyStopped(handler: EventHandler<'strategy:stopped'>): () => void {
    return this.on('strategy:stopped', handler);
  }
}

// Singleton instance
export const eventBus = new EventBus();

// Re-export types for convenience
export type { EventHandler };
