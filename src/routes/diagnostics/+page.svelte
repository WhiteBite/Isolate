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
  type ComponentCategory = 'network' | 'dpi' | 'system';
  
  interface SystemComponent {
    id: string;
    name: string;
    description: string;
    status: ComponentStatus;
    details: string;
    icon: string;
    category?: ComponentCategory;
  }

  interface SystemInfo {
    os: string;
    osVersion: string;
    arch: string;
    memory: string;
    adminRights: boolean;
  }

  interface DiagnosticsHistoryEntry {
    timestamp: string;
    overallStatus: ComponentStatus;
    percentage: number;
    healthy: number;
    warnings: number;
    errors: number;
  }

  // Category definitions for grouping
  const categoryConfig: Record<ComponentCategory, { title: string; icon: string; description: string }> = {
    network: { 
      title: 'Сеть и DNS', 
      icon: '🌐', 
      description: 'Проверка интернет-соединения и DNS-резолвинга' 
    },
    dpi: { 
      title: 'DPI-обход', 
      icon: '⚡', 
      description: 'Компоненты для обхода блокировок' 
    },
    system: { 
      title: 'Системные компоненты', 
      icon: '🔧', 
      description: 'Драйверы и системные настройки' 
    }
  };

  // Map component IDs to categories
  const componentCategories: Record<string, ComponentCategory> = {
    network: 'network',
    dns: 'network',
    windivert: 'system',
    firewall: 'system',
    tcp_timestamps: 'system',
    winws: 'dpi',
    singbox: 'dpi'
  };

  // State
  let components = $state<SystemComponent[]>([...mockDiagnosticsComponents].map(c => ({
    ...c,
    category: componentCategories[c.id] || 'system'
  })));
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
  let history = $state<DiagnosticsHistoryEntry[]>([]);
  let showHistory = $state(false);
  let overallHealth = $derived(calculateOverallHealth());

  // Grouped components by category
  let groupedComponents = $derived(groupComponentsByCategory());

  function groupComponentsByCategory(): Record<ComponentCategory, SystemComponent[]> {
    const groups: Record<ComponentCategory, SystemComponent[]> = {
      network: [],
      dpi: [],
      system: []
    };
    
    for (const component of components) {
      const category = component.category || 'system';
      groups[category].push(component);
    }
    
    return groups;
  }

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
  import { onMount } from 'svelte';
  onMount(() => {
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    isDemoMode = !isTauri;
    loadSystemInfo();
    checkConflicts();
    loadHistory();
  });

  // History management
  const HISTORY_KEY = 'isolate_diagnostics_history';
  const MAX_HISTORY_ENTRIES = 5;

  function loadHistory() {
    if (!browser) return;
    try {
      const stored = localStorage.getItem(HISTORY_KEY);
      if (stored) {
        history = JSON.parse(stored);
      }
    } catch (e) {
      console.error('Failed to load diagnostics history:', e);
      history = [];
    }
  }

  function saveToHistory() {
    if (!browser) return;
    
    const entry: DiagnosticsHistoryEntry = {
      timestamp: new Date().toISOString(),
      overallStatus: overallHealth.status,
      percentage: overallHealth.percentage,
      healthy: components.filter(c => c.status === 'healthy').length,
      warnings: components.filter(c => c.status === 'warning').length,
      errors: components.filter(c => c.status === 'error').length
    };
    
    // Add to beginning, keep max entries
    history = [entry, ...history].slice(0, MAX_HISTORY_ENTRIES);
    
    try {
      localStorage.setItem(HISTORY_KEY, JSON.stringify(history));
    } catch (e) {
      console.error('Failed to save diagnostics history:', e);
    }
  }

  function formatHistoryDate(isoString: string): string {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);
    
    if (diffMins < 1) return 'Только что';
    if (diffMins < 60) return `${diffMins} мин. назад`;
    if (diffHours < 24) return `${diffHours} ч. назад`;
    if (diffDays < 7) return `${diffDays} дн. назад`;
    
    return date.toLocaleDateString('ru-RU', { day: 'numeric', month: 'short' });
  }

  function clearHistory() {
    if (!browser) return;
    history = [];
    try {
      localStorage.removeItem(HISTORY_KEY);
      toasts.success('История очищена');
    } catch (e) {
      console.error('Failed to clear history:', e);
    }
  }

  async function loadSystemInfo() {
    if (!browser) return;
    
    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Wait for backend
        const ready = await waitForBackend(30, 300);
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
        
        const ready = await waitForBackend(20, 300);
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
    components = components.map(c => ({ ...c, status: 'checking' as ComponentStatus, details: 'Проверка...' }));
    
    try {
      if (isTauri) {
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Wait for backend
        const ready = await waitForBackend(20, 300);
        if (!ready) {
          console.warn('[Diagnostics] Backend not ready for diagnostics');
          toasts.error('Бэкенд не готов');
          return;
        }
        
        // Run diagnostics
        const results = await invoke<Record<string, { status: string; details: string }>>('run_diagnostics').catch(() => null);
        
        if (results) {
          components = components.map(c => ({
            ...c,
            status: (results[c.id]?.status || 'unknown') as ComponentStatus,
            details: results[c.id]?.details || 'Нет данных'
          }));
        }
      } else {
        // Demo mode - simulate checks
        await simulateDiagnostics();
      }
      
      lastCheck = new Date().toLocaleTimeString('ru-RU');
      saveToHistory();
      toasts.success('Диагностика завершена');
    } catch (e) {
      console.error('Diagnostics failed:', e);
      toasts.error(`Ошибка диагностики: ${e}`);
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

  // Recommendations based on diagnostics results
  interface Recommendation {
    title: string;
    description: string;
    action?: string;
    severity: 'info' | 'warning' | 'error';
  }

  let recommendations = $derived(generateRecommendations());

  function generateRecommendations(): Recommendation[] {
    const recs: Recommendation[] = [];
    const checked = components.filter(c => c.status !== 'unknown' && c.status !== 'checking');
    
    if (checked.length === 0) {
      return [{
        title: 'Запустите диагностику',
        description: 'Нажмите "Запустить диагностику" чтобы проверить состояние системы',
        severity: 'info'
      }];
    }

    // Network issues
    const network = components.find(c => c.id === 'network');
    if (network?.status === 'error') {
      recs.push({
        title: 'Нет подключения к интернету',
        description: 'Проверьте сетевое подключение. Без интернета обход блокировок невозможен.',
        action: 'Проверьте Wi-Fi или Ethernet кабель',
        severity: 'error'
      });
    }

    // DNS issues
    const dns = components.find(c => c.id === 'dns');
    if (dns?.status === 'error') {
      recs.push({
        title: 'Проблемы с DNS',
        description: 'DNS-сервер не отвечает или блокирует запросы.',
        action: 'Попробуйте сменить DNS на 8.8.8.8 или 1.1.1.1',
        severity: 'error'
      });
    } else if (dns?.status === 'warning') {
      recs.push({
        title: 'DNS работает медленно',
        description: 'Высокая задержка DNS может замедлять загрузку страниц.',
        action: 'Рекомендуем использовать DoH (DNS over HTTPS)',
        severity: 'warning'
      });
    }

    // WinDivert issues
    const windivert = components.find(c => c.id === 'windivert');
    if (windivert?.status === 'error') {
      recs.push({
        title: 'WinDivert не работает',
        description: 'Драйвер WinDivert необходим для Zapret-стратегий.',
        action: 'Запустите приложение от имени администратора',
        severity: 'error'
      });
    }

    // Sing-box issues
    const singbox = components.find(c => c.id === 'singbox');
    if (singbox?.status === 'warning') {
      recs.push({
        title: 'Sing-box не настроен',
        description: 'Для VLESS-стратегий требуется настройка прокси.',
        action: 'Перейдите в Маркетплейс и добавьте VLESS-конфигурацию',
        severity: 'warning'
      });
    } else if (singbox?.status === 'error') {
      recs.push({
        title: 'Ошибка Sing-box',
        description: 'Sing-box не найден или повреждён.',
        action: 'Переустановите приложение или проверьте антивирус',
        severity: 'error'
      });
    }

    // TCP Timestamps
    const tcpTimestamps = components.find(c => c.id === 'tcp_timestamps');
    if (tcpTimestamps?.status === 'warning') {
      recs.push({
        title: 'TCP Timestamps отключены',
        description: 'Включение TCP Timestamps улучшает обход некоторых DPI-систем.',
        action: 'Включите в Настройки → Дополнительно',
        severity: 'warning'
      });
    }

    // Firewall issues
    const firewall = components.find(c => c.id === 'firewall');
    if (firewall?.status === 'warning') {
      recs.push({
        title: 'Firewall может блокировать',
        description: 'Windows Firewall может мешать работе Isolate.',
        action: 'Добавьте исключение для Isolate в настройках Firewall',
        severity: 'warning'
      });
    }

    // All good!
    if (recs.length === 0 && overallHealth.status === 'healthy') {
      recs.push({
        title: 'Всё работает отлично! 🎉',
        description: 'Система готова к обходу блокировок. Выберите стратегию на главной странице.',
        severity: 'info'
      });
    }

    // Conflicts warning
    if (conflicts.length > 0) {
      recs.push({
        title: `Обнаружено ${conflicts.length} конфликтующих программ`,
        description: 'VPN или другое ПО может мешать работе Isolate.',
        action: 'Отключите конфликтующие программы или настройте исключения',
        severity: 'warning'
      });
    }

    return recs;
  }

  function getRecommendationIcon(severity: 'info' | 'warning' | 'error'): string {
    switch (severity) {
      case 'error': return '❌';
      case 'warning': return '⚠️';
      case 'info': return '💡';
    }
  }

  function getRecommendationColor(severity: 'info' | 'warning' | 'error'): string {
    switch (severity) {
      case 'error': return 'border-neon-red/30 bg-neon-red/5';
      case 'warning': return 'border-neon-yellow/30 bg-neon-yellow/5';
      case 'info': return 'border-electric/30 bg-electric/5';
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
        <h1 class="text-3xl font-bold text-white">Диагностика системы</h1>
        {#if isDemoMode}
          <span class="px-2 py-1 text-xs uppercase tracking-wider bg-amber-500/20 text-amber-400 rounded-md font-medium border border-amber-500/30">Демо</span>
        {/if}
      </div>
      <p class="text-text-muted mt-1">Проверка компонентов системы и сетевого подключения</p>
    </div>
    <div class="flex items-center gap-4">
      {#if lastCheck}
        <span class="text-text-muted text-sm">Последняя проверка: {lastCheck}</span>
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
          Запустить диагностику
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
          <h2 class="text-xl font-semibold text-white">Состояние системы</h2>
          <p class="text-text-muted">
            {#if overallHealth.status === 'healthy'}
              Все системы работают нормально
            {:else if overallHealth.status === 'warning'}
              Некоторые компоненты требуют внимания
            {:else if overallHealth.status === 'error'}
              Обнаружены критические проблемы
            {:else}
              Запустите диагностику для проверки
            {/if}
          </p>
        </div>
      </div>
      
      <!-- Quick Stats -->
      <div class="flex gap-6">
        <div class="text-center">
          <p class="text-2xl font-bold text-neon-green">{components.filter(c => c.status === 'healthy').length}</p>
          <p class="text-text-muted text-sm">Норма</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-neon-yellow">{components.filter(c => c.status === 'warning').length}</p>
          <p class="text-text-muted text-sm">Внимание</p>
        </div>
        <div class="text-center">
          <p class="text-2xl font-bold text-neon-red">{components.filter(c => c.status === 'error').length}</p>
          <p class="text-text-muted text-sm">Ошибки</p>
        </div>
      </div>
    </div>
  </div>

  <!-- Recommendations Section -->
  {#if recommendations.length > 0}
    <div class="bg-void-50 rounded-xl border border-glass-border p-5">
      <h3 class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <span>💡</span>
        Рекомендации
      </h3>
      <div class="space-y-3">
        {#each recommendations as rec}
          <div class="rounded-lg p-4 border {getRecommendationColor(rec.severity)}">
            <div class="flex items-start gap-3">
              <span class="text-xl flex-shrink-0">{getRecommendationIcon(rec.severity)}</span>
              <div class="flex-1">
                <h4 class="text-white font-medium">{rec.title}</h4>
                <p class="text-text-muted text-sm mt-1">{rec.description}</p>
                {#if rec.action}
                  <p class="text-electric text-sm mt-2 flex items-center gap-1">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
                    </svg>
                    {rec.action}
                  </p>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- History Section -->
  {#if history.length > 0}
    <div class="bg-void-50 rounded-xl border border-glass-border p-5">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-white flex items-center gap-2">
          <span>📊</span>
          История проверок
        </h3>
        <div class="flex items-center gap-2">
          <button
            onclick={() => showHistory = !showHistory}
            class="text-text-muted hover:text-white text-sm transition-colors"
          >
            {showHistory ? 'Свернуть' : 'Развернуть'}
          </button>
          <button
            onclick={clearHistory}
            class="text-text-muted hover:text-neon-red text-sm transition-colors"
            title="Очистить историю"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </div>
      
      {#if showHistory}
        <div class="space-y-2">
          {#each history as entry, i}
            <div class="flex items-center justify-between p-3 bg-void-100/50 rounded-lg">
              <div class="flex items-center gap-3">
                <span class="text-lg {getStatusColor(entry.overallStatus)}">{getStatusIcon(entry.overallStatus)}</span>
                <div>
                  <p class="text-white text-sm">{formatHistoryDate(entry.timestamp)}</p>
                  <p class="text-text-muted text-xs">
                    {entry.percentage}% — {entry.healthy} норма, {entry.warnings} внимание, {entry.errors} ошибок
                  </p>
                </div>
              </div>
              {#if i === 0}
                <span class="px-2 py-0.5 bg-electric/20 text-electric text-xs rounded-full">Последняя</span>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <div class="flex items-center gap-4 text-sm">
          <div class="flex items-center gap-2">
            <span class="{getStatusColor(history[0].overallStatus)}">{getStatusIcon(history[0].overallStatus)}</span>
            <span class="text-text-muted">Последняя: {formatHistoryDate(history[0].timestamp)}</span>
          </div>
          <span class="text-text-muted">•</span>
          <span class="text-text-muted">{history.length} из {MAX_HISTORY_ENTRIES} записей</span>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Software Conflicts Section -->
  {#if isCheckingConflicts}
    <div class="bg-void-50 rounded-xl border border-glass-border p-5">
      <div class="flex items-center gap-3">
        <ScanningIndicator active={true} text="" variant="pulse" />
        <span class="text-text-muted">Проверка конфликтующего ПО...</span>
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
          <h3 class="text-white font-semibold">Обнаружены конфликты ПО</h3>
          <p class="text-text-muted text-sm">
            Найдено {conflicts.length} {conflicts.length === 1 ? 'программа' : 'программ'}, которые могут мешать работе Isolate
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
        Перепроверить конфликты
      </button>
    </div>
  {/if}

  <!-- Main Grid: Components + System Info -->
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Components Grid (2 columns) - Grouped by Category -->
    <div class="lg:col-span-2 space-y-6">
      {#each Object.entries(groupedComponents) as [category, categoryComponents]}
        {#if categoryComponents.length > 0}
          <div class="space-y-4">
            <!-- Category Header -->
            <div class="flex items-center gap-3">
              <span class="text-2xl">{categoryConfig[category as ComponentCategory].icon}</span>
              <div>
                <h3 class="text-lg font-semibold text-white">{categoryConfig[category as ComponentCategory].title}</h3>
                <p class="text-text-muted text-sm">{categoryConfig[category as ComponentCategory].description}</p>
              </div>
            </div>
            
            <!-- Category Components -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              {#each categoryComponents as component}
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
        {/if}
      {/each}
    </div>

    <!-- System Info Sidebar -->
    <div class="space-y-4">
      <h3 class="text-lg font-semibold text-white">Информация о системе</h3>
      
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
              <p class="text-text-muted text-sm">Операционная система</p>
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
              <p class="text-text-muted text-sm">Архитектура</p>
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
              <p class="text-text-muted text-sm">Память</p>
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
              <p class="text-text-muted text-sm">Права администратора</p>
              <p class="font-medium {systemInfo.adminRights ? 'text-neon-green' : 'text-neon-red'}">
                {systemInfo.adminRights ? 'Есть' : 'Нет'}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="bg-void-50 rounded-xl border border-glass-border p-4">
        <h4 class="text-white font-medium mb-3">Быстрые действия</h4>
        <div class="space-y-2">
          <button
            onclick={runDiagnostics}
            disabled={isRunning}
            class="w-full flex items-center gap-3 px-4 py-3 bg-void-100/50 hover:bg-void-100 rounded-lg text-left transition-colors disabled:opacity-50"
          >
            <svg class="w-5 h-5 text-electric" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span class="text-white text-sm">Перезапустить проверки</span>
          </button>
          
          <button
            onclick={exportReport}
            class="w-full flex items-center gap-3 px-4 py-3 bg-void-100/50 hover:bg-void-100 rounded-lg text-left transition-colors"
          >
            <svg class="w-5 h-5 text-neon-cyan" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <span class="text-white text-sm">Экспорт отчёта</span>
          </button>
          
          <button
            onclick={autoFix}
            disabled={isFixing || isRunning || isDemoMode}
            class="w-full flex items-center gap-3 px-4 py-3 bg-void-100/50 hover:bg-void-100 rounded-lg text-left transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            title={isDemoMode ? 'Недоступно в демо-режиме' : 'Остановить стратегии, очистить прокси, сбросить состояние'}
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
              <span class="text-white text-sm">{isFixing ? 'Исправление...' : 'Автоисправление'}</span>
              <span class="text-text-muted/60 text-xs">Остановить стратегии, очистить прокси</span>
            </div>
          </button>
        </div>
      </div>
    </div>
  </div>

</div>
