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
  }

  let { 
    actions = [
      { id: 'scan', label: 'Scan All' },
      { id: 'test', label: 'Test Current' },
      { id: 'proxy', label: 'Add Proxy' },
      { id: 'settings', label: 'Settings' }
    ],
    onAction
  }: Props = $props();

  function handleClick(action: QuickAction) {
    if (action.disabled || action.loading) return;
    action.onClick?.();
    onAction?.(action.id);
  }

  // SVG icons for actions
  const actionIcons: Record<string, string> = {
    scan: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z',
    test: 'M13 10V3L4 14h7v7l9-11h-7z',
    proxy: 'M12 4v16m8-8H4',
    settings: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z'
  };

  const actionColors: Record<string, string> = {
    scan: 'text-zinc-400 group-hover:text-zinc-200',
    test: 'text-zinc-400 group-hover:text-zinc-200',
    proxy: 'text-zinc-400 group-hover:text-zinc-200',
    settings: 'text-zinc-400 group-hover:text-zinc-200'
  };
</script>

<div class="grid grid-cols-2 gap-2 h-full">
  {#each actions as action}
    <button
      class="group
        flex flex-col items-center justify-center gap-2 p-3
        rounded-lg bg-zinc-900/40 border border-white/5
        transition-all duration-200
        hover:bg-zinc-800/60 hover:border-white/10 hover:-translate-y-0.5
        active:scale-95
        disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:scale-100 disabled:hover:translate-y-0
        {action.loading ? 'animate-pulse' : ''}
      "
      disabled={action.disabled || action.loading}
      onclick={() => handleClick(action)}
    >
      <div class="w-8 h-8 rounded-lg bg-zinc-800/50 border border-white/5 flex items-center justify-center
                  group-hover:bg-zinc-700/50 group-hover:border-white/10 transition-all duration-200">
        {#if action.loading}
          <svg class="w-4 h-4 text-zinc-400 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        {:else}
          <svg class="w-4 h-4 text-zinc-400 group-hover:text-zinc-200 transition-colors duration-200" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d={actionIcons[action.id] || 'M13 10V3L4 14h7v7l9-11h-7z'}/>
          </svg>
        {/if}
      </div>
      <span class="text-xs text-zinc-400 font-medium group-hover:text-zinc-200 transition-colors duration-200">
        {action.label}
      </span>
    </button>
  {/each}
</div>
