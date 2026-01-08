import type { PluginInfo, PluginType } from '$lib/stores/plugins';

export interface MarketplacePlugin extends PluginInfo {
  downloads: number;
  rating: number;
  category: 'strategies' | 'services' | 'tools';
  featured?: boolean;
  updatedAt?: string;
}

export type SortOption = 'popular' | 'recent' | 'name' | 'rating' | 'author';
export type LevelFilter = 'all' | 1 | 2 | 3;
export type TypeFilter = 'all' | PluginType;

export const typeTabs: { id: TypeFilter; label: string; icon: string; color: string; description: string }[] = [
  { id: 'all', label: 'All', icon: '📦', color: 'indigo', description: 'All plugins' },
  { id: 'service-checker', label: 'Services', icon: '📡', color: 'emerald', description: 'Service availability checks' },
  { id: 'strategy-provider', label: 'Strategies', icon: '🎯', color: 'violet', description: 'DPI bypass configurations' },
  { id: 'hostlist-provider', label: 'Hostlists', icon: '📋', color: 'cyan', description: 'Domain lists' },
  { id: 'ui-widget', label: 'UI Widgets', icon: '🎨', color: 'pink', description: 'Visual components' },
  { id: 'script-plugin', label: 'Scripts', icon: '📜', color: 'amber', description: 'Lua scripts' }
];

export const levelFilters: { id: LevelFilter; label: string; shortLabel: string; description: string; color: string }[] = [
  { id: 'all', label: 'All levels', shortLabel: 'All', description: '', color: 'zinc' },
  { id: 1, label: 'L1 — Declarative', shortLabel: 'L1', description: 'JSON/YAML configs', color: 'emerald' },
  { id: 2, label: 'L2 — UI plugins', shortLabel: 'L2', description: 'Svelte components', color: 'indigo' },
  { id: 3, label: 'L3 — Scripts', shortLabel: 'L3', description: 'Lua scripts', color: 'amber' }
];

export const sortOptions: { id: SortOption; label: string; icon: string }[] = [
  { id: 'popular', label: 'Popular', icon: '🔥' },
  { id: 'rating', label: 'By Rating', icon: '⭐' },
  { id: 'recent', label: 'Recent', icon: '🆕' },
  { id: 'name', label: 'By Name', icon: '🔤' },
  { id: 'author', label: 'By Author', icon: '👤' }
];

export function formatDownloads(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`;
  if (count >= 1000) return `${(count / 1000).toFixed(1)}K`;
  return count.toString();
}

export function renderStars(rating: number): { full: number; half: boolean; empty: number } {
  const full = Math.floor(rating);
  const half = rating % 1 >= 0.5;
  const empty = 5 - full - (half ? 1 : 0);
  return { full, half, empty };
}

export function getTabColorClasses(color: string, isActive: boolean): string {
  if (isActive) {
    const activeColors: Record<string, string> = {
      indigo: 'bg-indigo-500 text-white shadow-lg shadow-indigo-500/25',
      emerald: 'bg-emerald-500 text-white shadow-lg shadow-emerald-500/25',
      violet: 'bg-violet-500 text-white shadow-lg shadow-violet-500/25',
      cyan: 'bg-cyan-500 text-white shadow-lg shadow-cyan-500/25',
      pink: 'bg-pink-500 text-white shadow-lg shadow-pink-500/25',
      amber: 'bg-amber-500 text-white shadow-lg shadow-amber-500/25'
    };
    return activeColors[color] || activeColors.indigo;
  }
  return 'bg-void-100 text-text-secondary hover:text-text-primary hover:bg-void-200 border border-glass-border';
}

export function getLevelBadgeClasses(level: LevelFilter, isActive: boolean): string {
  if (!isActive) return 'bg-void-100 text-text-secondary border-glass-border';
  
  const colors: Record<number, string> = {
    1: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
    2: 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30',
    3: 'bg-amber-500/20 text-amber-400 border-amber-500/30'
  };
  return colors[level as number] || 'bg-void-200 text-text-primary border-glass-border';
}
