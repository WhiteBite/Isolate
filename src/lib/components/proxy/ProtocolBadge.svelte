<script lang="ts">
  type Protocol = 'vless' | 'vmess' | 'shadowsocks' | 'trojan' | 'ss' | 'http' | 'socks5';
  
  interface Props {
    protocol: Protocol;
    size?: 'sm' | 'md';
  }
  
  let { protocol, size = 'md' }: Props = $props();
  
  const protocolConfig: Record<Protocol, { label: string; color: string; bg: string }> = {
    vless: { 
      label: 'VLESS', 
      color: 'text-purple-400', 
      bg: 'bg-purple-500/20 border-purple-500/30' 
    },
    vmess: { 
      label: 'VMess', 
      color: 'text-blue-400', 
      bg: 'bg-blue-500/20 border-blue-500/30' 
    },
    shadowsocks: { 
      label: 'Shadowsocks', 
      color: 'text-cyan-400', 
      bg: 'bg-cyan-500/20 border-cyan-500/30' 
    },
    ss: { 
      label: 'SS', 
      color: 'text-cyan-400', 
      bg: 'bg-cyan-500/20 border-cyan-500/30' 
    },
    trojan: { 
      label: 'Trojan', 
      color: 'text-red-400', 
      bg: 'bg-red-500/20 border-red-500/30' 
    },
    http: { 
      label: 'HTTP', 
      color: 'text-gray-400', 
      bg: 'bg-gray-500/20 border-gray-500/30' 
    },
    socks5: { 
      label: 'SOCKS5', 
      color: 'text-orange-400', 
      bg: 'bg-orange-500/20 border-orange-500/30' 
    }
  };
  
  const sizeClasses = {
    sm: 'px-1.5 py-0.5 text-[10px]',
    md: 'px-2 py-1 text-xs'
  };
  
  let config = $derived(protocolConfig[protocol] || protocolConfig.vless);
</script>

<span 
  class="inline-flex items-center font-medium rounded border {config.bg} {config.color} {sizeClasses[size]} uppercase tracking-wide"
>
  {config.label}
</span>
