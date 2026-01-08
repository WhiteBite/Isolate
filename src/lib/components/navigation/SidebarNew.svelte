<script lang="ts">
  import { page } from '$app/stores';
  import { navigationStore } from '$lib/stores/navigation.svelte';
  import NavGroup from './NavGroup.svelte';
  import LogsButton from './LogsButton.svelte';

  interface Props {
    collapsed?: boolean;
    onToggle?: (collapsed: boolean) => void;
  }

  let { collapsed = $bindable(false), onToggle }: Props = $props();

  // Sync collapsed state with navigation store
  $effect(() => {
    navigationStore.collapsed = collapsed;
  });

  // Update active route
  $effect(() => {
    navigationStore.setActive($page.url.pathname);
  });

  function toggleSidebar() {
    collapsed = !collapsed;
    onToggle?.(collapsed);
  }
</script>

<aside
  class="h-full flex flex-col backdrop-blur-xl bg-black/20 transition-all duration-200 ease-out"
  style="width: {collapsed ? '60px' : '200px'}"
  aria-label="Main navigation"
>
  <!-- Logo -->
  <div class="flex items-center h-14 px-3">
    <div class="flex items-center gap-3 overflow-hidden">
      <div class="w-8 h-8 flex-shrink-0 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-glow-indigo">
        <svg class="w-4 h-4 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
        </svg>
      </div>
      {#if !collapsed}
        <span class="font-semibold text-zinc-100 tracking-tight whitespace-nowrap">Isolate</span>
      {/if}
    </div>
  </div>

  <!-- Navigation Groups -->
  <nav class="flex-1 flex flex-col py-3 overflow-hidden" aria-label="Primary">
    <div class="px-2 space-y-3">
      {#each navigationStore.groups as group, index (group.id)}
        <NavGroup 
          {group} 
          {collapsed} 
          showDivider={index > 0}
        />
      {/each}
    </div>

    <!-- Spacer -->
    <div class="flex-1"></div>

    <!-- Logs Button -->
    <div class="px-2 pt-2 border-t border-white/5 mt-2">
      <LogsButton {collapsed} />
    </div>
  </nav>

  <!-- Collapse Toggle -->
  <div class="p-2">
    <button
      onclick={toggleSidebar}
      class="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-zinc-400 hover:bg-white/5 hover:text-zinc-300 transition-all duration-150"
      aria-expanded={!collapsed}
      aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
    >
      <span 
        class="w-5 h-5 flex-shrink-0 transition-transform duration-200" 
        style="transform: rotate({collapsed ? '180deg' : '0deg'})" 
        aria-hidden="true"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M11 17l-5-5 5-5"/>
          <path d="M18 17l-5-5 5-5"/>
        </svg>
      </span>
      {#if !collapsed}
        <span class="text-sm">Collapse</span>
      {/if}
    </button>
  </div>
</aside>
