<script lang="ts">
  interface Props {
    value: string;
    onchange?: (value: string) => void;
    readonly?: boolean;
    height?: string;
  }
  
  let { value = $bindable(), onchange, readonly = false, height = '300px' }: Props = $props();
  
  let error = $state<string | null>(null);
  
  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    value = target.value;
    
    // Validate JSON
    try {
      JSON.parse(value);
      error = null;
    } catch (e) {
      error = (e as Error).message;
    }
    
    onchange?.(value);
  }
  
  function formatJson() {
    try {
      const parsed = JSON.parse(value);
      value = JSON.stringify(parsed, null, 2);
      error = null;
      onchange?.(value);
    } catch {}
  }
  
  function minifyJson() {
    try {
      const parsed = JSON.parse(value);
      value = JSON.stringify(parsed);
      error = null;
      onchange?.(value);
    } catch {}
  }
  
  function copyToClipboard() {
    navigator.clipboard.writeText(value);
  }
</script>

<div class="relative">
  <div class="flex items-center justify-between mb-2">
    <span class="text-xs text-zinc-400 font-medium">JSON Editor</span>
    <div class="flex items-center gap-2">
      <button
        type="button"
        onclick={copyToClipboard}
        class="text-xs text-zinc-400 hover:text-zinc-300 transition-colors"
        title="Copy to clipboard"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      </button>
      <button
        type="button"
        onclick={minifyJson}
        class="text-xs text-zinc-400 hover:text-zinc-300 transition-colors"
        title="Minify"
      >
        Minify
      </button>
      <button
        type="button"
        onclick={formatJson}
        class="text-xs text-indigo-400 hover:text-indigo-300 transition-colors"
        title="Format JSON"
      >
        Format
      </button>
    </div>
  </div>
  
  <div class="relative">
    <textarea
      {value}
      oninput={handleInput}
      readonly={readonly}
      spellcheck="false"
      class="w-full p-4 bg-zinc-900/60 border rounded-xl font-mono text-sm
             text-zinc-100 resize-none focus:outline-none focus:border-indigo-500/50
             focus:ring-1 focus:ring-indigo-500/20 transition-all duration-200
             {error ? 'border-red-500/50' : 'border-white/10'}"
      style="height: {height}; tab-size: 2;"
    ></textarea>
    
    <!-- Line numbers overlay hint -->
    {#if !error}
      <div class="absolute top-4 right-4 text-xs text-zinc-600">
        {value.split('\n').length} lines
      </div>
    {/if}
  </div>
  
  {#if error}
    <div class="mt-2 p-2 bg-red-500/10 border border-red-500/20 rounded-lg">
      <p class="text-xs text-red-400 font-mono">{error}</p>
    </div>
  {:else if value.trim()}
    <div class="mt-2 flex items-center gap-2">
      <div class="w-2 h-2 rounded-full bg-emerald-400"></div>
      <span class="text-xs text-emerald-400">Valid JSON</span>
    </div>
  {/if}
</div>
