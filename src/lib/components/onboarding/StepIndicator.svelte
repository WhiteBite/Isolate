<script lang="ts">
  type Step = 1 | 2 | 3 | 4 | 5;
  
  interface Props {
    currentStep: Step;
    stepTitles: string[];
    onStepClick: (step: Step) => void;
  }
  
  let { currentStep, stepTitles, onStepClick }: Props = $props();
  
  // Generate step numbers based on stepTitles length
  let steps = $derived(stepTitles.map((_, i) => i + 1));
</script>

<div class="flex justify-center items-center gap-3 mb-6">
  {#each steps as step}
    <button
      onclick={() => { if (step < currentStep) onStepClick(step as Step); }}
      disabled={step > currentStep}
      class="group relative flex flex-col items-center gap-1"
    >
      <div 
        class="w-3 h-3 rounded-full transition-all duration-300 
               {step < currentStep 
                 ? 'bg-emerald-400 scale-100' 
                 : step === currentStep 
                   ? 'bg-indigo-500 scale-125 ring-4 ring-indigo-500/20' 
                   : 'bg-zinc-700 scale-100'}"
      ></div>
      <span class="text-[10px] font-medium transition-colors
                   {step === currentStep ? 'text-indigo-400' : step < currentStep ? 'text-emerald-400' : 'text-zinc-600'}">
        {stepTitles[step - 1]}
      </span>
    </button>
  {/each}
</div>
