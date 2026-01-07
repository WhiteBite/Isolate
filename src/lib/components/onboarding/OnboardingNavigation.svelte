<script lang="ts">
  type Step = 1 | 2 | 3 | 4 | 5;
  
  interface Props {
    currentStep: Step;
    canProceed: boolean;
    setupComplete: boolean;
    isSettingUp: boolean;
    onNext: () => void;
    onPrev: () => void;
    onSkip: () => void;
    onComplete: () => void;
  }
  
  let { currentStep, canProceed, setupComplete, isSettingUp, onNext, onPrev, onSkip, onComplete }: Props = $props();
</script>

<div class="flex gap-3 pt-6 mt-auto border-t border-white/5">
  {#if currentStep > 1 && currentStep < 5}
    <button
      onclick={onPrev}
      class="flex items-center justify-center gap-2 px-5 py-3 
             bg-zinc-800/60 hover:bg-zinc-800 border border-white/5 hover:border-white/10
             text-zinc-300 rounded-xl font-medium transition-all"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
      Back
    </button>
  {:else if currentStep === 1}
    <button
      onclick={onSkip}
      class="px-5 py-3 text-zinc-500 hover:text-zinc-400 font-medium transition-colors"
    >
      Skip
    </button>
  {:else}
    <div></div>
  {/if}
  
  <div class="flex-1"></div>
  
  {#if currentStep < 5}
    <button
      onclick={onNext}
      disabled={!canProceed}
      class="flex items-center justify-center gap-2 px-6 py-3 
             bg-indigo-500 hover:bg-indigo-600 disabled:bg-zinc-800 disabled:text-zinc-600
             text-white rounded-xl font-medium transition-all
             disabled:cursor-not-allowed shadow-lg shadow-indigo-500/20 disabled:shadow-none
             hover:-translate-y-0.5 disabled:translate-y-0"
    >
      {currentStep === 1 ? 'Get Started' : 'Next'}
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
      </svg>
    </button>
  {:else if setupComplete}
    <button
      onclick={onComplete}
      class="flex items-center justify-center gap-2 px-8 py-3 
             bg-gradient-to-r from-emerald-500 to-cyan-500 hover:from-emerald-600 hover:to-cyan-600
             text-white rounded-xl font-semibold transition-all
             shadow-lg shadow-emerald-500/20
             hover:-translate-y-0.5"
    >
      Start Using
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
      </svg>
    </button>
  {:else}
    <div class="flex items-center gap-2 px-6 py-3 text-zinc-500">
      <div class="w-4 h-4 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
      Setting up...
    </div>
  {/if}
</div>
