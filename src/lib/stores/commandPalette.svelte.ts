/**
 * Command Palette Store
 * Управление состоянием Command Palette с историей команд
 */

export interface Command {
  id: string;
  label: string;
  description?: string;
  icon?: string;
  shortcut?: string;
  category?: string;
  action: () => void | Promise<void>;
}

export interface RecentCommand {
  id: string;
  label: string;
  timestamp: number;
}

const RECENT_COMMANDS_KEY = 'isolate:recent-commands';
const MAX_RECENT_COMMANDS = 5;

function loadRecentCommands(): RecentCommand[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const stored = localStorage.getItem(RECENT_COMMANDS_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

function saveRecentCommands(commands: RecentCommand[]): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(RECENT_COMMANDS_KEY, JSON.stringify(commands));
  } catch {
    // Ignore storage errors
  }
}

class CommandPaletteStore {
  isOpen = $state(false);
  query = $state('');
  selectedIndex = $state(0);
  recentCommands = $state<RecentCommand[]>(loadRecentCommands());
  registeredCommands = $state<Command[]>([]);

  /**
   * Открыть Command Palette
   */
  open() {
    this.isOpen = true;
    this.query = '';
    this.selectedIndex = 0;
  }

  /**
   * Закрыть Command Palette
   */
  close() {
    this.isOpen = false;
    this.query = '';
    this.selectedIndex = 0;
  }

  /**
   * Переключить состояние
   */
  toggle() {
    if (this.isOpen) {
      this.close();
    } else {
      this.open();
    }
  }

  /**
   * Установить поисковый запрос
   */
  setQuery(value: string) {
    this.query = value;
    this.selectedIndex = 0;
  }

  /**
   * Выбрать следующий элемент
   */
  selectNext(maxIndex: number) {
    this.selectedIndex = Math.min(this.selectedIndex + 1, maxIndex - 1);
  }

  /**
   * Выбрать предыдущий элемент
   */
  selectPrevious() {
    this.selectedIndex = Math.max(this.selectedIndex - 1, 0);
  }

  /**
   * Добавить команду в историю
   */
  addRecent(command: { id: string; label: string }) {
    const newRecent: RecentCommand = {
      id: command.id,
      label: command.label,
      timestamp: Date.now()
    };

    this.recentCommands = [
      newRecent,
      ...this.recentCommands.filter(c => c.id !== command.id)
    ].slice(0, MAX_RECENT_COMMANDS);

    saveRecentCommands(this.recentCommands);
  }

  /**
   * Очистить историю команд
   */
  clearRecent() {
    this.recentCommands = [];
    saveRecentCommands([]);
  }

  /**
   * Зарегистрировать команды
   */
  registerCommands(commands: Command[]) {
    this.registeredCommands = commands;
  }

  /**
   * Добавить команду
   */
  addCommand(command: Command) {
    if (!this.registeredCommands.find(c => c.id === command.id)) {
      this.registeredCommands = [...this.registeredCommands, command];
    }
  }

  /**
   * Удалить команду
   */
  removeCommand(id: string) {
    this.registeredCommands = this.registeredCommands.filter(c => c.id !== id);
  }

  /**
   * Выполнить команду
   */
  async executeCommand(command: Command) {
    this.addRecent({ id: command.id, label: command.label });
    this.close();
    await command.action();
  }

  /**
   * Получить отфильтрованные команды
   */
  getFilteredCommands(): Command[] {
    if (!this.query.trim()) {
      return this.registeredCommands;
    }

    const lowerQuery = this.query.toLowerCase();
    return this.registeredCommands.filter(cmd => 
      cmd.label.toLowerCase().includes(lowerQuery) ||
      cmd.description?.toLowerCase().includes(lowerQuery) ||
      cmd.category?.toLowerCase().includes(lowerQuery)
    );
  }

  /**
   * Получить команды сгруппированные по категориям
   */
  getGroupedCommands(): Map<string, Command[]> {
    const filtered = this.getFilteredCommands();
    const grouped = new Map<string, Command[]>();

    for (const cmd of filtered) {
      const category = cmd.category || 'Другое';
      const existing = grouped.get(category) || [];
      grouped.set(category, [...existing, cmd]);
    }

    return grouped;
  }
}

export const commandPaletteStore = new CommandPaletteStore();

/**
 * Глобальный обработчик клавиш для Command Palette
 * Вызывать в корневом layout
 */
export function setupCommandPaletteShortcuts() {
  if (typeof window === 'undefined') return () => {};

  function handleKeydown(e: KeyboardEvent) {
    // Cmd/Ctrl + K для открытия
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      commandPaletteStore.toggle();
    }

    // Escape для закрытия
    if (e.key === 'Escape' && commandPaletteStore.isOpen) {
      e.preventDefault();
      commandPaletteStore.close();
    }
  }

  window.addEventListener('keydown', handleKeydown);
  return () => window.removeEventListener('keydown', handleKeydown);
}
