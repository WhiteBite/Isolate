import { invoke } from '@tauri-apps/api/core';
import type { DomainRoute, AppRoute, InstalledApp, RoutingRule } from './types';

// ============================================================================
// Domain Routing API Functions
// ============================================================================

/**
 * Get all domain routing rules.
 * @returns Array of domain routes
 */
export async function getDomainRoutes(): Promise<DomainRoute[]> {
    return invoke('get_domain_routes');
}

/**
 * Add a domain routing rule.
 * @param domain - Domain pattern
 * @param proxyId - Proxy ID to route through
 */
export async function addDomainRoute(domain: string, proxyId: string): Promise<void> {
    return invoke('add_domain_route', { domain, proxyId });
}

/**
 * Remove a domain routing rule.
 * @param domain - Domain pattern to remove
 */
export async function removeDomainRoute(domain: string): Promise<void> {
    return invoke('remove_domain_route', { domain });
}

// ============================================================================
// Application Routing API Functions
// ============================================================================

/**
 * Get all application routing rules.
 * @returns Array of app routes
 */
export async function getAppRoutes(): Promise<AppRoute[]> {
    return invoke('get_app_routes');
}

/**
 * Add an application routing rule.
 * @param appName - Application display name
 * @param appPath - Full path to executable
 * @param proxyId - Proxy ID to route through
 */
export async function addAppRoute(appName: string, appPath: string, proxyId: string): Promise<void> {
    return invoke('add_app_route', { appName, appPath, proxyId });
}

/**
 * Remove an application routing rule.
 * @param appPath - Application path to remove
 */
export async function removeAppRoute(appPath: string): Promise<void> {
    return invoke('remove_app_route', { appPath });
}

/**
 * Get list of installed applications.
 * @returns Array of installed apps with names, paths, and icons
 */
export async function getInstalledApps(): Promise<InstalledApp[]> {
    return invoke('get_installed_apps');
}

// ============================================================================
// High-Level Routing Rules API
// ============================================================================

/**
 * Get all routing rules ordered by priority.
 * @returns Array of routing rules
 */
export async function getRoutingRules(): Promise<RoutingRule[]> {
    return invoke('get_routing_rules');
}

/**
 * Add a new routing rule.
 * @param rule - Routing rule to add
 * @returns The created routing rule
 */
export async function addRoutingRule(rule: RoutingRule): Promise<RoutingRule> {
    return invoke('add_routing_rule', { rule });
}

/**
 * Update an existing routing rule.
 * @param rule - Routing rule with updated values
 */
export async function updateRoutingRule(rule: RoutingRule): Promise<void> {
    return invoke('update_routing_rule', { rule });
}

/**
 * Delete a routing rule.
 * @param ruleId - ID of the rule to delete
 */
export async function deleteRoutingRule(ruleId: string): Promise<void> {
    return invoke('delete_routing_rule', { ruleId });
}

/**
 * Reorder routing rules.
 * @param ruleIds - Array of rule IDs in new order
 */
export async function reorderRoutingRules(ruleIds: string[]): Promise<void> {
    return invoke('reorder_routing_rules', { ruleIds });
}

/**
 * Toggle routing rule enabled state.
 * @param ruleId - ID of the rule to toggle
 * @param enabled - New enabled state
 */
export async function toggleRoutingRule(ruleId: string, enabled: boolean): Promise<void> {
    return invoke('toggle_routing_rule', { ruleId, enabled });
}
