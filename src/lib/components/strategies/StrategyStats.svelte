<script lang="ts">
  interface Props {
    totalStrategies: number;
    filteredCount: number;
    zapretCount: number;
    vlessCount: number;
  }

  let { totalStrategies, filteredCount, zapretCount, vlessCount }: Props = $props();
</script>

<div class="bg-void-50 rounded-xl p-4 border border-glass-border">
  <div class="flex items-center justify-between text-sm">
    <div class="flex items-center gap-6">
      <span class="text-text-muted">
        Total: <span class="text-text-primary font-medium">{totalStrategies}</span>
      </span>
      <span class="text-text-muted">
        Shown: <span class="text-text-primary font-medium">{filteredCount}</span>
      </span>
      <span class="text-text-muted flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-neon-cyan"></span>
        Zapret: <span class="text-neon-cyan font-medium">{zapretCount}</span>
      </span>
      <span class="text-text-muted flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-neon-green"></span>
        VLESS: <span class="text-neon-green font-medium">{vlessCount}</span>
      </span>
    </div>
    <a 
      href="/testing"
      class="text-indigo-400 hover:text-indigo-300 flex items-center gap-1 transition-colors"
    >
      Testing
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
      </svg>
    </a>
  </div>
</div>
