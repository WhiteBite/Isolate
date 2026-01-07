/**
 * Centralized mock data exports
 * Used for browser preview and development
 */

// Services
export {
  mockDashboardServices,
  mockOnboardingServices,
  mockHealthServices,
  mockTestingServices,
  mockOrchestraServices,
  type MockService,
  type MockServiceItem,
  type MockServiceHealth,
  type MockServiceBasic,
  type MockServiceInfo,
} from './services';

// Proxies/Gateways
export {
  mockGateways,
} from './proxies';

// Network rules
export {
  mockNetworkRules,
} from './network';

// Diagnostics
export {
  mockDiagnosticsComponents,
  mockSystemInfo,
  mockDiagnosticsResults,
  mockConflicts,
  type ComponentStatus,
  type ConflictSeverity,
  type ConflictCategory,
  type MockSystemComponent,
  type MockSystemInfo,
  type MockConflictInfo,
} from './diagnostics';

// Plugins
export {
  mockMarketPlugins,
  type MockMarketPlugin,
} from './plugins';
