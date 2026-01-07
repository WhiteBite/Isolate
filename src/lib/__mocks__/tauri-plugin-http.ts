// Mock for @tauri-apps/plugin-http
export const fetch = async (url: string, options?: any) => {
  return {
    ok: true,
    status: 200,
    statusText: 'OK',
    text: async () => '',
    json: async () => ({})
  };
};
