<script lang="ts">
  import { toasts } from '$lib/stores/toast';
  import { logs } from '$lib/stores/logs';
  import { browser } from '$app/environment';
  import { waitForBackend } from '$lib/utils/backend';
  import { ContextMenu, ContextMenuItem, ContextMenuSeparator } from '$lib/components';
  import { ServicesSkeleton } from '$lib/components/skeletons';
  import { ServiceList, ServiceDetails, ServiceConfigModal, AddServiceModal } from '$lib/components/services';
  import BaseModal from '$lib/components/BaseModal.svelte';

  // Types
  interface ServiceWithStatus {
    id: string;
    name: string;
    status: 'working' | 'blocked' | 'unknown' | 'checking' | 'error';
    ping?: number;
    category: string;
    isCustom?: boolean;
    error?: string | null;
  }

  interface ServiceConfig {
    autoCheck: boolean;
    checkInterval: string;
    notifyBlocked: boolean;
    notifyRestored: boolean;
    priority: 'low' | 'normal' | 'high';
  }

  // State
  let services: ServiceWithStatus[] = $state([]);
  let selectedId: string | null = $state(null);
  let loading = $state(true);
  let scanning = $state(false);
  let loadError = $state<string | null>(null);
  let retrying = $state(false);
  
  // Ping history for each service (serviceId -> array of ping values)
  // Limited to MAX_PING_HISTORY entries per service to prevent memory leak
  const MAX_PING_HISTORY = 30;
  const MAX_SERVICES_IN_HISTORY = 100; // Limit total services tracked
  let pingHistory: Map<string, number[]> = $state(new Map());
  
  // Modals
  let showAddModal = $state(false);
  let showConfigModal = $state(false);
  let configService: ServiceWithStatus | null = $state(null);
  let showDeleteConfirm = $state(false);
  let serviceToDelete: ServiceWithStatus | null = $state(null);
  
  // Config state
  let serviceConfig: ServiceConfig = $state({
    autoCheck: false,
    checkInterval: '15',
    notifyBlocked: true,
    notifyRestored: false,
    priority: 'normal'
  });
  
  // Context menu
  let contextMenu: ReturnType<typeof ContextMenu> | null = $state(null);
  let contextMenuService: ServiceWithStatus | null = $state(null);

  // Search and filter state
  let searchQuery = $state('');
  let statusFilter = $state<'all' | 'working' | 'blocked' | 'unknown'>('all');

  // Derived
  let selected = $derived(services.find(s => s.id === selectedId) || null);
  let selectedPingHistory = $derived(selectedId ? (pingHistory.get(selectedId) || []) : []);
  
  // Filtered services
  let filteredServices = $derived(
    services.filter(s => {
      const matchesSearch = s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           s.category.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesStatus = statusFilter === 'all' || 
                           (statusFilter === 'unknown' && (s.status === 'unknown' || s.status === 'checking' || s.status === 'error')) ||
                           s.status === statusFilter;
      return matchesSearch && matchesStatus;
    })
  );

  // Load services on mount - use onMount to run only once
  import { onMount } from 'svelte';
  
  onMount(() => {
    loadFromBackend();
  });

  // ===== Backend Operations =====
  
  async function loadFromBackend() {
    loading = true;
    loadError = null;
    logs.info('services', 'Loading services from backend...');
    
    const ready = await waitForBackend(10, 200);
    if (!ready) {
      const errorMsg = 'Backend not ready. Please wait and try again.';
      logs.error('services', 'Failed to load services - backend not ready');
      loadError = errorMsg;
      loading = false;
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<any[]>('get_registry_services');
      
      if (result && result.length > 0) {
        services = result.map((s: any) => ({
          id: s.id,
          name: s.name,
          status: 'unknown' as const,
          category: s.category?.toLowerCase() || 'other',
          ping: undefined,
          isCustom: s.plugin_id === 'user-custom'
        }));
        if (services.length > 0 && !selectedId) {
          selectedId = services[0].id;
        }
        logs.success('services', `Loaded ${services.length} services`);
        loadError = null;
      }
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      logs.error('services', `Load failed: ${errorMsg}`);
      loadError = `Failed to load services: ${errorMsg}`;
      console.warn('Load failed:', e);
    }
    
    loading = false;
  }

  async function checkAllServices() {
    if (!browser || scanning) return;
    scanning = true;
    logs.info('services', 'Starting check of all services...');
    
    const ready = await waitForBackend(5, 200);
    if (!ready) {
      logs.error('services', 'Backend not ready');
      toasts.error('Backend not ready. Please wait and try again.');
      scanning = false;
      return;
    }
    
    services = services.map(s => ({ ...s, status: 'checking' as const, error: null }));
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const timeoutPromise = new Promise<null>((_, reject) => 
        setTimeout(() => reject(new Error('Timeout: check took too long')), 30000)
      );
      
      const results = await Promise.race([
        invoke<any[]>('check_all_registry_services'),
        timeoutPromise
      ]);
      
      if (results && results.length > 0) {
        for (const r of results) {
          const idx = services.findIndex(s => s.id === r.service_id);
          if (idx >= 0) {
            services[idx] = {
              ...services[idx],
              status: r.accessible ? 'working' : 'blocked',
              ping: r.avg_latency_ms,
              error: null
            };
            addPingToHistory(r.service_id, r.avg_latency_ms);
            if (r.accessible) {
              logs.success(r.service_id, `Service accessible (${r.avg_latency_ms || '-'}ms)`);
            } else {
              logs.error(r.service_id, `Service blocked or unreachable`);
            }
          }
        }
        services = [...services];
        const working = services.filter(s => s.status === 'working').length;
        logs.success('services', `Check complete: ${working}/${services.length} services available`);
        toasts.success(`Checked: ${working}/${services.length} available`);
      } else {
        logs.warn('services', 'No results from backend');
        services = services.map(s => ({ ...s, status: 'unknown' as const, error: 'No response from backend' }));
        toasts.warning('No results received from backend');
      }
    } catch (e) {
      const errorMsg = String(e);
      logs.error('services', `Backend check failed: ${errorMsg}`);
      services = services.map(s => ({ ...s, status: 'error' as const, error: errorMsg }));
      toasts.error(`Check failed: ${errorMsg}`);
    }
    scanning = false;
  }

  async function checkSingleService(id: string, retries = 3) {
    const idx = services.findIndex(s => s.id === id);
    if (idx < 0) return;
    
    services[idx] = { ...services[idx], status: 'checking', error: null };
    services = [...services];
    logs.info(id, `Checking service...`);
    
    const { invoke } = await import('@tauri-apps/api/core');
    for (let attempt = 1; attempt <= retries; attempt++) {
      try {
        const timeoutPromise = new Promise<null>((_, reject) => 
          setTimeout(() => reject(new Error('Timeout')), 10000)
        );
        
        const result = await Promise.race([
          invoke<any>('check_single_service', { serviceId: id }),
          timeoutPromise
        ]);
        
        if (result) {
          services[idx] = {
            ...services[idx],
            status: result.accessible ? 'working' : 'blocked',
            ping: result.avg_latency_ms,
            error: null
          };
          addPingToHistory(id, result.avg_latency_ms);
          services = [...services];
          if (result.accessible) {
            logs.success(id, `Service accessible (${result.avg_latency_ms || '-'}ms)`);
          } else {
            logs.error(id, `Service blocked or unreachable`);
          }
          return;
        }
      } catch (e) {
        const errorMsg = String(e);
        if (attempt === retries) {
          services[idx] = { ...services[idx], status: 'error', error: errorMsg };
          services = [...services];
          logs.error(id, `Check failed after ${retries} attempts: ${errorMsg}`);
          toasts.error(`${services[idx].name}: check failed`);
        } else {
          logs.warn(id, `Attempt ${attempt}/${retries} failed: ${errorMsg}, retrying...`);
          await new Promise(r => setTimeout(r, 1000));
        }
      }
    }
  }

  // ===== Service Management =====
  
  async function addCustomService(data: { name: string; url: string; category: string }) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const id = data.name.toLowerCase().replace(/[^a-z0-9]/g, '-');
      await invoke('register_custom_service', {
        id,
        name: data.name,
        category: data.category,
        endpoints: [data.url]
      });
      
      toasts.success(`Service "${data.name}" added`);
      showAddModal = false;
      await loadFromBackend();
    } catch (e) {
      toasts.error(`Error: ${e}`);
      throw e;
    }
  }

  function confirmDeleteService(service: ServiceWithStatus) {
    serviceToDelete = service;
    showDeleteConfirm = true;
  }

  async function removeService() {
    if (!serviceToDelete) return;
    
    const id = serviceToDelete.id;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('unregister_custom_service', { serviceId: id });
      toasts.success('Service removed');
      if (selectedId === id) {
        selectedId = services.find(s => s.id !== id)?.id || null;
      }
      await loadFromBackend();
    } catch (e) {
      toasts.error(`Error: ${e}`);
    } finally {
      showDeleteConfirm = false;
      serviceToDelete = null;
    }
  }

  // ===== Config Modal =====
  
  async function openConfigModal(service: ServiceWithStatus) {
    configService = service;
    await loadServiceConfig(service.id);
    showConfigModal = true;
  }

  async function loadServiceConfig(serviceId: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const config = await invoke<ServiceConfig | null>('get_service_config', { serviceId });
      
      if (config) {
        serviceConfig = config;
      } else {
        serviceConfig = {
          autoCheck: false,
          checkInterval: '15',
          notifyBlocked: true,
          notifyRestored: false,
          priority: 'normal'
        };
      }
    } catch {
      serviceConfig = {
        autoCheck: false,
        checkInterval: '15',
        notifyBlocked: true,
        notifyRestored: false,
        priority: 'normal'
      };
    }
  }

  async function saveServiceConfig(config: ServiceConfig) {
    if (!configService) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_service_config', {
        serviceId: configService.id,
        config
      });
      toasts.success(`${configService.name} settings saved`);
    } catch (e) {
      toasts.error(`Failed to save settings: ${e}`);
    }
    showConfigModal = false;
    configService = null;
  }

  // ===== Context Menu =====
  
  function handleServiceContextMenu(event: MouseEvent, service: ServiceWithStatus) {
    contextMenuService = service;
    contextMenu?.show(event);
  }

  function handleContextCheckNow() {
    if (contextMenuService) {
      checkSingleService(contextMenuService.id);
    }
  }

  function handleContextConfigure() {
    if (contextMenuService) {
      openConfigModal(contextMenuService);
    }
  }

  function handleContextViewLogs() {
    if (contextMenuService) {
      selectedId = contextMenuService.id;
    }
  }

  function handleContextDisable() {
    if (contextMenuService) {
      toasts.info(`Disable functionality coming soon for ${contextMenuService.name}`);
    }
  }

  // ===== Helpers =====
  
  function addPingToHistory(serviceId: string, ping: number | undefined) {
    if (ping === undefined) return;
    
    // MEMORY SAFETY: Limit total services tracked to prevent unbounded growth
    if (!pingHistory.has(serviceId) && pingHistory.size >= MAX_SERVICES_IN_HISTORY) {
      // Remove oldest entry (first key in Map)
      const firstKey = pingHistory.keys().next().value;
      if (firstKey) {
        pingHistory.delete(firstKey);
      }
    }
    
    const history = pingHistory.get(serviceId) || [];
    const newHistory = [...history, ping].slice(-MAX_PING_HISTORY);
    pingHistory.set(serviceId, newHistory);
    pingHistory = new Map(pingHistory);
  }
  
  // Cleanup ping history when services are removed
  function cleanupPingHistory() {
    const serviceIds = new Set(services.map(s => s.id));
    for (const key of pingHistory.keys()) {
      if (!serviceIds.has(key)) {
        pingHistory.delete(key);
      }
    }
    pingHistory = new Map(pingHistory);
  }
</script>

{#if loading}
  <ServicesSkeleton />
{:else if loadError}
  <!-- Error State -->
  <div class="h-full flex items-center justify-center p-8">
    <div class="max-w-md w-full bg-red-500/10 border border-red-500/30 rounded-2xl p-6 text-center">
      <div class="w-14 h-14 mx-auto mb-4 rounded-full bg-red-500/20 border border-red-500/30 flex items-center justify-center">
        <svg class="w-7 h-7 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
      </div>
      <h3 class="text-lg font-semibold text-zinc-100 mb-2">Failed to Load Services</h3>
      <p class="text-sm text-red-300/80 mb-6">{loadError}</p>
      <button
        onclick={async () => {
          retrying = true;
          await loadFromBackend();
          retrying = false;
        }}
        disabled={retrying}
        class="inline-flex items-center gap-2 px-5 py-2.5 bg-red-500/20 border border-red-500/30 rounded-xl
               text-red-400 text-sm font-medium
               hover:bg-red-500/30 hover:border-red-500/40
               disabled:opacity-50 disabled:cursor-not-allowed
               transition-all duration-200"
      >
        <svg class="w-4 h-4 {retrying ? 'animate-spin' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
          <path d="M3 3v5h5"/>
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
          <path d="M16 16h5v5"/>
        </svg>
        {retrying ? 'Retrying...' : 'Retry'}
      </button>
    </div>
  </div>
{:else}
  <div class="h-full flex">
    <ServiceList
      services={filteredServices}
      totalCount={services.length}
      {selectedId}
      {scanning}
      {searchQuery}
      {statusFilter}
      onsearchchange={(q) => searchQuery = q}
      onfilterchange={(f) => statusFilter = f}
      onselect={(id) => selectedId = id}
      oncontextmenu={handleServiceContextMenu}
      oncheckall={checkAllServices}
      onadd={() => showAddModal = true}
    />

    <ServiceDetails
      service={selected}
      pingHistory={selectedPingHistory}
      oncheck={checkSingleService}
      ondelete={confirmDeleteService}
      onconfigure={openConfigModal}
    />
  </div>
{/if}

<!-- Add Service Modal -->
<AddServiceModal
  open={showAddModal}
  onclose={() => showAddModal = false}
  onadd={addCustomService}
/>

<!-- Configure Service Modal -->
<ServiceConfigModal
  open={showConfigModal}
  service={configService}
  config={serviceConfig}
  onclose={() => { showConfigModal = false; configService = null; }}
  onsave={saveServiceConfig}
/>

<!-- Delete Confirmation Modal -->
<BaseModal open={showDeleteConfirm} onclose={() => { showDeleteConfirm = false; serviceToDelete = null; }} class="w-full max-w-sm">
  {#if serviceToDelete}
    <div class="p-6 text-center">
      <div class="w-14 h-14 mx-auto mb-4 rounded-full bg-red-500/10 border border-red-500/20 flex items-center justify-center">
        <svg class="w-7 h-7 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          <line x1="10" y1="11" x2="10" y2="17"/>
          <line x1="14" y1="11" x2="14" y2="17"/>
        </svg>
      </div>
      <h3 class="text-lg font-semibold text-zinc-100 mb-2">Delete service?</h3>
      <p class="text-sm text-zinc-400 mb-6">
        Are you sure you want to delete <span class="font-medium text-zinc-200">"{serviceToDelete.name}"</span>? This action cannot be undone.
      </p>
      <div class="flex gap-3">
        <button
          onclick={() => { showDeleteConfirm = false; serviceToDelete = null; }}
          class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                 text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={removeService}
          class="flex-1 px-4 py-2.5 bg-red-500 rounded-xl
                 text-white font-medium text-sm hover:bg-red-600 transition-colors"
        >
          Delete
        </button>
      </div>
    </div>
  {/if}
</BaseModal>

<!-- Service Context Menu -->
<ContextMenu bind:this={contextMenu}>
  <ContextMenuItem icon="🔄" onclick={handleContextCheckNow}>
    Check Now
  </ContextMenuItem>
  <ContextMenuItem icon="⚙️" onclick={handleContextConfigure}>
    Configure
  </ContextMenuItem>
  <ContextMenuItem icon="📋" onclick={handleContextViewLogs}>
    View Logs
  </ContextMenuItem>
  <ContextMenuSeparator />
  <ContextMenuItem icon="🚫" variant="danger" onclick={handleContextDisable}>
    Disable
  </ContextMenuItem>
</ContextMenu>
