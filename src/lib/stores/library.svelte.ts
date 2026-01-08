import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Backend Integration Helpers
// ============================================================================

/**
 * Проверяет, запущено ли приложение в Tauri окружении
 * @returns true если работаем в Tauri, false если в браузере (demo режим)
 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Проверяет готовность backend перед вызовом команд
 * @returns true если backend готов к работе
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
// Backend Types (синхронизированы с library.rs)
// ============================================================================

/**
 * Метод доступа к ресурсу (соответствует Rust AccessMethod)
 * @see src-tauri/src/commands/library.rs
 */
export type BackendAccessMethod = 'zapret' | 'vless' | 'direct' | 'block';

/**
 * Правило библиотеки из backend (соответствует Rust LibraryRule)
 * @see src-tauri/src/commands/library.rs
 */
export interface BackendLibraryRule {
  id: string;
  serviceId: string | null;
  pattern: string;
  method: BackendAccessMethod;
  isEnabled: boolean;
  strategyId: string | null;
  priority: number;
  createdAt: number;
  updatedAt: number;
}

/**
 * Входные данные для создания правила (соответствует Rust CreateRuleInput)
 */
export interface CreateRuleInput {
  serviceId?: string | null;
  pattern: string;
  method: BackendAccessMethod;
  strategyId?: string | null;
  priority?: number | null;
}

/**
 * Входные данные для обновления правила (соответствует Rust UpdateRuleInput)
 */
export interface UpdateRuleInput {
  id: string;
  serviceId?: string | null;
  pattern?: string | null;
  method?: BackendAccessMethod | null;
  isEnabled?: boolean | null;
  strategyId?: string | null;
  priority?: number | null;
}

/**
 * Пресет библиотеки (соответствует Rust LibraryPreset)
 */
export interface LibraryPreset {
  id: string;
  name: string;
  description: string;
  rules: BackendLibraryRule[];
  isBuiltin: boolean;
}

// ============================================================================
// Frontend Types (для UI совместимости)
// ============================================================================

export type AccessMethodType = 'direct' | 'auto' | 'strategy' | 'vless' | 'proxy' | 'tor' | 'block';
export type ServiceStatus = 'accessible' | 'blocked' | 'unknown' | 'checking';

export interface AccessMethod {
  type: AccessMethodType;
  strategyId?: string;
  strategyName?: string;
  proxyId?: string;
  proxyName?: string;
}

export interface ServiceRule {
  id: string;
  name: string;
  domain: string;
  icon: string;
  category: string;
  status: ServiceStatus;
  currentMethod: AccessMethod;
  availableMethods: AccessMethod[];
  isCustom: boolean;
  isEnabled: boolean;
  lastChecked?: number;
  ping?: number;
  priority: number;
  createdAt: number;
  updatedAt: number;
}

export interface LibraryFilters {
  search: string;
  status: 'all' | ServiceStatus;
  method: 'all' | AccessMethodType;
  category: string;
  criticalOnly: boolean;
}

// Критичные сервисы для пресета "Critical only"
export const CRITICAL_SERVICES = ['youtube', 'discord', 'telegram', 'twitch', 'steam'] as const;

// ============================================================================
// LocalStorage Helpers
// ============================================================================

const FILTERS_STORAGE_KEY = 'isolate_library_filters';

function loadFiltersFromStorage(): Partial<LibraryFilters> {
  if (typeof window === 'undefined') return {};
  try {
    const stored = localStorage.getItem(FILTERS_STORAGE_KEY);
    if (stored) {
      return JSON.parse(stored);
    }
  } catch (e) {
    console.warn('[Library] Failed to load filters from localStorage:', e);
  }
  return {};
}

function saveFiltersToStorage(filters: LibraryFilters): void {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(FILTERS_STORAGE_KEY, JSON.stringify(filters));
  } catch (e) {
    console.warn('[Library] Failed to save filters to localStorage:', e);
  }
}

// ============================================================================
// Type Converters
// ============================================================================

/**
 * Конвертирует backend AccessMethod в frontend AccessMethod
 */
function backendMethodToFrontend(method: BackendAccessMethod, strategyId?: string | null): AccessMethod {
  switch (method) {
    case 'zapret':
      return { 
        type: 'strategy', 
        strategyId: strategyId ?? undefined,
        strategyName: strategyId ? `Strategy ${strategyId}` : 'Zapret'
      };
    case 'vless':
      return { type: 'vless' };
    case 'direct':
      return { type: 'direct' };
    case 'block':
      return { type: 'block' };
    default:
      return { type: 'direct' };
  }
}

/**
 * Конвертирует frontend AccessMethod в backend AccessMethod
 */
function frontendMethodToBackend(method: AccessMethod): { method: BackendAccessMethod; strategyId?: string } {
  switch (method.type) {
    case 'strategy':
    case 'auto':
      return { method: 'zapret', strategyId: method.strategyId };
    case 'vless':
    case 'proxy':
    case 'tor':
      return { method: 'vless' };
    case 'block':
      return { method: 'block' };
    case 'direct':
    default:
      return { method: 'direct' };
  }
}

/**
 * Извлекает имя сервиса из паттерна домена
 */
function extractServiceName(pattern: string): string {
  // Убираем wildcard и извлекаем основной домен
  const domain = pattern.replace(/^\*\./, '').replace(/\*$/, '');
  const parts = domain.split('.');
  
  // Берём первую значимую часть
  const name = parts[0] || domain;
  
  // Капитализируем первую букву
  return name.charAt(0).toUpperCase() + name.slice(1);
}

/**
 * Определяет категорию сервиса по домену
 */
function detectCategory(pattern: string): string {
  const domain = pattern.toLowerCase();
  
  if (domain.includes('youtube') || domain.includes('twitch') || domain.includes('vimeo')) {
    return 'video';
  }
  if (domain.includes('discord') || domain.includes('telegram') || domain.includes('twitter') || 
      domain.includes('instagram') || domain.includes('facebook')) {
    return 'social';
  }
  if (domain.includes('spotify') || domain.includes('soundcloud') || domain.includes('music')) {
    return 'music';
  }
  if (domain.includes('steam') || domain.includes('epic') || domain.includes('game')) {
    return 'gaming';
  }
  
  return 'other';
}

/**
 * Определяет иконку для сервиса
 */
function detectIcon(pattern: string, serviceId?: string | null): string {
  const domain = (serviceId || pattern).toLowerCase();
  
  const iconMap: Record<string, string> = {
    'youtube': '📺',
    'discord': '💬',
    'telegram': '✈️',
    'twitter': '🐦',
    'instagram': '📷',
    'facebook': '👤',
    'spotify': '🎵',
    'twitch': '🎮',
    'steam': '🎮',
    'github': '💻',
    'google': '🔍',
  };
  
  for (const [key, icon] of Object.entries(iconMap)) {
    if (domain.includes(key)) {
      return icon;
    }
  }
  
  return '🌐';
}

/**
 * Конвертирует BackendLibraryRule в ServiceRule для UI
 */
function backendRuleToServiceRule(rule: BackendLibraryRule): ServiceRule {
  const currentMethod = backendMethodToFrontend(rule.method, rule.strategyId);
  
  // Генерируем доступные методы
  const availableMethods: AccessMethod[] = [
    { type: 'direct' },
    { type: 'auto' },
  ];
  
  if (rule.strategyId) {
    availableMethods.push({
      type: 'strategy',
      strategyId: rule.strategyId,
      strategyName: `Strategy ${rule.strategyId}`
    });
  }
  
  availableMethods.push({ type: 'vless' });
  availableMethods.push({ type: 'block' });
  
  return {
    id: rule.id,
    name: extractServiceName(rule.pattern),
    domain: rule.pattern.replace(/^\*\./, ''),
    icon: detectIcon(rule.pattern, rule.serviceId),
    category: detectCategory(rule.pattern),
    status: rule.isEnabled ? 'unknown' : 'blocked',
    currentMethod,
    availableMethods,
    isCustom: !rule.serviceId,
    isEnabled: rule.isEnabled,
    priority: rule.priority,
    createdAt: rule.createdAt,
    updatedAt: rule.updatedAt,
  };
}

/**
 * Конвертирует ServiceRule в CreateRuleInput для backend
 */
function serviceRuleToCreateInput(rule: Partial<ServiceRule> & { domain: string; currentMethod: AccessMethod }): CreateRuleInput {
  const { method, strategyId } = frontendMethodToBackend(rule.currentMethod);
  
  return {
    serviceId: rule.id?.startsWith('custom_') ? null : rule.id,
    pattern: rule.domain.includes('*') ? rule.domain : `*.${rule.domain}`,
    method,
    strategyId: strategyId ?? null,
    priority: rule.priority ?? 0,
  };
}

// ============================================================================
// Backend API Functions
// ============================================================================

/**
 * Загружает правила библиотеки из backend
 */
async function fetchRules(): Promise<ServiceRule[]> {
  if (!isTauri()) {
    return getDemoRules();
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    console.warn('[Library] Backend not ready, using demo data');
    return getDemoRules();
  }
  
  try {
    const backendRules = await invoke<BackendLibraryRule[]>('get_library_rules');
    return backendRules.map(backendRuleToServiceRule);
  } catch (e) {
    console.error('[Library] Failed to fetch rules from backend:', e);
    return getDemoRules();
  }
}

/**
 * Добавляет новое правило в backend
 */
async function addRuleToBackend(input: CreateRuleInput): Promise<BackendLibraryRule> {
  if (!isTauri()) {
    console.log('[Demo] Would add rule:', input);
    // Возвращаем mock для demo режима
    return {
      id: `demo_${Date.now()}`,
      serviceId: input.serviceId ?? null,
      pattern: input.pattern,
      method: input.method,
      isEnabled: true,
      strategyId: input.strategyId ?? null,
      priority: input.priority ?? 0,
      createdAt: Date.now() / 1000,
      updatedAt: Date.now() / 1000,
    };
  }
  
  return await invoke<BackendLibraryRule>('add_library_rule', { input });
}

/**
 * Обновляет правило в backend
 */
async function updateRuleInBackend(input: UpdateRuleInput): Promise<BackendLibraryRule> {
  if (!isTauri()) {
    console.log('[Demo] Would update rule:', input);
    throw new Error('Demo mode: update not persisted');
  }
  
  return await invoke<BackendLibraryRule>('update_library_rule', { input });
}

/**
 * Удаляет правило из backend
 */
async function deleteRuleFromBackend(ruleId: string): Promise<boolean> {
  if (!isTauri()) {
    console.log('[Demo] Would delete rule:', ruleId);
    return true;
  }
  
  return await invoke<boolean>('delete_library_rule', { ruleId });
}

/**
 * Устанавливает метод доступа для правила
 */
async function setRuleMethodInBackend(ruleId: string, method: BackendAccessMethod): Promise<BackendLibraryRule> {
  if (!isTauri()) {
    console.log('[Demo] Would set method for rule:', ruleId, method);
    throw new Error('Demo mode: method change not persisted');
  }
  
  return await invoke<BackendLibraryRule>('set_rule_method', { ruleId, method });
}

/**
 * Включает/выключает правило
 */
async function toggleRuleInBackend(ruleId: string, enabled: boolean): Promise<BackendLibraryRule> {
  if (!isTauri()) {
    console.log('[Demo] Would toggle rule:', ruleId, enabled);
    throw new Error('Demo mode: toggle not persisted');
  }
  
  return await invoke<BackendLibraryRule>('toggle_library_rule', { ruleId, enabled });
}

/**
 * Загружает пресеты из backend
 */
async function fetchPresets(): Promise<LibraryPreset[]> {
  if (!isTauri()) {
    return getDemoPresets();
  }
  
  const ready = await isBackendReady();
  if (!ready) {
    console.warn('[Library] Backend not ready, using demo presets');
    return getDemoPresets();
  }
  
  try {
    return await invoke<LibraryPreset[]>('get_library_presets');
  } catch (e) {
    console.error('[Library] Failed to fetch presets:', e);
    return getDemoPresets();
  }
}

/**
 * Применяет пресет
 */
async function applyPresetInBackend(presetId: string): Promise<BackendLibraryRule[]> {
  if (!isTauri()) {
    console.log('[Demo] Would apply preset:', presetId);
    const presets = getDemoPresets();
    const preset = presets.find(p => p.id === presetId);
    return preset?.rules ?? [];
  }
  
  return await invoke<BackendLibraryRule[]>('apply_library_preset', { presetId });
}

/**
 * Проверяет доступность сервиса через backend
 */
async function checkService(domain: string): Promise<{ status: ServiceStatus; ping?: number }> {
  if (!isTauri()) {
    // Demo: имитация проверки
    await new Promise(resolve => setTimeout(resolve, 800 + Math.random() * 400));
    return {
      status: Math.random() > 0.3 ? 'accessible' : 'blocked',
      ping: Math.floor(Math.random() * 100) + 20
    };
  }
  
  try {
    return await invoke<{ status: ServiceStatus; ping?: number }>('check_service_availability', { domain });
  } catch {
    // Если команда не реализована, возвращаем unknown
    return { status: 'unknown' };
  }
}

// ============================================================================
// Demo Data
// ============================================================================

/**
 * Возвращает демо-данные для режима без backend
 */
function getDemoRules(): ServiceRule[] {
  const now = Date.now() / 1000;
  return [
    {
      id: 'youtube',
      name: 'YouTube',
      domain: 'youtube.com',
      icon: '📺',
      category: 'video',
      status: 'accessible',
      currentMethod: { type: 'strategy', strategyId: 'fake_tls', strategyName: 'Fake TLS' },
      availableMethods: [
        { type: 'direct' },
        { type: 'auto' },
        { type: 'strategy', strategyId: 'fake_tls', strategyName: 'Fake TLS' },
        { type: 'vless' },
        { type: 'block' }
      ],
      isCustom: false,
      isEnabled: true,
      ping: 45,
      priority: 100,
      createdAt: now - 86400,
      updatedAt: now,
    },
    {
      id: 'discord',
      name: 'Discord',
      domain: 'discord.com',
      icon: '💬',
      category: 'social',
      status: 'accessible',
      currentMethod: { type: 'auto' },
      availableMethods: [
        { type: 'direct' },
        { type: 'auto' },
        { type: 'strategy', strategyId: 'discord_voice', strategyName: 'Discord Voice' },
        { type: 'vless' },
        { type: 'block' }
      ],
      isCustom: false,
      isEnabled: true,
      ping: 32,
      priority: 100,
      createdAt: now - 86400,
      updatedAt: now,
    },
    {
      id: 'twitter',
      name: 'Twitter/X',
      domain: 'twitter.com',
      icon: '🐦',
      category: 'social',
      status: 'blocked',
      currentMethod: { type: 'direct' },
      availableMethods: [
        { type: 'direct' },
        { type: 'auto' },
        { type: 'strategy', strategyId: 'split_tls', strategyName: 'Split TLS' },
        { type: 'vless' },
        { type: 'block' }
      ],
      isCustom: false,
      isEnabled: true,
      priority: 90,
      createdAt: now - 86400,
      updatedAt: now,
    },
    {
      id: 'instagram',
      name: 'Instagram',
      domain: 'instagram.com',
      icon: '📷',
      category: 'social',
      status: 'unknown',
      currentMethod: { type: 'auto' },
      availableMethods: [
        { type: 'direct' },
        { type: 'auto' },
        { type: 'vless' },
        { type: 'block' }
      ],
      isCustom: false,
      isEnabled: true,
      priority: 80,
      createdAt: now - 86400,
      updatedAt: now,
    },
    {
      id: 'spotify',
      name: 'Spotify',
      domain: 'spotify.com',
      icon: '🎵',
      category: 'music',
      status: 'accessible',
      currentMethod: { type: 'direct' },
      availableMethods: [
        { type: 'direct' },
        { type: 'auto' },
        { type: 'vless' },
        { type: 'block' }
      ],
      isCustom: false,
      isEnabled: true,
      ping: 28,
      priority: 50,
      createdAt: now - 86400,
      updatedAt: now,
    }
  ];
}

/**
 * Возвращает демо-пресеты
 */
function getDemoPresets(): LibraryPreset[] {
  const now = Date.now() / 1000;
  return [
    {
      id: 'preset-default',
      name: 'По умолчанию',
      description: 'Базовый набор правил для популярных сервисов',
      rules: [
        {
          id: 'preset-rule-1',
          serviceId: 'youtube',
          pattern: '*.youtube.com',
          method: 'zapret',
          isEnabled: true,
          strategyId: null,
          priority: 100,
          createdAt: now,
          updatedAt: now,
        },
        {
          id: 'preset-rule-2',
          serviceId: 'discord',
          pattern: '*.discord.com',
          method: 'zapret',
          isEnabled: true,
          strategyId: null,
          priority: 100,
          createdAt: now,
          updatedAt: now,
        },
      ],
      isBuiltin: true,
    },
    {
      id: 'preset-vless-all',
      name: 'Всё через VLESS',
      description: 'Направить весь трафик через VLESS прокси',
      rules: [
        {
          id: 'preset-vless-rule',
          serviceId: null,
          pattern: '*',
          method: 'vless',
          isEnabled: true,
          strategyId: null,
          priority: 1,
          createdAt: now,
          updatedAt: now,
        },
      ],
      isBuiltin: true,
    },
    {
      id: 'preset-gaming',
      name: 'Игровой',
      description: 'Оптимизированные правила для игровых сервисов',
      rules: [
        {
          id: 'preset-gaming-discord',
          serviceId: 'discord',
          pattern: '*.discord.com',
          method: 'zapret',
          isEnabled: true,
          strategyId: null,
          priority: 100,
          createdAt: now,
          updatedAt: now,
        },
        {
          id: 'preset-gaming-steam',
          serviceId: 'steam',
          pattern: '*.steampowered.com',
          method: 'direct',
          isEnabled: true,
          strategyId: null,
          priority: 90,
          createdAt: now,
          updatedAt: now,
        },
      ],
      isBuiltin: true,
    },
  ];
}

// ============================================================================
// Library Store
// ============================================================================

class LibraryStore {
  rules = $state<ServiceRule[]>([]);
  presets = $state<LibraryPreset[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  
  // Загружаем сохранённые фильтры из localStorage
  private defaultFilters: LibraryFilters = {
    search: '',
    status: 'all',
    method: 'all',
    category: 'all',
    criticalOnly: false
  };
  
  filters = $state<LibraryFilters>({
    ...this.defaultFilters,
    ...loadFiltersFromStorage()
  });

  // Derived
  filteredRules = $derived.by(() => {
    let result = this.rules;
    
    // Фильтр "Critical only" — только критичные сервисы
    if (this.filters.criticalOnly) {
      result = result.filter(r => 
        CRITICAL_SERVICES.some(critical => 
          r.id.toLowerCase().includes(critical) || 
          r.name.toLowerCase().includes(critical) ||
          r.domain.toLowerCase().includes(critical)
        )
      );
    }
    
    if (this.filters.search) {
      const search = this.filters.search.toLowerCase();
      result = result.filter(r => 
        r.name.toLowerCase().includes(search) || 
        r.domain.toLowerCase().includes(search)
      );
    }
    
    if (this.filters.status !== 'all') {
      result = result.filter(r => r.status === this.filters.status);
    }
    
    if (this.filters.method !== 'all') {
      result = result.filter(r => r.currentMethod.type === this.filters.method);
    }
    
    if (this.filters.category !== 'all') {
      result = result.filter(r => r.category === this.filters.category);
    }
    
    return result;
  });

  categories = $derived([...new Set(this.rules.map(r => r.category))]);
  
  accessibleCount = $derived(this.rules.filter(r => r.status === 'accessible').length);
  blockedCount = $derived(this.rules.filter(r => r.status === 'blocked').length);
  enabledCount = $derived(this.rules.filter(r => r.isEnabled).length);

  /**
   * Загружает правила библиотеки
   * В Tauri режиме — из backend, иначе — demo данные
   */
  async load() {
    this.loading = true;
    this.error = null;
    try {
      this.rules = await fetchRules();
    } catch (e) {
      console.error('[Library] Failed to load rules:', e);
      this.error = String(e);
      // Fallback to demo data on error
      this.rules = getDemoRules();
    } finally {
      this.loading = false;
    }
  }

  /**
   * Загружает пресеты библиотеки
   */
  async loadPresets() {
    try {
      this.presets = await fetchPresets();
    } catch (e) {
      console.error('[Library] Failed to load presets:', e);
      this.presets = getDemoPresets();
    }
  }

  /**
   * Устанавливает метод доступа для правила
   */
  async setRuleMethod(ruleId: string, method: AccessMethod) {
    try {
      const { method: backendMethod } = frontendMethodToBackend(method);
      
      if (isTauri() && await isBackendReady()) {
        const updatedRule = await setRuleMethodInBackend(ruleId, backendMethod);
        // Обновляем локальное состояние с данными из backend
        this.rules = this.rules.map(r => 
          r.id === ruleId ? backendRuleToServiceRule(updatedRule) : r
        );
      } else {
        // Demo режим: обновляем только локально
        this.rules = this.rules.map(r => 
          r.id === ruleId ? { ...r, currentMethod: method } : r
        );
      }
    } catch (e) {
      console.error('[Library] Failed to set method:', e);
      // В случае ошибки всё равно обновляем UI
      this.rules = this.rules.map(r => 
        r.id === ruleId ? { ...r, currentMethod: method } : r
      );
    }
  }

  /**
   * Включает/выключает правило
   */
  async toggleRule(ruleId: string, enabled: boolean) {
    try {
      if (isTauri() && await isBackendReady()) {
        const updatedRule = await toggleRuleInBackend(ruleId, enabled);
        this.rules = this.rules.map(r => 
          r.id === ruleId ? backendRuleToServiceRule(updatedRule) : r
        );
      } else {
        // Demo режим
        this.rules = this.rules.map(r => 
          r.id === ruleId ? { ...r, isEnabled: enabled } : r
        );
      }
    } catch (e) {
      console.error('[Library] Failed to toggle rule:', e);
      // Обновляем UI даже при ошибке
      this.rules = this.rules.map(r => 
        r.id === ruleId ? { ...r, isEnabled: enabled } : r
      );
    }
  }

  /**
   * Проверяет доступность сервиса
   */
  async checkRule(ruleId: string) {
    const rule = this.rules.find(r => r.id === ruleId);
    if (!rule) return;

    // Устанавливаем статус "checking"
    this.rules = this.rules.map(r => 
      r.id === ruleId ? { ...r, status: 'checking' as ServiceStatus } : r
    );

    try {
      const result = await checkService(rule.domain);
      
      this.rules = this.rules.map(r => 
        r.id === ruleId ? { 
          ...r, 
          status: result.status,
          lastChecked: Date.now(),
          ping: result.ping
        } : r
      );
    } catch (e) {
      console.error('[Library] Failed to check rule:', e);
      this.rules = this.rules.map(r => 
        r.id === ruleId ? { ...r, status: 'unknown' as ServiceStatus, lastChecked: Date.now() } : r
      );
    }
  }

  /**
   * Добавляет новое правило
   */
  async addRule(domain: string, name: string, category: string, method: AccessMethod) {
    try {
      const input = serviceRuleToCreateInput({
        domain,
        currentMethod: method,
        priority: 0,
      });
      
      const backendRule = await addRuleToBackend(input);
      const newRule = backendRuleToServiceRule(backendRule);
      
      // Переопределяем имя и категорию, если указаны пользователем
      newRule.name = name || newRule.name;
      newRule.category = category || newRule.category;
      newRule.isCustom = true;
      
      this.rules = [...this.rules, newRule];
    } catch (e) {
      console.error('[Library] Failed to add rule:', e);
      throw e;
    }
  }

  /**
   * Удаляет правило
   */
  async removeRule(ruleId: string) {
    try {
      await deleteRuleFromBackend(ruleId);
      this.rules = this.rules.filter(r => r.id !== ruleId);
    } catch (e) {
      console.error('[Library] Failed to remove rule:', e);
      throw e;
    }
  }

  /**
   * Применяет пресет
   */
  async applyPreset(presetId: string) {
    try {
      const backendRules = await applyPresetInBackend(presetId);
      this.rules = backendRules.map(backendRuleToServiceRule);
    } catch (e) {
      console.error('[Library] Failed to apply preset:', e);
      throw e;
    }
  }

  /**
   * Обновляет правило
   */
  async updateRule(ruleId: string, updates: Partial<Pick<ServiceRule, 'domain' | 'currentMethod' | 'priority' | 'isEnabled'>>) {
    try {
      const input: UpdateRuleInput = {
        id: ruleId,
      };
      
      if (updates.domain !== undefined) {
        input.pattern = updates.domain.includes('*') ? updates.domain : `*.${updates.domain}`;
      }
      if (updates.currentMethod !== undefined) {
        const { method, strategyId } = frontendMethodToBackend(updates.currentMethod);
        input.method = method;
        input.strategyId = strategyId ?? null;
      }
      if (updates.priority !== undefined) {
        input.priority = updates.priority;
      }
      if (updates.isEnabled !== undefined) {
        input.isEnabled = updates.isEnabled;
      }
      
      if (isTauri() && await isBackendReady()) {
        const updatedRule = await updateRuleInBackend(input);
        this.rules = this.rules.map(r => 
          r.id === ruleId ? backendRuleToServiceRule(updatedRule) : r
        );
      } else {
        // Demo режим: обновляем локально
        this.rules = this.rules.map(r => {
          if (r.id !== ruleId) return r;
          return {
            ...r,
            domain: updates.domain ?? r.domain,
            currentMethod: updates.currentMethod ?? r.currentMethod,
            priority: updates.priority ?? r.priority,
            isEnabled: updates.isEnabled ?? r.isEnabled,
            updatedAt: Date.now() / 1000,
          };
        });
      }
    } catch (e) {
      console.error('[Library] Failed to update rule:', e);
      throw e;
    }
  }

  /**
   * Получает правило по ID
   */
  getRule(ruleId: string): ServiceRule | undefined {
    return this.rules.find(r => r.id === ruleId);
  }

  setFilter<K extends keyof LibraryFilters>(key: K, value: LibraryFilters[K]) {
    this.filters = { ...this.filters, [key]: value };
    saveFiltersToStorage(this.filters);
  }

  clearFilters() {
    this.filters = { search: '', status: 'all', method: 'all', category: 'all', criticalOnly: false };
    saveFiltersToStorage(this.filters);
  }
  
  /**
   * Переключает фильтр "Critical only"
   */
  toggleCriticalOnly() {
    this.filters = { ...this.filters, criticalOnly: !this.filters.criticalOnly };
    saveFiltersToStorage(this.filters);
  }
}

export const libraryStore = new LibraryStore();

// Экспорт вспомогательных функций для использования в других модулях
export { isTauri, isBackendReady, checkService };

// Экспорт конвертеров для использования в других модулях
export { 
  backendMethodToFrontend, 
  frontendMethodToBackend, 
  backendRuleToServiceRule,
  serviceRuleToCreateInput 
};
