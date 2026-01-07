/**
 * Layout state persistence store
 * Saves and restores UI layout preferences to localStorage
 */

import { browser } from '$app/environment';

const STORAGE_KEY = 'isolate-layout';
const WIDGET_ORDER_KEY = 'isolate:dashboard:widget-order';

export interface LayoutState {
  sidebar: {
    collapsed: boolean;
  };
  terminal: {
    isOpen: boolean;
    height: number;
  };
  navigation: {
    lastRoute: string;
  };
  dashboard: {
    widgetOrder: string[];
  };
}

const DEFAULT_WIDGET_ORDER = ['status', 'health', 'method', 'actions', 'network'];

const defaultState: LayoutState = {
  sidebar: {
    collapsed: false,
  },
  terminal: {
    isOpen: false,
    height: 200,
  },
  navigation: {
    lastRoute: '/',
  },
  dashboard: {
    widgetOrder: DEFAULT_WIDGET_ORDER,
  },
};

/**
 * Load layout state from localStorage
 */
export function loadLayoutState(): LayoutState {
  if (!browser) return defaultState;
  
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      // Merge with defaults to handle missing fields
      return {
        sidebar: { ...defaultState.sidebar, ...parsed.sidebar },
        terminal: { ...defaultState.terminal, ...parsed.terminal },
        navigation: { ...defaultState.navigation, ...parsed.navigation },
        dashboard: { ...defaultState.dashboard, ...parsed.dashboard },
      };
    }
  } catch (e) {
    console.warn('Failed to load layout state:', e);
  }
  
  // Try to migrate old widget order from separate key
  try {
    const oldWidgetOrder = localStorage.getItem(WIDGET_ORDER_KEY);
    if (oldWidgetOrder) {
      const order = JSON.parse(oldWidgetOrder) as string[];
      if (DEFAULT_WIDGET_ORDER.every(id => order.includes(id))) {
        return {
          ...defaultState,
          dashboard: { widgetOrder: order },
        };
      }
    }
  } catch (e) {
    // Ignore migration errors
  }
  
  return defaultState;
}

/**
 * Save layout state to localStorage
 */
export function saveLayoutState(state: Partial<LayoutState>): void {
  if (!browser) return;
  
  try {
    const current = loadLayoutState();
    const updated: LayoutState = {
      sidebar: { ...current.sidebar, ...state.sidebar },
      terminal: { ...current.terminal, ...state.terminal },
      navigation: { ...current.navigation, ...state.navigation },
      dashboard: { ...current.dashboard, ...state.dashboard },
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  } catch (e) {
    console.warn('Failed to save layout state:', e);
  }
}

/**
 * Save sidebar collapsed state
 */
export function saveSidebarState(collapsed: boolean): void {
  saveLayoutState({ sidebar: { collapsed } });
}

/**
 * Save terminal panel state
 */
export function saveTerminalState(isOpen: boolean, height: number): void {
  saveLayoutState({ terminal: { isOpen, height } });
}

/**
 * Save last visited route
 */
export function saveLastRoute(route: string): void {
  // Don't save onboarding or special routes
  if (route === '/onboarding' || route.startsWith('/_')) return;
  saveLayoutState({ navigation: { lastRoute: route } });
}

/**
 * Get last visited route
 */
export function getLastRoute(): string {
  if (!browser) return '/';
  const state = loadLayoutState();
  return state.navigation.lastRoute || '/';
}

/**
 * Save dashboard widget order
 */
export function saveWidgetOrder(order: string[]): void {
  if (!browser) return;
  
  // Validate that all required widgets are present
  if (!DEFAULT_WIDGET_ORDER.every(id => order.includes(id))) {
    console.warn('Invalid widget order, missing required widgets');
    return;
  }
  
  saveLayoutState({ dashboard: { widgetOrder: order } });
  
  // Also save to old key for backward compatibility
  try {
    localStorage.setItem(WIDGET_ORDER_KEY, JSON.stringify(order));
  } catch (e) {
    // Ignore
  }
}

/**
 * Get dashboard widget order
 */
export function getWidgetOrder(): string[] {
  if (!browser) return DEFAULT_WIDGET_ORDER;
  const state = loadLayoutState();
  return state.dashboard.widgetOrder || DEFAULT_WIDGET_ORDER;
}

/**
 * Check if this is the first app launch (no saved state)
 */
export function isFirstLaunch(): boolean {
  if (!browser) return true;
  return localStorage.getItem(STORAGE_KEY) === null;
}
