<script lang="ts">
  import { proxyImportStore } from '$lib/stores/proxyImport.svelte';
  import ImportPreview from './ImportPreview.svelte';
  
  interface Props {
    onImport?: (proxy: ParsedProxy) => void;
    onCancel?: () => void;
  }
  
  interface ParsedProxy {
    protocol: string;
    name: string;
    server: string;
    port: number;
    uuid?: string;
    password?: string;
    method?: string;
    security?: string;
    sni?: string;
    raw: string;
  }
  
  let { onImport, onCancel }: Props = $props();
  
  let isDragging = $state(false);
  let textareaValue = $state('');
  
  // Sync with store
  $effect(() => {
    if (textareaValue !== proxyImportStore.rawInput) {
      proxyImportStore.parseInput(textareaValue);
    }
  });
  
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }
  
  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
  }
  
  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    
    const text = e.dataTransfer?.getData('text/plain');
    if (text) {
      textareaValue = text;
    }
  }
  
  function handlePaste(e: ClipboardEvent) {
    // Let the default paste happen, textarea will update
  }
  
  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    textareaValue = target.value;
  }
  
  function handleImport() {
    if (proxyImportStore.parsedProxy && proxyImportStore.isValid) {
      onImport?.(proxyImportStore.parsedProxy);
      handleClear();
    }
  }
  
  function handleClear() {
    textareaValue = '';
    proxyImportStore.clear();
    onCancel?.();
  }
  
  // Detect protocol from input
  let detectedProtocol = $derived.by(() => {
    const input = textareaValue.trim().toLowerCase();
    if (input.startsWith('vless://')) return 'VLESS';
    if (input.startsWith('vmess://')) return 'VMess';
    if (input.startsWith('ss://')) return 'Shadowsocks';
    if (input.startsWith('trojan://')) return 'Trojan';
    return null;
  });
</script>

<div class="space-y-4">
  <!-- Drop zone -->
  <div
    class="relative rounded-xl border-2 border-dashed transition-all duration-200
           {isDragging 
             ? 'border-blue-500 bg-blue-500/10' 
             : 'border-white/20 hover:border-white/30 bg-white/5'}"
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-white/10">
      <div class="flex items-center gap-2">
        <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
        </svg>
        <span class="text-sm font-medium text-white">Import Proxy</span>
        {#if detectedProtocol}
          <span class="px-2 py-0.5 text-xs font-medium rounded bg-blue-500/20 text-blue-400">
            {detectedProtocol}
          </span>
        {/if}
      </div>
      
      {#if textareaValue}
        <button
          onclick={handleClear}
          class="text-xs text-gray-400 hover:text-white transition-colors"
        >
          Clear
        </button>
      {/if}
    </div>
    
    <!-- Textarea -->
    <textarea
      value={textareaValue}
      oninput={handleInput}
      onpaste={handlePaste}
      placeholder="Paste your proxy link here...&#10;&#10;Supported formats:&#10;• vless://...&#10;• vmess://...&#10;• ss://...&#10;• trojan://..."
      class="w-full h-32 px-4 py-3 bg-transparent text-white placeholder-gray-500 
             text-sm font-mono resize-none focus:outline-none"
      spellcheck="false"
    ></textarea>
    
    <!-- Drag overlay -->
    {#if isDragging}
      <div class="absolute inset-0 flex items-center justify-center bg-blue-500/10 rounded-xl">
        <div class="text-center">
          <svg class="w-12 h-12 mx-auto mb-2 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          <p class="text-blue-400 font-medium">Drop to import</p>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Error message -->
  {#if proxyImportStore.error}
    <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20">
      <svg class="w-4 h-4 text-red-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <span class="text-sm text-red-400">{proxyImportStore.error}</span>
    </div>
  {/if}
  
  <!-- Preview -->
  {#if proxyImportStore.parsedProxy && proxyImportStore.isValid}
    <ImportPreview 
      proxy={proxyImportStore.parsedProxy}
      onConfirm={handleImport}
      onCancel={handleClear}
    />
  {:else if textareaValue && !proxyImportStore.error}
    <!-- Import button when no preview yet -->
    <div class="flex justify-end gap-2">
      <button
        onclick={handleClear}
        class="px-4 py-2 text-sm font-medium rounded-lg
               text-gray-400 hover:text-white hover:bg-white/10
               transition-colors"
      >
        Cancel
      </button>
      <button
        onclick={handleImport}
        disabled={!proxyImportStore.isValid}
        class="px-4 py-2 text-sm font-medium rounded-lg
               bg-blue-500 text-white hover:bg-blue-600
               disabled:opacity-50 disabled:cursor-not-allowed
               transition-colors"
      >
        Import
      </button>
    </div>
  {/if}
</div>
