// Mock for $app/navigation
export const goto = async (url: string) => {};
export const invalidate = async (url: string) => {};
export const invalidateAll = async () => {};
export const preloadData = async (url: string) => ({ type: 'loaded' as const, status: 200, data: {} });
export const preloadCode = async (...urls: string[]) => {};
export const beforeNavigate = (callback: (navigation: any) => void) => {};
export const afterNavigate = (callback: (navigation: any) => void) => {};
export const onNavigate = (callback: (navigation: any) => void) => {};
export const disableScrollHandling = () => {};
export const pushState = (url: string, state: any) => {};
export const replaceState = (url: string, state: any) => {};
