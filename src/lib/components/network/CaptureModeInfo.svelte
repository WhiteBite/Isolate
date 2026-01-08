<script lang="ts">
  interface Props {
    mode: 'system' | 'tun';
    expanded?: boolean;
  }

  let { mode, expanded = false }: Props = $props();
  
  let isExpanded = $state(expanded);
</script>

<div class="mt-2">
  <!-- Toggle button -->
  <button
    type="button"
    onclick={() => isExpanded = !isExpanded}
    class="flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-400 transition-colors"
  >
    <svg 
      class="w-3.5 h-3.5 transition-transform duration-200 {isExpanded ? 'rotate-180' : ''}" 
      fill="none" 
      viewBox="0 0 24 24" 
      stroke="currentColor"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
    <span>Что это значит?</span>
  </button>

  <!-- Info panel -->
  {#if isExpanded}
    <div class="mt-2 p-3 bg-zinc-900/50 rounded-lg border border-white/5 text-xs space-y-3 animate-in fade-in slide-in-from-top-1 duration-200">
      <!-- TUN Mode -->
      <div class="space-y-1.5">
        <div class="flex items-center gap-2">
          <span class="text-sm">🛡️</span>
          <span class="font-medium text-zinc-300">TUN Driver</span>
          {#if mode === 'tun'}
            <span class="px-1.5 py-0.5 bg-emerald-500/20 text-emerald-400 rounded text-[10px] font-medium">Выбран</span>
          {/if}
        </div>
        <p class="text-zinc-500 leading-relaxed pl-6">
          Перехватывает <span class="text-zinc-400">весь сетевой трафик</span> на уровне драйвера. 
          Работает для всех приложений, включая игры, торренты и программы без поддержки прокси.
        </p>
        <div class="pl-6 flex flex-wrap gap-1.5">
          <span class="px-1.5 py-0.5 bg-emerald-500/10 text-emerald-400/80 rounded text-[10px]">✓ Все приложения</span>
          <span class="px-1.5 py-0.5 bg-emerald-500/10 text-emerald-400/80 rounded text-[10px]">✓ Игры</span>
          <span class="px-1.5 py-0.5 bg-emerald-500/10 text-emerald-400/80 rounded text-[10px]">✓ UDP трафик</span>
        </div>
      </div>

      <!-- System Proxy Mode -->
      <div class="space-y-1.5">
        <div class="flex items-center gap-2">
          <span class="text-sm">🌍</span>
          <span class="font-medium text-zinc-300">System Proxy</span>
          {#if mode === 'system'}
            <span class="px-1.5 py-0.5 bg-emerald-500/20 text-emerald-400 rounded text-[10px] font-medium">Выбран</span>
          {/if}
        </div>
        <p class="text-zinc-500 leading-relaxed pl-6">
          Использует <span class="text-zinc-400">системные настройки прокси</span>. 
          Работает только для приложений, которые поддерживают системный прокси (браузеры, мессенджеры).
        </p>
        <div class="pl-6 flex flex-wrap gap-1.5">
          <span class="px-1.5 py-0.5 bg-blue-500/10 text-blue-400/80 rounded text-[10px]">✓ Браузеры</span>
          <span class="px-1.5 py-0.5 bg-blue-500/10 text-blue-400/80 rounded text-[10px]">✓ Легче для системы</span>
          <span class="px-1.5 py-0.5 bg-amber-500/10 text-amber-400/80 rounded text-[10px]">⚠ Не все приложения</span>
        </div>
      </div>

      <!-- Recommendation -->
      <div class="pt-2 border-t border-white/5">
        <p class="text-zinc-500 leading-relaxed">
          <span class="text-zinc-400 font-medium">💡 Рекомендация:</span> 
          Используйте <span class="text-zinc-300">TUN</span> для полного покрытия, 
          <span class="text-zinc-300">System Proxy</span> — если нужен только браузер или возникают проблемы с TUN.
        </p>
      </div>

      <!-- Warning -->
      <div class="p-2 bg-amber-500/5 border border-amber-500/10 rounded">
        <p class="text-amber-400/80 leading-relaxed">
          <span class="font-medium">⚠️ Внимание:</span> 
          TUN режим требует прав администратора и может конфликтовать с VPN или антивирусами.
        </p>
      </div>
    </div>
  {/if}
</div>
