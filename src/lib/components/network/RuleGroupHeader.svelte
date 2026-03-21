<script lang="ts">
  interface Props {
    action: 'direct' | 'proxy' | 'block' | 'dpi-bypass';
    count: number;
    enabledCount: number;
    expanded: boolean;
    ontoggle: () => void;
  }

  let { action, count, enabledCount, expanded, ontoggle }: Props = $props();

  // Action configuration
  const actionConfig = {
    direct: {
      label: 'Direct',
      description: 'Bypass proxy, connect directly',
      icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />`,
      bgColor: 'bg-emerald-500/10',
      borderColor: 'border-emerald-500/20',
      textColor: 'text-emerald-400',
      dotColor: 'bg-emerald-400',
      hoverBg: 'hover:bg-emerald-500/15',
    },
    proxy: {
      label: 'Proxy',
      description: 'Route through proxy gateway',
      icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />`,
      bgColor: 'bg-indigo-500/10',
      borderColor: 'border-indigo-500/20',
      textColor: 'text-indigo-400',
      dotColor: 'bg-indigo-400',
      hoverBg: 'hover:bg-indigo-500/15',
    },
    block: {
      label: 'Block',
      description: 'Block all traffic',
      icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />`,
      bgColor: 'bg-red-500/10',
      borderColor: 'border-red-500/20',
      textColor: 'text-red-400',
      dotColor: 'bg-red-400',
      hoverBg: 'hover:bg-red-500/15',
    },
    'dpi-bypass': {
      label: 'DPI Bypass',
      description: 'Bypass DPI filtering',
      icon: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />`,
      bgColor: 'bg-amber-500/10',
      borderColor: 'border-amber-500/20',
      textColor: 'text-amber-400',
      dotColor: 'bg-amber-400',
      hoverBg: 'hover:bg-amber-500/15',
    },
  };

  let config = $derived(actionConfig[action]);
  let disabledCount = $derived(count - enabledCount);
</script>

<button
  type="button"
  onclick={ontoggle}
  class="w-full flex items-center gap-3 p-3 rounded-xl border transition-all duration-200 group
         {config.bgColor} {config.borderColor} {config.hoverBg}"
  aria-expanded={expanded}
  aria-controls={`rule-group-${action}`}
>
  <!-- Expand/Collapse chevron -->
  <div class="flex-shrink-0 transition-transform duration-200 {expanded ? 'rotate-90' : ''}">
    <svg class="w-4 h-4 {config.textColor}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
    </svg>
  </div>

  <!-- Icon -->
  <div class="flex-shrink-0 w-8 h-8 rounded-lg {config.bgColor} flex items-center justify-center">
    <svg class="w-4 h-4 {config.textColor}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      {@html config.icon}
    </svg>
  </div>

  <!-- Label and description -->
  <div class="flex-1 text-left">
    <div class="flex items-center gap-2">
      <span class="text-sm font-semibold {config.textColor}">{config.label}</span>
      <span class="w-1.5 h-1.5 rounded-full {config.dotColor}"></span>
    </div>
    <p class="text-xs text-zinc-500">{config.description}</p>
  </div>

  <!-- Count badges -->
  <div class="flex items-center gap-2">
    {#if enabledCount > 0}
      <span class="px-2 py-0.5 text-xs font-medium rounded-full {config.bgColor} {config.textColor} border {config.borderColor}">
        {enabledCount} active
      </span>
    {/if}
    {#if disabledCount > 0}
      <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-zinc-800/60 text-zinc-500 border border-zinc-700/50">
        {disabledCount} disabled
      </span>
    {/if}
    {#if count === 0}
      <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-zinc-800/60 text-zinc-500 border border-zinc-700/50">
        empty
      </span>
    {/if}
  </div>
</button>
