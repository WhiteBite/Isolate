<script lang="ts">
  import { browser } from '$app/environment';
  import type { ProxyConfig } from '$lib/api/types';
  import {
    type Subscription,
    getSubscriptions,
    saveSubscriptions,
    addSubscription,
    updateSubscription,
    removeSubscription,
    fetchSubscription
  } from '$lib/api/subscription';
  import BaseModal from '$lib/components/BaseModal.svelte';
  
  // Props
  interface Props {
    onProxiesImported?: (proxies: ProxyConfig[]) => void;
  }
  
  let { onProxiesImported }: Props = $props();
  
  // State
  let subscriptions = $state<Subscription[]>([]);
  let loading = $state(false);
  let updatingId = $state<string | null>(null);
  let showAddModal = $state(false);
  let showDeleteConfirm = $state(false);
  let subscriptionToDelete = $state<Subscription | null>(null);
  
  // Add modal state
  let newName = $state('');
  let newUrl = $state('');
  let newAutoUpdate = $state(true);
  let newUpdateInterval = $state(60);
  let addError = $state('');
  let adding = $state(false);
  
  // Load subscriptions on mount
  $effect(() => {
    if (!browser) return;
    subscriptions = getSubscriptions();
  });
  
  // Auto-update effect
  $effect(() => {
    if (!browser) return;
    
    const intervals: ReturnType<typeof setInterval>[] = [];
    
    for (const sub of subscriptions) {
      if (sub.enabled && sub.autoUpdate && sub.updateInterval > 0) {
        const interval = setInterval(() => {
          handleUpdateSubscription(sub.id);
        }, sub.updateInterval * 60 * 1000);
        intervals.push(interval);
      }
    }

    return () => {
      intervals.forEach(clearInterval);
    };
  });
  
  async function handleAddSubscription() {
    if (!newUrl.trim()) {
      addError = 'URL is required';
      return;
    }
    
    // Validate URL
    try {
      new URL(newUrl);
    } catch {
      addError = 'Invalid URL format';
      return;
    }
    
    adding = true;
    addError = '';
    
    try {
      // Create subscription
      const name = newName.trim() || new URL(newUrl).hostname;
      const subscription = addSubscription(name, newUrl.trim(), newAutoUpdate, newUpdateInterval);
      
      // Fetch and parse
      const result = await fetchSubscription(newUrl.trim());
      
      if (result.success && result.proxies.length > 0) {
        // Update subscription with results
        updateSubscription(subscription.id, {
          lastUpdated: new Date().toISOString(),
          proxyCount: result.proxies.length,
          proxyIds: result.proxies.map(p => p.id),
          error: null
        });
        
        // Notify parent
        onProxiesImported?.(result.proxies);
        
        // Refresh list
        subscriptions = getSubscriptions();
        
        // Close modal
        showAddModal = false;
        resetAddForm();
      } else {
        // Update with error
        updateSubscription(subscription.id, {
          error: result.error || 'No proxies found'
        });
        subscriptions = getSubscriptions();
        addError = result.error || 'No proxies found';
      }
    } catch (e) {
      addError = e instanceof Error ? e.message : String(e);
    } finally {
      adding = false;
    }
  }
  
  async function handleUpdateSubscription(id: string) {
    const subscription = subscriptions.find(s => s.id === id);
    if (!subscription) return;
    
    updatingId = id;
    
    try {
      const result = await fetchSubscription(subscription.url);
      
      if (result.success && result.proxies.length > 0) {
        updateSubscription(id, {
          lastUpdated: new Date().toISOString(),
          proxyCount: result.proxies.length,
          proxyIds: result.proxies.map(p => p.id),
          error: null
        });
        
        onProxiesImported?.(result.proxies);
      } else {
        updateSubscription(id, {
          error: result.error || 'No proxies found'
        });
      }
      
      subscriptions = getSubscriptions();
    } catch (e) {
      updateSubscription(id, {
        error: e instanceof Error ? e.message : String(e)
      });
      subscriptions = getSubscriptions();
    } finally {
      updatingId = null;
    }
  }
  
  async function handleUpdateAll() {
    loading = true;
    
    for (const sub of subscriptions.filter(s => s.enabled)) {
      await handleUpdateSubscription(sub.id);
    }
    
    loading = false;
  }
  
  function handleToggleSubscription(id: string) {
    const subscription = subscriptions.find(s => s.id === id);
    if (!subscription) return;
    
    updateSubscription(id, { enabled: !subscription.enabled });
    subscriptions = getSubscriptions();
  }
  
  function handleDeleteSubscription(subscription: Subscription) {
    subscriptionToDelete = subscription;
    showDeleteConfirm = true;
  }
  
  function confirmDelete() {
    if (!subscriptionToDelete) return;
    
    removeSubscription(subscriptionToDelete.id);
    subscriptions = getSubscriptions();
    showDeleteConfirm = false;
    subscriptionToDelete = null;
  }
  
  function resetAddForm() {
    newName = '';
    newUrl = '';
    newAutoUpdate = true;
    newUpdateInterval = 60;
    addError = '';
  }
  
  function formatLastUpdated(dateStr: string | null): string {
    if (!dateStr) return 'Never';
    
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  }
</script>

<div class="space-y-4">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <svg class="w-5 h-5 text-violet-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
      <h3 class="text-sm font-medium text-white">Subscriptions</h3>
      <span class="px-1.5 py-0.5 text-[10px] bg-zinc-800 text-zinc-400 rounded">
        {subscriptions.length}
      </span>
    </div>
    
    <div class="flex items-center gap-2">
      {#if subscriptions.length > 0}
        <button
          type="button"
          onclick={handleUpdateAll}
          disabled={loading}
          class="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium rounded-lg
                 bg-zinc-800/60 border border-white/5 text-zinc-300
                 hover:bg-zinc-700/60 hover:text-white transition-all
                 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg class="w-3.5 h-3.5 {loading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Update All
        </button>
      {/if}
      
      <button
        type="button"
        onclick={() => showAddModal = true}
        class="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium rounded-lg
               bg-violet-500/20 border border-violet-500/30 text-violet-300
               hover:bg-violet-500/30 hover:text-violet-200 transition-all"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Add
      </button>
    </div>
  </div>
  
  <!-- Subscription List -->
  {#if subscriptions.length === 0}
    <div class="flex flex-col items-center justify-center py-8 text-center">
      <div class="w-12 h-12 rounded-full bg-zinc-800/60 flex items-center justify-center mb-3">
        <svg class="w-6 h-6 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
        </svg>
      </div>
      <p class="text-sm text-zinc-400 mb-1">No subscriptions</p>
      <p class="text-xs text-zinc-500">Add a subscription URL to import proxies automatically</p>
    </div>
  {:else}
    <div class="space-y-2">
      {#each subscriptions as subscription (subscription.id)}
        <div class="group relative p-3 rounded-xl bg-zinc-900/50 border border-white/5 hover:border-white/10 transition-all">
          <div class="flex items-start justify-between gap-3">
            <!-- Info -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <button
                  type="button"
                  onclick={() => handleToggleSubscription(subscription.id)}
                  class="w-4 h-4 rounded flex items-center justify-center transition-colors
                         {subscription.enabled ? 'bg-violet-500 text-white' : 'bg-zinc-700 text-zinc-500'}"
                >
                  {#if subscription.enabled}
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                  {/if}
                </button>
                <span class="text-sm font-medium text-white truncate">{subscription.name}</span>
                {#if subscription.autoUpdate}
                  <span class="px-1.5 py-0.5 text-[9px] uppercase tracking-wider bg-emerald-500/10 text-emerald-400 rounded border border-emerald-500/20">
                    Auto
                  </span>
                {/if}
              </div>
              
              <div class="flex items-center gap-3 text-xs text-zinc-500">
                <span class="flex items-center gap-1">
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                  </svg>
                  {subscription.proxyCount} proxies
                </span>
                <span class="flex items-center gap-1">
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {formatLastUpdated(subscription.lastUpdated)}
                </span>
              </div>
              
              {#if subscription.error}
                <p class="mt-1.5 text-xs text-red-400 truncate">{subscription.error}</p>
              {/if}
            </div>
            
            <!-- Actions -->
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                type="button"
                onclick={() => handleUpdateSubscription(subscription.id)}
                disabled={updatingId === subscription.id}
                class="p-1.5 rounded-lg text-zinc-400 hover:text-white hover:bg-white/5 transition-all
                       disabled:opacity-50 disabled:cursor-not-allowed"
                title="Update"
              >
                <svg class="w-4 h-4 {updatingId === subscription.id ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
              
              <button
                type="button"
                onclick={() => handleDeleteSubscription(subscription)}
                class="p-1.5 rounded-lg text-zinc-400 hover:text-red-400 hover:bg-red-500/10 transition-all"
                title="Remove"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Add Subscription Modal -->
<BaseModal open={showAddModal} onclose={() => { showAddModal = false; resetAddForm(); }} class="w-full max-w-md">
  <div class="p-6">
    <h3 class="text-lg font-semibold text-white mb-4">Add Subscription</h3>
    
    <div class="space-y-4">
      <!-- URL Input -->
      <div>
        <label for="sub-url" class="block text-sm font-medium text-zinc-300 mb-1.5">
          Subscription URL <span class="text-red-400">*</span>
        </label>
        <input
          id="sub-url"
          type="url"
          bind:value={newUrl}
          placeholder="https://example.com/sub?token=xxx"
          class="w-full px-3 py-2.5 bg-zinc-900/50 border border-white/10 rounded-xl
                 text-white placeholder-zinc-500 text-sm
                 focus:outline-none focus:border-violet-500/50 focus:ring-1 focus:ring-violet-500/20
                 transition-all"
        />
      </div>
      
      <!-- Name Input -->
      <div>
        <label for="sub-name" class="block text-sm font-medium text-zinc-300 mb-1.5">
          Name <span class="text-zinc-500">(optional)</span>
        </label>
        <input
          id="sub-name"
          type="text"
          bind:value={newName}
          placeholder="My Subscription"
          class="w-full px-3 py-2.5 bg-zinc-900/50 border border-white/10 rounded-xl
                 text-white placeholder-zinc-500 text-sm
                 focus:outline-none focus:border-violet-500/50 focus:ring-1 focus:ring-violet-500/20
                 transition-all"
        />
      </div>
      
      <!-- Auto Update Toggle -->
      <div class="flex items-center justify-between p-3 bg-zinc-900/30 rounded-xl border border-white/5">
        <div>
          <p class="text-sm font-medium text-white">Auto Update</p>
          <p class="text-xs text-zinc-500">Automatically refresh proxies</p>
        </div>
        <button
          type="button"
          onclick={() => newAutoUpdate = !newAutoUpdate}
          class="relative w-11 h-6 rounded-full transition-colors
                 {newAutoUpdate ? 'bg-violet-500' : 'bg-zinc-700'}"
        >
          <span
            class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform
                   {newAutoUpdate ? 'translate-x-5' : 'translate-x-0'}"
          ></span>
        </button>
      </div>
      
      <!-- Update Interval -->
      {#if newAutoUpdate}
        <div>
          <label for="sub-interval" class="block text-sm font-medium text-zinc-300 mb-1.5">
            Update Interval
          </label>
          <select
            id="sub-interval"
            bind:value={newUpdateInterval}
            class="w-full px-3 py-2.5 bg-zinc-900/50 border border-white/10 rounded-xl
                   text-white text-sm appearance-none cursor-pointer
                   focus:outline-none focus:border-violet-500/50 focus:ring-1 focus:ring-violet-500/20
                   transition-all"
          >
            <option value={30}>Every 30 minutes</option>
            <option value={60}>Every hour</option>
            <option value={180}>Every 3 hours</option>
            <option value={360}>Every 6 hours</option>
            <option value={720}>Every 12 hours</option>
            <option value={1440}>Every 24 hours</option>
          </select>
        </div>
      {/if}
      
      <!-- Error -->
      {#if addError}
        <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-xl">
          <p class="text-sm text-red-400">{addError}</p>
        </div>
      {/if}
    </div>
    
    <!-- Actions -->
    <div class="flex gap-3 mt-6">
      <button
        type="button"
        onclick={() => { showAddModal = false; resetAddForm(); }}
        class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
               text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={handleAddSubscription}
        disabled={adding || !newUrl.trim()}
        class="flex-1 px-4 py-2.5 bg-violet-500/20 border border-violet-500/30 rounded-xl
               text-violet-300 font-medium text-sm hover:bg-violet-500/30 transition-colors
               disabled:opacity-50 disabled:cursor-not-allowed
               flex items-center justify-center gap-2"
      >
        {#if adding}
          <svg class="w-4 h-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Importing...
        {:else}
          Add & Import
        {/if}
      </button>
    </div>
  </div>
</BaseModal>

<!-- Delete Confirmation Modal -->
<BaseModal open={showDeleteConfirm} onclose={() => { showDeleteConfirm = false; subscriptionToDelete = null; }} class="w-full max-w-sm">
  {#if subscriptionToDelete}
    <div class="p-6 text-center">
      <div class="w-14 h-14 mx-auto mb-4 rounded-full bg-red-500/10 flex items-center justify-center">
        <svg class="w-7 h-7 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </div>
      <h3 class="text-lg font-semibold text-white mb-2">Remove Subscription?</h3>
      <p class="text-zinc-400 text-sm mb-6">
        Are you sure you want to remove <span class="text-white font-medium">{subscriptionToDelete.name}</span>?
        Imported proxies will remain.
      </p>
      <div class="flex gap-3">
        <button
          onclick={() => { showDeleteConfirm = false; subscriptionToDelete = null; }}
          class="flex-1 px-4 py-2.5 bg-zinc-800/60 border border-white/10 rounded-xl
                 text-zinc-300 font-medium text-sm hover:bg-zinc-700/60 transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={confirmDelete}
          class="flex-1 px-4 py-2.5 bg-red-500/20 border border-red-500/30 rounded-xl
                 text-red-400 font-medium text-sm hover:bg-red-500/30 transition-colors"
        >
          Remove
        </button>
      </div>
    </div>
  {/if}
</BaseModal>
