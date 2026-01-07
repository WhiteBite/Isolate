<script lang="ts">
  import { Modal } from '$lib/components';
  import { toasts } from '$lib/stores/toast';
  import { generateProxyUrl } from '$lib/utils/proxy-url';
  import type { ProxyConfig } from '$lib/api';
  import QRCode from 'qrcode';

  interface Props {
    open: boolean;
    proxy: ProxyConfig | null;
    onClose: () => void;
  }

  let { open = $bindable(), proxy, onClose }: Props = $props();

  let canvasRef: HTMLCanvasElement | undefined = $state();
  let proxyUrl = $derived(proxy ? generateProxyUrl(proxy) : '');
  let copying = $state(false);

  // Generate QR code when modal opens or proxy changes
  $effect(() => {
    if (open && proxy && canvasRef) {
      generateQR();
    }
  });

  async function generateQR() {
    if (!canvasRef || !proxyUrl) return;
    
    try {
      await QRCode.toCanvas(canvasRef, proxyUrl, {
        width: 256,
        margin: 2,
        color: {
          dark: '#ffffff',
          light: '#00000000' // transparent background
        },
        errorCorrectionLevel: 'M'
      });
    } catch (err) {
      console.error('Failed to generate QR code:', err);
      toasts.error('Failed to generate QR code');
    }
  }

  async function handleCopyUrl() {
    if (!proxyUrl) return;
    
    copying = true;
    try {
      await navigator.clipboard.writeText(proxyUrl);
      toasts.success('URL copied to clipboard');
    } catch {
      toasts.error('Failed to copy URL');
    }
    copying = false;
  }

  async function handleDownloadQR() {
    if (!canvasRef) return;
    
    try {
      const dataUrl = canvasRef.toDataURL('image/png');
      const link = document.createElement('a');
      link.download = `${proxy?.name || 'proxy'}-qr.png`;
      link.href = dataUrl;
      link.click();
      toasts.success('QR code downloaded');
    } catch {
      toasts.error('Failed to download QR code');
    }
  }

  function handleClose() {
    open = false;
    onClose();
  }
</script>

<Modal bind:open title="Share Proxy" onclose={handleClose}>
  {#if proxy}
    <div class="flex flex-col items-center gap-6">
      <!-- Proxy Info -->
      <div class="text-center">
        <h3 class="text-lg font-medium text-white">{proxy.name}</h3>
        <p class="text-sm text-zinc-400 mt-1">
          {proxy.protocol.toUpperCase()} • {proxy.server}:{proxy.port}
        </p>
      </div>

      <!-- QR Code -->
      <div class="p-4 bg-white rounded-2xl">
        <canvas bind:this={canvasRef} class="block"></canvas>
      </div>

      <!-- URL Display -->
      <div class="w-full">
        <span class="block text-sm font-medium text-zinc-400 mb-2">Share URL</span>
        <div class="relative">
          <input
            type="text"
            readonly
            value={proxyUrl}
            class="w-full px-4 py-3 pr-12 bg-zinc-800/50 border border-white/5 rounded-xl text-zinc-300 text-sm font-mono truncate focus:outline-none"
          />
          <button
            onclick={handleCopyUrl}
            disabled={copying}
            class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-zinc-400 hover:text-emerald-400 hover:bg-emerald-500/10 rounded-lg transition-all"
            title="Copy URL"
          >
            {#if copying}
              <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
              </svg>
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                  d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            {/if}
          </button>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-3 w-full">
        <button
          onclick={handleCopyUrl}
          class="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 rounded-xl font-medium text-sm hover:bg-indigo-500/30 transition-all"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
          </svg>
          Copy URL
        </button>
        <button
          onclick={handleDownloadQR}
          class="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 bg-zinc-800 text-zinc-300 border border-white/5 rounded-xl font-medium text-sm hover:bg-zinc-700 transition-all"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          Download QR
        </button>
      </div>
    </div>
  {/if}
</Modal>
