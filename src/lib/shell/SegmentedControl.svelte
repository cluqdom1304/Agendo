<script lang="ts" generics="T extends string">
  interface Option {
    value: T;
    label: string;
  }

  interface Props {
    options: readonly Option[];
    value: T;
    label: string;
    onChange: (value: T) => void;
  }

  let { options, value, label, onChange }: Props = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
  {#each options as option (option.value)}
    <button
      type="button"
      role="radio"
      aria-checked={option.value === value}
      class:active={option.value === value}
      onclick={() => onChange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    padding: 2px;
    gap: 2px;
    border-radius: var(--radius-control);
    background: var(--bg-hover);
  }

  button {
    padding: 4px 14px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    transition:
      background var(--motion-fast) var(--ease-apple),
      color var(--motion-fast) var(--ease-apple);
  }

  button:hover {
    color: var(--text-primary);
  }

  button.active {
    background: var(--bg-elevated);
    color: var(--text-primary);
    font-weight: 600;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
