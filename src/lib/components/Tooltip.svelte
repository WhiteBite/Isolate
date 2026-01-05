<script lang="ts">
  interface Props {
    text: string;
    position?: 'top' | 'bottom' | 'left' | 'right';
    children: any;
  }
  
  let { text, position = 'top', children }: Props = $props();
  let visible = $state(false);
</script>

<div class="relative inline-block"
     onmouseenter={() => visible = true}
     onmouseleave={() => visible = false}>
  {@render children()}
  
  {#if visible}
    <div class="absolute z-50 px-3 py-2 text-sm text-white bg-[#2a2f4a] rounded-lg shadow-lg whitespace-nowrap
                {position === 'top' ? 'bottom-full left-1/2 -translate-x-1/2 mb-2' : ''}
                {position === 'bottom' ? 'top-full left-1/2 -translate-x-1/2 mt-2' : ''}
                {position === 'left' ? 'right-full top-1/2 -translate-y-1/2 mr-2' : ''}
                {position === 'right' ? 'left-full top-1/2 -translate-y-1/2 ml-2' : ''}">
      {text}
      <div class="absolute w-2 h-2 bg-[#2a2f4a] rotate-45
                  {position === 'top' ? 'top-full left-1/2 -translate-x-1/2 -mt-1' : ''}
                  {position === 'bottom' ? 'bottom-full left-1/2 -translate-x-1/2 -mb-1' : ''}"></div>
    </div>
  {/if}
</div>
