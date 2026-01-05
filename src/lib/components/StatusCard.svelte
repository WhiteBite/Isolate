<script lang="ts">
  interface Props {
    icon?: string;
    title: string;
    status?: 'active' | 'inactive' | 'error' | 'testing';
    ping?: number | null;
    method?: string | null;
  }

  let { 
    icon = '🌐', 
    title, 
    status = 'inactive', 
    ping = null, 
    method = null 
  }: Props = $props();

  const statusConfig = {
    active: { color: 'bg-green-500', text: 'Активен', textColor: 'text-green-500' },
    inactive: { color: 'bg-gray-500', text: 'Неактивен', textColor: 'text-gray-400' },
    error: { color: 'bg-red-500', text: 'Ошибка', textColor: 'text-red-500' },
    testing: { color: 'bg-yellow-500', text: 'Проверка...', textColor: 'text-yellow-500' }
  };

  let currentStatus = $derived(statusConfig[status]);
</script>

<div 
  class="bg-surface border border-white/10 rounded-xl p-4 w-[180px] h-[120px] 
         flex flex-col justify-between
         hover:border-white/20 hover:shadow-[0_0_15px_rgba(255,255,255,0.05)] 
         transition-all duration-200 cursor-pointer"
>
  <div class="flex items-start gap-3">
    <span class="text-2xl">{icon}</span>
    <div class="flex-1 min-w-0">
      <h3 class="text-white font-medium text-sm truncate">{title}</h3>
      <div class="flex items-center gap-1.5 mt-1">
        <span class="w-2 h-2 rounded-full {currentStatus.color} {status === 'testing' ? 'animate-pulse' : ''}"></span>
        <span class="text-xs {currentStatus.textColor}">{currentStatus.text}</span>
      </div>
    </div>
  </div>

  <div class="flex flex-col gap-0.5">
    {#if ping !== null}
      <span class="text-xs text-gray-400">
        Пинг: <span class="text-white">{ping}ms</span>
      </span>
    {/if}
    {#if method}
      <span class="text-xs text-gray-400 truncate">
        {method}
      </span>
    {/if}
  </div>
</div>
