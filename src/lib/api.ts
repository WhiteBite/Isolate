import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// Types
export interface AppStatus {
    is_active: boolean;
    current_strategy: string | null;
    services_status: Record<string, ServiceStatus>;
}

export interface ServiceStatus {
    name: string;
    is_available: boolean;
    latency_ms: number | null;
}

export interface Strategy {
    id: string;
    name: string;
    description: string;
    family: string;
    engine: string;
}

export interface Service {
    id: string;
    name: string;
    critical: boolean;
}

export interface OptimizationProgress {
    stage: string;
    percent: number;
    message: string;
    current_strategy: string | null;
    tested_count: number;
    total_count: number;
    best_score: number | null;
}

export interface DiagnosticResult {
    profile: {
        kind: string;
        details: string | null;
        candidate_families: string[];
    };
    tested_services: string[];
    blocked_services: string[];
}

// API Functions
export async function getStatus(): Promise<AppStatus> {
    return invoke('get_status');
}

export async function getStrategies(): Promise<Strategy[]> {
    return invoke('get_strategies');
}

export async function getServices(): Promise<Service[]> {
    return invoke('get_services');
}

export async function runOptimization(mode: 'turbo' | 'deep'): Promise<string> {
    return invoke('run_optimization', { mode });
}

export async function cancelOptimization(): Promise<void> {
    return invoke('cancel_optimization');
}

export async function applyStrategy(strategyId: string): Promise<void> {
    return invoke('apply_strategy', { strategyId });
}

export async function stopStrategy(): Promise<void> {
    return invoke('stop_strategy');
}

export async function diagnose(): Promise<DiagnosticResult> {
    return invoke('diagnose');
}

export async function panicReset(): Promise<void> {
    return invoke('panic_reset');
}

// Event Listeners
export function onOptimizationProgress(
    callback: (progress: OptimizationProgress) => void
): Promise<UnlistenFn> {
    return listen('optimization:progress', (event) => {
        callback(event.payload as OptimizationProgress);
    });
}

export function onOptimizationComplete(
    callback: (result: { strategy_id: string; score: number }) => void
): Promise<UnlistenFn> {
    return listen('optimization:complete', (event) => {
        callback(event.payload as { strategy_id: string; score: number });
    });
}

export function onOptimizationFailed(
    callback: (error: string) => void
): Promise<UnlistenFn> {
    return listen('optimization:failed', (event) => {
        callback(event.payload as string);
    });
}

export function onStrategyDegraded(
    callback: () => void
): Promise<UnlistenFn> {
    return listen('strategy:degraded', () => {
        callback();
    });
}
