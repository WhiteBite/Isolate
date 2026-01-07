import { writable } from 'svelte/store';

// Types
export interface AppStatus {
    isActive: boolean;
    currentStrategy: string | null;
    currentStrategyName: string | null;
}

export interface OptimizationProgress {
    step: string;
    progress: number;
    message: string;
    isComplete: boolean;
    error: string | null;
}

export interface Service {
    id: string;
    name: string;
    icon: string;
    enabled: boolean;
    status: 'unknown' | 'working' | 'blocked';
    ping?: number;
}

// Stores
export const appStatus = writable<AppStatus>({
    isActive: false,
    currentStrategy: null,
    currentStrategyName: null
});

export const optimizationProgress = writable<OptimizationProgress>({
    step: '',
    progress: 0,
    message: '',
    isComplete: false,
    error: null
});

export const isOptimizing = writable(false);

export const services = writable<Service[]>([]);

export const settings = writable({
    autoStart: false,
    autoApply: false,
    minimizeToTray: true,
    blockQuic: true,
    defaultMode: 'turbo' as 'turbo' | 'deep'
});

// Derived stores - kept for potential future use
// export const hasActiveStrategy = derived(appStatus, $status => $status.isActive);
// export const optimizationError = derived(optimizationProgress, $progress => $progress.error);
