<script lang="ts">
  import { pluginSlots, type PluginSlots } from '$lib/stores/pluginSlots';
  
  interface Props {
    name: keyof PluginSlots;
    context?: Record<string, unknown>;
  }
  
  let { name, context }: Props = $props();
  
  let entries = $derived($pluginSlots[name]);
</script>

{#each entries as entry (entry.pluginId)}
  {@const Component = entry.component}
  <Component {...entry.props} {context} />
{/each}
