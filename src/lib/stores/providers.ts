/**
 * Provider store for ISP profile management
 * Stores selected provider and provides recommendations
 */

import { browser } from '$app/environment';
import type { ProviderSummary, ProviderRecommendations } from '$lib/api/types';

const STORAGE_KEY = 'isolate-provider';

/**
 * Get the initial provider from localStorage
 */
function getInitialProvider(): string | null {
  if (!browser) return null;
  return localStorage.getItem(STORAGE_KEY);
}

/**
 * Save provider to localStorage
 */
function saveProvider(providerId: string | null): void {
  if (!browser) return;
  if (providerId) {
    localStorage.setItem(STORAGE_KEY, providerId);
  } else {
    localStorage.removeItem(STORAGE_KEY);
  }
}

// Store state
let selectedProviderId: string | null = getInitialProvider();
let providers: ProviderSummary[] = [];
let recommendations: ProviderRecommendations | null = null;
let loading = false;

const subscribers = new Set<() => void>();

function notify() {
  subscribers.forEach(cb => cb());
}

/**
 * Provider store with reactive state
 */
export const providerStore = {
  subscribe(callback: () => void) {
    subscribers.add(callback);
    callback();
    return () => {
      subscribers.delete(callback);
    };
  },

  /**
   * Get current state
   */
  getState() {
    return {
      selectedProviderId,
      providers,
      recommendations,
      loading,
      selectedProvider: providers.find(p => p.id === selectedProviderId) || null
    };
  },

  /**
   * Set selected provider
   */
  setProvider(providerId: string | null) {
    selectedProviderId = providerId;
    saveProvider(providerId);
    recommendations = null;
    notify();
    
    // Load recommendations if provider selected
    if (providerId) {
      this.loadRecommendations(providerId);
    }
  },

  /**
   * Set providers list
   */
  setProviders(newProviders: ProviderSummary[]) {
    providers = newProviders;
    notify();
  },

  /**
   * Set recommendations
   */
  setRecommendations(newRecommendations: ProviderRecommendations | null) {
    recommendations = newRecommendations;
    notify();
  },

  /**
   * Set loading state
   */
  setLoading(isLoading: boolean) {
    loading = isLoading;
    notify();
  },

  /**
   * Load providers from backend
   */
  async loadProviders(): Promise<void> {
    if (!browser) return;
    
    this.setLoading(true);
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const loadedProviders = await invoke<ProviderSummary[]>('get_providers');
      this.setProviders(loadedProviders);
      
      // Load recommendations for selected provider
      if (selectedProviderId) {
        await this.loadRecommendations(selectedProviderId);
      }
    } catch (e) {
      console.error('Failed to load providers:', e);
    } finally {
      this.setLoading(false);
    }
  },

  /**
   * Load recommendations for a provider
   */
  async loadRecommendations(providerId: string): Promise<void> {
    if (!browser) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const recs = await invoke<ProviderRecommendations | null>('get_provider_recommendations', {
        providerId
      });
      this.setRecommendations(recs);
    } catch (e) {
      console.error('Failed to load recommendations:', e);
    }
  },

  /**
   * Check if a strategy is recommended for current provider
   */
  isStrategyRecommended(strategyId: string): boolean {
    if (!recommendations) return false;
    return recommendations.strategies.includes(strategyId);
  },

  /**
   * Get priority of strategy (lower is better, -1 if not recommended)
   */
  getStrategyPriority(strategyId: string): number {
    if (!recommendations) return -1;
    const index = recommendations.strategies.indexOf(strategyId);
    return index;
  },

  /**
   * Clear selected provider
   */
  clear() {
    selectedProviderId = null;
    recommendations = null;
    saveProvider(null);
    notify();
  }
};

export type ProviderStore = typeof providerStore;
