<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';

  // Plugin type definition
  interface Plugin {
    id: string;
    name: string;
    version: string;
    author: string;
    description: string;
    enabled: boolean;
    icon?: string;
  }

  // Mock data for plugins
  const plugins: Record<string, Plugin> = {
    'discord': {
      id: 'discord',
      name: 'Discord Fix',
      version: '1.0.0',
      author: 'Isolate Team',
      description: 'Исправляет проблемы с голосовыми каналами Discord',
      enabled: true,
      icon: '🎮',
    },
    'speed': {
      id: 'speed',
      name: 'Speed Test',
      version: '1.0.0',
      author: 'Isolate Team',
      description: 'Тестирование скорости соединения',
      enabled: true,
      icon: '⚡',
    },
    'speedtest': {
      id: 'speedtest',
      name: 'Speed Test',
      version: '1.0.0',
      author: 'Isolate Team',
      description: 'Тестирование скорости соединения',
      enabled: true,
      icon: '⚡',
    },
  };

  // Get plugin ID from route params
  let pluginId = $derived($page.params.id);
  let plugin = $derived(plugins[pluginId] || null);

  // Settings state
  let autoStart = $state(true);
  let notifications = $state(true);

  // Action handlers
  function handleBack() {
    goto('/');
  }

  function handleToggleEnabled() {
    if (plugin) {
      plugins[pluginId].enabled = !plugins[pluginId].enabled;
    }
  }

  function handleDelete() {
    // TODO: Implement actual delete
    console.log('Deleting plugin:', pluginId);
    goto('/');
  }
</script>

<div class="h-full bg-void overflow-y-auto">
  {#if plugin}
    <div class="p-6 max-w-3xl mx-auto">
      <!-- Header with Back Button -->
      <div class="flex items-center gap-4 mb-6">
        <button
          onclick={handleBack}
          class="w-10 h-10 flex items-center justify-center rounded-lg bg-void-50 border border-glass-border
                 text-text-secondary hover:text-text-primary hover:bg-void-100 transition-colors"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 12H5"/>
            <path d="M12 19l-7-7 7-7"/>
          </svg>
        </button>
        <h1 class="text-2xl font-semibold text-text-primary">{plugin.name}</h1>
      </div>

      <!-- Plugin Info Card -->
      <div class="bg-void-50 border border-glass-border rounded-xl p-5 mb-6">
        <div class="flex items-start gap-4">
          <!-- Plugin Icon -->
          <div class="w-14 h-14 rounded-xl bg-void-100 border border-glass-border flex items-center justify-center text-2xl flex-shrink-0">
            {plugin.icon || '🔧'}
          </div>
          
          <!-- Plugin Details -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-3 mb-2">
              <!-- Status Badge -->
              <span class="px-2.5 py-1 rounded-full text-xs font-medium
                {plugin.enabled 
                  ? 'bg-neon-green/10 text-neon-green border border-neon-green/30' 
                  : 'bg-void-200 text-text-muted border border-glass-border'}">
                {plugin.enabled ? '🔧 Активен' : 'Отключен'}
              </span>
            </div>
            
            <p class="text-text-secondary text-sm mb-3">{plugin.description}</p>
            
            <div class="flex items-center gap-4 text-xs text-text-muted">
              <span>Версия: <span class="text-text-secondary">{plugin.version}</span></span>
              <span>Автор: <span class="text-text-secondary">{plugin.author}</span></span>
            </div>
          </div>
        </div>
      </div>

      <!-- Settings Card -->
      <div class="bg-void-50 border border-glass-border rounded-xl p-5 mb-6">
        <h2 class="text-sm font-medium text-text-primary mb-4">Настройки плагина</h2>
        
        <div class="space-y-4">
          <!-- Auto Start Toggle -->
          <label class="flex items-center justify-between cursor-pointer group">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-lg bg-void-100 border border-glass-border flex items-center justify-center text-text-muted group-hover:text-text-secondary transition-colors">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="5 3 19 12 5 21 5 3"/>
                </svg>
              </div>
              <span class="text-sm text-text-secondary">Автозапуск</span>
            </div>
            <button
              onclick={() => autoStart = !autoStart}
              class="relative w-11 h-6 rounded-full transition-colors duration-200
                {autoStart ? 'bg-electric' : 'bg-void-200'}"
            >
              <span
                class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform duration-200
                  {autoStart ? 'translate-x-5' : 'translate-x-0'}"
              ></span>
            </button>
          </label>

          <!-- Notifications Toggle -->
          <label class="flex items-center justify-between cursor-pointer group">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-lg bg-void-100 border border-glass-border flex items-center justify-center text-text-muted group-hover:text-text-secondary transition-colors">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
                  <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
                </svg>
              </div>
              <span class="text-sm text-text-secondary">Уведомления</span>
            </div>
            <button
              onclick={() => notifications = !notifications}
              class="relative w-11 h-6 rounded-full transition-colors duration-200
                {notifications ? 'bg-electric' : 'bg-void-200'}"
            >
              <span
                class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform duration-200
                  {notifications ? 'translate-x-5' : 'translate-x-0'}"
              ></span>
            </button>
          </label>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex gap-3">
        <button
          onclick={handleToggleEnabled}
          class="flex-1 flex items-center justify-center gap-2 px-4 py-3 
                 bg-void-100 border border-glass-border rounded-xl
                 text-text-secondary font-medium text-sm
                 hover:bg-void-200 hover:text-text-primary
                 transition-colors duration-150"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/>
          </svg>
          {plugin.enabled ? 'Отключить' : 'Включить'}
        </button>

        <button
          onclick={handleDelete}
          class="flex-1 flex items-center justify-center gap-2 px-4 py-3 
                 bg-neon-red/10 border border-neon-red/30 rounded-xl
                 text-neon-red font-medium text-sm
                 hover:bg-neon-red/20 hover:border-neon-red/50
                 transition-colors duration-150"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            <line x1="10" y1="11" x2="10" y2="17"/>
            <line x1="14" y1="11" x2="14" y2="17"/>
          </svg>
          Удалить
        </button>
      </div>
    </div>
  {:else}
    <!-- Plugin Not Found -->
    <div class="h-full flex flex-col items-center justify-center text-center p-6">
      <div class="w-20 h-20 rounded-2xl bg-void-50 border border-glass-border flex items-center justify-center mb-4">
        <svg class="w-10 h-10 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 8v4"/>
          <path d="M12 16h.01"/>
        </svg>
      </div>
      <h2 class="text-xl font-semibold text-text-primary mb-2">Плагин не найден</h2>
      <p class="text-sm text-text-muted max-w-xs mb-6">
        Плагин «{pluginId}» не установлен или был удалён.
      </p>
      <button
        onclick={handleBack}
        class="flex items-center gap-2 px-4 py-2 
               bg-electric/10 border border-electric/30 rounded-lg
               text-electric font-medium text-sm
               hover:bg-electric/20 hover:border-electric/50
               transition-colors duration-150"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 12H5"/>
          <path d="M12 19l-7-7 7-7"/>
        </svg>
        Вернуться на главную
      </button>
    </div>
  {/if}
</div>
