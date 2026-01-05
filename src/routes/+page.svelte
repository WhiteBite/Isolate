<script lang="ts">
  import { browser } from '$app/environment';
  import { 
    BentoGrid, 
    BentoWidget,
    StatusWidget,
    HealthWidget,
    MethodWidget,
    QuickActionsWidget,
    DashboardSkeleton
  } from '$lib/components';
  import { 
    appStatus, 
    optimizationProgress, 
    isOptimizing, 
    services,
  } from '$lib/stores';
  import { logs } from '$lib/stores/logs';

  // Service icons mapping
  const serviceIcons: Record<string, string> = {
    youtube: '📺',
    discord: '💬',
    telegram: '✈️',
    twitch: '🎮',
    google: '🔍',
    instagram: '📷',
    twitter: '🐦',
    default: '🌐'
  };

  // Backend ServiceStatus type
  interface ServiceStatus {
    service_id: string;
    service_name: string;
    accessible: boolean;
    avg_latency_ms: number | null;
    success_rate: number;
    from_cache: boolean;
  }

  // Cleanup functions
  let cleanupFns: (() => void)[] = [];
  let healthCheckInterval: ReturnType<typeof setInterval> | null = null;
  let initialized = $state(false);
  
  // Loading state for skeleton
  let isLoading = $state(true);

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

  // Quick actions with loading states
  let quickActions = $derived([
    { id: 'scan', label: 'Scan All', loading: isScanning, disabled: isOptimizingValue },
    { id: 'test', label: 'Test Current', loading: isTesting },
    { id: 'proxy', label: 'Add Proxy' },
    { id: 'settings', label: 'Settings' }
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
              icon: serviceIcons[result.service_id.toLowerCase()] || serviceIcons.default,
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

  // Wait for backend to be ready with retry logic
  async function waitForBackend(retries = 10): Promise<boolean> {
    const { invoke } = await import('@tauri-apps/api/core');
    
    for (let i = 0; i < retries; i++) {
      try {
        const ready = await invoke<boolean>('is_backend_ready');
        if (ready) return true;
      } catch {
        // Backend not ready yet
      }
      await new Promise(r => setTimeout(r, 200));
    }
    return false;
  }

  // Initialize dashboard data
  async function initializeDashboard() {
    if (!browser || initialized) return;
    initialized = true;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    // Subscribe to stores
    const unsubAppStatus = appStatus.subscribe(v => { appStatusValue = v; });
    const unsubIsOptimizing = isOptimizing.subscribe(v => { isOptimizingValue = v; });
    const unsubProgress = optimizationProgress.subscribe(v => { optimizationProgressValue = v; });
    const unsubServices = services.subscribe(v => { servicesValue = v; });
    
    cleanupFns.push(unsubAppStatus, unsubIsOptimizing, unsubProgress, unsubServices);
    
    if (!isTauri) {
      // Mock services for browser preview
      services.set([
        { id: 'youtube', name: 'YouTube', icon: '📺', enabled: true, status: 'working', ping: 45 },
        { id: 'discord', name: 'Discord', icon: '💬', enabled: true, status: 'working', ping: 32 },
        { id: 'telegram', name: 'Telegram', icon: '✈️', enabled: true, status: 'unknown', ping: 120 },
        { id: 'twitch', name: 'Twitch', icon: '🎮', enabled: true, status: 'blocked' }
      ]);
      
      // Add demo logs
      logs.info('system', 'Dashboard loaded');
      logs.success('youtube', 'Connection established');
      
      // Short delay for browser preview to show skeleton
      await new Promise(r => setTimeout(r, 500));
      isLoading = false;
      return;
    }
    
    // Wait for backend to be ready before making any calls
    const backendReady = await waitForBackend();
    if (!backendReady) {
      logs.error('system', 'Backend failed to initialize after retries');
      isLoading = false;
      return;
    }
    
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
            icon: serviceIcons[s.id.toLowerCase()] || serviceIcons.default,
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
              icon: serviceIcons[s.id.toLowerCase()] || serviceIcons.default,
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

      // Subscribe to optimization events
      unlistenProgress = await listen('optimization:progress', (event) => {
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
      
      unlistenComplete = await listen('optimization:complete', (event) => {
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
      
      unlistenFailed = await listen('optimization:failed', (event) => {
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
    } catch (e) {
      logs.error('system', `Failed to initialize: ${e}`);
      isLoading = false;
    }
  }

  // Use $effect with cleanup for proper lifecycle management
  $effect(() => {
    initializeDashboard();
    
    // Cleanup function returned from $effect
    return () => {
      cleanupFns.forEach(fn => fn());
      cleanupFns = [];
      if (healthCheckInterval) {
        clearInterval(healthCheckInterval);
        healthCheckInterval = null;
      }
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
        await invoke('run_optimization', { mode: 'turbo' });
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
          await invoke('run_optimization', { mode: 'deep' });
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
        window.location.href = '/proxies';
        break;
      case 'settings':
        window.location.href = '/settings';
        break;
    }
  }
</script>

<div class="h-full p-8 overflow-auto bg-gradient-to-br from-zinc-950 to-black">
  <!-- Page Header -->
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-white tracking-tight">Dashboard</h1>
    <p class="text-sm text-zinc-500 mt-2">Monitor and control your network protection</p>
  </div>

  <!-- Skeleton while loading -->
  {#if isLoading}
    <DashboardSkeleton />
  {:else}
    <!-- Bento Grid -->
    <BentoGrid columns={4} gap={4}>
      <!-- Global Status Widget (2x2) - Main Control -->
      <BentoWidget colspan={2} rowspan={2}>
        <div class="relative h-full">
          <!-- Glow background when active -->
          {#if appStatusValue.isActive && !isOptimizingValue}
            <div class="absolute inset-0 bg-neon-green/5 rounded-xl blur-2xl animate-pulse-glow pointer-events-none"></div>
          {/if}
          <StatusWidget 
            active={appStatusValue.isActive}
            loading={isOptimizingValue}
            onToggle={handleToggle}
          />
        </div>
      </BentoWidget>

      <!-- Health Monitor Widget (2x1) -->
      <BentoWidget colspan={2} title="Health Monitor" icon="💚">
        <HealthWidget services={healthServices.length > 0 ? healthServices : undefined} />
      </BentoWidget>

      <!-- Active Method Widget (1x1) -->
      <BentoWidget title="Active Method" icon="⚡">
        <MethodWidget 
          method={currentMethod}
          methodName={appStatusValue.currentStrategyName || undefined}
          active={appStatusValue.isActive}
        />
      </BentoWidget>

      <!-- Quick Actions Widget (1x1) -->
      <BentoWidget title="Quick Actions" icon="🚀">
        <QuickActionsWidget actions={quickActions} onAction={handleQuickAction} />
      </BentoWidget>
    </BentoGrid>

    <!-- Progress Bar (shown during optimization) -->
    {#if isOptimizingValue}
      <div class="mt-6 p-4 bg-void-50 border border-glass-border rounded-xl">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm text-text-secondary">{optimizationProgressValue.message || 'Optimizing...'}</span>
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
            <p class="text-sm font-medium text-neon-red">Optimization Failed</p>
            <p class="text-xs text-text-muted mt-1">{optimizationProgressValue.error}</p>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>
