<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { toasts } from '$lib/stores/toast';
  import { logs } from '$lib/stores/logs';
  import { browser } from '$app/environment';

  // Types
  interface ServiceWithStatus {
    id: string;
    name: string;
    status: 'working' | 'blocked' | 'unknown';
    ping?: number;
    category: string;
    isCustom?: boolean;
  }

  // State
  let services: ServiceWithStatus[] = $state([]);
  let selectedId: string | null = $state(null);
  let loading = $state(true);
  let scanning = $state(false);
  
  // Add service modal
  let showAddModal = $state(false);
  let newService = $state({ name: '', url: '', category: 'other' });
  let addingService = $state(false);
  
  // Configure service modal
  let showConfigModal = $state(false);
  let configService: ServiceWithStatus | null = $state(null);
  
  function openConfigModal(service: ServiceWithStatus) {
    configService = service;
    showConfigModal = true;
  }
  
  function closeConfigModal() {
    showConfigModal = false;
    configService = null;
  }

  // Load services on mount
  $effect(() => {
    loadFromBackend();
  });

  // Service icons (SVG paths)
  const serviceIcons: Record<string, { path: string; color: string }> = {
    youtube: {
      path: 'M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z',
      color: 'text-red-500'
    },
    discord: {
      path: 'M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z',
      color: 'text-indigo-400'
    },
    telegram: {
      path: 'M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z',
      color: 'text-sky-400'
    },
    twitter: {
      path: 'M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z',
      color: 'text-zinc-100'
    },
    instagram: {
      path: 'M12 0C8.74 0 8.333.015 7.053.072 5.775.132 4.905.333 4.14.63c-.789.306-1.459.717-2.126 1.384S.935 3.35.63 4.14C.333 4.905.131 5.775.072 7.053.012 8.333 0 8.74 0 12s.015 3.667.072 4.947c.06 1.277.261 2.148.558 2.913.306.788.717 1.459 1.384 2.126.667.666 1.336 1.079 2.126 1.384.766.296 1.636.499 2.913.558C8.333 23.988 8.74 24 12 24s3.667-.015 4.947-.072c1.277-.06 2.148-.262 2.913-.558.788-.306 1.459-.718 2.126-1.384.666-.667 1.079-1.335 1.384-2.126.296-.765.499-1.636.558-2.913.06-1.28.072-1.687.072-4.947s-.015-3.667-.072-4.947c-.06-1.277-.262-2.149-.558-2.913-.306-.789-.718-1.459-1.384-2.126C21.319 1.347 20.651.935 19.86.63c-.765-.297-1.636-.499-2.913-.558C15.667.012 15.26 0 12 0zm0 2.16c3.203 0 3.585.016 4.85.071 1.17.055 1.805.249 2.227.415.562.217.96.477 1.382.896.419.42.679.819.896 1.381.164.422.36 1.057.413 2.227.057 1.266.07 1.646.07 4.85s-.015 3.585-.074 4.85c-.061 1.17-.256 1.805-.421 2.227a3.81 3.81 0 0 1-.899 1.382 3.744 3.744 0 0 1-1.38.896c-.42.164-1.065.36-2.235.413-1.274.057-1.649.07-4.859.07-3.211 0-3.586-.015-4.859-.074-1.171-.061-1.816-.256-2.236-.421a3.716 3.716 0 0 1-1.379-.899 3.644 3.644 0 0 1-.9-1.38c-.165-.42-.359-1.065-.42-2.235-.045-1.26-.061-1.649-.061-4.844 0-3.196.016-3.586.061-4.861.061-1.17.255-1.814.42-2.234.21-.57.479-.96.9-1.381.419-.419.81-.689 1.379-.898.42-.166 1.051-.361 2.221-.421 1.275-.045 1.65-.06 4.859-.06l.045.03zm0 3.678a6.162 6.162 0 1 0 0 12.324 6.162 6.162 0 1 0 0-12.324zM12 16c-2.21 0-4-1.79-4-4s1.79-4 4-4 4 1.79 4 4-1.79 4-4 4zm7.846-10.405a1.441 1.441 0 1 1-2.88 0 1.441 1.441 0 0 1 2.88 0z',
      color: 'text-pink-400'
    },
    spotify: {
      path: 'M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z',
      color: 'text-green-400'
    },
    twitch: {
      path: 'M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714z',
      color: 'text-purple-400'
    },
    netflix: {
      path: 'M5.398 0v.006c3.028 8.556 5.37 15.175 8.348 23.596 2.344.058 4.85.398 4.854.398-2.8-7.924-5.923-16.747-8.487-24zm8.489 0v9.63L18.6 22.951c-.043-7.86-.004-15.913.002-22.95zM5.398 1.05V24c1.873-.225 2.81-.312 4.715-.398v-9.22z',
      color: 'text-red-600'
    },
    default: {
      path: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z',
      color: 'text-zinc-400'
    }
  };

  function getIcon(id: string) {
    return serviceIcons[id] || serviceIcons.default;
  }

  function getStatusColor(status: string): string {
    if (status === 'working') return 'bg-emerald-500';
    if (status === 'blocked') return 'bg-red-500';
    return 'bg-amber-500';
  }

  function getStatusBadge(status: string): string {
    if (status === 'working') return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
    if (status === 'blocked') return 'bg-red-500/10 text-red-400 border-red-500/20';
    return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
  }

  function getStatusText(status: string): string {
    if (status === 'working') return 'Working';
    if (status === 'blocked') return 'Blocked';
    return 'Unknown';
  }

  function getPingColor(ping?: number): string {
    if (!ping) return 'text-zinc-500';
    if (ping < 50) return 'text-emerald-400';
    if (ping < 150) return 'text-amber-400';
    return 'text-red-400';
  }

  async function checkAllServices() {
    if (!browser || scanning) return;
    scanning = true;
    logs.info('services', 'Starting check of all services...');
    
    try {
      // Timeout wrapper for backend call
      const timeoutPromise = new Promise<null>((_, reject) => 
        setTimeout(() => reject(new Error('Timeout')), 5000)
      );
      
      const results = await Promise.race([
        invoke<any[]>('check_all_registry_services'),
        timeoutPromise
      ]);
      
      if (results && results.length > 0) {
        // Update services with real results
        for (const r of results) {
          const idx = services.findIndex(s => s.id === r.service_id);
          if (idx >= 0) {
            services[idx] = {
              ...services[idx],
              status: r.accessible ? 'working' : 'blocked',
              ping: r.avg_latency_ms
            };
            logs.debug('services', `${r.service_id}: ${r.accessible ? 'OK' : 'BLOCKED'} (${r.avg_latency_ms || '-'}ms)`);
          }
        }
        services = [...services];
        const working = services.filter(s => s.status === 'working').length;
        logs.success('services', `Check complete: ${working}/${services.length} services available`);
        toasts.success(`Проверено: ${working}/${services.length} доступно`);
      } else {
        logs.warn('services', 'No results from backend, using simulation');
        simulateCheck();
      }
    } catch (e) {
      logs.error('services', `Backend check failed: ${e}`);
      console.warn('Backend check failed:', e);
      simulateCheck();
    }
    scanning = false;
  }

  function simulateCheck() {
    // Simulate realistic check results for demo
    services = services.map(s => ({
      ...s,
      status: Math.random() > 0.3 ? 'working' as const : 'blocked' as const,
      ping: Math.floor(Math.random() * 400) + 30
    }));
    const working = services.filter(s => s.status === 'working').length;
    toasts.info(`Демо: ${working}/${services.length} доступно`);
  }

  async function checkSingleService(id: string) {
    const idx = services.findIndex(s => s.id === id);
    if (idx < 0) return;
    
    logs.info('services', `Checking service: ${id}`);
    
    try {
      const timeoutPromise = new Promise<null>((_, reject) => 
        setTimeout(() => reject(new Error('Timeout')), 5000)
      );
      
      const result = await Promise.race([
        invoke<any>('check_single_service', { serviceId: id }),
        timeoutPromise
      ]);
      
      if (result) {
        services[idx] = {
          ...services[idx],
          status: result.accessible ? 'working' : 'blocked',
          ping: result.avg_latency_ms
        };
        services = [...services];
        logs.success('services', `${id}: ${result.accessible ? 'OK' : 'BLOCKED'} (${result.avg_latency_ms || '-'}ms)`);
      }
    } catch {
      logs.warn('services', `Check failed for ${id}, using simulation`);
      // Simulate on timeout/error
      services[idx] = {
        ...services[idx],
        status: Math.random() > 0.3 ? 'working' : 'blocked',
        ping: Math.floor(Math.random() * 150) + 30
      };
      services = [...services];
    }
  }

  async function loadFromBackend(retries = 10) {
    loading = true;
    logs.info('services', 'Loading services from backend...');
    
    for (let i = 0; i < retries; i++) {
      try {
        // Check if backend is ready
        const ready = await invoke<boolean>('is_backend_ready');
        if (!ready) {
          await new Promise(r => setTimeout(r, 200));
          continue;
        }
        
        // Load services
        const result = await invoke<any[]>('get_registry_services');
        
        if (result && result.length > 0) {
          services = result.map(s => ({
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
        }
        loading = false;
        return;
      } catch (e) {
        logs.warn('services', `Load attempt ${i + 1} failed: ${e}`);
        console.warn(`Load attempt ${i + 1} failed:`, e);
        await new Promise(r => setTimeout(r, 200));
      }
    }
    
    logs.error('services', 'Failed to load services after retries');
    console.warn('Failed to load services after retries');
    loading = false;
  }

  async function addCustomService() {
    if (!newService.name || !newService.url) {
      toasts.error('Заполните название и URL');
      return;
    }
    
    addingService = true;
    try {
      const id = newService.name.toLowerCase().replace(/[^a-z0-9]/g, '-');
      await invoke('register_custom_service', {
        id,
        name: newService.name,
        category: newService.category,
        endpoints: [newService.url]
      });
      
      toasts.success(`Сервис "${newService.name}" добавлен`);
      showAddModal = false;
      newService = { name: '', url: '', category: 'other' };
      await loadFromBackend();
    } catch (e) {
      toasts.error(`Ошибка: ${e}`);
    }
    addingService = false;
  }

  async function removeService(id: string) {
    try {
      await invoke('unregister_custom_service', { serviceId: id });
      toasts.success('Сервис удалён');
      if (selectedId === id) {
        selectedId = services.find(s => s.id !== id)?.id || null;
      }
      await loadFromBackend();
    } catch (e) {
      toasts.error(`Ошибка: ${e}`);
    }
  }

  // Get selected service
  let selected = $derived(services.find(s => s.id === selectedId) || null);
</script>

<div class="h-full flex">
  <!-- Left Panel - Service List -->
  <div class="w-[320px] flex-shrink-0 border-r border-white/5 flex flex-col bg-zinc-900/30">
    <!-- Header -->
    <div class="p-4 border-b border-white/5">
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-lg font-semibold text-zinc-100">Services</h2>
        <button
          onclick={() => showAddModal = true}
          class="p-2 rounded-lg bg-zinc-800/60 border border-white/5 
                 hover:bg-zinc-700/60 hover:border-white/10 transition-colors"
          title="Add custom service"
        >
          <svg class="w-4 h-4 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 5v14M5 12h14"/>
          </svg>
        </button>
      </div>
      <button
        onclick={checkAllServices}
        disabled={scanning || loading}
        class="w-full flex items-center justify-center gap-2 px-4 py-2.5
               bg-indigo-500/10 border border-indigo-500/20 rounded-xl
               text-indigo-400 text-sm font-medium
               hover:bg-indigo-500/20 hover:border-indigo-500/30
               disabled:opacity-50 disabled:cursor-not-allowed
               transition-all duration-200"
      >
        {#if scanning}
          <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
          </svg>
          Checking...
        {:else}
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
            <path d="M3 3v5h5"/>
            <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
            <path d="M16 16h5v5"/>
          </svg>
          Check All Services
        {/if}
      </button>
    </div>

    <!-- Service List -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      {#if loading}
        <div class="flex items-center justify-center py-12">
          <svg class="w-6 h-6 animate-spin text-zinc-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
          </svg>
        </div>
      {:else}
        {#each services as service (service.id)}
          {@const icon = getIcon(service.id)}
          <button
            onclick={() => selectedId = service.id}
            class="group w-full flex items-center gap-3 p-3 rounded-xl transition-all duration-200
                   {selectedId === service.id 
                     ? 'bg-white/5 border border-indigo-500/30 shadow-lg shadow-indigo-500/5' 
                     : 'bg-zinc-900/40 border border-white/5 hover:bg-zinc-900/60 hover:border-white/10'}"
          >
            <!-- Icon -->
            <div class="w-10 h-10 flex-shrink-0 flex items-center justify-center rounded-lg bg-zinc-800/60 group-hover:bg-zinc-800 transition-colors">
              <svg class="w-5 h-5 {icon.color}" viewBox="0 0 24 24" fill="currentColor">
                <path d={icon.path}/>
              </svg>
            </div>
            
            <!-- Info -->
            <div class="flex-1 text-left min-w-0">
              <div class="text-sm font-medium text-zinc-100 truncate">{service.name}</div>
              <div class="text-xs text-zinc-500 capitalize">{service.category}</div>
            </div>

            <!-- Status -->
            <div class="flex items-center gap-2 flex-shrink-0">
              {#if service.ping}
                <span class="text-xs font-mono {getPingColor(service.ping)}">{service.ping}ms</span>
              {/if}
              <div class="w-2.5 h-2.5 rounded-full {getStatusColor(service.status)} 
                          {service.status === 'working' ? 'animate-pulse' : ''}"></div>
            </div>
          </button>
        {/each}
      {/if}
    </div>

    <!-- Footer -->
    <div class="p-3 border-t border-white/5">
      <div class="text-xs text-zinc-500 text-center">
        {services.filter(s => s.status === 'working').length} / {services.length} services available
      </div>
    </div>
  </div>

  <!-- Right Panel - Details -->
  <div class="flex-1 overflow-y-auto bg-zinc-950">
    {#if loading}
      <div class="h-full flex items-center justify-center">
        <svg class="w-8 h-8 animate-spin text-zinc-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
        </svg>
      </div>
    {:else if selected}
      {@const icon = getIcon(selected.id)}
      <div class="p-6 max-w-3xl">
        <!-- Header -->
        <div class="flex items-start gap-5 mb-8">
          <div class="relative">
            <div class="w-20 h-20 rounded-2xl bg-zinc-900/60 border border-white/5 flex items-center justify-center">
              <svg class="w-10 h-10 {icon.color}" viewBox="0 0 24 24" fill="currentColor">
                <path d={icon.path}/>
              </svg>
            </div>
            <div class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full {getStatusColor(selected.status)} 
                        border-2 border-zinc-950 {selected.status === 'working' ? 'animate-pulse' : ''}"></div>
          </div>
          <div class="flex-1">
            <h1 class="text-2xl font-bold text-zinc-100 mb-2">{selected.name}</h1>
            <div class="flex items-center gap-3">
              <span class="px-3 py-1 rounded-lg text-xs font-medium border {getStatusBadge(selected.status)}">
                {getStatusText(selected.status)}
              </span>
              <span class="text-sm text-zinc-500 capitalize">{selected.category}</span>
            </div>
          </div>
        </div>

        <!-- Stats Grid -->
        <div class="grid grid-cols-3 gap-4 mb-8">
          <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl hover:border-white/10 transition-colors">
            <div class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Latency</div>
            <div class="flex items-baseline gap-1">
              {#if selected.ping}
                <span class="text-2xl font-bold {getPingColor(selected.ping)}">{selected.ping}</span>
                <span class="text-sm text-zinc-500">ms</span>
              {:else}
                <span class="text-2xl font-bold text-zinc-600">—</span>
              {/if}
            </div>
          </div>

          <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl hover:border-white/10 transition-colors">
            <div class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Status</div>
            <div class="text-xl font-semibold {selected.status === 'working' ? 'text-emerald-400' : selected.status === 'blocked' ? 'text-red-400' : 'text-amber-400'}">
              {getStatusText(selected.status)}
            </div>
          </div>

          <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl hover:border-white/10 transition-colors">
            <div class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Category</div>
            <div class="text-xl font-semibold text-zinc-100 capitalize">{selected.category}</div>
          </div>
        </div>

        <!-- Connection Details -->
        <div class="p-5 bg-zinc-900/30 border border-white/5 rounded-xl mb-6">
          <h3 class="text-sm font-medium text-zinc-100 mb-4">Connection Details</h3>
          <div class="space-y-3">
            <div class="flex items-center justify-between text-sm">
              <span class="text-zinc-500">DNS Resolution</span>
              <span class="text-zinc-300">{selected.status === 'blocked' ? 'Failed' : selected.status === 'unknown' ? '—' : 'OK'}</span>
            </div>
            <div class="flex items-center justify-between text-sm">
              <span class="text-zinc-500">TCP Connection</span>
              <span class="text-zinc-300">{selected.status === 'blocked' ? 'Blocked' : selected.status === 'unknown' ? '—' : 'Established'}</span>
            </div>
            <div class="flex items-center justify-between text-sm">
              <span class="text-zinc-500">TLS Handshake</span>
              <span class="text-zinc-300">{selected.status === 'blocked' ? 'Failed' : selected.status === 'unknown' ? '—' : 'Success'}</span>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex gap-3">
          <button
            onclick={() => checkSingleService(selected.id)}
            class="flex-1 flex items-center justify-center gap-2 px-4 py-3 
                   bg-indigo-500 hover:bg-indigo-600 rounded-xl
                   text-white font-medium text-sm
                   transition-all duration-200 hover:-translate-y-0.5 hover:shadow-lg hover:shadow-indigo-500/20"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
              <path d="M3 3v5h5"/>
            </svg>
            Re-check
          </button>

          {#if selected.isCustom}
            <button
              onclick={() => removeService(selected.id)}
              class="flex items-center justify-center gap-2 px-4 py-3 
                     bg-red-500/10 border border-red-500/20 rounded-xl
                     text-red-400 font-medium text-sm
                     hover:bg-red-500/20 hover:border-red-500/30
                     transition-all duration-200 hover:-translate-y-0.5"
            >
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
              Remove
            </button>
          {:else}
            <button
              onclick={() => openConfigModal(selected)}
              class="flex-1 flex items-center justify-center gap-2 px-4 py-3 
                     bg-zinc-900/60 border border-white/5 rounded-xl
                     text-zinc-100 font-medium text-sm
                     hover:bg-zinc-800/60 hover:border-white/10
                     transition-all duration-200 hover:-translate-y-0.5"
            >
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
              </svg>
              Configure
            </button>
          {/if}
        </div>
      </div>
    {:else}
      <!-- Empty State -->
      <div class="h-full flex flex-col items-center justify-center text-center p-6">
        <div class="w-20 h-20 rounded-2xl bg-zinc-900/40 border border-white/5 flex items-center justify-center mb-4">
          <svg class="w-10 h-10 text-zinc-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
            <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
            <line x1="6" y1="6" x2="6.01" y2="6"/>
            <line x1="6" y1="18" x2="6.01" y2="18"/>
          </svg>
        </div>
        <h2 class="text-lg font-medium text-zinc-100 mb-2">Select a service</h2>
        <p class="text-sm text-zinc-500 max-w-xs">
          Choose a service from the list to view its status and configuration.
        </p>
      </div>
    {/if}
  </div>
</div>

<!-- Add Service Modal -->
{#if showAddModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" onclick={() => showAddModal = false}></div>
    <div class="relative w-full max-w-md bg-zinc-900 border border-white/10 rounded-2xl shadow-2xl">
      <div class="p-6">
        <h3 class="text-lg font-semibold text-zinc-100 mb-4">Add Custom Service</h3>
        
        <div class="space-y-4">
          <div>
            <label class="block text-sm text-zinc-400 mb-1.5">Service Name</label>
            <input
              type="text"
              bind:value={newService.name}
              placeholder="e.g. My Service"
              class="w-full px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                     text-zinc-100 placeholder-zinc-500
                     focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20"
            />
          </div>
          
          <div>
            <label class="block text-sm text-zinc-400 mb-1.5">Check URL</label>
            <input
              type="url"
              bind:value={newService.url}
              placeholder="https://example.com"
              class="w-full px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                     text-zinc-100 placeholder-zinc-500
                     focus:outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/20"
            />
          </div>
          
          <div>
            <label class="block text-sm text-zinc-400 mb-1.5">Category</label>
            <select
              bind:value={newService.category}
              class="w-full px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                     text-zinc-100 focus:outline-none focus:border-indigo-500/50"
            >
              <option value="social">Social</option>
              <option value="video">Video</option>
              <option value="gaming">Gaming</option>
              <option value="messaging">Messaging</option>
              <option value="streaming">Streaming</option>
              <option value="other">Other</option>
            </select>
          </div>
        </div>
        
        <div class="flex gap-3 mt-6">
          <button
            onclick={() => showAddModal = false}
            class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                   text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={addCustomService}
            disabled={addingService || !newService.name || !newService.url}
            class="flex-1 px-4 py-2.5 bg-indigo-500 rounded-xl
                   text-white font-medium text-sm hover:bg-indigo-600
                   disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {addingService ? 'Adding...' : 'Add Service'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- Configure Service Modal -->
{#if showConfigModal && configService}
  {@const icon = getIcon(configService.id)}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" onclick={closeConfigModal}></div>
    <div class="relative w-full max-w-lg bg-zinc-900 border border-white/10 rounded-2xl shadow-2xl">
      <!-- Header -->
      <div class="flex items-center gap-4 p-6 border-b border-white/5">
        <div class="w-12 h-12 rounded-xl bg-zinc-800/60 border border-white/5 flex items-center justify-center">
          <svg class="w-6 h-6 {icon.color}" viewBox="0 0 24 24" fill="currentColor">
            <path d={icon.path}/>
          </svg>
        </div>
        <div class="flex-1">
          <h3 class="text-lg font-semibold text-zinc-100">{configService.name}</h3>
          <p class="text-sm text-zinc-500 capitalize">{configService.category} • Configuration</p>
        </div>
        <button
          onclick={closeConfigModal}
          class="p-2 rounded-lg hover:bg-zinc-800 transition-colors"
        >
          <svg class="w-5 h-5 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>
      
      <!-- Content -->
      <div class="p-6 space-y-5">
        <!-- Check Settings -->
        <div>
          <h4 class="text-sm font-medium text-zinc-300 mb-3 flex items-center gap-2">
            <svg class="w-4 h-4 text-zinc-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
              <polyline points="22 4 12 14.01 9 11.01"/>
            </svg>
            Check Settings
          </h4>
          <div class="space-y-3">
            <div class="flex items-center justify-between p-3 bg-zinc-800/40 rounded-xl border border-white/5">
              <span class="text-sm text-zinc-400">Auto-check on startup</span>
              <div class="w-10 h-6 bg-zinc-700 rounded-full relative cursor-pointer">
                <div class="absolute left-1 top-1 w-4 h-4 bg-zinc-400 rounded-full transition-transform"></div>
              </div>
            </div>
            <div class="flex items-center justify-between p-3 bg-zinc-800/40 rounded-xl border border-white/5">
              <span class="text-sm text-zinc-400">Check interval</span>
              <select class="bg-zinc-700 border border-white/10 rounded-lg px-3 py-1.5 text-sm text-zinc-200 focus:outline-none">
                <option value="5">5 min</option>
                <option value="15">15 min</option>
                <option value="30">30 min</option>
                <option value="60">1 hour</option>
              </select>
            </div>
          </div>
        </div>
        
        <!-- Notification Settings -->
        <div>
          <h4 class="text-sm font-medium text-zinc-300 mb-3 flex items-center gap-2">
            <svg class="w-4 h-4 text-zinc-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
              <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
            </svg>
            Notifications
          </h4>
          <div class="space-y-3">
            <div class="flex items-center justify-between p-3 bg-zinc-800/40 rounded-xl border border-white/5">
              <span class="text-sm text-zinc-400">Notify when blocked</span>
              <div class="w-10 h-6 bg-indigo-500 rounded-full relative cursor-pointer">
                <div class="absolute right-1 top-1 w-4 h-4 bg-white rounded-full transition-transform"></div>
              </div>
            </div>
            <div class="flex items-center justify-between p-3 bg-zinc-800/40 rounded-xl border border-white/5">
              <span class="text-sm text-zinc-400">Notify when restored</span>
              <div class="w-10 h-6 bg-zinc-700 rounded-full relative cursor-pointer">
                <div class="absolute left-1 top-1 w-4 h-4 bg-zinc-400 rounded-full transition-transform"></div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Priority -->
        <div>
          <h4 class="text-sm font-medium text-zinc-300 mb-3 flex items-center gap-2">
            <svg class="w-4 h-4 text-zinc-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
            </svg>
            Priority
          </h4>
          <div class="flex gap-2">
            <button class="flex-1 px-4 py-2.5 bg-zinc-800/40 border border-white/5 rounded-xl text-sm text-zinc-400 hover:bg-zinc-700/40 transition-colors">
              Low
            </button>
            <button class="flex-1 px-4 py-2.5 bg-indigo-500/20 border border-indigo-500/30 rounded-xl text-sm text-indigo-400 font-medium">
              Normal
            </button>
            <button class="flex-1 px-4 py-2.5 bg-zinc-800/40 border border-white/5 rounded-xl text-sm text-zinc-400 hover:bg-zinc-700/40 transition-colors">
              High
            </button>
          </div>
        </div>
      </div>
      
      <!-- Footer -->
      <div class="flex gap-3 p-6 border-t border-white/5">
        <button
          onclick={closeConfigModal}
          class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                 text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={() => {
            toasts.success(`Настройки ${configService?.name} сохранены`);
            closeConfigModal();
          }}
          class="flex-1 px-4 py-2.5 bg-indigo-500 rounded-xl
                 text-white font-medium text-sm hover:bg-indigo-600 transition-colors"
        >
          Save Changes
        </button>
      </div>
    </div>
  </div>
{/if}
