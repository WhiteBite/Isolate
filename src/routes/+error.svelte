<script lang="ts">
  import { page } from '$app/stores';
</script>

<div class="min-h-screen bg-[#0a0e27] flex items-center justify-center p-8">
  <div class="bg-[#1a1f3a] rounded-2xl p-8 border border-[#ff3333]/30 max-w-2xl w-full">
    <div class="flex items-center gap-4 mb-6">
      <div class="w-12 h-12 rounded-full bg-[#ff3333]/20 flex items-center justify-center">
        <svg class="w-6 h-6 text-[#ff3333]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
        </svg>
      </div>
      <div>
        <h1 class="text-2xl font-bold text-[#ff3333]">Ошибка {$page.status}</h1>
        <p class="text-[#a0a0a0]">Что-то пошло не так</p>
      </div>
    </div>
    
    <div class="bg-[#0a0e27] rounded-lg p-4 mb-6 font-mono text-sm overflow-auto max-h-64">
      <p class="text-[#ff6666] whitespace-pre-wrap">{$page.error?.message || 'Unknown error'}</p>
      {#if $page.error?.stack}
        <details class="mt-4">
          <summary class="text-[#a0a0a0] cursor-pointer hover:text-white">Stack trace</summary>
          <pre class="text-[#666] mt-2 text-xs whitespace-pre-wrap">{$page.error.stack}</pre>
        </details>
      {/if}
    </div>
    
    <div class="flex gap-3">
      <a href="/" class="px-4 py-2 bg-[#00d4ff] hover:bg-[#00b8e0] text-white rounded-lg font-medium transition-colors">
        На главную
      </a>
      <button 
        onclick={() => window.location.reload()}
        class="px-4 py-2 bg-[#2a2f4a] hover:bg-[#3a3f5a] text-white rounded-lg font-medium transition-colors"
      >
        Перезагрузить
      </button>
    </div>
  </div>
</div>
