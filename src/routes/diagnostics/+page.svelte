<script lang="ts">
  import { browser } from '$app/environment';
  import { Button, ScanningIndicator } from '$lib/components';
  import { toasts } from '$lib/stores/toast';
  import { waitForBackend } from '$lib/utils/backend';
  import { 
    mockDiagnosticsComponents, 
    mockSystemInfo as defaultSystemInfo, 
    mockDiagnosticsResults,
    mockConflicts
  } from '$lib/mocks';
  import type { ConflictInfo, ConflictSeverity, ConflictCategory } from '$lib/api/types';

  // Types
  type ComponentStatus = 'healthy' | 'warning' | 'error' | 'unknown' | 'checking';
  
  interface SystemComponent {
    id: string;
    name: string;
    description: string;
    status: ComponentStatus;
    details: string;
    icon: string;
  }

  interface SystemInfo {
    os: string;
    osVersion: string;
    arch: string;
    memory: string;
    adminRights: boolean;
  }

  // State
  let components = $state<SystemComponent[]>([...mockDiagnosticsComponents]);
  let conflicts = $state<ConflictInfo[]>([]);
  let isCheckingConflicts = $state(false);
  
  // Demo mode flag (browser preview without Tauri)
  let isDemoMode = $state(false);

  let systemInfo = $state<SystemInfo>({
    os: 'Windows',
    osVersion: '...',
    arch: '...',
    memory: '...',
    adminRights: false
  });

  let isRunning = $state(false);
  let isTauri = $state(false);
  let lastCheck = $state<string | null>(null);
  let overallHealth = $derived(calculateOverallHealth());

  function calculateOverallHealth(): { status: ComponentStatus; percentage: number } {
    const checked = components.filter(c => c.status !== 'unknown' && c.status !== 'checking');
    if (checked.length === 0) return { status: 'unknown', percentage: 0 };
    
    const healthy = checked.filter(c => c.status === 'healthy').length;
    const warnings = checked.filter(c => c.status === 'warning').length;
    const errors = checked.filter(c => c.status === 'error').length;
    
    const percentage = Math.round((healthy / checked.length) * 100);
    
    if (errors > 0) return { status: 'error', percentage };
    if (warnings > 0) return { status: 'warning', percentage };
    return { status: 'healthy', percentage };
  }

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    isDemoMode = !isTauri;
    loadSystemInfo();
    checkConflicts();
  });

  async function loadSystemInfo() {
    if (!browser) return;
    
    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Wait for backend
        const ready = await waitForBackend(10, 200);
        if (!ready) {
          console.warn('[Diagnostics] Backend not ready after retries');
          return;
        }
        
        const info = await invoke<SystemInfo>('get_system_info').catch(() => null);
        if (info) systemInfo = info;
      } catch (e) {
        console.error('Failed to load system info:', e);
      }
    } else {
      // Demo data
      systemInfo = { ...defaultSystemInfo };
    }
  }

  async function checkConflicts() {
    if (!browser) return;
    
    isCheckingConflicts = true;
    
    try {
      if (isTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        
        const ready = await waitForBackend(10, 200);
        if (!ready) {
          console.warn('[Diagnostics] Backend not ready for conflict check');
          return;
        }
        
        const result = await invoke<ConflictInfo[]>('check_conflicts').catch(() => []);
        conflicts = result;
      } else {
        // Demo mode - show mock conflicts
        await new Promise(r => setTimeout(r, 500));
        conflicts = [...mockConflicts] as ConflictInfo[];
      }
    } catch (e) {
      console.error('Failed to check conflicts:', e);
    } finally {
      isCheckingConflicts = false;
    }
  }

  async function runDiagnostics() {
    isRunning = true;
    
    // Reset all to checking
    components = components.map(c => ({ ...c, status: 'checking' as ComponentStatus, details: 'Checking...' }));
    
    try {
      if (isTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Wait for backend
        const ready = await waitForBackend(10, 200);
        if (!ready) {
          console.warn('[Diagnostics] Backend not ready for diagnostics');
          toasts.error('Backend not ready');
          return;
        }
        
        // Run diagnostics
        const results = await invoke<Record<string, { status: string; details: string }>>('run_diagnostics').catch(() => null);
        
        if (results) {
          components = components.map(c => ({
            ...c,
            status: (results[c.id]?.status || 'unknown') as ComponentStatus,
            details: results[c.id]?.details || 'No data'
          }));
        }
      } else {
        // Demo mode - simulate checks
        await simulateDiagnostics();
      }
      
      lastCheck = new Date().toLocaleTimeString();
      toasts.success('Diagnostics completed');
    } catch (e) {
      console.error('Diagnostics failed:', e);
      toasts.error(`Diagnostics failed: ${e}`);
    } finally {
      isRunning = false;
    }
  }

  // Demo mode simulation - generates mock diagnostic results for browser preview
  async function simulateDiagnostics() {
    for (const check of mockDiagnosticsResults) {
      await new Promise(r => setTimeout(r, check.delay));
      components = components.map(c => 
        c.id === check.id 
          ? { ...c, status: check.status as ComponentStatus, details: check.details }
          : c
      );
    }
  }

  function getStatusColor(status: ComponentStatus): string {
    switch (status) {
      case 'healthy': return 'text-neon-green';
      case 'warning': return 'text-neon-yellow';
      case 'error': return 'text-neon-red';
      case 'checking': return 'text-electric';
      default: return 'text-text-muted';
    }
  }

  function getStatusBgColor(status: ComponentStatus): string {
    switch (status) {
      case 'healthy': return 'bg-neon-green/20 border-neon-green/30';
      case 'warning': return 'bg-neon-yellow/20 border-neon-yellow/30';
      case 'error': return 'bg-neon-red/20 border-neon-red/30';
      case 'checking': return 'bg-electric/20 border-electric/30';
      default: return 'bg-void-200 border-void-300';
    }
  }

  function getStatusIcon(status: ComponentStatus): string {
    switch (status) {
      case 'healthy': return '✓';
      case 'warning': return '⚠';
      case 'error': return '✗';
      case 'checking': return '◌';
      default: return '○';
    }
  }

  function getHealthGradient(status: ComponentStatus): string {
    switch (status) {
      case 'healthy': return 'from-neon-green to-neon-cyan';
      case 'warning': return 'from-neon-yellow to-neon-orange';
      case 'error': return 'from-neon-red to-neon-pink';
      default: return 'from-void-200 to-void-300';
    }
  }

  function getSeverityColor(severity: ConflictSeverity): string {
    switch (severity) {
      case 'critical': return 'text-neon-red';
      case 'high': return 'text-neon-orange';
      case 'medium': return 'text-neon-yellow';
      case 'low': return 'text-text-muted';
    }
  }

  function getSeverityBgColor(severity: ConflictSeverity): string {
    switch (severity) {
      case 'critical': return 'bg-neon-red/20 border-neon-red/30';
      case 'high': return 'bg-neon-orange/20 border-neon-orange/30';
      case 'medium': return 'bg-neon-yellow/20 border-neon-yellow/30';
      case 'low': return 'bg-void-200 border-void-300';
    }
  }

  function getCategoryIcon(category: ConflictCategory): string {
    switch (category) {
      case 'network_filter': return '🛡️';
      case 'vpn': return '🔐';
      case 'network_optimization': return '⚡';
      case 'security': return '🔒';
      case 'windivert': return '⚠️';
    }
  }

  function getCategoryLabel(category: ConflictCategory): string {
    switch (category) {
      case 'network_filter': return 'Network Filter';
      case 'vpn': return 'VPN';
      case 'network_optimization': return 'Network Optimization';
      case 'security': return 'Security Software';
      case 'windivert': return 'WinDivert Conflict';
    }
  }

  function exportReport() {
    const report = {
      timestamp: new Date().toISOString(),
      lastCheck,
      overallHealth: {
        status: overallHealth.status,
        percentage: overallHealth.percentage
      },
      systemInfo,
      components: components.map(c => ({
        id: c.id,
        name: c.name,
        status: c.status,
        details: c.details
      })),
      conflicts: conflicts.map(c => ({
        name: c.name,
        category: c.category,
        severity: c.severity,
        description: c.description,
        recommendation: c.recommendation,
        detected_processes: c.detected_processes,
        detected_services: c.detected_services
      }))
    };
    
    const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `isolate-diagnostics-${new Date().toISOString().slice(0, 10)}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    
    toasts.success('Report exported');
  }

  let isFixing = $state(false);

  async function autoFix() {
    if (!isTauri || isDemoMode) {
      toasts.info('Auto-fix is not available in demo mode');
      return;
    }

    isFixing = true;
    toasts.info('Running auto-fix...');

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // 1. Stop any running strategy
      try {
        await invoke('stop_strategy');
      } catch {
        // Ignore - may not have running strategy
      }

      // 2. Clear system proxy
      try {
        await invoke('clear_system_proxy');
      } catch {
        // Ignore - proxy may not be set
      }

      // 3. Panic reset (clears WinDivert state)
      try {
        await invoke('panic_reset');
      } catch {
        // Ignore - may fail if not admin
      }

      toasts.success('Auto-fix completed. Running diagnostics...');

      // Re-run diagnostics to check results
      await runDiagnostics();
    } catch (e) {
      toasts.error(`Auto-fix failed: ${e}`);
    } finally {
      isFixing = false;
    }
  }
</script>

<div class="p-8 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <div class="flex items-center gap-3">
        <h1 class="text-3xl font-bold text-white">System Diagnostics</h1>
        {#if isDemoMode}
          <span class="px-2 py-1 text-xs uppercase tracking-wider bg-amber-500/20 text-amber-400 rounded-md font-medium border border-amber-500/30">Demo</span>
        {/if}
      </div>
      <p class="text-text-muted mt-1">Check system components and network health</p>
    </div>
    <div class="flex items-center gap-4">
      {#if lastCheck}
        <span class="text-text-muted text-sm">Last check: {lastCheck}</span>
      {/if}
      <Button 
        variant="primary" 
        onclick={runDiagnostics}
        loading={isRunning}
        disabled={isRunning}
      >
        {#snippet children()}
          {#if !isRunning}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          {/if}
          Run Diagnostics
        {/snippet}
      </Button>
    </div>
  </div>

  <!-- Overall Health Card -->
  <div class="bg-void-50 rounded-2xl border border-glass-border p-6">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-4">
        <!-- Health Ring -->
        <div class="relative w-20 h-20">
          <svg class="w-20 h-20 transform -rotate-90">
            <circle
              cx="40" cy="40" r="36"
              stroke="currentColor"
              stroke-width="6"
              fill="none"
              class="text-void-200"
            />
            <circle
              cx="40" cy="40" r="36"
              stroke="currentColor"
              stroke-width="6"
              fill="none"
              stroke-linecap="round"
              stroke-dasharray="{overallHealth.percentage * 2.26} 226"
              class="{getStatusColor(overallHealth.status)}"
            />
          </svg>
          <div class="absolute inset-0 flex items-center justify-center">
            <span class="text-xl font-bold text-white">{overallHealth.percentage}%</span>
          </div>
        </div>
        
        <div>
          <h2 class="text-xl font-semibold text-white">System Health</h2>
          <p class="text-text-muted">
            {#if overallHealth.status === 'healthy'}
              All systems operational
            {:else if overallHealth.status === 'warning'}
              Some components need attention
            {:else if overallHealth.status === 'error'}
              Critical issues detected
            {:else}
              Run diagnostics to check
            {/if}
          </p>
        </div>
      </div>
      
      <!-- Quick Stats -->
      <div class="flex gap-6">
        <div class="text-center">
          <p class="text-2xl font-bold text-neon-green">{components.filter(c => c.status === 'healthy').length}</p>
          <p class="text-text-muted text-sm">Healthy</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-neon-yellow">{components.filter(c => c.status === 'warning').length}</p>
          <p class="text-text-muted text-sm">Warnings</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-neon-red">{components.filter(c => c.status === 'error').length}</p>
          <p class="text-text-muted text-sm">Errors</p>
        </div>
      </div>
    </div>
  </div>

  <!-- Software Conflicts Section -->
  {#if isCheckingConflicts}
    <div class="bg-void-50 rounded-xl border border-glass-border p-5">
      <div class="flex items-center gap-3">
        <ScanningIndicator active={true} text="" variant="pulse" />
        <span class="text-text-muted">Checking for software conflicts...</span>
      </div>
    </div>
  {:else if conflicts.length > 0}
    <div class="bg-void-50 rounded-xl border border-neon-red/30 p-5">
      <div class="flex items-start gap-3 mb-4">
        <div class="w-10 h-10 rounded-lg bg-neon-red/20 flex items-center justify-center flex-shrink-0">
          <svg class="w-5 h-5 text-neon-red" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        <div>
          <h3 class="text-white font-semibold">Software Conflicts Detected</h3>
          <p class="text-text-muted text-sm">
            {conflicts.length} conflicting {conflicts.length === 1 ? 'program' : 'programs'} found that may interfere with Isolate
          </p>
        </div>
      </div>
      
      <div class="space-y-3">
        {#each conflicts as conflict}
          <div class="bg-void-100/50 rounded-lg p-4 border border-glass-border">
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2">
                <span class="text-xl">{getCategoryIcon(conflict.category)}</span>
                <div>
                  <h4 class="text-white font-medium">{conflict.name}</h4>
                  <span class="text-text-muted text-xs">{getCategoryLabel(conflict.category)}</span>
                </div>
              </div>
              <span class="px-2 py-1 rounded-full text-xs font-medium border {getSeverityBgColor(conflict.severity)} {getSeverityColor(conflict.severity)} uppercase">
                {conflict.severity}
              </span>
            </div>
            
            <p class="text-text-muted text-sm mb-2">{conflict.description}</p>
            
            <div class="flex flex-wrap gap-2 mb-2">
              {#each conflict.detected_processes as proc}
                <span class="px-2 py-0.5 bg-void-200 rounded text-xs text-text-muted font-mono">{proc}</span>
              {/each}
              {#each conflict.detected_services as svc}
                <span class="px-2 py-0.5 bg-void-200 rounded text-xs text-text-muted font-mono">{svc}</span>
              {/each}
            </div>
            
            <div class="flex items-start gap-2 mt-3 pt-3 border-t border-glass-border">
              <svg class="w-4 h-4 text-neon-cyan flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <p class="text-neon-cyan text-sm">{conflict.recommendation}</p>
            </div>
          </div>
        {/each}
      </div>
      
      <button
        onclick={checkConflicts}
        class="mt-4 flex items-center gap-2 px-4 py-2 bg-void-100 hover:bg-void-200 rounded-lg text-white text-sm transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        Re-check Conflicts
      </button>
    </div>
  {/if}

  <!-- Main Grid: Components + System Info -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Components Grid (2 columns) -->
    <div class="lg:col-span-2 space-y-4">
      <h3 class="text-lg font-semibold text-white">Components</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {#each components as component}
          <div class="bg-void-50 rounded-xl border border-glass-border p-5 hover:border-electric/30 transition-colors">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-3">
                <span class="text-2xl">{component.icon}</span>
                <div>
                  <h4 class="text-white font-medium">{component.name}</h4>
                  <p class="text-text-muted text-sm">{component.description}</p>
                </div>
              </div>
              
              <!-- Status Badge -->
              <div class="flex items-center gap-2 px-2.5 py-1 rounded-full border {getStatusBgColor(component.status)}">
                {#if component.status === 'checking'}
                  <ScanningIndicator active={true} text="" variant="pulse" />
                {:else}
                  <span class="text-sm font-bold {getStatusColor(component.status)}">{getStatusIcon(component.status)}</span>
                {/if}
                <span class="text-xs font-medium {getStatusColor(component.status)} capitalize">{component.status}</span>
              </div>
            </div>
            
            <!-- Details -->
            <div class="mt-3 pt-3 border-t border-glass-border">
              <p class="text-sm {component.status === 'checking' ? 'text-electric animate-pulse' : 'text-text-muted'}">
                {component.details}
              </p>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- System Info Sidebar -->
    <div class="space-y-4">
      <h3 class="text-lg font-semibold text-white">System Information</h3>
      
      <div class="bg-void-50 rounded-xl border border-glass-border overflow-hidden">
        <!-- OS Info -->
        <div class="p-4 border-b border-glass-border">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-electric/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-electric" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <div>
              <p class="text-text-muted text-sm">Operating System</p>
              <p class="text-white font-medium">{systemInfo.os} {systemInfo.osVersion}</p>
            </div>
          </div>
        </div>
        
        <!-- Architecture -->
        <div class="p-4 border-b border-glass-border">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-neon-cyan/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-neon-cyan" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
            </div>
            <div>
              <p class="text-text-muted text-sm">Architecture</p>
              <p class="text-white font-medium">{systemInfo.arch}</p>
            </div>
          </div>
        </div>
        
        <!-- Memory -->
        <div class="p-4 border-b border-glass-border">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-neon-green/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-neon-green" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </div>
            <div>
              <p class="text-text-muted text-sm">Memory</p>
              <p class="text-white font-medium">{systemInfo.memory}</p>
            </div>
          </div>
        </div>
        
        <!-- Admin Rights -->
        <div class="p-4">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg {systemInfo.adminRights ? 'bg-neon-green/20' : 'bg-neon-red/20'} flex items-center justify-center">
              <svg class="w-5 h-5 {systemInfo.adminRights ? 'text-neon-green' : 'text-neon-red'}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
              </svg>
            </div>
            <div>
              <p class="text-text-muted text-sm">Admin Rights</p>
              <p class="font-medium {systemInfo.adminRights ? 'text-neon-green' : 'text-neon-red'}">
                {systemInfo.adminRights ? 'Elevated' : 'Not Elevated'}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="bg-void-50 rounded-xl border border-glass-border p-4">
        <h4 class="text-white font-medium mb-3">Quick Actions</h4>
        <div class="space-y-2">
          <button
            onclick={runDiagnostics}
            disabled={isRunning}
            class="w-full flex items-center gap-3 px-4 py-3 bg-void-100/50 hover:bg-void-100 rounded-lg text-left transition-colors disabled:opacity-50"
          >
            <svg class="w-5 h-5 text-electric" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span class="text-white text-sm">Re-run All Checks</span>
          </button>
          
          <button
            onclick={exportReport}
            class="w-full flex items-center gap-3 px-4 py-3 bg-void-100/50 hover:bg-void-100 rounded-lg text-left transition-colors"
          >
            <svg class="w-5 h-5 text-neon-cyan" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <span class="text-white text-sm">Export Report</span>
          </button>
          
          <button
            onclick={autoFix}
            disabled={isFixing || isRunning || isDemoMode}
            class="w-full flex items-center gap-3 px-4 py-3 bg-void-100/50 hover:bg-void-100 rounded-lg text-left transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            title={isDemoMode ? 'Not available in demo mode' : 'Stop strategies, clear proxy, reset network state'}
          >
            {#if isFixing}
              <svg class="w-5 h-5 text-neon-yellow animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            {:else}
              <svg class="w-5 h-5 text-neon-yellow" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
              </svg>
            {/if}
            <div class="flex flex-col">
              <span class="text-white text-sm">{isFixing ? 'Fixing...' : 'Auto-fix Issues'}</span>
              <span class="text-text-muted/60 text-xs">Stop strategies, clear proxy, reset state</span>
            </div>
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Troubleshooting Tips (shown when errors/warnings exist) -->
  {#if components.some(c => c.status === 'error' || c.status === 'warning')}
    <div class="bg-void-50 rounded-xl border border-neon-yellow/30 p-5">
      <div class="flex items-start gap-3">
        <div class="w-10 h-10 rounded-lg bg-neon-yellow/20 flex items-center justify-center flex-shrink-0">
          <svg class="w-5 h-5 text-neon-yellow" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <div>
          <h4 class="text-white font-medium mb-2">Troubleshooting Tips</h4>
          <ul class="text-text-muted text-sm space-y-1">
            {#if components.find(c => c.id === 'windivert')?.status === 'error'}
              <li>• <span class="text-neon-red">WinDivert:</span> Run as Administrator or reinstall the driver</li>
            {/if}
            {#if components.find(c => c.id === 'singbox')?.status === 'warning'}
              <li>• <span class="text-neon-yellow">Sing-box:</span> Configure proxy settings in Marketplace</li>
            {/if}
            {#if components.find(c => c.id === 'network')?.status === 'error'}
              <li>• <span class="text-neon-red">Network:</span> Check your internet connection</li>
            {/if}
            {#if components.find(c => c.id === 'firewall')?.status === 'warning'}
              <li>• <span class="text-neon-yellow">Firewall:</span> Allow Isolate through Windows Firewall</li>
            {/if}
            {#if components.find(c => c.id === 'tcp_timestamps')?.status === 'warning'}
              <li>• <span class="text-neon-yellow">TCP Timestamps:</span> Enable in Settings → Advanced for better DPI bypass</li>
            {/if}
          </ul>
        </div>
      </div>
    </div>
  {/if}
</div>
