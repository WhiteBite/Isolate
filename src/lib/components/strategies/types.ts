// Types for Strategy components

export type Category = 'all' | 'youtube' | 'discord' | 'telegram' | 'general' | 'games' | 'custom';
export type Label = 'recommended' | 'experimental' | 'stable' | null;
export type FamilyFilter = 'all' | 'zapret' | 'vless';

export interface Strategy {
  id: string;
  name: string;
  family: 'zapret' | 'vless' | 'custom';
  category: Category;
  description: string;
  services: string[];
  score: number | null;
  lastTested: Date | null;
  isActive: boolean;
  label: Label;
  author: string;
}

export interface CategoryInfo {
  id: Category;
  name: string;
  icon: string;
}

// Categories configuration
export const categories: CategoryInfo[] = [
  { id: 'all', name: 'All', icon: '📋' },
  { id: 'youtube', name: 'YouTube', icon: '📺' },
  { id: 'discord', name: 'Discord', icon: '💬' },
  { id: 'telegram', name: 'Telegram', icon: '✈️' },
  { id: 'general', name: 'General', icon: '🌐' },
  { id: 'games', name: 'Games', icon: '🎮' },
  { id: 'custom', name: 'Custom', icon: '⚙️' }
];

// Utility functions
export function getFamilyColor(family: string): string {
  switch (family) {
    case 'zapret': return '#00d4ff';
    case 'vless': return '#00ff88';
    case 'custom': return '#ffaa00';
    default: return '#a0a0a0';
  }
}

export function getCategoryColor(category: Category): string {
  switch (category) {
    case 'youtube': return '#ff0000';
    case 'discord': return '#5865f2';
    case 'telegram': return '#0088cc';
    case 'general': return '#00d4ff';
    case 'games': return '#9b59b6';
    case 'custom': return '#ffaa00';
    default: return '#a0a0a0';
  }
}

export function getLabelInfo(label: Label): { text: string; color: string; bg: string } {
  switch (label) {
    case 'recommended': return { text: 'Recommended', color: '#00ff88', bg: '#00ff88' };
    case 'experimental': return { text: 'Experimental', color: '#ffaa00', bg: '#ffaa00' };
    case 'stable': return { text: 'Stable', color: '#00d4ff', bg: '#00d4ff' };
    default: return { text: '', color: '', bg: '' };
  }
}

export function getScoreColor(score: number | null): string {
  if (score === null) return '#a0a0a0';
  if (score >= 80) return '#00ff88';
  if (score >= 50) return '#ffaa00';
  return '#ff3333';
}

export function formatDate(date: Date | null): string {
  if (!date) return 'Not tested';
  return date.toLocaleDateString('en-US', { 
    day: 'numeric', 
    month: 'short',
    hour: '2-digit',
    minute: '2-digit'
  });
}

export function mapServiceToCategory(services: string[]): Category {
  if (!services || services.length === 0) return 'general';
  const service = services[0].toLowerCase();
  if (service.includes('youtube')) return 'youtube';
  if (service.includes('discord')) return 'discord';
  if (service.includes('telegram')) return 'telegram';
  if (service.includes('steam') || service.includes('epic') || service.includes('riot') || service.includes('game')) return 'games';
  if (service.includes('custom')) return 'custom';
  return 'general';
}
