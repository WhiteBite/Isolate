<script lang="ts">
  /**
   * PrivacySettings Component
   * 
   * UI for managing crash reporting and privacy settings.
   * Uses Svelte 5 runes and Tailwind CSS.
   */
  import { browser } from '$app/environment';
  import Toggle from '$lib/components/Toggle.svelte';
  import type { CrashReportingInfo } from '$lib/api/crashReporting';

  // Props
  interface Props {
    /** Optional class for container */
    class?: string;
  }

  let { class: className = '' }: Props = $props();

  // State
  let crashReportingEnabled = $state(false);
  let crashReportingInfo = $state<CrashReportingInfo | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let message = $state<{ text: string; type: 'success' | 'error' } | null>(null);
  let isTauri = $state(false);
  let showDetails = $state(false);

  // Initialize
  $effect(() => {
    if (!browser) return;
    isTauri = '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
    if (isTauri) {
      loadSettings();
    } else {
      loading = false;
    }
  });

  async function loadSettings() {
    if (!browser || !isTauri) return;
    
    loading = true;
    try {
      const { isCrashReportingEnabled, getCrashReportingInfo } = await import('$lib/api/crashReporting');
      
      crashReportingEnabled = await isCrashReportingEnabled();
      crashReportingInfo = await getCrashReportingInfo();
    } catch (e) {
      console.error('Failed to load privacy settings:', e);
      showMessage('Failed to load settings', 'error');
    } finally {
      loading = false;
    }
  }

  async function handleToggleCrashReporting(enabled: boolean) {
    if (!browser || !isTauri || saving) return;
    
    saving = true;
    message = null;
    
    try {
      const { setCrashReportingEnabled } = await import('$lib/api/crashReporting');
      await setCrashReportingEnabled(enabled);
      crashReportingEnabled = enabled;
      
      showMessage(
        enabled 
          ? 'Crash reporting enabled. Thank you for helping improve Isolate!' 
          : 'Crash reporting disabled',
        'success'
      );
    } catch (e) {
      console.error('Failed to toggle crash reporting:', e);
      showMessage(`Failed: ${e}`, 'error');
    } finally {
      saving = false;
    }
  }

  function showMessage(text: string, type: 'success' | 'error') {
    message = { text, type };
    setTimeout(() => { message = null; }, 4000);
  }

  function openPrivacyPolicy() {
    if (crashReportingInfo?.privacy_url) {
      window.open(crashReportingInfo.privacy_url, '_blank');
    }
  }
</script>

<div class={className}>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-xl font-semibold text-text-primary">Privacy & Crash Reporting</h2>
    {#if message}
      <span class="text-sm animate-pulse {message.type === 'error' ? 'text-red-400' : 'text-indigo-400'}">
        {message.text}
      </span>
    {/if}
  </div>
  
  {#if loading}
    <div class="flex items-center justify-center py-12">
      <svg class="w-8 h-8 animate-spin text-indigo-500" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
    </div>
  {:else}
    <div class="space-y-4">
      <!-- Crash Reporting Toggle -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <div class="flex items-center justify-between">
          <div id="crash-reporting-label">
            <p class="text-text-primary font-medium">Crash Reporting</p>
            <p class="text-text-secondary text-sm">
              Help improve Isolate by sending anonymous crash reports
            </p>
          </div>
          <Toggle 
            checked={crashReportingEnabled}
            onchange={handleToggleCrashReporting}
            disabled={saving}
            aria-labelledby="crash-reporting-label"
          />
        </div>
        
        <!-- Details toggle -->
        <button
          onclick={() => showDetails = !showDetails}
          class="mt-3 text-sm text-indigo-400 hover:text-indigo-300 flex items-center gap-1 transition-colors"
        >
          <svg 
            class="w-4 h-4 transition-transform {showDetails ? 'rotate-90' : ''}" 
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
          {showDetails ? 'Hide details' : 'What data is collected?'}
        </button>
        
        {#if showDetails && crashReportingInfo}
          <div class="mt-4 space-y-4 text-sm">
            <!-- Data Collected -->
            <div>
              <p class="text-text-primary font-medium mb-2 flex items-center gap-2">
                <svg class="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                </svg>
                Data we collect:
              </p>
              <ul class="text-text-secondary space-y-1 ml-6">
                {#each crashReportingInfo.data_collected as item}
                  <li class="list-disc">{item}</li>
                {/each}
              </ul>
            </div>
            
            <!-- Data NOT Collected -->
            <div>
              <p class="text-text-primary font-medium mb-2 flex items-center gap-2">
                <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                </svg>
                Data we NEVER collect:
              </p>
              <ul class="text-text-secondary space-y-1 ml-6">
                {#each crashReportingInfo.data_not_collected as item}
                  <li class="list-disc">{item}</li>
                {/each}
              </ul>
            </div>
          </div>
        {/if}
      </div>

      <!-- Privacy Info Box -->
      <div class="p-4 bg-indigo-500/5 rounded-xl border border-indigo-500/20">
        <p class="text-indigo-400 text-sm flex items-start gap-2">
          <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
          </svg>
          <span>
            Your privacy is important to us. Crash reporting is <strong>disabled by default</strong> and only enabled with your explicit consent. 
            All data is anonymized before sending — we never collect IP addresses, usernames, or file paths.
          </span>
        </p>
      </div>

      <!-- Privacy Policy Link -->
      <button
        onclick={openPrivacyPolicy}
        class="text-sm text-text-secondary hover:text-indigo-400 flex items-center gap-2 transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
        </svg>
        Read our Privacy Policy
      </button>

      <!-- Opt-in Explanation -->
      <div class="p-4 bg-void-100 rounded-xl border border-glass-border">
        <p class="text-text-primary font-medium mb-2">Why enable crash reporting?</p>
        <ul class="text-text-secondary text-sm space-y-2">
          <li class="flex items-start gap-2">
            <svg class="w-4 h-4 text-emerald-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
            </svg>
            <span>Help us identify and fix bugs faster</span>
          </li>
          <li class="flex items-start gap-2">
            <svg class="w-4 h-4 text-emerald-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
            </svg>
            <span>Improve stability for all users</span>
          </li>
          <li class="flex items-start gap-2">
            <svg class="w-4 h-4 text-emerald-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
            </svg>
            <span>Zero impact on performance</span>
          </li>
          <li class="flex items-start gap-2">
            <svg class="w-4 h-4 text-emerald-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
            </svg>
            <span>You can disable it anytime</span>
          </li>
        </ul>
      </div>
    </div>
  {/if}
</div>
