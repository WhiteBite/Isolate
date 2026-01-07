import { invoke } from '@tauri-apps/api/core';
import type { AppStatus, Strategy, Service, DiagnosticResult, ConflictInfo } from './types';

// ============================================================================
// Core API Functions
// ============================================================================

export async function getStatus(): Promise<AppStatus> {
    return invoke('get_status');
}

export async function getStrategies(): Promise<Strategy[]> {
    return invoke('get_strategies');
}

export async function getServices(): Promise<Service[]> {
    return invoke('get_services');
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

// ============================================================================
// Conflict Detection API
// ============================================================================

/**
 * Check for software conflicts with WinDivert/winws.
 * 
 * Detects running processes and services that may interfere with
 * Isolate's packet filtering functionality, such as:
 * - Network filtering software (AdGuard, Simplewall)
 * - VPN clients (OpenVPN, WireGuard, NordVPN, etc.)
 * - Network optimization services (Killer Network, SmartByte)
 * - Security software with network filtering
 * 
 * @returns Array of detected conflicts with severity and recommendations
 */
export async function checkConflicts(): Promise<ConflictInfo[]> {
    return invoke('check_conflicts');
}

// ============================================================================
// QUIC Blocking API
// ============================================================================

/**
 * Enable QUIC blocking via Windows Firewall.
 * Blocks UDP port 443 to force browsers to use TCP/TLS.
 * Requires administrator privileges.
 */
export async function enableQuicBlock(): Promise<void> {
    return invoke('enable_quic_block');
}

/**
 * Disable QUIC blocking.
 * Removes the firewall rule that blocks QUIC protocol.
 * Requires administrator privileges.
 */
export async function disableQuicBlock(): Promise<void> {
    return invoke('disable_quic_block');
}

/**
 * Check if QUIC is currently blocked.
 * @returns true if the QUIC blocking firewall rule exists
 */
export async function isQuicBlocked(): Promise<boolean> {
    return invoke('is_quic_blocked');
}

/**
 * Check if the application is running with administrator privileges.
 * @returns true if running as admin
 */
export async function isAdmin(): Promise<boolean> {
    return invoke('is_admin');
}
