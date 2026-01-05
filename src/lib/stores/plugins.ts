import { writable } from 'svelte/store';

export interface PluginInfo {
  id: string;
  name: string;
  icon: string; // SVG path или emoji
  route?: string;
}

export const installedPlugins = writable<PluginInfo[]>([
  // Примеры для демо
  { id: 'discord-fix', name: 'Discord Fix', icon: '🎮', route: '/plugins/discord' },
  { id: 'speed-test', name: 'Speed Test', icon: '⚡', route: '/plugins/speed' },
]);
