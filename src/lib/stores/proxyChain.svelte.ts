/**
 * Proxy Chain Store
 * Управление визуальным конструктором цепочек прокси
 * Интегрирован с backend командами из proxy_chain.rs
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { eventBus } from './eventBus.svelte';

// ============================================================================
// Environment Detection
// ============================================================================

/**
 * Проверяет, запущено ли приложение в Tauri окружении
 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Проверяет готовность backend (AppState инициализирован)
 */
async function isBackendReady(): Promise<boolean> {
  if (!isTauri()) return false;
  try {
    return await invoke<boolean>('is_backend_ready');
  } catch {
    return false;
  }
}

// ============================================================================
// Backend Types (соответствуют Rust структурам в proxy_chain.rs)
// ============================================================================

/**
 * Proxy chain from backend
 * Rust: ProxyChain in commands/proxy_chain.rs
 */
export interface BackendProxyChain {
  id: string;
  name: string;
  proxies: string[];
  is_active: boolean;
  created_at: number;
}

/**
 * Parsed proxy info from backend
 * Rust: ParsedProxy in commands/proxy_chain.rs
 */
export interface ParsedProxy {
  protocol: string;
  host: string;
  port: number;
  username?: string;
  password?: string;
  country?: string;
  name?: string;
}

/**
 * Import result from backend
 * Rust: ImportResult in commands/proxy_chain.rs
 */
export interface ImportResult {
  success_count: number;
  failed_count: number;
  errors: string[];
  imported_ids: string[];
}

// ============================================================================
// Frontend Types
// ============================================================================

export interface ChainBlock {
  id: string;
  type: 'dpi' | 'proxy' | 'internet';
  position: { x: number; y: number };
  data: {
    name: string;
    description?: string;
    proxyId?: string;
    strategyId?: string;
    country?: string;
    protocol?: string;
  };
}

export interface ChainConnection {
  id: string;
  from: string;
  to: string;
}

export interface ChainPreset {
  id: string;
  name: string;
  description: string;
  blocks: Omit<ChainBlock, 'id'>[];
  connections: Omit<ChainConnection, 'id'>[];
}

interface ProxyChainState {
  blocks: ChainBlock[];
  connections: ChainConnection[];
  selectedBlockId: string | null;
  isDragging: boolean;
  dragOffset: { x: number; y: number };
}

// ============================================================================
// Backend API Functions
// ============================================================================

/**
 * Загружает сохранённые цепочки из backend
 */
async function fetchChains(): Promise<BackendProxyChain[]> {
  if (!isTauri()) {
    return getDemoChains();
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    console.warn('[ProxyChain] Backend not ready, using demo data');
    return getDemoChains();
  }
  
  try {
    return await invoke<BackendProxyChain[]>('get_proxy_chains');
  } catch (e) {
    console.error('[ProxyChain] Failed to fetch chains:', e);
    return getDemoChains();
  }
}

/**
 * Сохраняет цепочку в backend
 */
async function saveChainToBackend(chain: BackendProxyChain): Promise<BackendProxyChain> {
  if (!isTauri()) {
    console.log('[Demo] Would save chain:', chain);
    return chain;
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    throw new Error('Backend not ready');
  }
  
  return await invoke<BackendProxyChain>('save_proxy_chain', { chain });
}

/**
 * Удаляет цепочку из backend
 */
async function deleteChainFromBackend(chainId: string): Promise<void> {
  if (!isTauri()) {
    console.log('[Demo] Would delete chain:', chainId);
    return;
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    throw new Error('Backend not ready');
  }
  
  await invoke('delete_proxy_chain', { chainId });
}

/**
 * Применяет (активирует) цепочку
 */
async function applyChainInBackend(chainId: string): Promise<void> {
  if (!isTauri()) {
    console.log('[Demo] Would apply chain:', chainId);
    return;
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    throw new Error('Backend not ready');
  }
  
  await invoke('apply_proxy_chain', { chainId });
}

/**
 * Останавливает (деактивирует) цепочку
 */
async function stopChainInBackend(chainId: string): Promise<void> {
  if (!isTauri()) {
    console.log('[Demo] Would stop chain:', chainId);
    return;
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    throw new Error('Backend not ready');
  }
  
  await invoke('deactivate_proxy_chain', { chainId });
}

/**
 * Парсит URL прокси
 */
async function parseProxyUrlFromBackend(url: string): Promise<ParsedProxy> {
  if (!isTauri()) {
    // Demo: простой парсинг
    const match = url.match(/^(\w+):\/\/(?:([^:@]+):([^@]+)@)?([^:]+):(\d+)/);
    if (match) {
      return {
        protocol: match[1],
        host: match[4],
        port: parseInt(match[5]),
        username: match[2],
        password: match[3],
      };
    }
    throw new Error('Invalid proxy URL');
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    throw new Error('Backend not ready');
  }
  
  return await invoke<ParsedProxy>('parse_proxy_url_info', { url });
}

/**
 * Импортирует прокси из списка URL
 */
async function batchImportProxiesFromBackend(urls: string[]): Promise<ImportResult> {
  if (!isTauri()) {
    console.log('[Demo] Would import proxies:', urls.length);
    return {
      success_count: urls.length,
      failed_count: 0,
      errors: [],
      imported_ids: urls.map((_, i) => `demo_proxy_${i}`),
    };
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    throw new Error('Backend not ready');
  }
  
  return await invoke<ImportResult>('batch_import_proxies', { urls });
}

// ============================================================================
// Demo Data
// ============================================================================

function getDemoChains(): BackendProxyChain[] {
  return [
    {
      id: 'chain_default',
      name: 'Default Chain',
      proxies: [],
      is_active: false,
      created_at: Date.now(),
    },
  ];
}

// ============================================================================
// Local Storage
// ============================================================================

const STORAGE_KEY = 'isolate:proxy-chain';

function generateId(): string {
  return `block-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

function loadLocalState(): ProxyChainState {
  if (typeof localStorage === 'undefined') {
    return getDefaultState();
  }
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : getDefaultState();
  } catch {
    return getDefaultState();
  }
}

function getDefaultState(): ProxyChainState {
  return {
    blocks: [],
    connections: [],
    selectedBlockId: null,
    isDragging: false,
    dragOffset: { x: 0, y: 0 }
  };
}

function saveLocalState(state: ProxyChainState): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      blocks: state.blocks,
      connections: state.connections
    }));
  } catch {
    // Ignore storage errors
  }
}

// ============================================================================
// Proxy Chain Store Class
// ============================================================================

class ProxyChainStore {
  // Visual editor state
  blocks = $state<ChainBlock[]>([]);
  connections = $state<ChainConnection[]>([]);
  selectedBlockId = $state<string | null>(null);
  isDragging = $state(false);
  dragOffset = $state({ x: 0, y: 0 });
  
  // Backend state
  savedChains = $state<BackendProxyChain[]>([]);
  activeChainId = $state<string | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);
  
  // Event listeners
  private unlistenFns: UnlistenFn[] = [];
  private initialized = false;

  constructor() {
    const state = loadLocalState();
    this.blocks = state.blocks;
    this.connections = state.connections;
  }

  /**
   * Инициализация store: загрузка данных и подписка на события
   */
  async init() {
    if (this.initialized) return;
    this.initialized = true;
    
    await this.loadChains();
    await this.setupEventListeners();
  }

  /**
   * Очистка при уничтожении store
   */
  async destroy() {
    for (const unlisten of this.unlistenFns) {
      await unlisten();
    }
    this.unlistenFns = [];
    this.initialized = false;
  }

  /**
   * Настройка подписок на Tauri события
   */
  private async setupEventListeners() {
    if (!isTauri()) return;
    
    try {
      // Подписка на событие применения цепочки
      const unlistenApplied = await listen<{ chain_id: string; name: string }>(
        'proxy_chain:applied',
        (event) => {
          console.log('[ProxyChain] Chain applied event:', event.payload);
          this.activeChainId = event.payload.chain_id;
          this.updateChainActiveStatus(event.payload.chain_id, true);
          
          // Emit to eventBus for other components
          eventBus.emit('proxy:chain_activated', {
            chain_id: event.payload.chain_id,
            name: event.payload.name,
          });
        }
      );
      this.unlistenFns.push(unlistenApplied);
      
      // Подписка на событие остановки цепочки
      const unlistenStopped = await listen<{ chain_id: string }>(
        'proxy_chain:stopped',
        (event) => {
          console.log('[ProxyChain] Chain stopped event:', event.payload);
          if (this.activeChainId === event.payload.chain_id) {
            this.activeChainId = null;
          }
          this.updateChainActiveStatus(event.payload.chain_id, false);
          
          // Emit to eventBus for other components
          eventBus.emit('proxy:chain_deactivated', {
            chain_id: event.payload.chain_id,
          });
        }
      );
      this.unlistenFns.push(unlistenStopped);
      
      console.log('[ProxyChain] Event listeners set up');
    } catch (e) {
      console.error('[ProxyChain] Failed to setup event listeners:', e);
    }
  }

  /**
   * Обновляет статус активности цепочки в локальном состоянии
   */
  private updateChainActiveStatus(chainId: string, isActive: boolean) {
    this.savedChains = this.savedChains.map(chain =>
      chain.id === chainId ? { ...chain, is_active: isActive } : chain
    );
  }

  // ==========================================================================
  // Backend Integration Methods
  // ==========================================================================

  /**
   * Загружает сохранённые цепочки из backend
   */
  async loadChains() {
    this.loading = true;
    this.error = null;
    
    try {
      this.savedChains = await fetchChains();
      
      // Найти активную цепочку
      const activeChain = this.savedChains.find(c => c.is_active);
      this.activeChainId = activeChain?.id ?? null;
      
      console.log('[ProxyChain] Loaded chains:', this.savedChains.length);
    } catch (e) {
      console.error('[ProxyChain] Failed to load chains:', e);
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  /**
   * Сохраняет текущую цепочку в backend
   */
  async saveChain(name: string): Promise<BackendProxyChain | null> {
    this.error = null;
    
    try {
      // Собираем proxy IDs из блоков
      const proxyIds = this.blocks
        .filter(b => b.type === 'proxy' && b.data.proxyId)
        .map(b => b.data.proxyId!);
      
      const chain: BackendProxyChain = {
        id: '', // Backend сгенерирует ID
        name,
        proxies: proxyIds,
        is_active: false,
        created_at: Date.now(),
      };
      
      const savedChain = await saveChainToBackend(chain);
      
      // Обновляем локальный список
      const existingIndex = this.savedChains.findIndex(c => c.id === savedChain.id);
      if (existingIndex >= 0) {
        this.savedChains = [
          ...this.savedChains.slice(0, existingIndex),
          savedChain,
          ...this.savedChains.slice(existingIndex + 1),
        ];
      } else {
        this.savedChains = [...this.savedChains, savedChain];
      }
      
      console.log('[ProxyChain] Chain saved:', savedChain.id);
      return savedChain;
    } catch (e) {
      console.error('[ProxyChain] Failed to save chain:', e);
      this.error = String(e);
      return null;
    }
  }

  /**
   * Удаляет цепочку
   */
  async deleteChain(chainId: string): Promise<boolean> {
    this.error = null;
    
    try {
      await deleteChainFromBackend(chainId);
      this.savedChains = this.savedChains.filter(c => c.id !== chainId);
      
      if (this.activeChainId === chainId) {
        this.activeChainId = null;
      }
      
      console.log('[ProxyChain] Chain deleted:', chainId);
      return true;
    } catch (e) {
      console.error('[ProxyChain] Failed to delete chain:', e);
      this.error = String(e);
      return false;
    }
  }

  /**
   * Применяет (активирует) цепочку
   */
  async applyChain(chainId: string): Promise<boolean> {
    this.error = null;
    
    try {
      await applyChainInBackend(chainId);
      
      // Обновляем локальное состояние (событие тоже обновит)
      this.activeChainId = chainId;
      this.updateChainActiveStatus(chainId, true);
      
      console.log('[ProxyChain] Chain applied:', chainId);
      return true;
    } catch (e) {
      console.error('[ProxyChain] Failed to apply chain:', e);
      this.error = String(e);
      return false;
    }
  }

  /**
   * Останавливает активную цепочку
   */
  async stopChain(chainId?: string): Promise<boolean> {
    const targetId = chainId ?? this.activeChainId;
    if (!targetId) {
      console.warn('[ProxyChain] No chain to stop');
      return false;
    }
    
    this.error = null;
    
    try {
      await stopChainInBackend(targetId);
      
      // Обновляем локальное состояние (событие тоже обновит)
      this.activeChainId = null;
      this.updateChainActiveStatus(targetId, false);
      
      console.log('[ProxyChain] Chain stopped:', targetId);
      return true;
    } catch (e) {
      console.error('[ProxyChain] Failed to stop chain:', e);
      this.error = String(e);
      return false;
    }
  }

  /**
   * Парсит URL прокси
   */
  async parseProxyUrl(url: string): Promise<ParsedProxy | null> {
    try {
      return await parseProxyUrlFromBackend(url);
    } catch (e) {
      console.error('[ProxyChain] Failed to parse proxy URL:', e);
      return null;
    }
  }

  /**
   * Импортирует прокси из списка URL
   */
  async importProxies(urls: string[]): Promise<ImportResult | null> {
    this.error = null;
    
    try {
      const result = await batchImportProxiesFromBackend(urls);
      
      if (result.success_count > 0) {
        // Emit event for other components
        eventBus.emit('proxy:imported', {
          count: result.success_count,
          source: 'batch_import',
        });
      }
      
      console.log('[ProxyChain] Proxies imported:', result.success_count);
      return result;
    } catch (e) {
      console.error('[ProxyChain] Failed to import proxies:', e);
      this.error = String(e);
      return null;
    }
  }

  // ==========================================================================
  // Visual Editor Methods
  // ==========================================================================

  /**
   * Добавить новый блок
   */
  addBlock(type: ChainBlock['type'], position: { x: number; y: number }, data?: Partial<ChainBlock['data']>) {
    const defaultData = this.getDefaultBlockData(type);
    const block: ChainBlock = {
      id: generateId(),
      type,
      position,
      data: { ...defaultData, ...data }
    };
    
    this.blocks = [...this.blocks, block];
    this.saveLocal();
    return block;
  }

  /**
   * Получить данные по умолчанию для типа блока
   */
  private getDefaultBlockData(type: ChainBlock['type']): ChainBlock['data'] {
    switch (type) {
      case 'dpi':
        return { name: 'DPI Bypass', description: 'Обход блокировок' };
      case 'proxy':
        return { name: 'Proxy', description: 'Прокси-сервер' };
      case 'internet':
        return { name: 'Internet', description: 'Выход в сеть' };
    }
  }

  /**
   * Удалить блок
   */
  removeBlock(blockId: string) {
    this.blocks = this.blocks.filter(b => b.id !== blockId);
    this.connections = this.connections.filter(
      c => c.from !== blockId && c.to !== blockId
    );
    if (this.selectedBlockId === blockId) {
      this.selectedBlockId = null;
    }
    this.saveLocal();
  }

  /**
   * Обновить позицию блока
   */
  updateBlockPosition(blockId: string, position: { x: number; y: number }) {
    this.blocks = this.blocks.map(b =>
      b.id === blockId ? { ...b, position } : b
    );
    this.saveLocal();
  }

  /**
   * Обновить данные блока
   */
  updateBlockData(blockId: string, data: Partial<ChainBlock['data']>) {
    this.blocks = this.blocks.map(b =>
      b.id === blockId ? { ...b, data: { ...b.data, ...data } } : b
    );
    this.saveLocal();
  }

  /**
   * Выбрать блок
   */
  selectBlock(blockId: string | null) {
    this.selectedBlockId = blockId;
  }

  /**
   * Добавить соединение между блоками
   */
  addConnection(fromId: string, toId: string) {
    // Проверяем, что соединение не существует
    const exists = this.connections.some(
      c => c.from === fromId && c.to === toId
    );
    if (exists) return;

    // Проверяем, что блоки существуют
    const fromBlock = this.blocks.find(b => b.id === fromId);
    const toBlock = this.blocks.find(b => b.id === toId);
    if (!fromBlock || !toBlock) return;

    const connection: ChainConnection = {
      id: `conn-${Date.now()}`,
      from: fromId,
      to: toId
    };
    
    this.connections = [...this.connections, connection];
    this.saveLocal();
  }

  /**
   * Удалить соединение
   */
  removeConnection(connectionId: string) {
    this.connections = this.connections.filter(c => c.id !== connectionId);
    this.saveLocal();
  }

  /**
   * Начать перетаскивание
   */
  startDrag(offset: { x: number; y: number }) {
    this.isDragging = true;
    this.dragOffset = offset;
  }

  /**
   * Закончить перетаскивание
   */
  endDrag() {
    this.isDragging = false;
    this.dragOffset = { x: 0, y: 0 };
  }

  /**
   * Применить пресет
   */
  applyPreset(preset: ChainPreset) {
    // Очищаем текущую цепочку
    this.clear();
    
    // Создаём маппинг старых ID на новые
    const idMap = new Map<string, string>();
    
    // Добавляем блоки
    preset.blocks.forEach((blockData, index) => {
      const oldId = `preset-${index}`;
      const block = this.addBlock(blockData.type, blockData.position, blockData.data);
      idMap.set(oldId, block.id);
    });
    
    // Добавляем соединения с новыми ID
    preset.connections.forEach((conn, index) => {
      const fromId = idMap.get(`preset-${index}`) || this.blocks[index]?.id;
      const toId = idMap.get(`preset-${index + 1}`) || this.blocks[index + 1]?.id;
      if (fromId && toId) {
        this.addConnection(fromId, toId);
      }
    });
  }

  /**
   * Очистить цепочку
   */
  clear() {
    this.blocks = [];
    this.connections = [];
    this.selectedBlockId = null;
    this.saveLocal();
  }

  /**
   * Получить блок по ID
   */
  getBlock(blockId: string): ChainBlock | undefined {
    return this.blocks.find(b => b.id === blockId);
  }

  /**
   * Получить позицию центра блока
   */
  getBlockCenter(blockId: string): { x: number; y: number } | null {
    const block = this.getBlock(blockId);
    if (!block) return null;
    
    // Предполагаем размер блока 160x80
    return {
      x: block.position.x + 80,
      y: block.position.y + 40
    };
  }

  /**
   * Сохранить локальное состояние (visual editor)
   */
  private saveLocal() {
    saveLocalState({
      blocks: this.blocks,
      connections: this.connections,
      selectedBlockId: this.selectedBlockId,
      isDragging: this.isDragging,
      dragOffset: this.dragOffset
    });
  }

  /**
   * Экспортировать цепочку
   */
  exportChain(): { blocks: ChainBlock[]; connections: ChainConnection[] } {
    return {
      blocks: this.blocks,
      connections: this.connections
    };
  }

  /**
   * Импортировать цепочку
   */
  importChain(data: { blocks: ChainBlock[]; connections: ChainConnection[] }) {
    this.blocks = data.blocks;
    this.connections = data.connections;
    this.saveLocal();
  }

  // ==========================================================================
  // Derived State
  // ==========================================================================

  /**
   * Проверяет, есть ли активная цепочка
   */
  get isActive(): boolean {
    return this.activeChainId !== null;
  }

  /**
   * Получает активную цепочку
   */
  get activeChain(): BackendProxyChain | null {
    if (!this.activeChainId) return null;
    return this.savedChains.find(c => c.id === this.activeChainId) ?? null;
  }
}

export const proxyChainStore = new ProxyChainStore();

/**
 * Предустановленные пресеты цепочек
 */
export const chainPresets: ChainPreset[] = [
  {
    id: 'basic-dpi',
    name: 'Базовый обход DPI',
    description: 'Простой обход блокировок без прокси',
    blocks: [
      { type: 'dpi', position: { x: 50, y: 100 }, data: { name: 'DPI Bypass', description: 'Zapret/GoodbyeDPI' } },
      { type: 'internet', position: { x: 300, y: 100 }, data: { name: 'Internet', description: 'Прямое подключение' } }
    ],
    connections: [
      { from: 'preset-0', to: 'preset-1' }
    ]
  },
  {
    id: 'single-proxy',
    name: 'Через прокси',
    description: 'Весь трафик через один прокси-сервер',
    blocks: [
      { type: 'dpi', position: { x: 50, y: 100 }, data: { name: 'DPI Bypass', description: 'Обход блокировок' } },
      { type: 'proxy', position: { x: 250, y: 100 }, data: { name: 'VLESS Proxy', description: 'Netherlands', country: 'NL', protocol: 'vless' } },
      { type: 'internet', position: { x: 450, y: 100 }, data: { name: 'Internet', description: 'Выход в сеть' } }
    ],
    connections: [
      { from: 'preset-0', to: 'preset-1' },
      { from: 'preset-1', to: 'preset-2' }
    ]
  },
  {
    id: 'double-proxy',
    name: 'Двойной прокси',
    description: 'Цепочка из двух прокси для максимальной анонимности',
    blocks: [
      { type: 'dpi', position: { x: 50, y: 100 }, data: { name: 'DPI Bypass', description: 'Обход блокировок' } },
      { type: 'proxy', position: { x: 220, y: 100 }, data: { name: 'Proxy 1', description: 'Germany', country: 'DE', protocol: 'vless' } },
      { type: 'proxy', position: { x: 390, y: 100 }, data: { name: 'Proxy 2', description: 'Netherlands', country: 'NL', protocol: 'vmess' } },
      { type: 'internet', position: { x: 560, y: 100 }, data: { name: 'Internet', description: 'Выход в сеть' } }
    ],
    connections: [
      { from: 'preset-0', to: 'preset-1' },
      { from: 'preset-1', to: 'preset-2' },
      { from: 'preset-2', to: 'preset-3' }
    ]
  }
];

// Экспорт вспомогательных функций
export { isTauri, isBackendReady };
