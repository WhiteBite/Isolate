import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Strategy Composition Types
// ============================================================================

/**
 * A rule that maps a service to a specific strategy.
 */
export interface CompositionRule {
    id: string;
    service_id: string;
    service_name: string;
    strategy_id: string;
    strategy_name: string;
    priority: number;
    enabled: boolean;
}

/**
 * Configuration for strategy composition.
 */
export interface CompositionConfig {
    enabled: boolean;
    rules: CompositionRule[];
    fallback_strategy_id: string | null;
}

// ============================================================================
// Strategy Composition API Functions
// ============================================================================

/**
 * Get the current composition configuration.
 * @returns Current composition config with all rules
 */
export async function getCompositionConfig(): Promise<CompositionConfig> {
    return invoke('get_composition_config');
}

/**
 * Save the composition configuration.
 * @param config - Configuration to save
 */
export async function saveCompositionConfig(config: CompositionConfig): Promise<void> {
    return invoke('save_composition_config', { config });
}

/**
 * Add a new composition rule.
 * @param rule - Rule to add (without id, will be generated)
 * @returns The created rule with generated id
 */
export async function addCompositionRule(rule: Omit<CompositionRule, 'id'>): Promise<CompositionRule> {
    return invoke('add_composition_rule', { rule });
}

/**
 * Update an existing composition rule.
 * @param rule - Rule with updated values
 */
export async function updateCompositionRule(rule: CompositionRule): Promise<void> {
    return invoke('update_composition_rule', { rule });
}

/**
 * Delete a composition rule.
 * @param ruleId - ID of the rule to delete
 */
export async function deleteCompositionRule(ruleId: string): Promise<void> {
    return invoke('delete_composition_rule', { ruleId });
}

/**
 * Reorder composition rules by priority.
 * @param ruleIds - Array of rule IDs in the new order
 */
export async function reorderCompositionRules(ruleIds: string[]): Promise<void> {
    return invoke('reorder_composition_rules', { ruleIds });
}

/**
 * Toggle composition feature on/off.
 * @param enabled - Whether composition should be enabled
 */
export async function toggleComposition(enabled: boolean): Promise<void> {
    return invoke('toggle_composition', { enabled });
}

/**
 * Set the fallback strategy for services without specific rules.
 * @param strategyId - Strategy ID to use as fallback, or null for none
 */
export async function setFallbackStrategy(strategyId: string | null): Promise<void> {
    return invoke('set_fallback_strategy', { strategyId });
}
