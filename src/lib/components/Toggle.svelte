<script lang="ts">
  interface Props {
    checked: boolean;
    onchange?: (checked: boolean) => void;
    disabled?: boolean;
    label?: string;
    'aria-labelledby'?: string;
    'aria-label'?: string;
  }

  let { 
    checked = $bindable(false), 
    onchange, 
    disabled = false, 
    label,
    'aria-labelledby': ariaLabelledby,
    'aria-label': ariaLabel
  }: Props = $props();

  function handleToggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleToggle();
    }
  }
</script>

<label class="inline-flex items-center gap-3 cursor-pointer {disabled ? 'opacity-50 cursor-not-allowed' : ''}">
  <button
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={ariaLabel || label || 'Toggle'}
    aria-labelledby={ariaLabelledby}
    disabled={disabled}
    onclick={handleToggle}
    onkeydown={handleKeydown}
    class="relative inline-flex h-6 w-11 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:ring-offset-2 focus:ring-offset-zinc-900 {checked ? 'bg-indigo-500' : 'bg-zinc-700'}"
  >
    <span
      class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {checked ? 'translate-x-5' : 'translate-x-0'}"
    ></span>
  </button>
  {#if label}
    <span class="text-sm text-zinc-300">{label}</span>
  {/if}
</label>
