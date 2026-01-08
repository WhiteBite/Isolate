// Dashboard components
export { default as ShieldIndicator } from './ShieldIndicator.svelte';
export { default as ModeSelector } from './ModeSelector.svelte';
export { default as TrafficChart } from './TrafficChart.svelte';
export { default as ActiveConnectionItem } from './ActiveConnectionItem.svelte';
export { default as LiveActivityPanel } from './LiveActivityPanel.svelte';
export { default as BackendNotReady } from './BackendNotReady.svelte';
export { default as ActivityLog } from './ActivityLog.svelte';

// Re-export types from store
export type { 
  ProtectionStatus, 
  OperationMode, 
  Issue, 
  ActiveConnection, 
  TrafficPoint 
} from '$lib/stores/dashboard.svelte';
