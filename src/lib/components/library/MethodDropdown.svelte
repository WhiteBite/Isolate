<script lang="ts">
  import type { AccessMethod, AccessMethodType } from '$lib/stores/library.svelte';

  interface Props {
    currentMethod: AccessMethod;
    availableMethods: AccessMethod[];
    onSelect: (method: AccessMethod) => void;
    disabled?: boolean;
  }

  let { currentMethod, availableMethods, onSelect, disabled = false }: Props = $props();

  let isOpen = $state(false);
  let dropdownRef = $state<HTMLDivElement | null>(null);

  const methodIcons: Record<AccessMethodType, string> = {
    direct: '🔗',
    auto: '✨',
    strategy: '🛡️',
    vless: '🔒',
    proxy: '🌐',
    tor: '🧅',
    block: '🚫'
  };

  const methodLabels: Record<AccessMethodType, string> = {
    direct: 'Напрямую',
    auto: 'Авто',
    strategy: 'Стратегия',
    vless: 'VLESS',
    proxy: 'Прокси',
    tor: 'Tor',
    block: 'Заблокировать'
  };

  function getMethodLabel(method: AccessMethod): string {
    if (method.type === 'strategy' && method.strategyName) {
      return method.strategyName;
    }
    if (method.type === 'proxy' && method.proxyName) {
      return method.proxyName;
    }
    return methodLabels[method.type];
  }

  function handleSelect(method: AccessMethod) {
    onSelect(method);
    isOpen = false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      isOpen = false;
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
      isOpen = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      document.addEventListener('click', handleClickOutside);
      document.addEventListener('keydown', handleKeydown);
    }
    return () => {
      document.removeEventListener('click', handleClickOutside);
      document.removeEventListener('keydown', handleKeydown);
    };
  });

  // Group methods by type
  const groupedMethods = $derived(() => {
    const groups: Record<string, AccessMethod[]> = {
      basic: [],
      strategies: [],
      proxies: []
    };

    for (const method of availableMethods) {
      if (method.type === 'direct' || method.type === 'auto') {
        groups.basic.push(method);
      } else if (method.type === 'strategy') {
        groups.strategies.push(method);
      } else {
        groups.proxies.push(method);
      }
    }

    return groups;
  });
</script>

<div class="relative" bind:this={dropdownRef}>
  <button
    type="button"
    class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium
           bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 hover:border-zinc-600
           transition-colors duration-150 disabled:opacity-50 disabled:cursor-not-allowed"
    onclick={() => !disabled && (isOpen = !isOpen)}
    aria-haspopup="listbox"
    aria-expanded={isOpen}
    {disabled}
  >
    <span>{methodIcons[currentMethod.type]}</span>
    <span class="text-zinc-200">{getMethodLabel(currentMethod)}</span>
    <svg 
      class="w-4 h-4 text-zinc-400 transition-transform duration-150 {isOpen ? 'rotate-180' : ''}" 
      fill="none" 
      viewBox="0 0 24 24" 
      stroke="currentColor"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if isOpen}
    <div 
      class="absolute right-0 mt-1 w-56 py-1 bg-zinc-800 border border-zinc-700 rounded-lg shadow-xl z-50"
      role="listbox"
      aria-label="Выбор метода доступа"
    >
      {#if groupedMethods().basic.length > 0}
        <div class="px-3 py-1.5 text-xs font-medium text-zinc-400 uppercase tracking-wider">
          Базовые
        </div>
        {#each groupedMethods().basic as method}
          <button
            type="button"
            class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left
                   hover:bg-zinc-700 transition-colors duration-150
                   {currentMethod.type === method.type ? 'bg-zinc-700/50 text-white' : 'text-zinc-300'}"
            onclick={() => handleSelect(method)}
            role="option"
            aria-selected={currentMethod.type === method.type}
          >
            <span>{methodIcons[method.type]}</span>
            <span>{getMethodLabel(method)}</span>
            {#if currentMethod.type === method.type}
              <svg class="w-4 h-4 ml-auto text-emerald-400" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            {/if}
          </button>
        {/each}
      {/if}

      {#if groupedMethods().strategies.length > 0}
        <div class="border-t border-zinc-700 mt-1 pt-1">
          <div class="px-3 py-1.5 text-xs font-medium text-zinc-400 uppercase tracking-wider">
            Стратегии
          </div>
          {#each groupedMethods().strategies as method}
            <button
              type="button"
              class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left
                     hover:bg-zinc-700 transition-colors duration-150
                     {currentMethod.type === method.type && currentMethod.strategyId === method.strategyId 
                       ? 'bg-zinc-700/50 text-white' : 'text-zinc-300'}"
              onclick={() => handleSelect(method)}
              role="option"
              aria-selected={currentMethod.type === method.type && currentMethod.strategyId === method.strategyId}
            >
              <span>{methodIcons[method.type]}</span>
              <span>{getMethodLabel(method)}</span>
              {#if currentMethod.type === method.type && currentMethod.strategyId === method.strategyId}
                <svg class="w-4 h-4 ml-auto text-emerald-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
              {/if}
            </button>
          {/each}
        </div>
      {/if}

      {#if groupedMethods().proxies.length > 0}
        <div class="border-t border-zinc-700 mt-1 pt-1">
          <div class="px-3 py-1.5 text-xs font-medium text-zinc-400 uppercase tracking-wider">
            Прокси
          </div>
          {#each groupedMethods().proxies as method}
            <button
              type="button"
              class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left
                     hover:bg-zinc-700 transition-colors duration-150
                     {currentMethod.type === method.type ? 'bg-zinc-700/50 text-white' : 'text-zinc-300'}"
              onclick={() => handleSelect(method)}
              role="option"
              aria-selected={currentMethod.type === method.type}
            >
              <span>{methodIcons[method.type]}</span>
              <span>{getMethodLabel(method)}</span>
              {#if currentMethod.type === method.type}
                <svg class="w-4 h-4 ml-auto text-emerald-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
