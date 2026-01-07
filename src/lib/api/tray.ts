import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { TrayState } from './types';

// ============================================================================
// Tray API Functions
// ============================================================================

/**
 * Update tray status and icon.
 * @param state - New tray state
 * @param strategyName - Optional strategy name to display
 */
export async function updateTray(state: TrayState, strategyName?: string): Promise<void> {
    return invoke('update_tray', { state, strategyName });
}

/**
 * Set tray to optimizing state.
 */
export async function setTrayOptimizing(): Promise<void> {
    return invoke('set_tray_optimizing');
}

/**
 * Set tray to error state.
 * @param errorMsg - Error message to display in tooltip
 */
export async function setTrayError(errorMsg: string): Promise<void> {
    return invoke('set_tray_error', { errorMsg });
}

/**
 * Get current tray state.
 * @returns Current tray state
 */
export async function getTrayState(): Promise<TrayState> {
    return invoke('get_tray_state');
}

// ============================================================================
// Tray Event Listeners
// ============================================================================

/**
 * Subscribe to tray optimize events.
 * @param callback - Function to call when optimize is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayOptimize(callback: (mode: 'turbo' | 'deep') => void): Promise<UnlistenFn> {
    return listen('tray:optimize', (event) => {
        callback(event.payload as 'turbo' | 'deep');
    });
}

/**
 * Subscribe to tray toggle events.
 * @param callback - Function to call when toggle is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayToggle(callback: () => void): Promise<UnlistenFn> {
    return listen('tray:toggle', () => {
        callback();
    });
}

/**
 * Subscribe to tray stop events.
 * @param callback - Function to call when stop is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayStop(callback: () => void): Promise<UnlistenFn> {
    return listen('tray:stop', () => {
        callback();
    });
}

/**
 * Subscribe to tray panic reset events.
 * @param callback - Function to call when panic reset is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayPanicReset(callback: () => void): Promise<UnlistenFn> {
    return listen('tray:panic_reset', () => {
        callback();
    });
}

/**
 * Subscribe to tray navigation events.
 * @param callback - Function to call with the route to navigate to
 * @returns Unsubscribe function
 */
export function onTrayNavigate(callback: (route: string) => void): Promise<UnlistenFn> {
    return listen('tray:navigate', (event) => {
        callback(event.payload as string);
    });
}

/**
 * Subscribe to tray quit events.
 * @param callback - Function to call when quit is triggered from tray
 * @returns Unsubscribe function
 */
export function onTrayQuit(callback: () => void): Promise<UnlistenFn> {
    return listen('tray:quit', () => {
        callback();
    });
}

/**
 * Subscribe to tray QUIC block events.
 * @param callback - Function to call with block state (true = block, false = unblock)
 * @returns Unsubscribe function
 */
export function onTrayQuicBlock(callback: (block: boolean) => void): Promise<UnlistenFn> {
    return listen('tray:quic_block', (event) => {
        callback(event.payload as boolean);
    });
}
