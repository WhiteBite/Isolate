<script lang="ts">
  interface ServiceItem {
    id: string;
    name: string;
    icon: string;
    description: string;
  }
  
  interface Props {
    services: ServiceItem[];
    selectedServices: Set<string>;
    loading: boolean;
    onToggle: (serviceId: string) => void;
    onSelectAll: () => void;
    onDeselectAll: () => void;
  }
  
  let { services, selectedServices, loading, onToggle, onSelectAll, onDeselectAll }: Props = $props();
</script>

<div class="flex-1 flex flex-col animate-fade-in">
  <div class="text-center mb-6">
    <h2 class="text-2xl font-bold text-white mb-2">Select Services</h2>
    <p class="text-zinc-400">Which services do you want to unblock?</p>
  </div>
  
  <!-- Quick actions -->
  <div class="flex justify-center gap-2 mb-4">
    <button
      onclick={onSelectAll}
      class="px-3 py-1.5 text-xs font-medium text-indigo-400 hover:text-indigo-300 
             bg-indigo-500/10 hover:bg-indigo-500/20 rounded-lg transition-colors"
    >
      Select all
    </button>
    <button
      onclick={onDeselectAll}
      class="px-3 py-1.5 text-xs font-medium text-zinc-400 hover:text-zinc-300 
             bg-zinc-800/60 hover:bg-zinc-800 rounded-lg transition-colors"
    >
      Clear
    </button>
  </div>
  
  {#if loading}
    <div class="flex-1 flex items-center justify-center">
      <div class="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else}
    <div class="grid grid-cols-2 gap-3 flex-1 overflow-y-auto">
      {#each services as service}
        <button
          onclick={() => onToggle(service.id)}
          class="group p-4 rounded-xl border transition-all duration-200 text-left
                 {selectedServices.has(service.id) 
                   ? 'border-indigo-500/50 bg-indigo-500/10 shadow-lg shadow-indigo-500/10' 
                   : 'border-white/5 bg-zinc-800/30 hover:border-white/10 hover:bg-zinc-800/50'}"
        >
          <div class="flex items-start gap-3">
            <span class="text-2xl">{service.icon}</span>
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-white truncate">{service.name}</div>
              <div class="text-xs text-zinc-400">{service.description}</div>
            </div>
            <div class="w-5 h-5 rounded-md border-2 flex items-center justify-center transition-all
                        {selectedServices.has(service.id) 
                          ? 'border-indigo-500 bg-indigo-500' 
                          : 'border-zinc-600 group-hover:border-zinc-500'}">
              {#if selectedServices.has(service.id)}
                <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                </svg>
              {/if}
            </div>
          </div>
        </button>
      {/each}
    </div>
    
    <!-- Selection counter -->
    <div class="mt-4 text-center">
      <span class="text-sm {selectedServices.size > 0 ? 'text-indigo-400' : 'text-amber-400'}">
        {selectedServices.size > 0 
          ? `Selected: ${selectedServices.size} service${selectedServices.size === 1 ? '' : 's'}`
          : 'Select at least one service'}
      </span>
    </div>
  {/if}
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
