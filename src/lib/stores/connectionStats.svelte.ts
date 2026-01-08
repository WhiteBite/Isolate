/**
 * Connection Statistics Store
 * 
 * Provides real-time connection monitoring via Tauri events.
 * Subscribes to:
 * - connection:stats - periodic statistics updates
 * - connection:new - new connection established
 * - connection:closed - connection closed
 */

import { browser } from '$app/environment';

// ============================================================================
// Types
// ============================================================================

export interface ConnectionInfo {
  id: string;
  protocol: 'tcp' | 'udp';
  remoteHost: string;
  remotePort: number;
  localPort: number;
  bytesIn: number;
  bytesOut: number;
  startTime: number;
  duration: number;
}

export interface ConnectionStats {
  /** Number of currently active connections */
  activeConnections: number;
  /** Total bytes received */
  totalBytesIn: number;
  /** Total bytes sent */
  totalBytesOut: number;
  /** Average connection duration in ms */
  avgDuration: number;
  /** Connections per minute (last 5 minutes) */
  connectionsPerMinute: number;
  /** Peak connections in last 5 minutes */
  peakConnections: number;
  /** Whether data is from real backend or waiting */
  isReady: boolean;
  /** Last update timestamp */
  lastUpdate: number;
}

export interface ConnectionHistoryPoint {
  timestamp: number;
  connections: number;
}

// ============================================================================
// Store State
// ============================================================================

let stats = $state<ConnectionStats>({
  activeConnections: 0,
  totalBytesIn: 0,
  totalBytesOut: 0,
  avgDuration: 0,
  connectionsPerMinute: 0,
  peakConnections: 0,
  isReady: false,
  lastUpdate: 0
});

// History for chart (last 5 minutes, 1 point per 5 seconds = 60 points)
let history = $state<ConnectionHistoryPoint[]>([]);
const MAX_HISTORY_POINTS = 60;
const HISTORY_INTERVAL_MS = 5000;

// Active connections map
let activeConnectionsMap = $state<Map<string, ConnectionInfo>>(new Map());

// Cleanup functions
let cleanupFns: (() => void)[] = [];
let historyInterval: ReturnType<typeof setInterval> | null = null;
let initialized = false;

// ============================================================================
// Event Handlers
// ============================================================================

interface StatsEventPayload {
  active_connections: number;
  total_bytes_in: number;
  total_bytes_out: number;
  avg_duration_ms: number;
  connections_per_minute: number;
}

interface NewConnectionPayload {
  id: string;
  protocol: 'tcp' | 'udp';
  remote_host: string;
  remote_port: number;
  local_port: number;
}

interface ClosedConnectionPayload {
  id: string;
  bytes_in: number;
  bytes_out: number;
  duration_ms: number;
}

function handleStatsUpdate(payload: StatsEventPayload) {
  const now = Date.now();
  
  stats = {
    activeConnections: payload.active_connections,
    totalBytesIn: payload.total_bytes_in,
    totalBytesOut: payload.total_bytes_out,
    avgDuration: payload.avg_duration_ms,
    connectionsPerMinute: payload.connections_per_minute,
    peakConnections: Math.max(stats.peakConnections, payload.active_connections),
    isReady: true,
    lastUpdate: now
  };
}

function handleNewConnection(payload: NewConnectionPayload) {
  const conn: ConnectionInfo = {
    id: payload.id,
    protocol: payload.protocol,
    remoteHost: payload.remote_host,
    remotePort: payload.remote_port,
    localPort: payload.local_port,
    bytesIn: 0,
    bytesOut: 0,
    startTime: Date.now(),
    duration: 0
  };
  
  activeConnectionsMap = new Map(activeConnectionsMap).set(payload.id, conn);
  
  // Update active count
  stats = {
    ...stats,
    activeConnections: activeConnectionsMap.size,
    peakConnections: Math.max(stats.peakConnections, activeConnectionsMap.size),
    isReady: true,
    lastUpdate: Date.now()
  };
}

function handleClosedConnection(payload: ClosedConnectionPayload) {
  const newMap = new Map(activeConnectionsMap);
  newMap.delete(payload.id);
  activeConnectionsMap = newMap;
  
  // Update stats
  stats = {
    ...stats,
    activeConnections: activeConnectionsMap.size,
    totalBytesIn: stats.totalBytesIn + payload.bytes_in,
    totalBytesOut: stats.totalBytesOut + payload.bytes_out,
    lastUpdate: Date.now()
  };
}

// ============================================================================
// History Management
// ============================================================================

function recordHistoryPoint() {
  const now = Date.now();
  const point: ConnectionHistoryPoint = {
    timestamp: now,
    connections: stats.activeConnections
  };
  
  // Add new point and trim to max size
  history = [...history.slice(-(MAX_HISTORY_POINTS - 1)), point];
}

// ============================================================================
// Initialization
// ============================================================================

async function initializeStore() {
  if (!browser || initialized) return;
  initialized = true;
  
  const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
  if (!isTauri) {
    // Browser preview - no real data
    return;
  }
  
  try {
    const { listen } = await import('@tauri-apps/api/event');
    
    // Subscribe to connection events
    const unlistenStats = await listen<StatsEventPayload>('connection:stats', (event) => {
      handleStatsUpdate(event.payload);
    });
    
    const unlistenNew = await listen<NewConnectionPayload>('connection:new', (event) => {
      handleNewConnection(event.payload);
    });
    
    const unlistenClosed = await listen<ClosedConnectionPayload>('connection:closed', (event) => {
      handleClosedConnection(event.payload);
    });
    
    cleanupFns.push(unlistenStats, unlistenNew, unlistenClosed);
    
    // Start history recording
    historyInterval = setInterval(recordHistoryPoint, HISTORY_INTERVAL_MS);
    
    // Record initial point
    recordHistoryPoint();
    
  } catch (e) {
    console.error('Failed to initialize connection stats store:', e);
  }
}

function cleanup() {
  cleanupFns.forEach(fn => fn());
  cleanupFns = [];
  
  if (historyInterval) {
    clearInterval(historyInterval);
    historyInterval = null;
  }
  
  initialized = false;
}

// ============================================================================
// Exported Store Interface
// ============================================================================

export const connectionStats = {
  /** Get current stats (reactive) */
  get stats() {
    return stats;
  },
  
  /** Get connection history for chart (reactive) */
  get history() {
    return history;
  },
  
  /** Get active connections map (reactive) */
  get activeConnections() {
    return activeConnectionsMap;
  },
  
  /** Initialize the store (call once on app start) */
  init: initializeStore,
  
  /** Cleanup subscriptions */
  cleanup,
  
  /** Reset peak connections counter */
  resetPeak() {
    stats = { ...stats, peakConnections: stats.activeConnections };
  },
  
  /** Reset all stats */
  reset() {
    stats = {
      activeConnections: 0,
      totalBytesIn: 0,
      totalBytesOut: 0,
      avgDuration: 0,
      connectionsPerMinute: 0,
      peakConnections: 0,
      isReady: false,
      lastUpdate: 0
    };
    history = [];
    activeConnectionsMap = new Map();
  }
};
