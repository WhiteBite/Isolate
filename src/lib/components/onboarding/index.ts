// Onboarding components
export { default as StepIndicator } from './StepIndicator.svelte';
export { default as ServiceSelector } from './ServiceSelector.svelte';
export { default as MethodSelector } from './MethodSelector.svelte';
export { default as SetupStep } from './SetupStep.svelte';
export { default as WelcomeStep } from './WelcomeStep.svelte';
export { default as OnboardingNavigation } from './OnboardingNavigation.svelte';
export { default as ProviderStep } from './ProviderStep.svelte';

// Types
export type Step = 1 | 2 | 3 | 4 | 5;
export type ConnectionMode = 'auto' | 'proxy';

export interface ServiceItem {
  id: string;
  name: string;
  icon: string;
  description: string;
}

export interface SetupTask {
  id: string;
  label: string;
  status: 'pending' | 'running' | 'done' | 'error';
  error?: string;
}

export interface DownloadProgress {
  name: string;
  percent: number;
}
