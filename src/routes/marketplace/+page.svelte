<script lang="ts">
  import { toasts } from '$lib/stores/toast';

  interface Plugin {
    id: string;
    name: string;
    description: string;
    author: string;
    version: string;
    downloads: number;
    rating: number;
    installed: boolean;
    category: 'bypass' | 'utility' | 'monitoring';
  }

  let plugins = $state<Plugin[]>([
    {
      id: 'discord-fix',
      name: 'Discord Fix',
      description: 'Исправляет проблемы с голосовыми каналами Discord',
      author: 'Isolate Team',
      version: '1.0.0',
      downloads: 15420,
      rating: 4.8,
      installed: true,
      category: 'bypass'
    },
    {
      id: 'speed-test',
      name: 'Speed Test',
      description: 'Тестирование скорости соединения',
      author: 'Isolate Team',
      version: '1.0.0',
      downloads: 12350,
      rating: 4.6,
      installed: true,
      category: 'utility'
    },
    {
      id: 'youtube-optimizer',
      name: 'YouTube Optimizer',
      description: 'Оптимизация качества видео на YouTube',
      author: 'Community',
      version: '0.9.2',
      downloads: 8920,
      rating: 4.3,
      installed: false,
      category: 'bypass'
    },
    {
      id: 'traffic-monitor',
      name: 'Traffic Monitor',
      description: 'Мониторинг сетевого трафика в реальном времени',
      author: 'Community',
      version: '1.2.0',
      downloads: 6540,
      rating: 4.5,
      installed: false,
      category: 'monitoring'
    }
  ]);

  let searchQuery = $state('');
  let selectedCategory = $state<string>('all');

  let filteredPlugins = $derived(
    plugins.filter(p => {
      const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           p.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesCategory = selectedCategory === 'all' || p.category === selectedCategory;
      return matchesSearch && matchesCategory;
    })
  );

  function installPlugin(id: string) {
    const plugin = plugins.find(p => p.id === id);
    if (plugin) {
      plugin.installed = true;
      plugins = [...plugins];
      toasts.success(`${plugin.name} установлен`);
    }
  }

  function getCategoryLabel(cat: string): string {
    const labels: Record<string, string> = {
      bypass: 'Обход',
      utility: 'Утилиты',
      monitoring: 'Мониторинг'
    };
    return labels[cat] || cat;
  }
</script>

<div class="p-6 max-w-6xl mx-auto">
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-2xl font-bold text-zinc-100">Marketplace</h1>
      <p class="text-zinc-500 mt-1">Расширения и плагины для Isolate</p>
    </div>
  </div>

  <!-- Filters -->
  <div class="flex gap-4 mb-6">
    <div class="flex-1">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Поиск плагинов..."
        class="w-full px-4 py-2.5 bg-zinc-900/60 border border-white/10 rounded-xl
               text-zinc-100 placeholder-zinc-500
               focus:outline-none focus:border-indigo-500/50"
      />
    </div>
    <select
      bind:value={selectedCategory}
      class="px-4 py-2.5 bg-zinc-900/60 border border-white/10 rounded-xl
             text-zinc-100 focus:outline-none focus:border-indigo-500/50"
    >
      <option value="all">Все категории</option>
      <option value="bypass">Обход</option>
      <option value="utility">Утилиты</option>
      <option value="monitoring">Мониторинг</option>
    </select>
  </div>

  <!-- Plugin Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    {#each filteredPlugins as plugin (plugin.id)}
      <div class="p-5 bg-zinc-900/40 border border-white/5 rounded-xl hover:border-white/10 transition-colors">
        <div class="flex items-start justify-between mb-3">
          <div>
            <h3 class="text-lg font-semibold text-zinc-100">{plugin.name}</h3>
            <p class="text-sm text-zinc-500">{plugin.author} • v{plugin.version}</p>
          </div>
          <span class="px-2 py-1 text-xs rounded-lg bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
            {getCategoryLabel(plugin.category)}
          </span>
        </div>
        
        <p class="text-sm text-zinc-400 mb-4">{plugin.description}</p>
        
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4 text-xs text-zinc-500">
            <span class="flex items-center gap-1">
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
              </svg>
              {plugin.downloads.toLocaleString()}
            </span>
            <span class="flex items-center gap-1">
              <svg class="w-4 h-4 text-amber-400" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
              </svg>
              {plugin.rating}
            </span>
          </div>
          
          {#if plugin.installed}
            <span class="px-3 py-1.5 text-sm rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              Установлен
            </span>
          {:else}
            <button
              onclick={() => installPlugin(plugin.id)}
              class="px-3 py-1.5 text-sm rounded-lg bg-indigo-500 text-white
                     hover:bg-indigo-600 transition-colors"
            >
              Установить
            </button>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  {#if filteredPlugins.length === 0}
    <div class="text-center py-12">
      <p class="text-zinc-500">Плагины не найдены</p>
    </div>
  {/if}
</div>
