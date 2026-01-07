<script lang="ts">
  interface Props {
    totalRules: number;
    activeRules: number;
    proxiedRules: number;
    blockedRules: number;
  }

  let { totalRules, activeRules, proxiedRules, blockedRules }: Props = $props();
</script>

<div class="grid grid-cols-4 gap-4 mb-8">
  <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
    <div class="text-xs text-zinc-400 uppercase tracking-wider mb-1">Total Rules</div>
    <div class="text-2xl font-bold text-white">{totalRules}</div>
  </div>
  <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
    <div class="text-xs text-zinc-400 uppercase tracking-wider mb-1">Active</div>
    <div class="text-2xl font-bold text-emerald-400">{activeRules}</div>
  </div>
  <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
    <div class="text-xs text-zinc-400 uppercase tracking-wider mb-1">Via Proxy</div>
    <div class="text-2xl font-bold text-indigo-400">{proxiedRules}</div>
  </div>
  <div class="p-4 bg-zinc-900/40 border border-white/5 rounded-xl">
    <div class="text-xs text-zinc-400 uppercase tracking-wider mb-1">Blocked</div>
    <div class="text-2xl font-bold text-red-400">{blockedRules}</div>
  </div>
</div>
