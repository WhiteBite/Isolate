<script lang="ts">
  import { Modal, Button } from '$lib/components';
  import type { ProxyConfig, ProxyProtocol } from '$lib/api';

  interface Props {
    open: boolean;
    proxy: ProxyConfig | null;
    onUpdate: (proxy: Partial<ProxyConfig>) => Promise<void>;
    onClose: () => void;
  }

  let { open = $bindable(), proxy, onUpdate, onClose }: Props = $props();

  // Form state
  let formProtocol = $state<ProxyProtocol>('vless');
  let formName = $state('');
  let formServer = $state('');
  let formPort = $state(443);
  let formUuid = $state('');
  let formTls = $state(true);
  let formSni = $state('');
  let formTransport = $state('tcp');
  let formMethod = $state('aes-256-gcm');
  let formPassword = $state('');

  const ssMethodOptions = [
    'aes-128-gcm', 'aes-256-gcm', 'chacha20-ietf-poly1305',
    '2022-blake3-aes-128-gcm', '2022-blake3-aes-256-gcm', '2022-blake3-chacha20-poly1305'
  ];
  const transportOptions = ['tcp', 'ws', 'grpc', 'http'];

  // Sync form with proxy when it changes
  $effect(() => {
    if (proxy) {
      formProtocol = proxy.protocol;
      formName = proxy.name;
      formServer = proxy.server;
      formPort = proxy.port;
      formUuid = proxy.uuid || '';
      formTls = proxy.tls ?? true;
      formSni = proxy.sni || '';
      formTransport = proxy.transport || 'tcp';
      formPassword = proxy.password || '';
    }
  });

  async function handleUpdateProxy() {
    if (!proxy) return;
    
    const updatedProxy: Partial<ProxyConfig> = {
      id: proxy.id,
      name: formName,
      protocol: formProtocol,
      server: formServer,
      port: formPort,
    };
    
    if (formProtocol === 'vless') {
      updatedProxy.uuid = formUuid;
      updatedProxy.tls = formTls;
      updatedProxy.sni = formSni;
      updatedProxy.transport = formTransport;
    } else if (formProtocol === 'vmess') {
      updatedProxy.uuid = formUuid;
    } else if (formProtocol === 'shadowsocks') {
      updatedProxy.password = formPassword;
    }
    
    await onUpdate(updatedProxy);
    open = false;
  }

  function handleClose() {
    onClose();
  }
</script>

<Modal bind:open title="Edit Proxy" onclose={handleClose}>
  <form onsubmit={(e) => { e.preventDefault(); handleUpdateProxy(); }} class="space-y-4">
    <div>
      <label for="edit-protocol" class="block text-sm font-medium text-text-secondary mb-1">Protocol</label>
      <select id="edit-protocol" bind:value={formProtocol} class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active transition-all duration-200">
        <option value="vless">VLESS</option>
        <option value="vmess">VMess</option>
        <option value="shadowsocks">Shadowsocks</option>
        <option value="socks5">SOCKS5</option>
        <option value="http">HTTP</option>
      </select>
    </div>
    
    <div>
      <label for="edit-name" class="block text-sm font-medium text-text-secondary mb-1">Name</label>
      <input id="edit-name" type="text" bind:value={formName} placeholder="My Proxy" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active transition-all duration-200" required />
    </div>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label for="edit-server" class="block text-sm font-medium text-text-secondary mb-1">Server</label>
        <input id="edit-server" type="text" bind:value={formServer} placeholder="example.com" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active font-mono transition-all duration-200" required />
      </div>
      <div>
        <label for="edit-port" class="block text-sm font-medium text-text-secondary mb-1">Port</label>
        <input id="edit-port" type="number" bind:value={formPort} min="1" max="65535" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active font-mono transition-all duration-200" required />
      </div>
    </div>
    
    {#if formProtocol === 'vless' || formProtocol === 'vmess'}
      <div>
        <label for="edit-uuid" class="block text-sm font-medium text-text-secondary mb-1">UUID</label>
        <input id="edit-uuid" type="text" bind:value={formUuid} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active font-mono text-sm transition-all duration-200" required />
      </div>
    {/if}
    
    {#if formProtocol === 'vless'}
      <div class="flex items-center gap-3">
        <input type="checkbox" id="edit-tls" bind:checked={formTls} class="w-4 h-4 rounded bg-void-50 border-glass-border text-indigo-500 focus:ring-indigo-500/30 transition-all duration-200" />
        <label for="edit-tls" class="text-sm text-text-secondary">TLS</label>
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label for="edit-sni" class="block text-sm font-medium text-text-secondary mb-1">SNI</label>
          <input id="edit-sni" type="text" bind:value={formSni} placeholder="example.com" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active font-mono transition-all duration-200" />
        </div>
        <div>
          <label for="edit-transport" class="block text-sm font-medium text-text-secondary mb-1">Transport</label>
          <select id="edit-transport" bind:value={formTransport} class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active transition-all duration-200">
            {#each transportOptions as opt}
              <option value={opt}>{opt.toUpperCase()}</option>
            {/each}
          </select>
        </div>
      </div>
    {/if}
    
    {#if formProtocol === 'shadowsocks'}
      <div>
        <label for="edit-method" class="block text-sm font-medium text-text-secondary mb-1">Encryption Method</label>
        <select id="edit-method" bind:value={formMethod} class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active transition-all duration-200">
          {#each ssMethodOptions as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      </div>
      <div>
        <label for="edit-password" class="block text-sm font-medium text-text-secondary mb-1">Password</label>
        <input id="edit-password" type="password" bind:value={formPassword} class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active transition-all duration-200" required />
      </div>
    {/if}
    
    <div class="flex justify-end gap-3 pt-4">
      <Button variant="secondary" onclick={handleClose}>Cancel</Button>
      <button type="submit" class="px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 text-white font-medium text-sm rounded-xl transition-all duration-200">Save</button>
    </div>
  </form>
</Modal>
