<script lang="ts">
  interface QuickAction {
    id: string;
    label: string;
    onClick?: () => void;
    disabled?: boolean;
    loading?: boolean;
  }

  interface Props {
    actions?: QuickAction[];
    onAction?: (actionId: string) => void;
    backendReady?: boolean;
    hasActiveStrategy?: boolean;
  }

  let { 
    actions = [
      { id: 'scan', label: 'Scan All' },
      { id: 'test', label: 'Test Current' },
      { id: 'proxy', label: 'Add Proxy' },
      { id: 'settings', label: 'Settings' }
    ],
    onAction,
    backendReady = true,
    hasActiveStrategy = false
  }: Props = $props();

  // Tooltip visibility state
  let visibleTooltip = $state<string | null>(null);
  let tooltipTimeout = $state<ReturnType<typeof setTimeout> | null>(null);

  function handleClick(action: QuickAction) {
    const isDisabled = getDisabledState(action);
    if (isDisabled || action.loading) return;
    action.onClick?.();
    onAction?.(action.id);
  }

  // Determine if action should be disabled based on context
  function getDisabledState(action: QuickAction): boolean {
    if (action.disabled) return true;
    if (!backendReady && action.id !== 'settings') return true;
    if (action.id === 'test' && !hasActiveStrategy) return true;
    return false;
  }

  // Get disabled reason for tooltip
  function getDisabledReason(action: QuickAction): string | null {
    if (!backendReady && action.id !== 'settings') {
      return 'Бэкенд загружается...';
    }
    if (action.id === 'test' && !hasActiveStrategy) {
      return 'Нет активной стратегии';
    }
    if (action.disabled) {
      return 'Действие недоступно';
    }
    return null;
  }

  // Show tooltip with delay
  function showTooltip(actionId: string) {
    if (tooltipTimeout) clearTimeout(tooltipTimeout);
    tooltipTimeout = setTimeout(() => {
      visibleTooltip = actionId;
    }, 400);
  }

  // Hide tooltip
  function hideTooltip() {
    if (tooltipTimeout) clearTimeout(tooltipTimeout);
    visibleTooltip = null;
  }

  // SVG icons for actions
  const actionIcons: Record<string, string> = {
    scan: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z',
    test: 'M13 10V3L4 14h7v7l9-11h-7z',
    proxy: 'M12 4v16m8-8H4',
    settings: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z'
  };

  // Tooltips with descriptions and side effects
  const actionTooltips: Record<string, { title: string; description: string; sideEffects?: string }> = {
    scan: {
      title: 'Полное сканирование',
      description: 'Тестирует все доступные стратегии и выбирает лучшую',
      sideEffects: '⚠️ Может занять 1-3 минуты. Текущая защита будет временно отключена.'
    },
    test: {
      title: 'Тест текущей стратегии',
      description: 'Проверяет работоспособность активной стратегии',
      sideEffects: '⚠️ Кратковременные прерывания соединений во время теста.'
    },
    proxy: {
      title: 'Добавить прокси',
      description: 'Открывает страницу настройки VLESS/прокси серверов'
    },
    settings: {
      title: 'Настройки',
      description: 'Открывает страницу настроек приложения'
    }
  };

  // Loading text for actions
  const loadingText: Record<string, string> = {
    scan: 'Сканирование...',
    test: 'Тестирование...'
  };
</script>

<div class="grid grid-cols-2 gap-2 h-full">
  {#each actions as action}
    {@const isDisabled = getDisabledState(action)}
    {@const disabledReason = getDisabledReason(action)}
    {@const tooltip = actionTooltips[action.id]}
    
    <div class="relative">
      <button
        class="group w-full
          flex flex-col items-center justify-center gap-2 p-3
          rounded-lg border transition-all duration-200
          {action.loading 
            ? 'bg-electric/10 border-electric/30 animate-pulse' 
            : isDisabled 
              ? 'bg-zinc-900/20 border-white/5 opacity-40 cursor-not-allowed' 
              : 'bg-zinc-900/40 border-white/5 hover:bg-zinc-800/60 hover:border-white/10 hover:-translate-y-0.5 active:scale-95'}
        "
        disabled={isDisabled || action.loading}
        onclick={() => handleClick(action)}
        onmouseenter={() => showTooltip(action.id)}
        onmouseleave={hideTooltip}
        onfocus={() => showTooltip(action.id)}
        onblur={hideTooltip}
      >
        <!-- Icon container with loading state -->
        <div class="w-8 h-8 rounded-lg flex items-center justify-center transition-all duration-200
                    {action.loading 
                      ? 'bg-electric/20 border border-electric/30' 
                      : isDisabled 
                        ? 'bg-zinc-800/30 border border-white/5' 
                        : 'bg-zinc-800/50 border border-white/5 group-hover:bg-zinc-700/50 group-hover:border-white/10'}">
          {#if action.loading}
            <!-- Enhanced loading spinner -->
            <svg class="w-4 h-4 text-electric animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          {:else}
            <svg class="w-4 h-4 transition-colors duration-200
                        {isDisabled ? 'text-zinc-400' : 'text-zinc-400 group-hover:text-zinc-200'}" 
                 fill="none" stroke="currentColor" viewBox="0 0 24 24" 
                 stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d={actionIcons[action.id] || 'M13 10V3L4 14h7v7l9-11h-7z'}/>
            </svg>
          {/if}
        </div>
        
        <!-- Label with loading text -->
        <span class="text-xs font-medium transition-colors duration-200
                     {action.loading 
                       ? 'text-electric' 
                       : isDisabled 
                         ? 'text-zinc-400' 
                         : 'text-zinc-400 group-hover:text-zinc-200'}">
          {action.loading ? (loadingText[action.id] || action.label) : action.label}
        </span>
        
        <!-- Loading progress bar -->
        {#if action.loading}
          <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-zinc-800 rounded-b-lg overflow-hidden">
            <div class="h-full bg-electric animate-loading-bar"></div>
          </div>
        {/if}
      </button>
      
      <!-- Tooltip -->
      {#if visibleTooltip === action.id && tooltip}
        <div class="absolute z-50 bottom-full left-1/2 -translate-x-1/2 mb-2 
                    w-56 p-3 rounded-lg 
                    bg-zinc-900 border border-white/10 shadow-xl
                    animate-tooltip-in pointer-events-none">
          <!-- Arrow -->
          <div class="absolute -bottom-1.5 left-1/2 -translate-x-1/2 
                      w-3 h-3 rotate-45 bg-zinc-900 border-r border-b border-white/10"></div>
          
          <!-- Content -->
          <div class="relative">
            <p class="text-xs font-semibold text-white mb-1">{tooltip.title}</p>
            <p class="text-xs text-zinc-400 leading-relaxed">{tooltip.description}</p>
            
            {#if disabledReason}
              <p class="text-xs text-amber-400 mt-2 flex items-center gap-1">
                <span>🔒</span> {disabledReason}
              </p>
            {:else if tooltip.sideEffects}
              <p class="text-xs text-amber-400 mt-2 leading-relaxed">{tooltip.sideEffects}</p>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  @keyframes loading-bar {
    0% {
      width: 0%;
      margin-left: 0%;
    }
    50% {
      width: 60%;
      margin-left: 20%;
    }
    100% {
      width: 0%;
      margin-left: 100%;
    }
  }
  
  .animate-loading-bar {
    animation: loading-bar 1.5s ease-in-out infinite;
  }
  
  @keyframes tooltip-in {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }
  
  .animate-tooltip-in {
    animation: tooltip-in 0.15s ease-out forwards;
  }
</style>
