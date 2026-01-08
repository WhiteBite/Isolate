/**
 * Game Mode Store
 * Управление игровым режимом с автодетектом игр
 * 
 * Backend Integration:
 * Этот store готов к интеграции с Rust backend.
 * В Tauri режиме использует реальные команды, иначе — demo режим.
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TODO: Backend Commands (src-tauri/src/commands/game_mode.rs)
// ============================================================================
// 
// Необходимо реализовать следующие Tauri команды:
//
// #[tauri::command]
// pub async fn detect_running_games(
//     state: State<'_, Arc<AppState>>
// ) -> Result<Vec<DetectedGame>, String>
// Возвращает список запущенных игр из KNOWN_GAMES
//
// #[tauri::command]
// pub async fn start_game_monitor(
//     state: State<'_, Arc<AppState>>
// ) -> Result<(), String>
// Запускает фоновый мониторинг процессов (интервал ~5 сек)
// При обнаружении игры отправляет event: "game-detected"
// При закрытии игры отправляет event: "game-closed"
//
// #[tauri::command]
// pub async fn stop_game_monitor(
//     state: State<'_, Arc<AppState>>
// ) -> Result<(), String>
// Останавливает фоновый мониторинг
//
// #[tauri::command]
// pub fn get_game_mode_status(
//     state: State<'_, Arc<AppState>>
// ) -> Result<GameModeStatus, String>
// Возвращает текущий статус: { isMonitoring: bool, detectedGame: Option<String> }
//
// ============================================================================

export interface GameInfo {
  name: string;
  processName: string;
  icon?: string;
}

/**
 * Результат детекции игры от backend
 */
export interface DetectedGame {
  name: string;
  processName: string;
  pid?: number;
}

/**
 * Статус игрового режима от backend
 */
export interface GameModeStatus {
  isMonitoring: boolean;
  detectedGame: string | null;
}

// Известные игры для автодетекта
const KNOWN_GAMES: GameInfo[] = [
  { name: 'Counter-Strike 2', processName: 'cs2.exe' },
  { name: 'Dota 2', processName: 'dota2.exe' },
  { name: 'Valorant', processName: 'VALORANT.exe' },
  { name: 'League of Legends', processName: 'League of Legends.exe' },
  { name: 'Fortnite', processName: 'FortniteClient-Win64-Shipping.exe' },
  { name: 'Apex Legends', processName: 'r5apex.exe' },
  { name: 'PUBG', processName: 'TslGame.exe' },
  { name: 'Overwatch 2', processName: 'Overwatch.exe' },
  { name: 'Minecraft', processName: 'javaw.exe' },
  { name: 'GTA V', processName: 'GTA5.exe' },
  { name: 'Rust', processName: 'RustClient.exe' },
  { name: 'Escape from Tarkov', processName: 'EscapeFromTarkov.exe' },
  { name: 'World of Warcraft', processName: 'Wow.exe' },
  { name: 'Genshin Impact', processName: 'GenshinImpact.exe' },
  { name: 'Discord', processName: 'Discord.exe' },
];

const GAME_MODE_SETTINGS_KEY = 'isolate:game-mode-settings';

interface GameModeSettings {
  autoDetect: boolean;
  customGames: GameInfo[];
  preferredStrategy?: string;
}

// ============================================================================
// Tauri Detection
// ============================================================================

/**
 * Проверяет, запущено ли приложение в Tauri окружении
 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// ============================================================================
// Backend API Functions
// ============================================================================

/**
 * Получить список запущенных игр от backend
 * В demo режиме возвращает пустой массив
 * 
 * TODO: Backend command `detect_running_games`
 */
export async function detectRunningGames(): Promise<DetectedGame[]> {
  if (!isTauri()) {
    // Demo режим: возвращаем пустой массив
    console.log('[GameMode] Demo mode: detectRunningGames returns empty array');
    return [];
  }

  try {
    // TODO: Раскомментировать после реализации backend команды
    // return await invoke<DetectedGame[]>('detect_running_games');
    
    // Временная заглушка пока backend не готов
    console.warn('[GameMode] Backend command detect_running_games not implemented yet');
    return [];
  } catch (error) {
    console.error('[GameMode] Failed to detect running games:', error);
    return [];
  }
}

/**
 * Запустить фоновый мониторинг игр на backend
 * В demo режиме использует локальный интервал
 * 
 * TODO: Backend command `start_game_monitor`
 */
export async function startGameMonitor(): Promise<boolean> {
  if (!isTauri()) {
    console.log('[GameMode] Demo mode: using local detection interval');
    return false; // Сигнал использовать локальный fallback
  }

  try {
    // TODO: Раскомментировать после реализации backend команды
    // await invoke('start_game_monitor');
    // return true;
    
    // Временная заглушка
    console.warn('[GameMode] Backend command start_game_monitor not implemented yet');
    return false;
  } catch (error) {
    console.error('[GameMode] Failed to start game monitor:', error);
    return false;
  }
}

/**
 * Остановить фоновый мониторинг игр на backend
 * 
 * TODO: Backend command `stop_game_monitor`
 */
export async function stopGameMonitor(): Promise<void> {
  if (!isTauri()) {
    console.log('[GameMode] Demo mode: stop_game_monitor is no-op');
    return;
  }

  try {
    // TODO: Раскомментировать после реализации backend команды
    // await invoke('stop_game_monitor');
    
    console.warn('[GameMode] Backend command stop_game_monitor not implemented yet');
  } catch (error) {
    console.error('[GameMode] Failed to stop game monitor:', error);
  }
}

/**
 * Получить текущий статус игрового режима от backend
 * 
 * TODO: Backend command `get_game_mode_status`
 */
export async function getGameModeStatus(): Promise<GameModeStatus | null> {
  if (!isTauri()) {
    return null; // Demo режим не имеет backend статуса
  }

  try {
    // TODO: Раскомментировать после реализации backend команды
    // return await invoke<GameModeStatus>('get_game_mode_status');
    
    console.warn('[GameMode] Backend command get_game_mode_status not implemented yet');
    return null;
  } catch (error) {
    console.error('[GameMode] Failed to get game mode status:', error);
    return null;
  }
}

// ============================================================================
// Local Storage
// ============================================================================

function loadSettings(): GameModeSettings {
  if (typeof localStorage === 'undefined') {
    return { autoDetect: true, customGames: [] };
  }
  try {
    const stored = localStorage.getItem(GAME_MODE_SETTINGS_KEY);
    return stored ? JSON.parse(stored) : { autoDetect: true, customGames: [] };
  } catch {
    return { autoDetect: true, customGames: [] };
  }
}

function saveSettings(settings: GameModeSettings): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(GAME_MODE_SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    // Ignore storage errors
  }
}

// ============================================================================
// Game Mode Store
// ============================================================================

class GameModeStore {
  isActive = $state(false);
  detectedGame = $state<string | null>(null);
  autoDetect = $state(true);
  customGames = $state<GameInfo[]>([]);
  preferredStrategy = $state<string | undefined>(undefined);
  isMonitoring = $state(false);
  
  private detectionInterval: ReturnType<typeof setInterval> | null = null;
  private useBackendMonitor = false;

  constructor() {
    const settings = loadSettings();
    this.autoDetect = settings.autoDetect;
    this.customGames = settings.customGames;
    this.preferredStrategy = settings.preferredStrategy;
  }

  /**
   * Активировать игровой режим
   */
  activate(gameName?: string) {
    this.isActive = true;
    this.detectedGame = gameName || null;
  }

  /**
   * Деактивировать игровой режим
   */
  deactivate() {
    this.isActive = false;
    this.detectedGame = null;
  }

  /**
   * Переключить игровой режим
   */
  toggle() {
    if (this.isActive) {
      this.deactivate();
    } else {
      this.activate();
    }
  }

  /**
   * Включить/выключить автодетект
   */
  setAutoDetect(enabled: boolean) {
    this.autoDetect = enabled;
    this.saveSettings();
    
    if (enabled) {
      this.startDetection();
    } else {
      this.stopDetection();
    }
  }

  /**
   * Добавить кастомную игру
   */
  addCustomGame(game: GameInfo) {
    if (!this.customGames.find(g => g.processName === game.processName)) {
      this.customGames = [...this.customGames, game];
      this.saveSettings();
    }
  }

  /**
   * Удалить кастомную игру
   */
  removeCustomGame(processName: string) {
    this.customGames = this.customGames.filter(g => g.processName !== processName);
    this.saveSettings();
  }

  /**
   * Установить предпочтительную стратегию для игрового режима
   */
  setPreferredStrategy(strategyId: string | undefined) {
    this.preferredStrategy = strategyId;
    this.saveSettings();
  }

  /**
   * Получить все известные игры (встроенные + кастомные)
   */
  getAllGames(): GameInfo[] {
    return [...KNOWN_GAMES, ...this.customGames];
  }

  /**
   * Найти игру по имени процесса
   */
  findGameByProcess(processName: string): GameInfo | undefined {
    return this.getAllGames().find(
      g => g.processName.toLowerCase() === processName.toLowerCase()
    );
  }

  /**
   * Запустить автодетект игр
   * Пытается использовать backend мониторинг, fallback на локальный интервал
   */
  async startDetection() {
    if (this.isMonitoring || !this.autoDetect) return;
    
    // Пробуем запустить backend мониторинг
    this.useBackendMonitor = await startGameMonitor();
    
    if (this.useBackendMonitor) {
      // Backend мониторинг запущен
      // TODO: Подписаться на события game-detected и game-closed
      // import { listen } from '@tauri-apps/api/event';
      // await listen('game-detected', (event) => { ... });
      // await listen('game-closed', (event) => { ... });
      this.isMonitoring = true;
      console.log('[GameMode] Using backend game monitor');
    } else {
      // Fallback: локальный интервал (demo режим)
      this.startLocalDetection();
    }
  }

  /**
   * Локальный fallback для детекции (demo режим)
   */
  private startLocalDetection() {
    if (this.detectionInterval) return;
    
    this.isMonitoring = true;
    console.log('[GameMode] Using local detection interval (demo mode)');
    
    // Demo режим: периодически проверяем через backend API
    this.detectionInterval = setInterval(async () => {
      const games = await detectRunningGames();
      
      if (games.length > 0 && !this.isActive) {
        // Найдена игра — активируем режим
        const game = games[0];
        this.activate(game.name);
        console.log(`[GameMode] Detected game: ${game.name}`);
      } else if (games.length === 0 && this.isActive && this.detectedGame) {
        // Игра закрыта — деактивируем
        console.log(`[GameMode] Game closed: ${this.detectedGame}`);
        this.deactivate();
      }
    }, 5000);
  }

  /**
   * Остановить автодетект
   */
  async stopDetection() {
    this.isMonitoring = false;
    
    if (this.useBackendMonitor) {
      await stopGameMonitor();
      this.useBackendMonitor = false;
    }
    
    if (this.detectionInterval) {
      clearInterval(this.detectionInterval);
      this.detectionInterval = null;
    }
  }

  /**
   * Синхронизировать состояние с backend
   */
  async syncWithBackend() {
    const status = await getGameModeStatus();
    if (status) {
      this.isMonitoring = status.isMonitoring;
      if (status.detectedGame && !this.isActive) {
        this.activate(status.detectedGame);
      } else if (!status.detectedGame && this.isActive && this.detectedGame) {
        this.deactivate();
      }
    }
  }

  /**
   * Сохранить настройки
   */
  private saveSettings() {
    saveSettings({
      autoDetect: this.autoDetect,
      customGames: this.customGames,
      preferredStrategy: this.preferredStrategy
    });
  }

  /**
   * Очистка при уничтожении
   */
  async destroy() {
    await this.stopDetection();
  }
}

export const gameModeStore = new GameModeStore();

/**
 * Инициализация Game Mode при старте приложения
 */
export async function initGameMode() {
  // Синхронизируем с backend если доступен
  if (isTauri()) {
    await gameModeStore.syncWithBackend();
  }
  
  if (gameModeStore.autoDetect) {
    await gameModeStore.startDetection();
  }
  
  return () => gameModeStore.destroy();
}

// Экспорт для использования в других модулях
export { isTauri, KNOWN_GAMES };
