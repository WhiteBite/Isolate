<script lang="ts">
  import type { NavGroup } from '$lib/stores/navigation.svelte';
  import NavItem from './NavItem.svelte';

  interface Props {
    group: NavGroup;
    collapsed?: boolean;
    showDivider?: boolean;
  }

  let { group, collapsed = false, showDivider = true }: Props = $props();
</script>

<div class="nav-group" role="group" aria-label={group.label}>
  <!-- Group Label (hidden when collapsed) -->
  {#if !collapsed}
    <div class="px-3 py-1.5 text-[10px] font-semibold text-zinc-400 uppercase tracking-widest">
      {group.label}
    </div>
  {:else if showDivider}
    <!-- Divider when collapsed -->
    <div class="h-px bg-white/5 mx-2 my-2" aria-hidden="true"></div>
  {/if}
  
  <!-- Items -->
  <div class="space-y-0.5">
    {#each group.items as item (item.id)}
      <NavItem {item} {collapsed} />
    {/each}
  </div>
</div>
