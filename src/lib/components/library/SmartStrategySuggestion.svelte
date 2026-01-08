<script lang="ts">
  interface Props {
    domain: string;
    onApply?: (strategyId: string) => void;
  }

  interface StrategySuggestion {
    id: string;
    name: string;
    reason: string;
  }

  let { domain, onApply }: Props = $props();

  // Паттерны для определения категории домена
  const domainPatterns: Array<{
    patterns: RegExp[];
    strategy: StrategySuggestion;
  }> = [
    // YouTube / Google
    {
      patterns: [
        /youtube\.com$/i,
        /youtu\.be$/i,
        /googlevideo\.com$/i,
        /ytimg\.com$/i,
        /ggpht\.com$/i,
        /google\.(com|ru|co\.\w+)$/i,
        /googleapis\.com$/i,
        /gstatic\.com$/i,
      ],
      strategy: {
        id: 'zapret_youtube_google',
        name: 'YouTube Google',
        reason: 'Оптимизирована для Google/YouTube с multisplit фрагментацией'
      }
    },
    // Discord
    {
      patterns: [
        /discord\.com$/i,
        /discord\.gg$/i,
        /discordapp\.com$/i,
        /discordapp\.net$/i,
        /discord\.media$/i,
      ],
      strategy: {
        id: 'discord_multisplit',
        name: 'Discord Multisplit',
        reason: 'Специализированная стратегия для Discord с UDP/QUIC обходом'
      }
    },
    // Telegram
    {
      patterns: [
        /telegram\.org$/i,
        /t\.me$/i,
        /telegram\.me$/i,
        /telesco\.pe$/i,
        /tg\.dev$/i,
      ],
      strategy: {
        id: 'telegram_multisplit',
        name: 'Telegram Multisplit',
        reason: 'Оптимизирована для Telegram с обходом MTProto блокировок'
      }
    },
    // Twitter / X
    {
      patterns: [
        /twitter\.com$/i,
        /x\.com$/i,
        /twimg\.com$/i,
        /t\.co$/i,
      ],
      strategy: {
        id: 'twitter_multisplit',
        name: 'Twitter/X Multisplit',
        reason: 'Стратегия для Twitter/X с SNI фрагментацией'
      }
    },
    // Meta (Instagram, Facebook, WhatsApp)
    {
      patterns: [
        /instagram\.com$/i,
        /facebook\.com$/i,
        /fb\.com$/i,
        /whatsapp\.com$/i,
        /whatsapp\.net$/i,
        /fbcdn\.net$/i,
        /cdninstagram\.com$/i,
        /meta\.com$/i,
      ],
      strategy: {
        id: 'meta_multisplit',
        name: 'Meta Multisplit',
        reason: 'Оптимизирована для Instagram, Facebook и WhatsApp'
      }
    },
    // AI сервисы
    {
      patterns: [
        /openai\.com$/i,
        /anthropic\.com$/i,
        /claude\.ai$/i,
        /chatgpt\.com$/i,
        /perplexity\.ai$/i,
        /gemini\.google\.com$/i,
        /bard\.google\.com$/i,
      ],
      strategy: {
        id: 'ai_multisplit',
        name: 'AI Services Multisplit',
        reason: 'Стратегия для AI-сервисов (OpenAI, Anthropic, etc.)'
      }
    },
    // Gaming
    {
      patterns: [
        /steampowered\.com$/i,
        /steamcommunity\.com$/i,
        /steamstatic\.com$/i,
        /epicgames\.com$/i,
        /unrealengine\.com$/i,
        /riotgames\.com$/i,
        /leagueoflegends\.com$/i,
        /blizzard\.com$/i,
        /battle\.net$/i,
        /ea\.com$/i,
        /origin\.com$/i,
      ],
      strategy: {
        id: 'gaming_multisplit',
        name: 'Gaming Multisplit',
        reason: 'Оптимизирована для игровых платформ (Steam, Epic, Riot)'
      }
    },
    // Streaming
    {
      patterns: [
        /spotify\.com$/i,
        /scdn\.co$/i,
        /netflix\.com$/i,
        /nflxvideo\.net$/i,
        /twitch\.tv$/i,
        /ttvnw\.net$/i,
        /soundcloud\.com$/i,
        /deezer\.com$/i,
      ],
      strategy: {
        id: 'streaming_multisplit',
        name: 'Streaming Multisplit',
        reason: 'Стратегия для стриминговых сервисов (Spotify, Netflix, Twitch)'
      }
    },
  ];

  // Определяем рекомендуемую стратегию на основе домена
  let suggestion = $derived.by(() => {
    if (!domain || domain.trim() === '') {
      return null;
    }

    const normalizedDomain = domain.trim().toLowerCase();

    for (const { patterns, strategy } of domainPatterns) {
      for (const pattern of patterns) {
        if (pattern.test(normalizedDomain)) {
          return strategy;
        }
      }
    }

    // Дефолтная стратегия для неизвестных доменов
    return {
      id: 'general_multisplit',
      name: 'General Multisplit',
      reason: 'Универсальная стратегия для большинства сервисов'
    };
  });

  function handleApply() {
    if (suggestion && onApply) {
      onApply(suggestion.id);
    }
  }
</script>

{#if suggestion}
  <div 
    class="flex items-start gap-3 p-4 bg-amber-500/5 border border-amber-500/20 rounded-xl"
    role="region"
    aria-label="Рекомендация стратегии"
  >
    <!-- Icon -->
    <div class="flex-shrink-0 w-10 h-10 flex items-center justify-center 
                bg-amber-500/10 rounded-lg text-xl">
      💡
    </div>

    <!-- Content -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2 mb-1">
        <span class="text-sm font-medium text-amber-400">
          Рекомендуемая стратегия
        </span>
      </div>
      <h4 class="text-base font-semibold text-white mb-1">
        {suggestion.name}
      </h4>
      <p class="text-sm text-zinc-400 leading-relaxed">
        {suggestion.reason}
      </p>
    </div>

    <!-- Apply button -->
    <button
      type="button"
      class="flex-shrink-0 px-4 py-2 text-sm font-medium text-amber-400 
             bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/30
             rounded-lg transition-colors duration-150"
      onclick={handleApply}
    >
      Применить
    </button>
  </div>
{/if}
