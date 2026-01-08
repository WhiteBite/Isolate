// Store для Dashboard с Svelte 5 runes
// Интегрирован с реальными backend командами и EventBus

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { eventBus, type EventPayload } from './eventBus.svelte';

export type ProtectionStatus = 'protected' | 'bypassing' | 'issues' | 'disabled';
export type OperationMode = 'auto' | 'tun' | 'proxy';

// ============================================================================
// Backend Types (соответствуют Rust структурам в dashboard.rs)
// ============================================================================

/**
 * Live network connection from backend
 * Rust: LiveConnection in commands/dashboard.rs
 */
export interface LiveConnection {
  id: string;
  domain: string;
  ip: string;
  port: number;
  protocol: string;
  status: string;
  bytes_sent: number;
  bytes_received: number;
  started_at: number;
}

/**
 * Traffic statistics from backend
 * Rust: TrafficStats in commands/dashboard.rs
 */
export interface TrafficStats {
  total_sent: number;
  total_received: number;
  active_connections: number;
  protected_connections: number;
  last_updated: number;
}

/**
 * Protection issue from backend
 * Rust: ProtectionIssue in commands/dashboard.rs
 */
export interface ProtectionIssue {
  id: string;
  severity: string;
  message: string;
  service_id: string | null;
  detected_at: number;
}

/**
 * Result of fixing an issue
 * Rust: FixResult in commands/dashboard.rs
 */
export interface FixResult {
  success: boolean;
  message: string;
  issue_id: string;
}

// ============================================================================
// Legacy Types (для обратной совместимости с UI)
// ============================================================================

export interface Issue {
  id: string;
  type: 'service_blocked' | 'strategy_failed' | 'connection_error';
  message: string;
  serviceId?: string;
  timestamp: number;
  canAutoFix: boolean;
}

export interface ActiveConnection {
  domain: string;
  method: 'direct' | 'strategy' | 'proxy' | 'vless';
  strategyName?: string;
  proxyName?: string;
  bytesTransferred: number;
  duration: number;
}

export interface TrafficPoint {
  timestamp: number;
  download: number;
  upload: number;
}

// ============================================================================
// Environment Detection
// ============================================================================

/**
 * Проверяет, запущено ли приложение в Tauri окружении
 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window;
}

/**
 * Проверяет готовность backend (AppState инициализирован)
 */
async function isBackendReady(): Promise<boolean> {
  if (!isTauri()) return false;
  try {
    return await invoke<boolean>('is_backend_ready');
  } catch {
    return false;
  }
}

// ============================================================================
// Backend API Functions
// ============================================================================

/**
 * Загрузить live connections с backend
 */
async function loadConnections(): Promise<LiveConnection[]> {
  if (!isTauri()) {
    // Demo mode: mock данные
    const now = Math.floor(Date.now() / 1000);
    return [
      {
        id: 'demo-001',
        domain: 'youtube.com',
        ip: '142.250.185.206',
        port: 443,
        protocol: 'TCP',
        status: 'active',
        bytes_sent: Math.floor(Math.random() * 50000),
        bytes_received: Math.floor(Math.random() * 5000000),
        started_at: now - 120,
      },
      {
        id: 'demo-002',
        domain: 'discord.com',
        ip: '162.159.135.234',
        port: 443,
        protocol: 'TCP',
        status: 'active',
        bytes_sent: Math.floor(Math.random() * 20000),
        bytes_received: Math.floor(Math.random() * 100000),
        started_at: now - 300,
      },
      {
        id: 'demo-003',
        domain: 'google.com',
        ip: '142.250.185.100',
        port: 443,
        protocol: 'TCP',
        status: 'active',
        bytes_sent: Math.floor(Math.random() * 10000),
        bytes_received: Math.floor(Math.random() * 50000),
        started_at: now - 60,
      },
    ];
  }

  try {
    const ready = await isBackendReady();
    if (!ready) {
      console.warn('[Dashboard] Backend not ready for loadConnections');
      return [];
    }
    return await invoke<LiveConnection[]>('get_live_connections');
  } catch (e) {
    console.error('[Dashboard] Failed to load connections:', e);
    return [];
  }
}

/**
 * Загрузить статистику трафика с backend
 */
async function loadTrafficStats(): Promise<TrafficStats | null> {
  if (!isTauri()) {
    // Demo mode: mock данные
    return {
      total_sent: Math.floor(Math.random() * 200_000_000),
      total_received: Math.floor(Math.random() * 3_000_000_000),
      active_connections: 3 + Math.floor(Math.random() * 5),
      protected_connections: 2 + Math.floor(Math.random() * 3),
      last_updated: Math.floor(Date.now() / 1000),
    };
  }

  try {
    const ready = await isBackendReady();
    if (!ready) {
      console.warn('[Dashboard] Backend not ready for loadTrafficStats');
      return null;
    }
    return await invoke<TrafficStats>('get_traffic_stats');
  } catch (e) {
    console.error('[Dashboard] Failed to load traffic stats:', e);
    return null;
  }
}

/**
 * Загрузить проблемы защиты с backend
 */
async function loadIssues(): Promise<ProtectionIssue[]> {
  if (!isTauri()) {
    // Demo mode: пустой список или mock проблемы
    return [];
  }

  try {
    const ready = await isBackendReady();
    if (!ready) {
      console.warn('[Dashboard] Backend not ready for loadIssues');
      return [];
    }
    return await invoke<ProtectionIssue[]>('get_protection_issues');
  } catch (e) {
    console.error('[Dashboard] Failed to load issues:', e);
    return [];
  }
}

/**
 * Исправить проблему через backend
 */
async function fixIssue(issueId: string): Promise<FixResult | null> {
  if (!isTauri()) {
    // Demo mode: всегда успешно
    return {
      success: true,
      message: 'Issue fixed (demo mode)',
      issue_id: issueId,
    };
  }

  try {
    const ready = await isBackendReady();
    if (!ready) {
      console.warn('[Dashboard] Backend not ready for fixIssue');
      return null;
    }
    return await invoke<FixResult>('fix_issue', { issueId });
  } catch (e) {
    console.error('[Dashboard] Failed to fix issue:', e);
    return null;
  }
}

// ============================================================================
// Conversion Helpers (Backend types → UI types)
// ============================================================================

/**
 * Конвертирует LiveConnection в ActiveConnection для UI
 */
function toActiveConnection(conn: LiveConnection): ActiveConnection {
  const now = Math.floor(Date.now() / 1000);
  return {
    domain: conn.domain,
    method: conn.status === 'active' ? 'strategy' : 'direct',
    strategyName: conn.status === 'active' ? 'DPI Bypass' : undefined,
    bytesTransferred: conn.bytes_sent + conn.bytes_received,
    duration: now - conn.started_at,
  };
}

/**
 * Конвертирует ProtectionIssue в Issue для UI
 */
function toIssue(issue: ProtectionIssue): Issue {
  const typeMap: Record<string, Issue['type']> = {
    warning: 'connection_error',
    error: 'strategy_failed',
    critical: 'service_blocked',
  };
  
  return {
    id: issue.id,
    type: typeMap[issue.severity] || 'connection_error',
    message: issue.message,
    serviceId: issue.service_id ?? undefined,
    timestamp: issue.detected_at * 1000, // convert to ms
    canAutoFix: true, // backend supports fix_issue
  };
}

// ============================================================================
// Dashboard Store
// ============================================================================

class DashboardStore {
  // State
  protectionStatus = $state<ProtectionStatus>('disabled');
  issues = $state<Issue[]>([]);
  currentMode = $state<OperationMode>('auto');
  activeConnections = $state<ActiveConnection[]>([]);
  trafficHistory = $state<TrafficPoint[]>([]);
  loading = $state(false);
  
  // Raw backend data
  liveConnections = $state<LiveConnection[]>([]);
  trafficStats = $state<TrafficStats | null>(null);
  protectionIssues = $state<ProtectionIssue[]>([]);
  
  // Session stats
  sessionStartTime = $state<number>(Date.now());
  totalDownload = $state(0);
  totalUpload = $state(0);
  
  // Backend state
  isRealMode = $state(false);
  
  // Event listeners
  private unlisteners: UnlistenFn[] = [];
  private eventBusUnsubscribers: (() => void)[] = [];
  
  // Derived
  hasIssues = $derived(this.issues.length > 0);
  issueCount = $derived(this.issues.length);
  currentDownloadSpeed = $derived(
    this.trafficHistory.length > 0 
      ? this.trafficHistory[this.trafficHistory.length - 1].download 
      : 0
  );
  currentUploadSpeed = $derived(
    this.trafficHistory.length > 0 
      ? this.trafficHistory[this.trafficHistory.length - 1].upload 
      : 0
  );
  
  // Derived from backend stats
  activeConnectionCount = $derived(this.trafficStats?.active_connections ?? 0);
  protectedConnectionCount = $derived(this.trafficStats?.protected_connections ?? 0);

  /**
   * Инициализация store — определяет режим работы и подписывается на события
   */
  async initialize() {
    this.isRealMode = isTauri() && await isBackendReady();
    console.log(`[Dashboard] Initialized in ${this.isRealMode ? 'REAL' : 'DEMO'} mode`);
    
    // Подписываемся на Tauri events
    await this.setupEventListeners();
    
    // Загружаем начальные данные
    await this.loadStatus();
  }

  /**
   * Настройка подписок на Tauri events и EventBus
   */
  private async setupEventListeners() {
    // Подписываемся на EventBus (работает всегда)
    this.setupEventBusListeners();
    
    // Подписываемся на Tauri events (только в Tauri окружении)
    if (isTauri()) {
      await this.setupTauriEventListeners();
    }
  }

  /**
   * Настройка подписок на EventBus
   * EventBus — единый источник событий для frontend
   */
  private setupEventBusListeners() {
    // Traffic updates через EventBus
    const unsubTraffic = eventBus.onTrafficUpdate((payload) => {
      console.log('[Dashboard] EventBus: traffic:update');
      // Обновляем историю трафика
      this.addTrafficPoint(payload.download, payload.upload);
      this.totalDownload += payload.download;
      this.totalUpload += payload.upload;
    });
    this.eventBusUnsubscribers.push(unsubTraffic);

    // Connection events через EventBus
    const unsubConnOpened = eventBus.onConnectionOpened((payload) => {
      console.log('[Dashboard] EventBus: connection:opened', payload.serviceId);
      // Добавляем соединение в список
      const newConnection: ActiveConnection = {
        domain: payload.serviceId,
        method: payload.method as ActiveConnection['method'] || 'strategy',
        bytesTransferred: 0,
        duration: 0,
      };
      this.activeConnections = [...this.activeConnections, newConnection];
    });
    this.eventBusUnsubscribers.push(unsubConnOpened);

    const unsubConnClosed = eventBus.onConnectionClosed((payload) => {
      console.log('[Dashboard] EventBus: connection:closed', payload.serviceId);
      // Удаляем соединение из списка
      this.activeConnections = this.activeConnections.filter(
        c => c.domain !== payload.serviceId
      );
    });
    this.eventBusUnsubscribers.push(unsubConnClosed);

    // Issue events через EventBus
    const unsubIssueDetected = eventBus.onIssueDetected((payload) => {
      console.log('[Dashboard] EventBus: issue:detected', payload.id);
      const issue: Issue = {
        id: payload.id,
        type: payload.severity === 'error' ? 'strategy_failed' : 'connection_error',
        message: payload.message,
        timestamp: Date.now(),
        canAutoFix: true,
      };
      this.issues = [...this.issues, issue];
      if (this.protectionStatus === 'protected') {
        this.protectionStatus = 'issues';
      }
    });
    this.eventBusUnsubscribers.push(unsubIssueDetected);

    const unsubIssueResolved = eventBus.onIssueResolved((payload) => {
      console.log('[Dashboard] EventBus: issue:resolved', payload.id);
      this.issues = this.issues.filter(i => i.id !== payload.id);
      if (this.issues.length === 0 && this.protectionStatus === 'issues') {
        this.protectionStatus = 'protected';
      }
    });
    this.eventBusUnsubscribers.push(unsubIssueResolved);

    // Strategy events через EventBus
    const unsubStrategyApplied = eventBus.onStrategyApplied((payload) => {
      console.log('[Dashboard] EventBus: strategy:applied', payload.strategyId);
      this.protectionStatus = 'protected';
    });
    this.eventBusUnsubscribers.push(unsubStrategyApplied);

    const unsubStrategyStopped = eventBus.onStrategyStopped(() => {
      console.log('[Dashboard] EventBus: strategy:stopped');
      this.protectionStatus = 'disabled';
    });
    this.eventBusUnsubscribers.push(unsubStrategyStopped);

    const unsubStrategyChanged = eventBus.onStrategyChanged((payload) => {
      console.log('[Dashboard] EventBus: strategy:changed', payload.serviceId, '->', payload.strategyId);
      // При смене стратегии статус остаётся protected
      if (this.protectionStatus === 'disabled') {
        this.protectionStatus = 'protected';
      }
    });
    this.eventBusUnsubscribers.push(unsubStrategyChanged);

    // AI Pilot events через EventBus
    const unsubAiPilotAction = eventBus.onAiPilotAction((payload) => {
      console.log('[Dashboard] EventBus: ai_pilot:action', payload.action_type);
      // AI Pilot переключил стратегию — обновляем статус
      if (payload.action_type === 'switch_strategy') {
        this.protectionStatus = 'bypassing';
      }
    });
    this.eventBusUnsubscribers.push(unsubAiPilotAction);

    // Proxy events через EventBus
    const unsubProxyActivated = eventBus.onProxyChainActivated((payload) => {
      console.log('[Dashboard] EventBus: proxy:chain_activated', payload.name);
      this.protectionStatus = 'protected';
    });
    this.eventBusUnsubscribers.push(unsubProxyActivated);

    const unsubProxyDeactivated = eventBus.onProxyChainDeactivated(() => {
      console.log('[Dashboard] EventBus: proxy:chain_deactivated');
      // Проверяем есть ли другие активные защиты
      if (this.activeConnections.length === 0) {
        this.protectionStatus = 'disabled';
      }
    });
    this.eventBusUnsubscribers.push(unsubProxyDeactivated);

    console.log('[Dashboard] EventBus listeners set up');
  }

  /**
   * Настройка подписок на Tauri events
   * Tauri events пробрасываются в EventBus для единообразия
   */
  private async setupTauriEventListeners() {
    try {
      // Traffic updates — пробрасываем в EventBus
      const unlistenTraffic = await listen<TrafficStats>('traffic:update', (event) => {
        console.log('[Dashboard] Tauri: traffic:update');
        this.trafficStats = event.payload;
        // Пробрасываем в EventBus
        eventBus.emit('traffic:update', {
          download: event.payload.total_received,
          upload: event.payload.total_sent,
        });
      });
      this.unlisteners.push(unlistenTraffic);
      
      // Connection events — пробрасываем в EventBus
      const unlistenConnOpened = await listen<LiveConnection>('connection:opened', (event) => {
        console.log('[Dashboard] Tauri: connection:opened');
        this.liveConnections = [...this.liveConnections, event.payload];
        this.activeConnections = this.liveConnections.map(toActiveConnection);
        // Пробрасываем в EventBus
        eventBus.emit('connection:opened', {
          serviceId: event.payload.domain,
          method: event.payload.protocol,
        });
      });
      this.unlisteners.push(unlistenConnOpened);
      
      const unlistenConnClosed = await listen<{ id: string }>('connection:closed', (event) => {
        console.log('[Dashboard] Tauri: connection:closed');
        const closedConn = this.liveConnections.find(c => c.id === event.payload.id);
        this.liveConnections = this.liveConnections.filter(c => c.id !== event.payload.id);
        this.activeConnections = this.liveConnections.map(toActiveConnection);
        // Пробрасываем в EventBus
        if (closedConn) {
          eventBus.emit('connection:closed', {
            serviceId: closedConn.domain,
          });
        }
      });
      this.unlisteners.push(unlistenConnClosed);
      
      // Issue events — пробрасываем в EventBus
      const unlistenIssueDetected = await listen<ProtectionIssue>('issue:detected', (event) => {
        console.log('[Dashboard] Tauri: issue:detected');
        this.protectionIssues = [...this.protectionIssues, event.payload];
        this.issues = this.protectionIssues.map(toIssue);
        // Пробрасываем в EventBus
        eventBus.emit('issue:detected', {
          id: event.payload.id,
          message: event.payload.message,
          severity: event.payload.severity as 'warning' | 'error',
        });
      });
      this.unlisteners.push(unlistenIssueDetected);
      
      const unlistenIssueResolved = await listen<{ id: string }>('issue:resolved', (event) => {
        console.log('[Dashboard] Tauri: issue:resolved');
        this.protectionIssues = this.protectionIssues.filter(i => i.id !== event.payload.id);
        this.issues = this.protectionIssues.map(toIssue);
        // Пробрасываем в EventBus
        eventBus.emit('issue:resolved', {
          id: event.payload.id,
        });
      });
      this.unlisteners.push(unlistenIssueResolved);
      
      // Strategy events — пробрасываем в EventBus
      const unlistenStrategyApplied = await listen<{ strategy_id?: string; service_id?: string }>('strategy:applied', (event) => {
        console.log('[Dashboard] Tauri: strategy:applied');
        // Пробрасываем в EventBus
        eventBus.emit('strategy:applied', {
          strategyId: event.payload?.strategy_id ?? 'unknown',
          serviceId: event.payload?.service_id,
          timestamp: Date.now(),
        });
      });
      this.unlisteners.push(unlistenStrategyApplied);
      
      const unlistenStrategyStopped = await listen<{ strategy_id?: string; service_id?: string }>('strategy:stopped', (event) => {
        console.log('[Dashboard] Tauri: strategy:stopped');
        // Пробрасываем в EventBus
        eventBus.emit('strategy:stopped', {
          strategyId: event.payload?.strategy_id,
          serviceId: event.payload?.service_id,
          timestamp: Date.now(),
        });
      });
      this.unlisteners.push(unlistenStrategyStopped);
      
      console.log('[Dashboard] Tauri event listeners set up');
    } catch (e) {
      console.error('[Dashboard] Failed to setup Tauri event listeners:', e);
    }
  }

  /**
   * Очистка подписок при уничтожении store
   */
  async cleanup() {
    // Очищаем Tauri event listeners
    for (const unlisten of this.unlisteners) {
      unlisten();
    }
    this.unlisteners = [];
    
    // Очищаем EventBus подписки
    for (const unsubscribe of this.eventBusUnsubscribers) {
      unsubscribe();
    }
    this.eventBusUnsubscribers = [];
    
    console.log('[Dashboard] All event listeners cleaned up');
  }

  /**
   * Загрузить все данные dashboard
   */
  async loadStatus() {
    this.loading = true;
    try {
      // Параллельно загружаем все данные
      const [connections, stats, issues] = await Promise.all([
        loadConnections(),
        loadTrafficStats(),
        loadIssues(),
      ]);

      // Обновляем raw данные
      this.liveConnections = connections;
      this.trafficStats = stats;
      this.protectionIssues = issues;
      
      // Конвертируем для UI
      this.activeConnections = connections.map(toActiveConnection);
      this.issues = issues.map(toIssue);
      
      // Обновляем статистику
      if (stats) {
        this.updateFromTrafficStats(stats);
      }
      
      // Определяем статус защиты
      if (issues.length > 0) {
        this.protectionStatus = 'issues';
      } else if (stats && stats.protected_connections > 0) {
        this.protectionStatus = 'protected';
      }
      
    } catch (e) {
      console.error('[Dashboard] Failed to load status:', e);
    } finally {
      this.loading = false;
    }
  }

  /**
   * Обновить данные из TrafficStats
   */
  private updateFromTrafficStats(stats: TrafficStats) {
    this.totalDownload = stats.total_received;
    this.totalUpload = stats.total_sent;
    
    // Добавляем точку в историю (симулируем скорость)
    const lastPoint = this.trafficHistory[this.trafficHistory.length - 1];
    const download = lastPoint 
      ? Math.max(0, stats.total_received - this.totalDownload) 
      : Math.random() * 1024 * 1024;
    const upload = lastPoint 
      ? Math.max(0, stats.total_sent - this.totalUpload) 
      : Math.random() * 256 * 1024;
    
    this.addTrafficPoint(download, upload);
  }

  /**
   * Обновить только статистику трафика (для частых обновлений)
   */
  async refreshTraffic() {
    const stats = await loadTrafficStats();
    if (stats) {
      this.trafficStats = stats;
      this.updateFromTrafficStats(stats);
    }
  }

  /**
   * Обновить активные соединения
   */
  async refreshConnections() {
    const connections = await loadConnections();
    this.liveConnections = connections;
    this.activeConnections = connections.map(toActiveConnection);
  }

  /**
   * Обновить проблемы
   */
  async refreshIssues() {
    const issues = await loadIssues();
    this.protectionIssues = issues;
    this.issues = issues.map(toIssue);
    
    if (issues.length > 0 && this.protectionStatus === 'protected') {
      this.protectionStatus = 'issues';
    } else if (issues.length === 0 && this.protectionStatus === 'issues') {
      this.protectionStatus = 'protected';
    }
  }

  async setMode(mode: OperationMode) {
    // TODO: Backend command set_operation_mode когда будет готов
    this.currentMode = mode;
  }

  /**
   * Исправить проблему
   */
  async fixIssue(issueId: string) {
    try {
      const result = await fixIssue(issueId);
      if (result?.success) {
        // Удаляем проблему из списка
        this.protectionIssues = this.protectionIssues.filter(i => i.id !== issueId);
        this.issues = this.protectionIssues.map(toIssue);
        
        // Обновляем статус если проблем больше нет
        if (this.protectionIssues.length === 0 && this.protectionStatus === 'issues') {
          this.protectionStatus = 'protected';
        }
        
        console.log(`[Dashboard] Issue ${issueId} fixed: ${result.message}`);
      }
    } catch (e) {
      console.error('[Dashboard] Failed to fix issue:', e);
    }
  }

  addTrafficPoint(download: number, upload: number) {
    const point: TrafficPoint = {
      timestamp: Date.now(),
      download,
      upload
    };
    // Keep last 60 points (1 minute at 1 point/sec)
    this.trafficHistory = [...this.trafficHistory.slice(-59), point];
  }

  updateConnections(connections: ActiveConnection[]) {
    this.activeConnections = connections;
  }

  setProtectionStatus(status: ProtectionStatus) {
    this.protectionStatus = status;
  }

  /**
   * Добавить проблему (для тестирования или внутреннего использования)
   */
  addIssue(issue: Issue) {
    this.issues = [...this.issues, issue];
    if (this.protectionStatus === 'protected') {
      this.protectionStatus = 'issues';
    }
  }

  /**
   * Сбросить сессию
   */
  resetSession() {
    this.sessionStartTime = Date.now();
    this.totalDownload = 0;
    this.totalUpload = 0;
    this.trafficHistory = [];
    this.issues = [];
    this.protectionIssues = [];
    this.liveConnections = [];
    this.activeConnections = [];
  }
}

export const dashboardStore = new DashboardStore();

// Экспорт функций для прямого использования
export {
  isTauri,
  isBackendReady,
  loadConnections,
  loadTrafficStats,
  loadIssues,
  fixIssue,
  toActiveConnection,
  toIssue,
};
