import { writable } from 'svelte/store';
import type { ComponentType } from 'svelte';

interface PluginSlotEntry {
  pluginId: string;
  component: ComponentType;
  props?: Record<string, unknown>;
  order?: number;
}

interface PluginSlots {
  sidebar: PluginSlotEntry[];
  dashboard: PluginSlotEntry[];
  settings: PluginSlotEntry[];
  contextMenu: PluginSlotEntry[];
}

function createPluginSlotsStore() {
  const { subscribe, update } = writable<PluginSlots>({
    sidebar: [],
    dashboard: [],
    settings: [],
    contextMenu: [],
  });

  return {
    subscribe,
    register: (slotName: keyof PluginSlots, entry: PluginSlotEntry) => {
      update(slots => ({
        ...slots,
        [slotName]: [...slots[slotName], entry].sort((a, b) => (a.order ?? 0) - (b.order ?? 0))
      }));
    },
    unregister: (slotName: keyof PluginSlots, pluginId: string) => {
      update(slots => ({
        ...slots,
        [slotName]: slots[slotName].filter(e => e.pluginId !== pluginId)
      }));
    },
  };
}

export const pluginSlots = createPluginSlotsStore();
export type { PluginSlotEntry, PluginSlots };
