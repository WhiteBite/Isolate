<script lang="ts">
  import BaseModal from '$lib/components/BaseModal.svelte';
  import { getServiceIconSvg } from '$lib/utils/icons';

  interface ServiceWithStatus {
    id: string;
    name: string;
    status: 'working' | 'blocked' | 'unknown' | 'checking' | 'error';
    ping?: number;
    category: string;
    isCustom?: boolean;
    error?: string | null;
  }

  interface ServiceConfig {
    autoCheck: boolean;
    checkInterval: string;
    notifyBlocked: boolean;
    notifyRestored: boolean;
    priority: 'low' | 'normal' | 'high';
  }

  interface Props {
    open: boolean;
    service: ServiceWithStatus | null;
    config: ServiceConfig;
    onclose: () => void;
    onsave: (config: ServiceConfig) => Promise<void>;
  }

  let { open, service, config, onclose, onsave }: Props = $props();

  // Local state for editing
  let autoCheck = $state(false);
  let checkInterval = $state('15');
  let notifyBlocked = $state(true);
  let notifyRestored = $state(false);
  let priority = $state<'low' | 'normal' | 'high'>('normal');

  // Sync local state with config prop when it changes
  $effect(() => {
    autoCheck = config.autoCheck;
    checkInterval = config.checkInterval;
    notifyBlocked = config.notifyBlocked;
    notifyRestored = config.notifyRestored;
    priority = config.priority;
  });

  function getIcon(id: string) {
    return getServiceIconSvg(id);
  }

  async function handleSave() {
    await onsave({
      autoCheck,
      checkInterval,
      notifyBlocked,
      notifyRestored,
      priority
    });
  }
</script>

<BaseModal {open} {onclose} class="w-full max-w-lg">
  {#if service}
    {@const icon = getIcon(service.id)}
    <!-- Header -->
    <div class="flex items-center gap-4 p-6 border-b border-white/5">
      <div class="w-12 h-12 rounded-xl bg-zinc-800/60 border border-white/5 flex items-center justify-center">
        <svg class="w-6 h-6 {icon.color}" viewBox="0 0 24 24" fill="currentColor">
          <path d={icon.path}/>
        </svg>
      </div>
      <div class="flex-1">
        <h3 class="text-lg font-semibold text-zinc-100">{service.name}</h3>
        <p class="text-sm text-zinc-400 capitalize">{service.category} • Configuration</p>
      </div>
      <button
        aria-label="Close"
        onclick={onclose}
        class="p-2 rounded-lg hover:bg-zinc-800 transition-colors"
      >
        <svg class="w-5 h-5 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
      </button>
    </div>
    
    <!-- Content -->
    <div class="p-6 space-y-5">
      <!-- Check Settings -->
      <div>
        <h4 class="text-sm font-medium text-zinc-300 mb-3 flex items-center gap-2">
          <svg class="w-4 h-4 text-zinc-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
            <polyline points="22 4 12 14.01 9 11.01"/>
          </svg>
          Check Settings
        </h4>
        <div class="space-y-3">
          <div class="flex items-center justify-between p-3 bg-zinc-800/40 rounded-xl border border-white/5">
            <span class="text-sm text-zinc-400">Auto-check on startup</span>
            <button 
              aria-label="Toggle auto-check on startup"
              role="switch"
              aria-checked={autoCheck}
              onclick={() => autoCheck = !autoCheck}
              class="w-10 h-6 rounded-full relative cursor-pointer transition-colors duration-200
                     {autoCheck ? 'bg-indigo-500' : 'bg-zinc-700'}"
            >
              <div class="absolute top-1 w-4 h-4 rounded-full transition-all duration-200
                          {autoCheck ? 'right-1 bg-white' : 'left-1 bg-zinc-400'}"></div>
            </button>
          </div>
          <div class="flex items-center justify-between p-3 bg-zinc-800/40 rounded-xl border border-white/5">
            <span class="text-sm text-zinc-400">Check interval</span>
            <select 
              bind:value={checkInterval}
              class="bg-zinc-700 border border-white/10 rounded-lg px-3 py-1.5 text-sm text-zinc-200 focus:outline-none"
            >
              <option value="5">5 min</option>
              <option value="15">15 min</option>
              <option value="30">30 min</option>
              <option value="60">1 hour</option>
            </select>
          </div>
        </div>
      </div>
      
      <!-- Notification Settings -->
      <div>
        <h4 class="text-sm font-medium text-zinc-300 mb-3 flex items-center gap-2">
          <svg class="w-4 h-4 text-zinc-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
            <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
          </svg>
          Notifications
        </h4>
        <div class="space-y-3">
          <div class="flex items-center justify-between p-3 bg-zinc-800/40 rounded-xl border border-white/5">
            <span class="text-sm text-zinc-400">Notify when blocked</span>
            <button 
              aria-label="Toggle notify when blocked"
              role="switch"
              aria-checked={notifyBlocked}
              onclick={() => notifyBlocked = !notifyBlocked}
              class="w-10 h-6 rounded-full relative cursor-pointer transition-colors duration-200
                     {notifyBlocked ? 'bg-indigo-500' : 'bg-zinc-700'}"
            >
              <div class="absolute top-1 w-4 h-4 rounded-full transition-all duration-200
                          {notifyBlocked ? 'right-1 bg-white' : 'left-1 bg-zinc-400'}"></div>
            </button>
          </div>
          <div class="flex items-center justify-between p-3 bg-zinc-800/40 rounded-xl border border-white/5">
            <span class="text-sm text-zinc-400">Notify when restored</span>
            <button 
              aria-label="Toggle notify when restored"
              role="switch"
              aria-checked={notifyRestored}
              onclick={() => notifyRestored = !notifyRestored}
              class="w-10 h-6 rounded-full relative cursor-pointer transition-colors duration-200
                     {notifyRestored ? 'bg-indigo-500' : 'bg-zinc-700'}"
            >
              <div class="absolute top-1 w-4 h-4 rounded-full transition-all duration-200
                          {notifyRestored ? 'right-1 bg-white' : 'left-1 bg-zinc-400'}"></div>
            </button>
          </div>
        </div>
      </div>
      
      <!-- Priority -->
      <div>
        <h4 class="text-sm font-medium text-zinc-300 mb-3 flex items-center gap-2">
          <svg class="w-4 h-4 text-zinc-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
          </svg>
          Priority
        </h4>
        <div class="flex gap-2">
          <button 
            onclick={() => priority = 'low'}
            class="flex-1 px-4 py-2.5 rounded-xl text-sm transition-colors
                   {priority === 'low' 
                     ? 'bg-indigo-500/20 border border-indigo-500/30 text-indigo-400 font-medium' 
                     : 'bg-zinc-800/40 border border-white/5 text-zinc-400 hover:bg-zinc-700/40'}"
          >
            Low
          </button>
          <button 
            onclick={() => priority = 'normal'}
            class="flex-1 px-4 py-2.5 rounded-xl text-sm transition-colors
                   {priority === 'normal' 
                     ? 'bg-indigo-500/20 border border-indigo-500/30 text-indigo-400 font-medium' 
                     : 'bg-zinc-800/40 border border-white/5 text-zinc-400 hover:bg-zinc-700/40'}"
          >
            Normal
          </button>
          <button 
            onclick={() => priority = 'high'}
            class="flex-1 px-4 py-2.5 rounded-xl text-sm transition-colors
                   {priority === 'high' 
                     ? 'bg-indigo-500/20 border border-indigo-500/30 text-indigo-400 font-medium' 
                     : 'bg-zinc-800/40 border border-white/5 text-zinc-400 hover:bg-zinc-700/40'}"
          >
            High
          </button>
        </div>
      </div>
    </div>
    
    <!-- Footer -->
    <div class="flex gap-3 p-6 border-t border-white/5">
      <button
        onclick={onclose}
        class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
               text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
      >
        Cancel
      </button>
      <button
        onclick={handleSave}
        class="flex-1 px-4 py-2.5 bg-indigo-500 rounded-xl
               text-white font-medium text-sm hover:bg-indigo-600 transition-colors"
      >
        Save Changes
      </button>
    </div>
  {/if}
</BaseModal>
