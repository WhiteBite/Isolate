/**
 * Utility functions exports
 */

// Backend utilities
export { 
  waitForBackend, 
  invokeWithBackendCheck, 
  isTauriEnv 
} from './backend';

// Service icons
export { 
  serviceIconsSvg, 
  serviceIconsEmoji, 
  getServiceIconSvg, 
  getServiceIconEmoji,
  type ServiceIcon 
} from './icons';

// Country flags
export { 
  countryFlags, 
  countryNames,
  getCountryFlag,
  getCountryName,
  detectCountryFromServer,
  getProxyFlag,
  getProxyCountryName
} from './countries';

// Lazy loading
export {
  createLazyLoader,
  lazyComponents,
  preloadAllLazyComponents,
  type LazyComponentState
} from './lazyComponent';

// Proxy testing
export {
  ProxyTester,
  proxyTester,
  sortByTestResults,
  getLatencyColor,
  formatLatency,
  type ProxyTestStatus,
  type ProxyTestState,
  type ProxyTesterProgress,
  type ProxyTesterOptions
} from './proxyTester';
