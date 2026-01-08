<script lang="ts">
  import { browser } from '$app/environment';
  import { waitForBackend } from '$lib/utils/backend';
  import { toasts } from '$lib/stores/toast';
  import { 
    CaptureMode,
    ConnectionStatus,
    GatewayList, 
    RuleList, 
    AdvancedDrawer,
    AddGatewayModal,
    AddRuleModal,
    EditRuleModal,
    EditGatewayModal
  } from '$lib/components/network';
  import { QRCodeModal, ProxyTester, SubscriptionManager } from '$lib/components/proxies';
  import type { ProxyTestState } from '$lib/utils/proxyTester';
  import BaseModal from '$lib/components/BaseModal.svelte';
  import type { NetworkRule } from '$lib/components/network';
  import type { ProxyConfig, RoutingRule } from '$lib/api';
  import { 
    getRoutingRules, 
    addRoutingRule, 
    updateRoutingRule, 
    deleteRoutingRule as apiDeleteRule,
    reorderRoutingRules,
    toggleRoutingRule,
    importProxyUrl
  } from '$lib/api';
  import { mockGateways, mockNetworkRules } from '$lib/mocks';
  
  // State
  let captureMode = $state<'system' | 'tun'>('system');
  let loading = $state(true);
  let initialized = $state(false);
  
  // Demo mode flag (browser preview without Tauri)
  let isDemoMode = $state(false);
  
  // TUN/System Proxy status
  let tunRunning = $state(false);
  let systemProxySet = $state(false);
  
  // Switching state
  let switching = $state(false);
  
  // Capture mode confirmation modal
  let showCaptureModeConfirm = $state(false);
  let pendingCaptureMode = $state<'system' | 'tun' | null>(null);
  let captureModeError = $state<string | null>(null);
  
  // Drawers/Modals
  let showAdvanced = $state(false);
  let showAddGateway = $state(false);
  let showEditGateway = $state(false);
  let editingGateway = $state<ProxyConfig | null>(null);
  let showAddRule = $state(false);
  let showEditRule = $state(false);
  let editingRule = $state<NetworkRule | null>(null);
  let showDeleteConfirm = $state(false);
  let gatewayToDelete = $state<ProxyConfig | null>(null);
  
  // QR Code Modal
  let showQRCode = $state(false);
  let qrCodeGateway = $state<ProxyConfig | null>(null);
  
  // Gateways (proxies)
  let gateways = $state<ProxyConfig[]>([]);
  let selectedGatewayId = $state<string | null>(null);
  
  // Derived: active gateway
  let activeGateway = $derived(gateways.find(g => g.active) ?? null);
  
  // Test All state
  let testingAllGateways = $state(false);
  let testAllProgress = $state<{ current: number; total: number } | null>(null);
  let showProxyTester = $state(false);
  
  // Import from clipboard state
  let importingFromClipboard = $state(false);
  
  // Rules
  let rules = $state<NetworkRule[]>([]);
  
  import { onMount } from 'svelte';
  onMount(() => {
    if (!initialized) {
      initialized = true;
      loadData();
    }
  });
  
  async function loadData() {
    loading = true;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    isDemoMode = !isTauri;
    
    if (!isTauri) {
      // Demo mode: mock data for browser preview
      await new Promise(r => setTimeout(r, 500));
      gateways = mockGateways;
      rules = mockNetworkRules;
      systemProxySet = false;
      loading = false;
      return;
    }
    
    try {
      const ready = await waitForBackend(30, 300);
      if (!ready) {
        gateways = mockGateways;
        rules = mockNetworkRules;
        loading = false;
        return;
      }
      
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Load proxies
      const proxies = await invoke<ProxyConfig[]>('get_proxies').catch(() => []);
      gateways = proxies;
      
      // Load TUN/System Proxy status
      tunRunning = await invoke<boolean>('is_tun_running').catch(() => false);
      systemProxySet = await invoke<boolean>('is_system_proxy_set').catch(() => false);
      
      // Determine current mode
      if (tunRunning) captureMode = 'tun';
      else if (systemProxySet) captureMode = 'system';
      
      // Load routing rules from backend
      const backendRules = await getRoutingRules().catch(() => []);
      rules = backendRules.map(r => ({
        id: r.id,
        name: r.name,
        enabled: r.enabled,
        source: (r.source as 'domain' | 'app' | 'ip') || 'domain',
        sourceValue: r.sourceValue || '',
        action: (r.action as 'direct' | 'proxy' | 'block' | 'dpi-bypass') || 'direct',
        proxyId: r.proxyId,
        priority: r.priority
      }));
      
      // Fallback to mock if no rules
      if (rules.length === 0) {
        rules = mockNetworkRules;
      }
      
    } catch (e) {
      console.error('Failed to load network data:', e);
      gateways = mockGateways;
      rules = mockNetworkRules;
    } finally {
      loading = false;
    }
  }
  
  async function handleCaptureModeChange(mode: 'system' | 'tun') {
    if (mode === captureMode || switching) return;
    
    // Show confirmation modal instead of switching immediately
    pendingCaptureMode = mode;
    captureModeError = null;
    showCaptureModeConfirm = true;
  }
  
  async function confirmCaptureModeChange() {
    if (!pendingCaptureMode || switching) return;
    
    const mode = pendingCaptureMode;
    switching = true;
    captureModeError = null;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      await new Promise(r => setTimeout(r, 500));
      captureMode = mode;
      switching = false;
      showCaptureModeConfirm = false;
      pendingCaptureMode = null;
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Stop previous mode
      if (captureMode === 'system' && systemProxySet) {
        await invoke('clear_system_proxy');
        systemProxySet = false;
      } else if (captureMode === 'tun' && tunRunning) {
        await invoke('stop_tun');
        tunRunning = false;
      }
      
      // Start new mode
      if (mode === 'system') {
        // Need active proxy for system proxy
        const activeGateway = gateways.find(g => g.active);
        if (activeGateway) {
          await invoke('set_system_proxy', { host: '127.0.0.1', port: 1080, scheme: 'socks5' });
          systemProxySet = true;
        } else {
          captureModeError = 'Сначала выберите активный шлюз';
          switching = false;
          return;
        }
      } else {
        await invoke('start_tun', { socksPort: 1080 });
        tunRunning = true;
      }
      
      captureMode = mode;
      toasts.success(`Переключено на ${mode === 'system' ? 'System Proxy' : 'TUN Driver'}`);
      showCaptureModeConfirm = false;
      pendingCaptureMode = null;
    } catch (e) {
      console.error('Failed to switch capture mode:', e);
      const errorMsg = e instanceof Error ? e.message : String(e);
      captureModeError = getCaptureModeErrorMessage(errorMsg, mode);
    } finally {
      switching = false;
    }
  }
  
  function getCaptureModeErrorMessage(error: string, mode: 'system' | 'tun'): string {
    const lowerError = error.toLowerCase();
    
    if (mode === 'tun') {
      if (lowerError.includes('admin') || lowerError.includes('privilege') || lowerError.includes('access denied')) {
        return 'TUN драйвер требует прав администратора. Перезапустите приложение от имени администратора.';
      }
      if (lowerError.includes('driver') || lowerError.includes('wintun')) {
        return 'Не удалось загрузить TUN драйвер. Возможно, он заблокирован антивирусом или VPN.';
      }
      if (lowerError.includes('conflict') || lowerError.includes('in use')) {
        return 'TUN интерфейс уже используется другим приложением (VPN, антивирус).';
      }
    }
    
    if (mode === 'system') {
      if (lowerError.includes('registry') || lowerError.includes('access denied')) {
        return 'Не удалось изменить системные настройки прокси. Проверьте права доступа.';
      }
    }
    
    return `Не удалось переключить режим: ${error}`;
  }
  
  function cancelCaptureModeChange() {
    showCaptureModeConfirm = false;
    pendingCaptureMode = null;
    captureModeError = null;
  }
  
  async function retryCaptureModeChange() {
    captureModeError = null;
    await confirmCaptureModeChange();
  }
  
  // Gateway handlers
  function handleSelectGateway(id: string) {
    selectedGatewayId = selectedGatewayId === id ? null : id;
  }
  
  function handleAddGateway() {
    showAddGateway = true;
  }
  
  async function handleGatewayAdded(gateway: Omit<ProxyConfig, 'id' | 'active'>) {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      // Mock add for browser preview
      const newGateway: ProxyConfig = {
        ...gateway,
        id: `gateway-${Date.now()}`,
        active: false,
        custom_fields: gateway.custom_fields || {}
      };
      gateways = [...gateways, newGateway];
      toasts.success(`Added ${gateway.name}`);
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const id = await invoke<string>('add_proxy', { proxy: gateway });
      const newGateway: ProxyConfig = {
        ...gateway,
        id,
        active: false,
        custom_fields: gateway.custom_fields || {}
      };
      gateways = [...gateways, newGateway];
      toasts.success(`Added ${gateway.name}`);
    } catch (e) {
      toasts.error(`Failed to add gateway: ${e}`);
    }
  }
  
  async function handleTestGateway(id: string) {
    const gateway = gateways.find(g => g.id === id);
    if (!gateway) return;
    
    toasts.info(`Testing ${gateway.name}...`);
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      // Demo mode: simulate ping test with random latency
      await new Promise(r => setTimeout(r, 1000));
      gateways = gateways.map(g => g.id === id ? { ...g, ping: Math.floor(Math.random() * 200) + 50 } : g);
      toasts.success(`[Demo] ${gateway.name}: ${gateways.find(g => g.id === id)?.ping}ms`);
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<{ success: boolean; latency?: number; error?: string }>('test_proxy', { id });
      if (result.success && result.latency) {
        gateways = gateways.map(g => g.id === id ? { ...g, ping: result.latency } : g);
        toasts.success(`${gateway.name}: ${result.latency}ms`);
      } else {
        toasts.error(`${gateway.name}: ${result.error || 'Failed'}`);
      }
    } catch (e) {
      toasts.error(`Test failed: ${e}`);
    }
  }
  
  async function handleTestAllGateways() {
    // Open ProxyTester panel instead of inline testing
    showProxyTester = true;
  }
  
  function handleProxyTestComplete(results: ProxyTestState[]) {
    // Update gateways with test results
    for (const result of results) {
      if (result.status === 'success' && result.latency !== null) {
        gateways = gateways.map(g => 
          g.id === result.proxyId ? { ...g, ping: result.latency! } : g
        );
      } else if (result.status === 'failed') {
        gateways = gateways.map(g => 
          g.id === result.proxyId ? { ...g, ping: undefined } : g
        );
      }
    }
    
    // Show summary toast
    const successful = results.filter(r => r.status === 'success').length;
    const failed = results.filter(r => r.status === 'failed').length;
    
    if (failed === 0) {
      toasts.success(`All ${successful} gateways tested successfully`);
    } else if (successful === 0) {
      toasts.error(`All ${failed} gateways failed`);
    } else {
      toasts.info(`Tested: ${successful} OK, ${failed} failed`);
    }
  }
  
  function handleProxySorted(sortedProxies: ProxyConfig[]) {
    gateways = sortedProxies;
  }
  
  async function handleImportFromClipboard() {
    if (importingFromClipboard) return;
    
    importingFromClipboard = true;
    
    try {
      const text = await navigator.clipboard.readText();
      
      if (!text.trim()) {
        toasts.error('Clipboard is empty');
        return;
      }
      
      // Supported protocols
      const supportedPrefixes = ['vless://', 'vmess://', 'ss://', 'trojan://', 'hysteria://', 'hysteria2://'];
      
      // Split by newlines and filter valid URLs
      const lines = text.split('\n')
        .map(l => l.trim())
        .filter(l => l && supportedPrefixes.some(prefix => l.toLowerCase().startsWith(prefix)));
      
      if (lines.length === 0) {
        toasts.error('No valid proxy URLs found. Supported: vless://, vmess://, ss://, trojan://');
        return;
      }
      
      const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
      
      let imported = 0;
      let failed = 0;
      
      for (const line of lines) {
        try {
          if (!isTauri) {
            // Mock for browser preview
            await new Promise(r => setTimeout(r, 200));
            const mockGateway: ProxyConfig = {
              id: `imported-${Date.now()}-${imported}`,
              name: `Imported Gateway ${imported + 1}`,
              protocol: line.split('://')[0] as ProxyConfig['protocol'],
              server: 'example.com',
              port: 443,
              tls: true,
              active: false,
              custom_fields: {}
            };
            gateways = [...gateways, mockGateway];
            imported++;
          } else {
            const config = await importProxyUrl(line);
            gateways = [...gateways, config];
            imported++;
          }
        } catch {
          failed++;
        }
      }
      
      // Show result toast
      if (imported > 0) {
        const failedMsg = failed > 0 ? `, ${failed} failed` : '';
        toasts.success(`Imported ${imported} proxy${imported !== 1 ? 'ies' : ''}${failedMsg}`);
      } else {
        toasts.error('Failed to import any proxies');
      }
      
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      toasts.error(`Import failed: ${errorMsg}`);
    } finally {
      importingFromClipboard = false;
    }
  }
  
  async function handleSubscriptionProxiesImported(proxies: ProxyConfig[]) {
    if (proxies.length === 0) return;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    
    let imported = 0;
    
    for (const proxy of proxies) {
      // Check if proxy with same server:port already exists
      const exists = gateways.some(g => g.server === proxy.server && g.port === proxy.port);
      if (exists) continue;
      
      if (!isTauri) {
        // Demo mode
        gateways = [...gateways, proxy];
        imported++;
      } else {
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          const id = await invoke<string>('add_proxy', { proxy });
          gateways = [...gateways, { ...proxy, id }];
          imported++;
        } catch {
          // Skip failed imports
        }
      }
    }
    
    if (imported > 0) {
      toasts.success(`Imported ${imported} proxy${imported !== 1 ? 'ies' : ''} from subscription`);
    }
  }
  
  async function handleDeleteGateway(id: string) {
    const gateway = gateways.find(g => g.id === id);
    if (!gateway) return;
    
    // Show confirmation dialog
    gatewayToDelete = gateway;
    showDeleteConfirm = true;
  }
  
  async function confirmDeleteGateway() {
    if (!gatewayToDelete) return;
    
    const id = gatewayToDelete.id;
    const name = gatewayToDelete.name;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      gateways = gateways.filter(g => g.id !== id);
      toasts.success(`Deleted ${name}`);
      showDeleteConfirm = false;
      gatewayToDelete = null;
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_proxy', { id });
      gateways = gateways.filter(g => g.id !== id);
      toasts.success(`Deleted ${name}`);
    } catch (e) {
      toasts.error(`Delete failed: ${e}`);
    } finally {
      showDeleteConfirm = false;
      gatewayToDelete = null;
    }
  }
  
  function handleEditGateway(id: string) {
    const gateway = gateways.find(g => g.id === id);
    if (gateway) {
      editingGateway = gateway;
      showEditGateway = true;
    }
  }
  
  function handleShareGateway(id: string) {
    const gateway = gateways.find(g => g.id === id);
    if (gateway) {
      qrCodeGateway = gateway;
      showQRCode = true;
    }
  }
  
  async function handleGatewaySaved(updatedGateway: ProxyConfig) {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      gateways = gateways.map(g => g.id === updatedGateway.id ? updatedGateway : g);
      toasts.success(`Updated ${updatedGateway.name}`);
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_proxy', { proxy: updatedGateway });
      gateways = gateways.map(g => g.id === updatedGateway.id ? updatedGateway : g);
      toasts.success(`Updated ${updatedGateway.name}`);
    } catch (e) {
      toasts.error(`Failed to update gateway: ${e}`);
    }
  }
  
  async function handleActivateGateway(id: string) {
    const gateway = gateways.find(g => g.id === id);
    if (!gateway) return;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      // Mock for browser preview
      gateways = gateways.map(g => ({ ...g, active: g.id === id }));
      toasts.success(`Activated ${gateway.name}`);
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      // Deactivate any currently active gateway first
      const currentActive = gateways.find(g => g.active);
      if (currentActive && currentActive.id !== id) {
        await invoke('deactivate_proxy', { id: currentActive.id });
      }
      
      // Activate the new gateway
      await invoke('apply_proxy', { id });
      
      // Update state: only one gateway can be active
      gateways = gateways.map(g => ({ ...g, active: g.id === id }));
      toasts.success(`Activated ${gateway.name}`);
    } catch (e) {
      toasts.error(`Failed to activate: ${e}`);
    }
  }
  
  async function handleDeactivateGateway(id: string) {
    const gateway = gateways.find(g => g.id === id);
    if (!gateway) return;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      // Mock for browser preview
      gateways = gateways.map(g => g.id === id ? { ...g, active: false } : g);
      toasts.success(`Deactivated ${gateway.name}`);
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('deactivate_proxy', { id });
      
      // Update state
      gateways = gateways.map(g => g.id === id ? { ...g, active: false } : g);
      toasts.success(`Deactivated ${gateway.name}`);
    } catch (e) {
      toasts.error(`Failed to deactivate: ${e}`);
    }
  }
  
  // Rule handlers
  function handleAddRule() {
    showAddRule = true;
  }
  
  async function handleRuleAdded(rule: Omit<NetworkRule, 'id' | 'priority'>) {
    const newRule: RoutingRule = {
      id: `rule-${Date.now()}`,
      name: rule.name,
      enabled: rule.enabled,
      source: rule.source,
      sourceValue: rule.sourceValue,
      action: rule.action,
      proxyId: rule.proxyId,
      priority: rules.length + 1
    };
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      rules = [...rules, newRule as NetworkRule];
      toasts.success(`Added rule: ${rule.name}`);
      return;
    }
    
    try {
      const created = await addRoutingRule(newRule);
      rules = [...rules, {
        id: created.id,
        name: created.name,
        enabled: created.enabled,
        source: created.source as 'domain' | 'app' | 'ip',
        sourceValue: created.sourceValue || '',
        action: created.action as 'direct' | 'proxy' | 'block' | 'dpi-bypass',
        proxyId: created.proxyId,
        priority: created.priority
      }];
      toasts.success(`Added rule: ${rule.name}`);
    } catch (e) {
      toasts.error(`Failed to add rule: ${e}`);
    }
  }
  
  function handleEditRule(rule: NetworkRule) {
    editingRule = rule;
    showEditRule = true;
  }
  
  async function handleRuleSaved(updatedRule: NetworkRule) {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      rules = rules.map(r => r.id === updatedRule.id ? updatedRule : r);
      toasts.success(`Updated rule: ${updatedRule.name}`);
      return;
    }
    
    try {
      await updateRoutingRule({
        id: updatedRule.id,
        name: updatedRule.name,
        enabled: updatedRule.enabled,
        source: updatedRule.source,
        sourceValue: updatedRule.sourceValue,
        action: updatedRule.action,
        proxyId: updatedRule.proxyId,
        priority: updatedRule.priority
      });
      rules = rules.map(r => r.id === updatedRule.id ? updatedRule : r);
      toasts.success(`Updated rule: ${updatedRule.name}`);
    } catch (e) {
      toasts.error(`Failed to update rule: ${e}`);
    }
  }
  
  async function handleDeleteRule(id: string) {
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      rules = rules.filter(r => r.id !== id);
      toasts.success('Rule deleted');
      return;
    }
    
    try {
      await apiDeleteRule(id);
      rules = rules.filter(r => r.id !== id);
      toasts.success('Rule deleted');
    } catch (e) {
      toasts.error(`Failed to delete rule: ${e}`);
    }
  }
  
  async function handleBulkDeleteRules(ids: string[]) {
    if (ids.length === 0) return;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      rules = rules.filter(r => !ids.includes(r.id));
      toasts.success(`Deleted ${ids.length} rule${ids.length !== 1 ? 's' : ''}`);
      return;
    }
    
    try {
      let deletedCount = 0;
      for (const id of ids) {
        await apiDeleteRule(id);
        deletedCount++;
      }
      rules = rules.filter(r => !ids.includes(r.id));
      toasts.success(`Deleted ${deletedCount} rule${deletedCount !== 1 ? 's' : ''}`);
    } catch (e) {
      toasts.error(`Failed to delete rules: ${e}`);
    }
  }
  
  async function handleBulkEnableRules(ids: string[]) {
    if (ids.length === 0) return;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      rules = rules.map(r => ids.includes(r.id) ? { ...r, enabled: true } : r);
      toasts.success(`Enabled ${ids.length} rule${ids.length !== 1 ? 's' : ''}`);
      return;
    }
    
    try {
      let enabledCount = 0;
      for (const id of ids) {
        await toggleRoutingRule(id, true);
        enabledCount++;
      }
      rules = rules.map(r => ids.includes(r.id) ? { ...r, enabled: true } : r);
      toasts.success(`Enabled ${enabledCount} rule${enabledCount !== 1 ? 's' : ''}`);
    } catch (e) {
      toasts.error(`Failed to enable rules: ${e}`);
    }
  }
  
  async function handleBulkDisableRules(ids: string[]) {
    if (ids.length === 0) return;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      rules = rules.map(r => ids.includes(r.id) ? { ...r, enabled: false } : r);
      toasts.success(`Disabled ${ids.length} rule${ids.length !== 1 ? 's' : ''}`);
      return;
    }
    
    try {
      let disabledCount = 0;
      for (const id of ids) {
        await toggleRoutingRule(id, false);
        disabledCount++;
      }
      rules = rules.map(r => ids.includes(r.id) ? { ...r, enabled: false } : r);
      toasts.success(`Disabled ${disabledCount} rule${disabledCount !== 1 ? 's' : ''}`);
    } catch (e) {
      toasts.error(`Failed to disable rules: ${e}`);
    }
  }
  
  async function handleToggleRule(id: string) {
    const rule = rules.find(r => r.id === id);
    if (!rule) return;
    
    const newEnabled = !rule.enabled;
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      rules = rules.map(r => r.id === id ? { ...r, enabled: newEnabled } : r);
      return;
    }
    
    try {
      await toggleRoutingRule(id, newEnabled);
      rules = rules.map(r => r.id === id ? { ...r, enabled: newEnabled } : r);
    } catch (e) {
      toasts.error(`Failed to toggle rule: ${e}`);
    }
  }
  
  async function handleReorderRules(fromIndex: number, toIndex: number) {
    const newRules = [...rules];
    const [removed] = newRules.splice(fromIndex, 1);
    newRules.splice(toIndex, 0, removed);
    // Update priorities
    const reorderedRules = newRules.map((r, i) => ({ ...r, priority: i + 1 }));
    
    const isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (!isTauri) {
      rules = reorderedRules;
      toasts.success('Rule order updated');
      return;
    }
    
    try {
      await reorderRoutingRules(reorderedRules.map(r => r.id));
      rules = reorderedRules;
      toasts.success('Rule order updated');
    } catch (e) {
      toasts.error(`Failed to reorder rules: ${e}`);
    }
  }
</script>

<div class="h-full flex flex-col bg-[#09090b]">
  <!-- Sticky Header -->
  <header class="sticky top-0 z-10 flex items-center justify-between px-6 py-4 bg-[#09090b]/80 backdrop-blur-xl border-b border-white/5">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-3">
        <h1 class="text-xl font-semibold text-white">Network</h1>
        {#if isDemoMode}
          <span class="px-2 py-0.5 text-[10px] uppercase tracking-wider bg-amber-500/20 text-amber-400 rounded font-medium border border-amber-500/30">Demo</span>
        {/if}
      </div>
      <CaptureMode 
        mode={captureMode}
        onchange={handleCaptureModeChange}
        systemActive={systemProxySet}
        tunActive={tunRunning}
        disabled={switching || loading}
      />
    </div>
    
    <div class="flex items-center gap-3">
      <ConnectionStatus
        {tunRunning}
        {systemProxySet}
        {activeGateway}
        {loading}
      />
      
      <button
        type="button"
        onclick={() => showAdvanced = true}
        class="p-2 rounded-lg bg-white/5 hover:bg-white/10 text-zinc-400 hover:text-white transition-all duration-200"
        title="Advanced Settings"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </button>
    </div>
  </header>
  
  <!-- Split View Content -->
  <div class="flex-1 flex min-h-0 overflow-hidden">
    <!-- Left Column: Gateways (~40%) -->
    <div class="w-[40%] flex flex-col border-r border-white/5 overflow-hidden p-4">
      <GatewayList
        {gateways}
        selectedId={selectedGatewayId}
        {loading}
        testingAll={testingAllGateways}
        testProgress={testAllProgress}
        importing={importingFromClipboard}
        onselect={handleSelectGateway}
        onadd={handleAddGateway}
        ontest={handleTestGateway}
        ondelete={handleDeleteGateway}
        onedit={handleEditGateway}
        onactivate={handleActivateGateway}
        ondeactivate={handleDeactivateGateway}
        onshare={handleShareGateway}
        ontestall={handleTestAllGateways}
        onimport={handleImportFromClipboard}
      />
      
      <!-- Subscriptions Section -->
      <div class="mt-4 pt-4 border-t border-white/5">
        <SubscriptionManager onProxiesImported={handleSubscriptionProxiesImported} />
      </div>
    </div>
    
    <!-- Right Column: Rules (~60%) -->
    <div class="flex-1 flex flex-col overflow-hidden p-4">
      <RuleList
        {rules}
        {gateways}
        {loading}
        onadd={handleAddRule}
        onedit={handleEditRule}
        ondelete={handleDeleteRule}
        ontoggle={handleToggleRule}
        onreorder={handleReorderRules}
        onbulkdelete={handleBulkDeleteRules}
        onbulkenable={handleBulkEnableRules}
        onbulkdisable={handleBulkDisableRules}
      />
    </div>
  </div>
</div>

<!-- Drawers/Modals -->
<AdvancedDrawer open={showAdvanced} onclose={() => showAdvanced = false} />
<AddGatewayModal open={showAddGateway} onclose={() => showAddGateway = false} onadd={handleGatewayAdded} />
<EditGatewayModal open={showEditGateway} onclose={() => { showEditGateway = false; editingGateway = null; }} onsave={handleGatewaySaved} gateway={editingGateway} />
<AddRuleModal open={showAddRule} onclose={() => showAddRule = false} onadd={handleRuleAdded} {gateways} />
<EditRuleModal open={showEditRule} onclose={() => { showEditRule = false; editingRule = null; }} onsave={handleRuleSaved} rule={editingRule} {gateways} />

<!-- QR Code Modal -->
<QRCodeModal 
  bind:open={showQRCode} 
  proxy={qrCodeGateway} 
  onClose={() => { showQRCode = false; qrCodeGateway = null; }} 
/>

<!-- Proxy Tester Modal -->
<BaseModal open={showProxyTester} onclose={() => showProxyTester = false} class="w-full max-w-lg">
  <div class="p-6">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-white">Test All Gateways</h3>
      <button
        type="button"
        onclick={() => showProxyTester = false}
        class="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-white/10 transition-colors"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
    
    <ProxyTester
      proxies={gateways}
      concurrency={5}
      oncomplete={handleProxyTestComplete}
      onsort={handleProxySorted}
    />
  </div>
</BaseModal>

<!-- Capture Mode Confirmation Modal -->
<BaseModal open={showCaptureModeConfirm} onclose={cancelCaptureModeChange} class="w-full max-w-md">
  <div class="p-6">
    <!-- Header -->
    <div class="flex items-center gap-3 mb-4">
      <div class="w-12 h-12 rounded-full bg-amber-500/10 flex items-center justify-center">
        <svg class="w-6 h-6 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
      </div>
      <div>
        <h3 class="text-lg font-semibold text-white">Изменить режим захвата?</h3>
        <p class="text-sm text-zinc-400">
          {captureMode === 'system' ? 'System Proxy' : 'TUN Driver'} → {pendingCaptureMode === 'system' ? 'System Proxy' : 'TUN Driver'}
        </p>
      </div>
    </div>
    
    <!-- Content -->
    {#if captureModeError}
      <!-- Error state -->
      <div class="mb-5 p-4 bg-red-500/10 border border-red-500/20 rounded-xl">
        <div class="flex items-start gap-3">
          <svg class="w-5 h-5 text-red-400 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div>
            <p class="text-sm text-red-300 font-medium mb-1">Ошибка переключения</p>
            <p class="text-sm text-red-400/80">{captureModeError}</p>
          </div>
        </div>
      </div>
    {:else}
      <!-- Warning info -->
      <div class="mb-5 p-4 bg-zinc-800/50 border border-white/5 rounded-xl space-y-3">
        <p class="text-sm text-zinc-300">
          Переключение режима захвата трафика может временно прервать активные соединения.
        </p>
        
        {#if pendingCaptureMode === 'tun'}
          <div class="flex items-start gap-2 text-xs text-amber-400/80">
            <svg class="w-4 h-4 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>TUN режим требует прав администратора и может конфликтовать с VPN или антивирусами.</span>
          </div>
        {/if}
        
        {#if pendingCaptureMode === 'system' && !gateways.some(g => g.active)}
          <div class="flex items-start gap-2 text-xs text-amber-400/80">
            <svg class="w-4 h-4 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <span>Для System Proxy необходимо сначала выбрать активный шлюз.</span>
          </div>
        {/if}
      </div>
    {/if}
    
    <!-- Actions -->
    <div class="flex gap-3">
      <button
        onclick={cancelCaptureModeChange}
        disabled={switching}
        class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
               text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Отмена
      </button>
      
      {#if captureModeError}
        <button
          onclick={retryCaptureModeChange}
          disabled={switching}
          class="flex-1 px-4 py-2.5 bg-amber-500/20 border border-amber-500/30 rounded-xl
                 text-amber-400 font-medium text-sm hover:bg-amber-500/30 transition-colors
                 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {#if switching}
            <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span>Повтор...</span>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span>Повторить</span>
          {/if}
        </button>
      {:else}
        <button
          onclick={confirmCaptureModeChange}
          disabled={switching}
          class="flex-1 px-4 py-2.5 bg-blue-500/20 border border-blue-500/30 rounded-xl
                 text-blue-400 font-medium text-sm hover:bg-blue-500/30 transition-colors
                 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {#if switching}
            <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span>Переключение...</span>
          {:else}
            <span>Переключить</span>
          {/if}
        </button>
      {/if}
    </div>
  </div>
</BaseModal>

<!-- Delete Gateway Confirmation Modal -->
<BaseModal open={showDeleteConfirm} onclose={() => { showDeleteConfirm = false; gatewayToDelete = null; }} class="w-full max-w-sm">
  {#if gatewayToDelete}
    <div class="p-6 text-center">
      <div class="w-14 h-14 mx-auto mb-4 rounded-full bg-red-500/10 flex items-center justify-center">
        <svg class="w-7 h-7 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </div>
      <h3 class="text-lg font-semibold text-white mb-2">Delete Gateway?</h3>
      <p class="text-zinc-400 text-sm mb-6">
        Are you sure you want to delete <span class="text-white font-medium">{gatewayToDelete.name}</span>? 
        This action cannot be undone.
      </p>
      <div class="flex gap-3">
        <button
          onclick={() => { showDeleteConfirm = false; gatewayToDelete = null; }}
          class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                 text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={confirmDeleteGateway}
          class="flex-1 px-4 py-2.5 bg-red-500/20 border border-red-500/30 rounded-xl
                 text-red-400 font-medium text-sm hover:bg-red-500/30 transition-colors"
        >
          Delete
        </button>
      </div>
    </div>
  {/if}
</BaseModal>
