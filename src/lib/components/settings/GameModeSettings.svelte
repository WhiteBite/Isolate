<script lang="ts">
  /**
   * GameModeSettings Component
   * 
   * UI for configuring Game Mode with auto-detection of games.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import Toggle from '$lib/components/Toggle.svelte';
  import { gameModeStore, type GameInfo } from '$lib/stores/gameMode.svelte';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // Local state
  let newProcessName = $state('');
  let newGameName = $state('');
  let checkInterval = $state(10);
  let gameAction = $state<'switch_profile' | 'notify_only'>('switch_profile');
  let message = $state<{ text: string; type: 'success' | 'error' | 'info' } | null>(null);

  // Derived from store
  let autoDetect = $derived(gameModeStore.autoDetect);
  let customGames = $derived(gameModeStore.customGames);

  // Interval options
  const intervalOptions = [
    { value: 5, label: '5 секунд' },
    { value: 10, label: '10 секунд' },
    { value: 30, label: '30 секунд' },
  ];

  // Action options
  const actionOptions = [
    { value: 'switch_profile', label: 'Переключить на Gaming профиль' },
    { value: 'notify_only', label: 'Только уведомить' },
  ];

  function handleToggleAutoDetect(value: boolean) {
    gameModeStore.setAutoDetect(value);
    showMessage(value ? 'Автодетект включён' : 'Автодетект выключен', 'info');
  }

  function handleAddGame() {
    const processName = newProcessName.trim();
    
    if (!processName) {
      showMessage('Введите имя процесса', 'error');
      return;
    }

    // Validate process name format
    if (!processName.toLowerCase().endsWith('.exe')) {
      showMessage('Имя процесса должно заканчиваться на .exe', 'error');
      return;
    }

    // Check if already exists
    const allGames = gameModeStore.getAllGames();
    if (allGames.some(g => g.processName.toLowerCase() === processName.toLowerCase())) {
      showMessage('Эта игра уже добавлена', 'error');
      return;
    }

    const game: GameInfo = {
      name: newGameName.trim() || processName.replace('.exe', ''),
      processName: processName,
    };

    gameModeStore.addCustomGame(game);
    newProcessName = '';
    newGameName = '';
    showMessage(`Игра "${game.name}" добавлена`, 'success');
  }

  function handleRemoveGame(processName: string) {
    const game = customGames.find(g => g.processName === processName);
    gameModeStore.removeCustomGame(processName);
    if (game) {
      showMessage(`Игра "${game.name}" удалена`, 'info');
    }
  }

  function handleIntervalChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    checkInterval = parseInt(select.value);
    // TODO: Save to store when backend supports it
  }

  function handleActionChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    gameAction = select.value as 'switch_profile' | 'notify_only';
    // TODO: Save to store when backend supports it
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleAddGame();
    }
  }

  function showMessage(text: string, type: 'success' | 'error' | 'info') {
    message = { text, type };
    setTimeout(() => { message = null; }, 3000);
  }
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-text-primary">Game Mode</h2>
    {#if message}
      <span class="text-sm animate-pulse {message.type === 'error' ? 'text-red-400' : message.type === 'success' ? 'text-emerald-400' : 'text-indigo-400'}">
        {message.text}
      </span>
    {/if}
  </div>

  <div class="space-y-6">
    <!-- Auto-detect Toggle -->
    <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-text-primary font-medium">Автоматический Game Mode</p>
          <p class="text-text-secondary text-sm">
            Автоматически определять запущенные игры и активировать игровой режим
          </p>
        </div>
        <Toggle 
          checked={autoDetect}
          onchange={handleToggleAutoDetect}
        />
      </div>
    </div>

    {#if autoDetect}
      <!-- Check Interval -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <label class="block">
          <span class="text-text-primary font-medium mb-2 block">Интервал проверки</span>
          <p class="text-text-secondary text-sm mb-3">
            Как часто проверять запущенные процессы
          </p>
          <select
            value={checkInterval}
            onchange={handleIntervalChange}
            class="w-full px-4 py-2.5 bg-zinc-900/50 border border-white/10 rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
          >
            {#each intervalOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </label>
      </div>

      <!-- Action on Detection -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <label class="block">
          <span class="text-text-primary font-medium mb-2 block">Действие при обнаружении игры</span>
          <p class="text-text-secondary text-sm mb-3">
            Что делать когда обнаружена запущенная игра
          </p>
          <select
            value={gameAction}
            onchange={handleActionChange}
            class="w-full px-4 py-2.5 bg-zinc-900/50 border border-white/10 rounded-lg text-text-primary focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
          >
            {#each actionOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </label>
      </div>

      <!-- Custom Games List -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <div class="mb-4">
          <p class="text-text-primary font-medium">Отслеживаемые игры</p>
          <p class="text-text-secondary text-sm">
            Добавьте свои игры для автоматического определения
          </p>
        </div>

        <!-- Add new game form -->
        <div class="flex flex-col gap-3 mb-4">
          <div class="flex gap-2">
            <input
              type="text"
              bind:value={newProcessName}
              onkeydown={handleKeydown}
              placeholder="Имя процесса (например: cs2.exe)"
              class="flex-1 px-4 py-2.5 bg-zinc-900/50 border border-white/10 rounded-lg text-text-primary placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
            />
            <button
              onclick={handleAddGame}
              disabled={!newProcessName.trim()}
              class="px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 disabled:bg-indigo-500/30 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors flex items-center gap-2"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
              </svg>
              Добавить
            </button>
          </div>
          <input
            type="text"
            bind:value={newGameName}
            onkeydown={handleKeydown}
            placeholder="Название игры (опционально)"
            class="w-full px-4 py-2.5 bg-zinc-900/50 border border-white/10 rounded-lg text-text-primary placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
          />
        </div>

        <!-- Custom games list -->
        {#if customGames.length > 0}
          <div class="space-y-2">
            <p class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Пользовательские игры</p>
            {#each customGames as game}
              <div class="flex items-center justify-between p-3 bg-zinc-900/50 rounded-lg border border-white/10">
                <div class="flex items-center gap-3">
                  <span class="text-lg">🎮</span>
                  <div>
                    <p class="text-text-primary">{game.name}</p>
                    <p class="text-text-secondary text-xs font-mono">{game.processName}</p>
                  </div>
                </div>
                <button
                  onclick={() => handleRemoveGame(game.processName)}
                  class="p-2 text-zinc-500 hover:text-red-400 transition-colors"
                  title="Удалить"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                  </svg>
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <div class="text-center py-4 text-zinc-500">
            <p class="text-sm">Нет пользовательских игр</p>
            <p class="text-xs mt-1">Встроенный список включает популярные игры</p>
          </div>
        {/if}
      </div>

      <!-- Built-in games info -->
      <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
        <p class="text-indigo-400 text-sm flex items-start gap-2">
          <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          <span>
            Встроенный список включает популярные игры: CS2, Dota 2, Valorant, League of Legends, 
            Fortnite, Apex Legends, PUBG, Overwatch 2, Minecraft, GTA V, Rust, Escape from Tarkov, 
            World of Warcraft, Genshin Impact и Discord.
          </span>
        </p>
      </div>
    {/if}
  </div>
</div>
