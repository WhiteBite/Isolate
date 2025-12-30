<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import Spinner from '$lib/components/Spinner.svelte';

  // Types
  interface AppSettings {
    autoStart: boolean;
    minimizeToTray: boolean;
    language: 'ru' | 'en';
    systemProxy: boolean;
    tunMode: boolean;
    blockQuic: boolean;
    perDomainRouting: boolean;
    perAppRouting: boolean;
    testTimeout: number;
    testServices: string[];
  }

  interface Service {
    id: string;
    name: string;
  }

  // State
  let settings = $state<AppSettings>({
    autoStart: false,
    minimizeToTray: true,
    language: 'ru',
    systemProxy: false,
    tunMode: false,
    blockQuic: true,
    perDomainRouting: false,
    perAppRouting: false,
    testTimeout: 5,
    testServices: ['discord', 'youtube']
  });

  let services = $state<Service[]>([
    { id: 'discord', name: 'Discord' },
    { id: 'youtube', name: 'YouTube' },
    { id: 'telegram', name: 'Telegram' },
    { id: 'twitch', name: 'Twitch' },
    { id: 'spotify', name: 'Spotify' }
  ]);

  let appVersion = $state('0.1.0');
  let checkingUpdates = $state(false);
  let exportingLogs = $state(false);
  let updateStatus = $state<string | null>(null);
  let saving = $state(false);
</script>
