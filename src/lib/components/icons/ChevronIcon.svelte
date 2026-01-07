<script lang="ts">
  interface Props {
    size?: number;
    class?: string;
    direction?: 'up' | 'down' | 'left' | 'right';
  }
  let { size = 20, class: className = '', direction = 'right' }: Props = $props();

  const rotations = {
    up: 'rotate(-90deg)',
    down: 'rotate(90deg)',
    left: 'rotate(180deg)',
    right: 'rotate(0deg)'
  };
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  style="transform: {rotations[direction]}"
>
  <path d="m9 18 6-6-6-6" />
</svg>
