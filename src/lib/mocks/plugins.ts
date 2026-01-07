/**
 * Mock data for plugins marketplace
 * Used for browser preview and development
 */

export interface MockMarketPlugin {
  id: string;
  name: string;
  desc: string;
  author: string;
  ver: string;
  icon: string;
  dl: number;
  inst: boolean;
  cat: string;
}

/**
 * Plugins marketplace mock
 */
export const mockMarketPlugins: MockMarketPlugin[] = [
  { id: 'discord', name: 'Discord Fix', desc: 'Голосовые и видео звонки', author: 'Isolate', ver: '1.2.0', icon: '🎮', dl: 24580, inst: true, cat: 'social' },
  { id: 'youtube', name: 'YouTube', desc: 'Видео стриминг', author: 'Isolate', ver: '2.0.1', icon: '📺', dl: 18920, inst: false, cat: 'media' },
  { id: 'telegram', name: 'Telegram', desc: 'Обход блокировок', author: 'Community', ver: '1.4.0', icon: '✈️', dl: 21340, inst: false, cat: 'social' },
  { id: 'speedtest', name: 'Speed Test', desc: 'Тест скорости', author: 'Isolate', ver: '1.1.0', icon: '⚡', dl: 15350, inst: true, cat: 'tools' },
  { id: 'steam', name: 'Steam', desc: 'Проверка серверов', author: 'Isolate', ver: '1.0.0', icon: '🎮', dl: 6540, inst: false, cat: 'gaming' },
  { id: 'twitch', name: 'Twitch', desc: 'Стриминг Twitch', author: 'Community', ver: '1.1.3', icon: '🎬', dl: 8930, inst: false, cat: 'media' },
];
