<script lang="ts">
  import CountryFlag from './CountryFlag.svelte';
  import ProtocolBadge from './ProtocolBadge.svelte';
  import { detectCountryFromServer, getCountryName } from '$lib/utils/countries';
  
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
    network?: string;
    raw: string;
  }
  
  interface Props {
    proxy: ParsedProxy;
    onConfirm?: () => void;
    onCancel?: () => void;
  }
  
  let { proxy, onConfirm, onCancel }: Props = $props();
  
  let editableName = $state(proxy.name || 'Imported Proxy');
  let detectedCountry = $derived(detectCountryFromServer(proxy.server));
  let countryName = $derived(getCountryName(detectedCountry));
</script>

<div class="rounded-xl border border-white/10 bg-white/5 overflow-hidden">
  <!-- Header -->
  <div class="flex items-center justify-between px-4 py-3 bg-white/5 border-b border-white/10">
    <span class="text-sm font-medium text-white">Preview</span>
    <ProtocolBadge protocol={proxy.protocol as any} size="sm" />
  </div>

  <!-- Content -->
  <div class="p-4 space-y-4">
    <!-- Name input -->
    <div>
      <label class="block text-xs font-medium text-gray-400 mb-1.5">Name</label>
      <input
        type="text"
        bind:value={editableName}
        class="w-full px-3 py-2 rounded-lg bg-white/5 border border-white/10 
               text-white text-sm placeholder-gray-500
               focus:outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/50
               transition-colors"
        placeholder="Enter proxy name"
      />
    </div>
    
    <!-- Server info -->
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-xs font-medium text-gray-400 mb-1.5">Server</label>
        <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-white/5 border border-white/10">
          {#if detectedCountry}
            <CountryFlag countryCode={detectedCountry} size="sm" />
          {/if}
          <span class="text-sm text-white truncate" title={proxy.server}>
            {proxy.server}
          </span>
        </div>
      </div>
      
      <div>
        <label class="block text-xs font-medium text-gray-400 mb-1.5">Port</label>
        <div class="px-3 py-2 rounded-lg bg-white/5 border border-white/10">
          <span class="text-sm text-white font-mono">{proxy.port}</span>
        </div>
      </div>
    </div>
    
    <!-- Country detection -->
    {#if detectedCountry}
      <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-blue-500/10 border border-blue-500/20">
        <svg class="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span class="text-sm text-blue-400">
          Detected location: <strong>{countryName}</strong>
        </span>
      </div>
    {/if}
    
    <!-- Additional details -->
    {#if proxy.security || proxy.network || proxy.method}
      <div class="pt-2 border-t border-white/10">
        <div class="flex flex-wrap gap-2">
          {#if proxy.security}
            <span class="px-2 py-1 text-xs rounded bg-white/5 text-gray-400">
              Security: {proxy.security}
            </span>
          {/if}
          {#if proxy.network}
            <span class="px-2 py-1 text-xs rounded bg-white/5 text-gray-400">
              Network: {proxy.network}
            </span>
          {/if}
          {#if proxy.method}
            <span class="px-2 py-1 text-xs rounded bg-white/5 text-gray-400">
              Method: {proxy.method}
            </span>
          {/if}
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Actions -->
  <div class="flex justify-end gap-2 px-4 py-3 bg-white/5 border-t border-white/10">
    <button
      onclick={onCancel}
      class="px-4 py-2 text-sm font-medium rounded-lg
             text-gray-400 hover:text-white hover:bg-white/10
             transition-colors"
    >
      Cancel
    </button>
    <button
      onclick={onConfirm}
      class="px-4 py-2 text-sm font-medium rounded-lg
             bg-blue-500 text-white hover:bg-blue-600
             transition-colors"
    >
      Import Proxy
    </button>
  </div>
</div>
