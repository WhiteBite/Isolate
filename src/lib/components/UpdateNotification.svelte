<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-shell';
  
  interface GitHubUpdateInfo {
    version: string;
    downloadUrl: string;
    releaseNotes: string | null;
    publishedAt: string | null;
  }
  
  let updateInfo = $state<GitHubUpdateInfo | null>(null);
  let checking = $state(false);
  let dismissed = $state(false);
  let showNotes = $state(false);
  
  async function checkForUpdates() {
    if (checking) return;
    checking = true;
    
    try {
      const result = await invoke<GitHubUpdateInfo | null>('check_github_updates');
      updateInfo = result;
    } catch (e) {
      console.error('Failed to check for updates:', e);
    } finally {
      checking = false;
    }
  }
  
  async function openDownloadPage() {
    if (updateInfo?.downloadUrl) {
      await open(updateInfo.downloadUrl);
    }
  }
  
  function dismiss() {
    dismissed = true;
  }
  
  // Check on mount
  $effect(() => {
    checkForUpdates();
  });
</script>

{#if updateInfo && !dismissed}
  <div class="fixed bottom-4 right-4 z-50 max-w-sm animate-slide-up">
    <div class="bg-zinc-800 border border-emerald-500/30 rounded-lg shadow-xl p-4">
      <div class="flex items-start gap-3">
        <div class="flex-shrink-0 w-10 h-10 bg-emerald-500/20 rounded-full flex items-center justify-center">
          <svg class="w-5 h-5 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
        </div>
        
        <div class="flex-1 min-w-0">
          <h3 class="text-sm font-medium text-white">
            Доступна новая версия
          </h3>
          <p class="text-sm text-zinc-400 mt-1">
            Версия {updateInfo.version}
            {#if updateInfo.publishedAt}
              <span class="text-zinc-500">• {new Date(updateInfo.publishedAt).toLocaleDateString()}</span>
            {/if}
          </p>
          
          {#if updateInfo.releaseNotes && showNotes}
            <div class="mt-2 p-2 bg-zinc-900 rounded text-xs text-zinc-400 max-h-32 overflow-y-auto">
              {updateInfo.releaseNotes}
            </div>
          {/if}
          
          <div class="flex items-center gap-2 mt-3">
            <button
              onclick={openDownloadPage}
              class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded transition-colors"
            >
              Скачать с GitHub
            </button>
            
            {#if updateInfo.releaseNotes}
              <button
                onclick={() => showNotes = !showNotes}
                class="px-3 py-1.5 text-zinc-400 hover:text-white text-sm transition-colors"
              >
                {showNotes ? 'Скрыть' : 'Что нового'}
              </button>
            {/if}
          </div>
        </div>
        
        <button
          onclick={dismiss}
          class="flex-shrink-0 text-zinc-500 hover:text-zinc-300 transition-colors"
          aria-label="Закрыть"
        >
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes slide-up {
    from {
      opacity: 0;
      transform: translateY(1rem);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .animate-slide-up {
    animation: slide-up 0.3s ease-out;
  }
</style>
