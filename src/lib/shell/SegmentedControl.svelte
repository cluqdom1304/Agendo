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

  const index = $derived(
    Math.max(
      0,
      options.findIndex((option) => option.value === value),
    ),
  );
</script>

<div
  class="segmented"
  role="radiogroup"
  aria-label={label}
  style:--count={options.length}
  style:--index={index}
>
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
    position: relative;
    display: inline-grid;
    grid-auto-columns: 1fr;
    grid-auto-flow: column;
    padding: 2px;
    border-radius: var(--radius-control);
    background: var(--bg-control);
  }

  /* One thumb that slides between equal columns: only its transform changes. */
  .segmented::before {
    content: "";
    position: absolute;
    top: 2px;
    bottom: 2px;
    left: 2px;
    width: calc((100% - 4px) / var(--count));
    border-radius: var(--radius-small);
    background: var(--thumb);
    box-shadow: var(--shadow-raised);
    transform: translateX(calc(var(--index) * 100%));
    transition: transform var(--motion-spring) var(--ease-spring);
  }

  button {
    position: relative;
    min-height: 24px;
    padding: 0 14px;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
    cursor: pointer;
    transition:
      color var(--motion-fast) var(--ease-out),
      transform var(--motion-instant) var(--ease-out);
  }

  button:hover,
  button.active {
    color: var(--text-primary);
  }

  button:active {
    transform: scale(0.96);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
