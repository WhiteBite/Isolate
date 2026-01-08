<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { 
    BentoGrid, 
    BentoWidget,
    StatusWidget,
    HealthWidget,
    MethodWidget,
    QuickActionsWidget,
    NetworkStatsWidget,
    LatencyWidget,
    DashboardSkeleton,
    ScanningIndicator,
    PluginSlot,
    ConnectionStatsWidget
  } from '$lib/components';
  import FailoverStatusWidget from '$lib/components/widgets/FailoverStatusWidget.svelte';
  // New dashboard components
  import ShieldIndicator from '$lib/components/dashboard/ShieldIndicator.svelte';
  import ModeSelector from '$lib/components/dashboard/ModeSelector.svelte';
  import LiveActivityPanel from '$lib/components/dashboard/LiveActivityPanel.svelte';
  import BackendNotReady from '$lib/components/dashboard/BackendNotReady.svelte';
  import { 
    appStatus, 
    optimizationProgress, 
    isOptimizing, 
    services,
  } from '$lib/stores';
  import { logs } from '$lib/stores/logs';
  import { getWidgetOrder, saveWidgetOrder } from '$lib/stores/layout';
  import { getServiceIconEmoji } from '$lib/utils/icons';
  import { waitForBackend } from '$lib/utils/backend';
  import { mockDashboardServices } from '$lib/mocks';
  import { dashboardStore, type ProtectionStatus, type OperationMode, type ActiveConnection } from '$lib/stores/dashboard.svelte';
  import { trafficMonitor } from '$lib/stores/trafficMonitor.svelte';
  import { t } from '$lib/i18n';

  // Backend ServiceStatus type
  interface ServiceStatus {
    service_id: string;
    service_name: string;
    accessible: boolean;
    avg_latency_ms: number | null;
    success_rate: number;
    from_cache: boolean;
  }

  // Widget definitions for drag-n-drop
  const DEFAULT_WIDGET_ORDER = ['shield', 'activity', 'health', 'method', 'actions', 'failover', 'network', 'latency', 'connections'];
  
  interface WidgetConfig {
    id: string;
    colspan: 1 | 2 | 3 | 4;
    rowspan: 1 | 2;
    title?: string;
    icon?: string;
  }
  
  const widgetConfigs: Record<string, WidgetConfig> = {
    shield: { id: 'shield', colspan: 2, rowspan: 2 },
    activity: { id: 'activity', colspan: 2, rowspan: 2, title: 'Live Activity', icon: '📡' },
    health: { id: 'health', colspan: 2, rowspan: 1, title: 'Health Monitor', icon: '💚' },
    method: { id: 'method', colspan: 1, rowspan: 1, title: 'Active Method', icon: '⚡' },
    actions: { id: 'actions', colspan: 1, rowspan: 1, title: 'Quick Actions', icon: '🚀' },
    failover: { id: 'failover', colspan: 1, rowspan: 1, title: 'Auto Recovery', icon: '🔄' },
    network: { id: 'network', colspan: 2, rowspan: 1, title: 'Network Stats', icon: '📊' },
    latency: { id: 'latency', colspan: 2, rowspan: 1, title: 'Latency Monitor', icon: '📈' },
    connections: { id: 'connections', colspan: 2, rowspan: 1, title: 'Connection Stats', icon: '🔗' }
  };

  // Widget order state - load from centralized layout store
  let widgetOrder = $state<string[]>(browser ? getWidgetOrder() : DEFAULT_WIDGET_ORDER);
  
  // Handle widget reorder
  function handleReorder(newOrder: string[]) {
    widgetOrder = newOrder;
    saveWidgetOrder(newOrder);
    logs.debug('dashboard', `Widget order saved: ${newOrder.join(', ')}`);
  }

  // Cleanup functions
  let cleanupFns: (() => void)[] = [];
  let healthCheckInterval: ReturnType<typeof setInterval> | null = null;
  let networkStatsInterval: ReturnType<typeof setInterval> | null = null;
  let initialized = false; // Not $state - should reset on component remount
  let isInitializing = false; // Guard against concurrent initialization
  
  // Loading state for skeleton
  let isLoading = $state(true);
  let backendFailed = $state(false);
  
  // Helper to clear all intervals safely
  function clearAllIntervals() {
    if (healthCheckInterval) {
      clearInterval(healthCheckInterval);
      healthCheckInterval = null;
    }
    if (networkStatsInterval) {
      clearInterval(networkStatsInterval);
      networkStatsInterval = null;
    }
  }

  // Local reactive copies of store values
  let appStatusValue = $state<{isActive: boolean; currentStrategy: string | null; currentStrategyName: string | null}>({
    isActive: false,
    currentStrategy: null,
    currentStrategyName: null
  });
  let isOptimizingValue = $state(false);
  let optimizationProgressValue = $state<{step: string; progress: number; message: string; isComplete: boolean; error: string | null}>({
    step: '',
    progress: 0,
    message: '',
    isComplete: false,
    error: null
  });
  let servicesValue = $state<{id: string; name: string; icon: string; enabled: boolean; status: 'unknown' | 'working' | 'blocked'; ping?: number}[]>([]);
  let isCheckingHealth = $state(false);
  let isScanning = $state(false);
  let isTesting = $state(false);

  // Network stats state
  let networkStats = $state({
    downloadSpeed: 0,
    uploadSpeed: 0,
    totalDownload: 0,
    totalUpload: 0,
    activeConnections: 0,
    isSimulated: true // Mark as simulated data
  });

  // Latency history state for chart
  let latencyHistory = $state<number[]>([]);
  let currentLatency = $derived(latencyHistory.length > 0 ? latencyHistory[latencyHistory.length - 1] : undefined);

  // Network stats interval is declared above with healthCheckInterval

  // Mock active connections for demo
  let activeConnections = $state<ActiveConnection[]>([]);

  // Protection status derived from app state
  let protectionStatus = $derived<ProtectionStatus>(
    isOptimizingValue ? 'bypassing' :
    appStatusValue.isActive ? 'protected' : 
    'disabled'
  );

  // Current operation mode
  let currentMode = $state<OperationMode>('auto');

  // Quick actions with loading states
  let quickActions = $derived([
    { id: 'scan', label: t('dashboard.quickActions.scanAll'), loading: isScanning, disabled: isOptimizingValue },
    { id: 'test', label: t('dashboard.quickActions.testCurrent'), loading: isTesting },
    { id: 'proxy', label: t('dashboard.quickActions.addProxy') },
    { id: 'settings', label: t('dashboard.quickActions.settings') }
  ]);

  // Derived health data for widget
  let healthServices = $derived(servicesValue.slice(0, 4).map(s => ({
    name: s.name,
    status: s.status === 'working' ? 'healthy' as const : s.status === 'blocked' ? 'down' as const : 'degraded' as const,
    ping: s.ping
  })));

  // Current method
  let currentMethod = $derived<'direct' | 'zapret' | 'vless' | 'proxy'>(
    appStatusValue.isActive 
      ? (appStatusValue.currentStrategyName?.toLowerCase().includes('vless') ? 'vless' : 'zapret')
      : 'direct'
  );

  // Handle mode change
  function handleModeChange(mode: OperationMode) {
    currentMode = mode;
    dashboardStore.setMode(mode);
    logs.info('system', `Mode changed to: ${mode}`);
  }

  // Retry backend connection
  async function handleRetryBackend() {
    isLoading = true;
    backendFailed = false;
    initialized = false;
    isInitializing = false;
    await initializeDashboard();
  }

  // Check all services health
  async function checkServicesHealth() {
    if (!browser || isCheckingHealth) return;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) return;
    
    isCheckingHealth = true;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const results = await invoke<ServiceStatus[]>('check_all_registry_services');
      
      if (results && results.length > 0) {
        // Calculate average latency from all accessible services
        const latencies = results
          .filter(r => r.accessible && r.avg_latency_ms !== null)
          .map(r => r.avg_latency_ms as number);
        
        if (latencies.length > 0) {
          const avgLatency = Math.round(latencies.reduce((a, b) => a + b, 0) / latencies.length);
          // Add to history, keep last 30 points
          latencyHistory = [...latencyHistory.slice(-29), avgLatency];
        }
        
        // Update services store with real status
        services.update(currentServices => {
          const updatedServices = [...currentServices];
          
          for (const result of results) {
            const existingIndex = updatedServices.findIndex(s => s.id === result.service_id);
            
            // Determine status based on accessibility and success rate
            let status: 'unknown' | 'working' | 'blocked' = 'unknown';
            if (result.accessible) {
              status = result.success_rate >= 0.5 ? 'working' : 'blocked';
            } else {
              status = 'blocked';
            }
            
            const serviceData = {
              id: result.service_id,
              name: result.service_name,
              icon: getServiceIconEmoji(result.service_id),
              enabled: true,
              status,
              ping: result.avg_latency_ms ?? undefined
            };
            
            if (existingIndex >= 0) {
              updatedServices[existingIndex] = serviceData;
            } else {
              updatedServices.push(serviceData);
            }
          }
          
          return updatedServices;
        });
        
        logs.debug('health', `Checked ${results.length} services`);
      }
    } catch (e) {
      logs.error('health', `Health check failed: ${e}`);
    } finally {
      isCheckingHealth = false;
    }
  }

  // Initialize dashboard data
  async function initializeDashboard() {
    // Guard: only run in browser and only once per mount
    if (!browser) return;
    if (initialized || isInitializing) return;
    isInitializing = true;
    
    // Clear any existing intervals before creating new ones (safety measure)
    clearAllIntervals();
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    // Subscribe to stores
    const unsubAppStatus = appStatus.subscribe(v => { appStatusValue = v; });
    const unsubIsOptimizing = isOptimizing.subscribe(v => { isOptimizingValue = v; });
    const unsubProgress = optimizationProgress.subscribe(v => { optimizationProgressValue = v; });
    const unsubServices = services.subscribe(v => { servicesValue = v; });
    
    cleanupFns.push(unsubAppStatus, unsubIsOptimizing, unsubProgress, unsubServices);
    
    if (!isTauri) {
      // Mock services for browser preview
      services.set(mockDashboardServices);
      
      // Mock latency history for browser preview
      latencyHistory = [45, 52, 48, 55, 42, 38, 65, 58, 45, 52, 48, 55, 42, 38, 65, 58, 45, 52, 48, 55];
      
      // Start traffic monitor for demo
      trafficMonitor.start();
      
      // Mock active connections
      activeConnections = [
        { domain: 'youtube.com', method: 'strategy', strategyName: 'Zapret v1', bytesTransferred: 15728640, duration: 125 },
        { domain: 'discord.com', method: 'strategy', strategyName: 'Zapret v1', bytesTransferred: 2097152, duration: 45 },
        { domain: 'twitch.tv', method: 'vless', bytesTransferred: 52428800, duration: 320 }
      ];
      
      // Add demo logs
      logs.info('system', 'Dashboard loaded');
      logs.success('youtube', 'Connection established');
      
      // Short delay for browser preview to show skeleton
      await new Promise(r => setTimeout(r, 500));
      isLoading = false;
      initialized = true;
      isInitializing = false;
      return;
    }
    
    // Wait for backend to be ready before making any calls
    // Backend initialization includes binary integrity verification which can take 3-5 seconds
    const backendReady = await waitForBackend(30, 300);
    if (!backendReady) {
      logs.error('system', 'Backend failed to initialize after retries');
      backendFailed = true;
      isLoading = false;
      isInitializing = false;
      return;
    }
    
    // Backend is ready, reset failed state
    backendFailed = false;
    
    // Async initialization for Tauri
    let unlistenProgress: (() => void) | undefined;
    let unlistenComplete: (() => void) | undefined;
    let unlistenFailed: (() => void) | undefined;
    let unlistenApplied: (() => void) | undefined;
    let unlistenStopped: (() => void) | undefined;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');
      
      // Load initial status
      try {
        const status = await invoke('get_status') as {is_active: boolean; current_strategy: string | null; current_strategy_name: string | null};
        appStatus.set({
          isActive: status.is_active,
          currentStrategy: status.current_strategy,
          currentStrategyName: status.current_strategy_name ?? null
        });
      } catch (e) {
        logs.error('system', `get_status failed: ${e}`);
      }
      
      // Load services from registry and check their health
      try {
        const registryServices = await invoke<{id: string; name: string}[]>('get_registry_services');
        if (registryServices && registryServices.length > 0) {
          services.set(registryServices.map(s => ({
            id: s.id,
            name: s.name,
            icon: getServiceIconEmoji(s.id),
            enabled: true,
            status: 'unknown' as const
          })));
          
          // Initial health check
          await checkServicesHealth();
        }
      } catch (e) {
        logs.error('system', `get_registry_services failed: ${e}`);
        
        // Fallback to old get_services
        try {
          const loadedServices = await invoke('get_services') as {id: string; name: string; critical: boolean}[];
          if (loadedServices && loadedServices.length > 0) {
            services.set(loadedServices.map(s => ({
              id: s.id,
              name: s.name,
              icon: getServiceIconEmoji(s.id),
              enabled: true,
              status: 'unknown' as const
            })));
          }
        } catch (e2) {
          logs.error('system', `get_services fallback failed: ${e2}`);
        }
      }
      
      // Start periodic health checks (every 30 seconds)
      healthCheckInterval = setInterval(() => {
        checkServicesHealth();
      }, 30000);

      // Start network stats simulation (updates every second when active)
      // TODO: Replace with real network stats from backend when available
      networkStatsInterval = setInterval(() => {
        if (appStatusValue.isActive) {
          // Simulate realistic network traffic when protection is active
          const baseDownload = 50000 + Math.random() * 200000; // 50-250 KB/s
          const baseUpload = 10000 + Math.random() * 50000;    // 10-60 KB/s
          
          networkStats = {
            downloadSpeed: Math.round(baseDownload + (Math.random() - 0.5) * 20000),
            uploadSpeed: Math.round(baseUpload + (Math.random() - 0.5) * 10000),
            totalDownload: networkStats.totalDownload + Math.round(baseDownload),
            totalUpload: networkStats.totalUpload + Math.round(baseUpload),
            activeConnections: Math.floor(3 + Math.random() * 8),
            isSimulated: true // Mark as simulated
          };
        } else {
          // Minimal traffic when inactive
          networkStats = {
            downloadSpeed: Math.round(Math.random() * 5000),
            uploadSpeed: Math.round(Math.random() * 2000),
            totalDownload: networkStats.totalDownload,
            totalUpload: networkStats.totalUpload,
            activeConnections: Math.floor(Math.random() * 2),
            isSimulated: true // Mark as simulated
          };
        }
      }, 1000);

      // Subscribe to optimization events
      unlistenProgress = await listen('automation:progress', (event) => {
        const payload = event.payload as { stage: string; percent: number; message: string };
        optimizationProgress.set({
          step: payload.stage,
          progress: payload.percent,
          message: payload.message,
          isComplete: false,
          error: null
        });
        logs.debug('optimization', payload.message);
      });
      
      unlistenComplete = await listen('automation:complete', (event) => {
        const result = event.payload as {strategy_id: string; strategy_name: string; score: number};
        appStatus.set({
          isActive: true,
          currentStrategy: result.strategy_id,
          currentStrategyName: result.strategy_name ?? null
        });
        isOptimizing.set(false);
        isScanning = false; // Reset scan loading state
        optimizationProgress.set({
          step: 'completed',
          progress: 100,
          message: 'Done',
          isComplete: true,
          error: null
        });
        logs.success('optimization', `Strategy applied: ${result.strategy_name}`);
        
        // Refresh health after strategy applied
        checkServicesHealth();
      });
      
      unlistenFailed = await listen('automation:error', (event) => {
        isOptimizing.set(false);
        isScanning = false; // Reset scan loading state
        optimizationProgress.set({
          step: 'failed',
          progress: 0,
          message: '',
          isComplete: false,
          error: event.payload as string
        });
        logs.error('optimization', `Failed: ${event.payload}`);
      });

      unlistenApplied = await listen('strategy:applied', (event) => {
        const payload = event.payload as {strategy_id: string; strategy_name: string};
        appStatus.set({
          isActive: true,
          currentStrategy: payload.strategy_id,
          currentStrategyName: payload.strategy_name ?? null
        });
        logs.success('strategy', `Applied: ${payload.strategy_name}`);
        
        // Refresh health after strategy applied
        checkServicesHealth();
      });

      unlistenStopped = await listen('strategy:stopped', () => {
        appStatus.set({
          isActive: false,
          currentStrategy: null,
          currentStrategyName: null
        });
        logs.info('strategy', 'Protection stopped');
        
        // Refresh health after strategy stopped
        checkServicesHealth();
      });
      
      cleanupFns.push(
        () => unlistenProgress?.(),
        () => unlistenComplete?.(),
        () => unlistenFailed?.(),
        () => unlistenApplied?.(),
        () => unlistenStopped?.()
      );
      
      // Data loaded successfully
      isLoading = false;
      initialized = true;
      isInitializing = false;
    } catch (e) {
      logs.error('system', `Failed to initialize: ${e}`);
      // CRITICAL: Clear intervals on error to prevent memory leak
      clearAllIntervals();
      isLoading = false;
      initialized = true; // Mark as initialized even on error to prevent infinite retries
      isInitializing = false;
    }
  }

  // Use $effect with cleanup for proper lifecycle management
  // CRITICAL: Guard prevents re-initialization on reactive updates
  $effect(() => {
    // Guard: prevent re-initialization if already initialized or initializing
    // This prevents memory leaks from duplicate intervals/subscriptions
    if (initialized || isInitializing) return;
    
    initializeDashboard();
    
    // Cleanup function returned from $effect
    // This runs when component unmounts or before effect re-runs
    return () => {
      // Clear all store subscriptions
      cleanupFns.forEach(fn => fn());
      cleanupFns = [];
      
      // Clear all intervals
      clearAllIntervals();
      
      // Stop traffic monitor
      trafficMonitor.stop();
      
      // Reset flags for potential remount
      initialized = false;
      isInitializing = false;
    };
  });

  async function handleToggle() {
    if (!browser) return;
    
    if (appStatusValue.isActive) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('stop_strategy');
        appStatus.set({
          isActive: false,
          currentStrategy: null,
          currentStrategyName: null
        });
      } catch (e) {
        logs.error('system', `Failed to stop: ${e}`);
      }
    } else {
      isOptimizing.set(true);
      optimizationProgress.set({
        step: 'initializing',
        progress: 0,
        message: 'Finding best strategy...',
        isComplete: false,
        error: null
      });
      
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('run_optimization_v2');
      } catch (e) {
        logs.error('optimization', `Failed: ${e}`);
        isOptimizing.set(false);
        optimizationProgress.update(p => ({ ...p, error: String(e) }));
      }
    }
  }

  async function handleQuickAction(actionId: string) {
    if (!browser) return;
    
    switch (actionId) {
      case 'scan':
        if (isScanning) return; // Prevent double-click
        logs.info('system', 'Starting full scan...');
        isScanning = true;
        isOptimizing.set(true);
        optimizationProgress.set({
          step: 'initializing',
          progress: 0,
          message: 'Starting deep scan...',
          isComplete: false,
          error: null
        });
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('run_optimization_v2');
        } catch (e) {
          logs.error('system', `Scan failed: ${e}`);
          isOptimizing.set(false);
          optimizationProgress.update(p => ({ ...p, error: String(e) }));
        } finally {
          isScanning = false;
        }
        break;
      case 'test':
        if (isTesting) return; // Prevent double-click
        logs.info('system', 'Testing services...');
        isTesting = true;
        // If we have an active strategy, test it; otherwise just check services
        if (appStatusValue.currentStrategy) {
          try {
            const { invoke } = await import('@tauri-apps/api/core');
            const result = await invoke<{
              strategy_id: string;
              score: number;
              success_rate: number;
              avg_latency_ms: number;
              services_passed: string[];
              services_failed: string[];
            }>('test_strategy', { strategyId: appStatusValue.currentStrategy });
            
            logs.success('test', `Strategy score: ${result.score.toFixed(1)}, ${result.services_passed.length}/${result.services_passed.length + result.services_failed.length} services passed`);
            
            // Refresh health after test
            await checkServicesHealth();
          } catch (e) {
            logs.error('system', `Test failed: ${e}`);
          } finally {
            isTesting = false;
          }
        } else {
          // No active strategy - just check services health
          try {
            await checkServicesHealth();
            logs.info('system', 'Health check completed');
          } finally {
            isTesting = false;
          }
        }
        break;
      case 'proxy':
        goto('/proxies');
        break;
      case 'settings':
        goto('/settings');
        break;
    }
  }
</script>

<div class="h-full p-8 overflow-auto bg-gradient-to-br from-zinc-950 to-black">
  <!-- Page Header -->
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-white tracking-tight">{t('dashboard.title')}</h1>
    <p class="text-sm text-zinc-400 mt-2">{t('dashboard.subtitle')}</p>
  </div>

  <!-- Skeleton while loading or Backend Error -->
  {#if backendFailed}
    <BackendNotReady onRetry={handleRetryBackend} />
  {:else if isLoading}
    <DashboardSkeleton />
  {:else}
    <!-- Bento Grid with Drag-n-Drop -->
    <BentoGrid columns={4} gap={4} draggable={true} order={widgetOrder} onReorder={handleReorder}>
      {#each widgetOrder as widgetId, index (widgetId)}
        {#if widgetId === 'shield'}
          <!-- Shield Indicator Widget (2x2) - Main Control -->
          <BentoWidget colspan={2} rowspan={2} widgetId="shield" {index}>
            <div class="relative h-full flex flex-col items-center justify-center gap-6">
              <!-- Glow background when active -->
              {#if appStatusValue.isActive && !isOptimizingValue}
                <div class="absolute inset-0 bg-neon-green/5 rounded-xl blur-2xl animate-pulse-glow pointer-events-none"></div>
              {/if}
              
              <!-- Shield Indicator -->
              <ShieldIndicator 
                status={protectionStatus}
                onclick={handleToggle}
              />
              
              <!-- Mode Selector -->
              <div class="w-full max-w-xs mt-4">
                <ModeSelector 
                  currentMode={currentMode}
                  onModeChange={handleModeChange}
                  disabled={isOptimizingValue}
                />
              </div>
            </div>
          </BentoWidget>
        {:else if widgetId === 'activity'}
          <!-- Live Activity Panel (2x2) -->
          <BentoWidget colspan={2} rowspan={2} title={t('dashboard.widgets.liveActivity')} icon="📡" widgetId="activity" {index}>
            <LiveActivityPanel 
              trafficHistory={trafficMonitor.history}
              connections={activeConnections}
              maxConnections={5}
            />
          </BentoWidget>
        {:else if widgetId === 'health'}
          <!-- Health Monitor Widget (2x1) -->
          <BentoWidget colspan={2} title={t('dashboard.widgets.healthMonitor')} icon="💚" widgetId="health" {index}>
            <HealthWidget services={healthServices.length > 0 ? healthServices : undefined} />
          </BentoWidget>
        {:else if widgetId === 'method'}
          <!-- Active Method Widget (1x1) -->
          <BentoWidget title={t('dashboard.widgets.activeMethod')} icon="⚡" widgetId="method" {index}>
            <MethodWidget 
              method={currentMethod}
              methodName={appStatusValue.currentStrategyName || undefined}
              active={appStatusValue.isActive}
            />
          </BentoWidget>
        {:else if widgetId === 'actions'}
          <!-- Quick Actions Widget (1x1) -->
          <BentoWidget title={t('dashboard.widgets.quickActions')} icon="🚀" widgetId="actions" {index}>
            <QuickActionsWidget 
              actions={quickActions} 
              onAction={handleQuickAction}
              backendReady={!isLoading}
              hasActiveStrategy={!!appStatusValue.currentStrategy}
            />
          </BentoWidget>
        {:else if widgetId === 'failover'}
          <!-- Auto Recovery Widget (1x1) -->
          <BentoWidget title={t('dashboard.widgets.autoRecovery')} icon="🔄" widgetId="failover" {index}>
            <FailoverStatusWidget compact={false} />
          </BentoWidget>
        {:else if widgetId === 'network'}
          <!-- Network Stats Widget (2x1) -->
          <BentoWidget colspan={2} title={t('dashboard.widgets.networkStats')} icon="📊" widgetId="network" {index}>
            <NetworkStatsWidget stats={networkStats} />
          </BentoWidget>
        {:else if widgetId === 'latency'}
          <!-- Latency Monitor Widget (2x1) -->
          <BentoWidget colspan={2} title={t('dashboard.widgets.latencyMonitor')} icon="📈" widgetId="latency" {index}>
            <LatencyWidget 
              history={latencyHistory} 
              currentLatency={currentLatency}
              label="Avg Service Latency"
            />
          </BentoWidget>
        {:else if widgetId === 'connections'}
          <!-- Connection Stats Widget (2x1) -->
          <BentoWidget colspan={2} title={t('dashboard.widgets.connectionStats')} icon="🔗" widgetId="connections" {index}>
            <ConnectionStatsWidget />
          </BentoWidget>
        {/if}
      {/each}
    </BentoGrid>

    <!-- Progress Bar (shown during optimization) -->
    {#if isOptimizingValue}
      <div class="mt-6 p-4 bg-void-50 border border-glass-border rounded-xl">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-3">
            <ScanningIndicator active={true} text="" variant="dots" />
            <span class="text-sm text-text-secondary">{optimizationProgressValue.message || 'Optimizing...'}</span>
          </div>
          <span class="text-sm text-electric font-mono">{optimizationProgressValue.progress}%</span>
        </div>
        <div class="h-1.5 bg-void-200 rounded-full overflow-hidden">
          <div 
            class="h-full bg-gradient-to-r from-electric to-neon-cyan transition-all duration-300 shadow-glow-cyan"
            style="width: {optimizationProgressValue.progress}%"
          ></div>
        </div>
      </div>
    {/if}

    <!-- Error State -->
    {#if optimizationProgressValue.error}
      <div class="mt-6 p-4 bg-neon-red/10 border border-neon-red/30 rounded-xl">
        <div class="flex items-center gap-3">
          <span class="text-neon-red text-xl">⚠️</span>
          <div>
            <p class="text-sm font-medium text-neon-red">{t('dashboard.optimization.failed')}</p>
            <p class="text-xs text-text-muted mt-1">{optimizationProgressValue.error}</p>
          </div>
        </div>
      </div>
    {/if}

    <!-- Plugin Widgets -->
    <div class="mt-6">
      <PluginSlot location="dashboard">
        {#snippet fallback()}
          <!-- Empty state - no plugins with dashboard UI -->
        {/snippet}
      </PluginSlot>
    </div>
  {/if}
</div>
