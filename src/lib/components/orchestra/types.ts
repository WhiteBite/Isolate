// Orchestra component types

export interface Strategy {
  id: string;
  name: string;
  family: string;
  engine: string;
}

export interface QueueItem {
  id: string;
  name: string;
  type: 'strategy' | 'service';
  status: 'pending' | 'testing' | 'success' | 'failed' | 'skipped';
  score?: number;
  latency?: number;
}

export interface OrchestraState {
  status: 'idle' | 'running' | 'learning' | 'paused' | 'completed' | 'error';
  currentItem: string | null;
  progress: number;
  totalItems: number;
  testedItems: number;
  bestStrategy: string | null;
  bestScore: number;
  startTime: number | null;
  elapsedTime: number;
}

export interface ServiceInfo {
  id: string;
  name: string;
  icon: string;
}

export type OptimizationMode = 'turbo' | 'deep';

// Helper functions
export function getStatusBadgeClass(status: QueueItem['status']): string {
  switch (status) {
    case 'testing': return 'bg-amber-500/20 text-amber-400 border-amber-500/30';
    case 'success': return 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30';
    case 'failed': return 'bg-red-500/20 text-red-400 border-red-500/30';
    case 'skipped': return 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30';
    default: return 'bg-zinc-800/50 text-zinc-500 border-zinc-700/50';
  }
}

export function getStatusIcon(status: QueueItem['status']): string {
  switch (status) {
    case 'testing': return '⏳';
    case 'success': return '✓';
    case 'failed': return '✗';
    case 'skipped': return '⏭';
    default: return '○';
  }
}

export function getOrchestraStatusColor(status: OrchestraState['status']): string {
  switch (status) {
    case 'running': return 'text-emerald-400';
    case 'learning': return 'text-amber-400';
    case 'paused': return 'text-blue-400';
    case 'completed': return 'text-cyan-400';
    case 'error': return 'text-red-400';
    default: return 'text-zinc-500';
  }
}

export function getOrchestraStatusIcon(status: OrchestraState['status']): string {
  switch (status) {
    case 'running': return '🟢';
    case 'learning': return '🔄';
    case 'paused': return '⏸️';
    case 'completed': return '✅';
    case 'error': return '❌';
    default: return '⏹️';
  }
}

export function getOrchestraStatusText(status: OrchestraState['status']): string {
  switch (status) {
    case 'running': return 'Running';
    case 'learning': return 'Learning';
    case 'paused': return 'Paused';
    case 'completed': return 'Completed';
    case 'error': return 'Error';
    default: return 'Idle';
  }
}

export function formatElapsedTime(elapsedTime: number): string {
  const seconds = Math.floor(elapsedTime / 1000);
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

// Service icon mapping
export const serviceIcons: Record<string, string> = {
  youtube: '📺',
  discord: '💬',
  telegram: '✈️',
  twitch: '🎮',
  spotify: '🎵',
  instagram: '📷',
  twitter: '🐦',
  facebook: '📘',
  whatsapp: '💚',
  tiktok: '🎵',
};

export function getServiceIcon(id: string): string {
  return serviceIcons[id.toLowerCase()] || '🌐';
}
