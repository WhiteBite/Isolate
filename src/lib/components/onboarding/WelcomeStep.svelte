<script lang="ts">
  // Welcome step - no props needed, purely presentational
</script>

<div class="flex-1 flex flex-col items-center justify-center text-center animate-fade-in">
  <div class="w-24 h-24 rounded-3xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 border border-indigo-500/20 flex items-center justify-center mb-6">
    <span class="text-5xl">👋</span>
  </div>
  <h2 class="text-3xl font-bold text-white mb-3">Welcome!</h2>
  <p class="text-zinc-400 text-lg mb-8 max-w-sm">
    Isolate helps you access blocked services quickly and securely
  </p>
  
  <!-- Features list -->
  <div class="grid grid-cols-1 gap-3 w-full max-w-sm text-left">
    <div class="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/40 border border-white/5">
      <div class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center">
        <span class="text-xl">⚡</span>
      </div>
      <div>
        <div class="text-sm font-medium text-white">Quick Setup</div>
        <div class="text-xs text-zinc-500">Automatic method selection</div>
      </div>
    </div>
    <div class="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/40 border border-white/5">
      <div class="w-10 h-10 rounded-lg bg-indigo-500/10 flex items-center justify-center">
        <span class="text-xl">🔒</span>
      </div>
      <div>
        <div class="text-sm font-medium text-white">Security</div>
        <div class="text-xs text-zinc-500">No logs or telemetry</div>
      </div>
    </div>
    <div class="flex items-center gap-3 p-3 rounded-xl bg-zinc-800/40 border border-white/5">
      <div class="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center">
        <span class="text-xl">🎯</span>
      </div>
      <div>
        <div class="text-sm font-medium text-white">Targeted Bypass</div>
        <div class="text-xs text-zinc-500">Only the services you need</div>
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  
  .animate-fade-in {
    animation: fade-in 0.4s ease-out forwards;
  }
</style>
