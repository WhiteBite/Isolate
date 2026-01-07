import { invoke } from '@tauri-apps/api/core';
import type { DpiTestResult } from './types';

// ============================================================================
// DPI Simulator Testing API Functions
// ============================================================================

/**
 * Test a strategy against the DPI simulator.
 * 
 * This runs a full test cycle:
 * 1. Reset DPI stats
 * 2. Verify domain is blocked without strategy
 * 3. Apply strategy
 * 4. Verify domain is accessible with strategy
 * 5. Stop strategy
 * 
 * Requires DPI simulator VM to be running.
 * 
 * @param strategyId - ID of the strategy to test
 * @returns Test result with blocking statistics
 */
export async function testStrategyWithDpi(strategyId: string): Promise<DpiTestResult> {
    return invoke('test_strategy_with_dpi', { strategyId });
}
