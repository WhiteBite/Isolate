<script lang="ts">
  import type { OperationMode } from '$lib/stores/dashboard.svelte';

  interface Props {
    currentMode: OperationMode;
    onModeChange?: (mode: OperationMode) => void;
    disabled?: boolean;
  }

  let { currentMode, onModeChange, disabled = false }: Props = $props();

  const modes: { id: OperationMode; label: string; icon: string; description: string }[] = [
    { id: 'auto', label: 'Авто', icon: '✨', description: 'Автоматический выбор' },
    { id: 'tun', label: 'TUN', icon: '🔒', description: 'Весь трафик через VPN' },
    { id: 'proxy', label: 'Proxy', icon: '🌐', description: 'Только через прокси' }
  ];

  function selectMode(mode: OperationMode) {
    if (!disabled && mode !== currentMode) {
      onModeChange?.(mode);
    }
  }
</script>

<div class="flex flex-col gap-2">
  <div 
    class="relative flex bg-slate-800/50 rounded-xl p-1 border border-slate-700/50"
    class:opacity-50={disabled}
    class:pointer-events-none={disabled}
  >
    <!-- Active indicator background -->
    <div 
      class="absolute top-1 bottom-1 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-lg transition-all duration-300 ease-out shadow-lg shadow-blue-500/20"
      style="width: calc(33.333% - 4px); left: calc({modes.findIndex(m => m.id === currentMode)} * 33.333% + 4px);"
    ></div>

    {#each modes as mode}
      <button
        type="button"
        class="relative flex-1 flex items-center justify-center gap-2 py-3 px-4 rounded-lg text-sm font-medium transition-colors duration-200 z-10
          {currentMode === mode.id 
            ? 'text-white' 
            : 'text-slate-400 hover:text-slate-200'}"
        onclick={() => selectMode(mode.id)}
        {disabled}
      >
        <span class="text-base">{mode.icon}</span>
        <span>{mode.label}</span>
      </button>
    {/each}
  </div>

  <!-- Description -->
  <p class="text-xs text-slate-500 text-center">
    {modes.find(m => m.id === currentMode)?.description}
  </p>
</div>
