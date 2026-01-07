<script lang="ts">
  import { Modal, Button } from '$lib/components';

  type ProxyProtocol = 'vless' | 'vmess' | 'shadowsocks' | 'socks5' | 'http' | 'trojan';
  type AddTab = 'paste' | 'file' | 'manual';

  interface Props {
    open: boolean;
    onImportPaste: (url: string) => Promise<void>;
    onImportFile: (event: Event) => Promise<void>;
    onAddManual: (proxy: ManualProxyData) => Promise<void>;
    onClose: () => void;
    onCheckClipboard: () => void;
  }

  interface ManualProxyData {
    name: string;
    protocol: ProxyProtocol;
    server: string;
    port: number;
    uuid?: string;
    tls?: boolean;
    sni?: string;
    transport?: string;
    password?: string;
    method?: string;
  }

  let { open = $bindable(), onImportPaste, onImportFile, onAddManual, onClose, onCheckClipboard }: Props = $props();

  // Tab state
  let addTab = $state<AddTab>('paste');
  
  // Paste tab state
  let pasteUrl = $state('');
  let pasteLoading = $state(false);

  // Manual form state
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

  function resetForm() {
    formProtocol = 'vless';
    formName = '';
    formServer = '';
    formPort = 443;
    formUuid = '';
    formTls = true;
    formSni = '';
    formTransport = 'tcp';
    formMethod = 'aes-256-gcm';
    formPassword = '';
    pasteUrl = '';
    addTab = 'paste';
  }

  async function handlePasteImport() {
    if (!pasteUrl.trim()) return;
    pasteLoading = true;
    try {
      await onImportPaste(pasteUrl);
      pasteUrl = '';
      open = false;
    } finally {
      pasteLoading = false;
    }
  }

  async function handleAddProxy() {
    const proxy: ManualProxyData = {
      name: formName,
      protocol: formProtocol,
      server: formServer,
      port: formPort,
    };
    
    if (formProtocol === 'vless') {
      proxy.uuid = formUuid;
      proxy.tls = formTls;
      proxy.sni = formSni;
      proxy.transport = formTransport;
    } else if (formProtocol === 'vmess') {
      proxy.uuid = formUuid;
    } else if (formProtocol === 'shadowsocks') {
      proxy.password = formPassword;
      proxy.method = formMethod;
    }
    
    await onAddManual(proxy);
    resetForm();
    open = false;
  }

  function handleTabChange(tab: AddTab) {
    addTab = tab;
    if (tab === 'paste') {
      onCheckClipboard();
    }
  }

  function handleClose() {
    resetForm();
    onClose();
  }

  // Export pasteUrl setter for parent to use
  export function setPasteUrl(url: string) {
    pasteUrl = url;
  }
</script>

<Modal bind:open title="Add Proxy" onclose={handleClose}>
  <!-- Tabs -->
  <div class="flex gap-1 mb-6 p-1 bg-void-50 rounded-xl" role="tablist" aria-label="Add proxy method">
    <button
      role="tab"
      aria-selected={addTab === 'paste'}
      aria-controls="tab-paste"
      class="flex-1 px-4 py-2.5 rounded-lg text-sm font-medium transition-all duration-200
        {addTab === 'paste' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-text-muted hover:text-text-secondary hover:bg-void-100'}"
      onclick={() => handleTabChange('paste')}
    >
      Paste Link
    </button>
    <button
      role="tab"
      aria-selected={addTab === 'file'}
      aria-controls="tab-file"
      class="flex-1 px-4 py-2.5 rounded-lg text-sm font-medium transition-all duration-200
        {addTab === 'file' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-text-muted hover:text-text-secondary hover:bg-void-100'}"
      onclick={() => addTab = 'file'}
    >
      Import File
    </button>
    <button
      role="tab"
      aria-selected={addTab === 'manual'}
      aria-controls="tab-manual"
      class="flex-1 px-4 py-2.5 rounded-lg text-sm font-medium transition-all duration-200
        {addTab === 'manual' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-text-muted hover:text-text-secondary hover:bg-void-100'}"
      onclick={() => addTab = 'manual'}
    >
      Manual
    </button>
  </div>

  <!-- Tab Content: Paste Link -->
  {#if addTab === 'paste'}
    <div id="tab-paste" role="tabpanel" aria-labelledby="tab-paste" class="space-y-4">
      <div>
        <label for="paste-url" class="block text-sm font-medium text-text-secondary mb-2">Proxy URL</label>
        <textarea
          id="paste-url"
          bind:value={pasteUrl}
          placeholder="vless://...&#10;vmess://...&#10;ss://...&#10;&#10;You can paste multiple links (one per line)"
          rows="6"
          class="w-full px-4 py-3 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active focus:ring-1 focus:ring-white/5 font-mono text-sm resize-none transition-all duration-200"
        ></textarea>
        <p class="mt-2 text-xs text-text-muted">Supported: VLESS, VMess, Shadowsocks, Trojan</p>
      </div>
      <div class="flex justify-end gap-3">
        <Button variant="secondary" onclick={handleClose}>Cancel</Button>
        <button 
          onclick={handlePasteImport}
          disabled={pasteLoading}
          class="px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 disabled:opacity-50 text-white font-medium text-sm rounded-xl transition-all duration-200"
        >
          {pasteLoading ? 'Importing...' : 'Import'}
        </button>
      </div>
    </div>
  {/if}

  <!-- Tab Content: Import File -->
  {#if addTab === 'file'}
    <div id="tab-file" role="tabpanel" aria-labelledby="tab-file" class="space-y-4">
      <div>
        <span class="block text-sm font-medium text-text-secondary mb-2">Select File</span>
        <div class="border-2 border-dashed border-glass-border rounded-xl p-8 text-center hover:border-indigo-500/30 transition-all duration-200">
          <input
            type="file"
            accept=".txt,.conf,.json"
            onchange={onImportFile}
            class="hidden"
            id="file-input"
          />
          <label for="file-input" class="cursor-pointer">
            <svg class="w-12 h-12 mx-auto text-text-muted mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            <p class="text-text-secondary">Click to select a file</p>
            <p class="text-xs text-text-muted mt-1">.txt, .conf, .json</p>
          </label>
        </div>
      </div>
      <div class="flex justify-end">
        <Button variant="secondary" onclick={handleClose}>Cancel</Button>
      </div>
    </div>
  {/if}

  <!-- Tab Content: Manual -->
  {#if addTab === 'manual'}
    <div id="tab-manual" role="tabpanel" aria-labelledby="tab-manual">
    <form onsubmit={(e) => { e.preventDefault(); handleAddProxy(); }} class="space-y-4">
      <div>
        <label for="add-protocol" class="block text-sm font-medium text-text-secondary mb-1">Protocol</label>
        <select id="add-protocol" bind:value={formProtocol} class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active transition-all duration-200">
          <option value="vless">VLESS</option>
          <option value="vmess">VMess</option>
          <option value="shadowsocks">Shadowsocks</option>
          <option value="socks5">SOCKS5</option>
          <option value="http">HTTP</option>
        </select>
      </div>
      
      <div>
        <label for="add-name" class="block text-sm font-medium text-text-secondary mb-1">Name</label>
        <input id="add-name" type="text" bind:value={formName} placeholder="My Proxy" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active transition-all duration-200" required />
      </div>
      
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label for="add-server" class="block text-sm font-medium text-text-secondary mb-1">Server</label>
          <input id="add-server" type="text" bind:value={formServer} placeholder="example.com" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active font-mono transition-all duration-200" required />
        </div>
        <div>
          <label for="add-port" class="block text-sm font-medium text-text-secondary mb-1">Port</label>
          <input id="add-port" type="number" bind:value={formPort} min="1" max="65535" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active font-mono transition-all duration-200" required />
        </div>
      </div>
      
      {#if formProtocol === 'vless' || formProtocol === 'vmess'}
        <div>
          <label for="add-uuid" class="block text-sm font-medium text-text-secondary mb-1">UUID</label>
          <input id="add-uuid" type="text" bind:value={formUuid} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active font-mono text-sm transition-all duration-200" required />
        </div>
      {/if}
      
      {#if formProtocol === 'vless'}
        <div class="flex items-center gap-3">
          <input type="checkbox" id="add-tls" bind:checked={formTls} class="w-4 h-4 rounded bg-void-50 border-glass-border text-indigo-500 focus:ring-indigo-500/30 transition-all duration-200" />
          <label for="add-tls" class="text-sm text-text-secondary">TLS</label>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="add-sni" class="block text-sm font-medium text-text-secondary mb-1">SNI</label>
            <input id="add-sni" type="text" bind:value={formSni} placeholder="example.com" class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary placeholder-text-muted focus:outline-none focus:border-glass-border-active font-mono transition-all duration-200" />
          </div>
          <div>
            <label for="add-transport" class="block text-sm font-medium text-text-secondary mb-1">Transport</label>
            <select id="add-transport" bind:value={formTransport} class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active transition-all duration-200">
              {#each transportOptions as opt}
                <option value={opt}>{opt.toUpperCase()}</option>
              {/each}
            </select>
          </div>
        </div>
      {/if}
      
      {#if formProtocol === 'shadowsocks'}
        <div>
          <label for="add-method" class="block text-sm font-medium text-text-secondary mb-1">Encryption Method</label>
          <select id="add-method" bind:value={formMethod} class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active transition-all duration-200">
            {#each ssMethodOptions as opt}
              <option value={opt}>{opt}</option>
            {/each}
          </select>
        </div>
        <div>
          <label for="add-password" class="block text-sm font-medium text-text-secondary mb-1">Password</label>
          <input id="add-password" type="password" bind:value={formPassword} class="w-full px-4 py-2.5 bg-void-50 border border-glass-border rounded-xl text-text-primary focus:outline-none focus:border-glass-border-active transition-all duration-200" required />
        </div>
      {/if}
      
      <div class="flex justify-end gap-3 pt-4">
        <Button variant="secondary" onclick={handleClose}>Cancel</Button>
        <button type="submit" class="px-4 py-2.5 bg-indigo-500 hover:bg-indigo-600 text-white font-medium text-sm rounded-xl transition-all duration-200">Save</button>
      </div>
    </form>
    </div>
  {/if}
</Modal>
