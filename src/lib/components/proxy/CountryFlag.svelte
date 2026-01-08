<script lang="ts">
  import { getCountryFlag } from '$lib/utils/countries';
  
  interface Props {
    countryCode: string;
    size?: 'sm' | 'md' | 'lg';
  }
  
  let { countryCode, size = 'md' }: Props = $props();
  
  const sizeClasses = {
    sm: 'text-lg leading-none',
    md: 'text-2xl leading-none',
    lg: 'text-4xl leading-none'
  };
  
  let flag = $derived(getCountryFlag(countryCode));
</script>

<span 
  class="{sizeClasses[size]} inline-flex items-center justify-center" 
  role="img" 
  aria-label="{countryCode || 'Unknown'} flag"
>
  {flag}
</span>
