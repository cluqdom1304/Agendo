<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    disabled?: boolean;
    /** 0 none … 3 urgent (D-93): the ring takes the colour of the level (F20-02). */
    priority?: number;
    onToggle: () => void;
  }

  let {
    checked,
    label,
    disabled = false,
    priority = 0,
    onToggle,
  }: Props = $props();

  const ring = $derived(
    priority > 0 ? `var(--priority-${Math.min(priority, 3)})` : undefined,
  );
</script>

<button
  type="button"
  role="checkbox"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  tabindex="-1"
  class="checkbox"
  class:checked
  class:prioritized={ring !== undefined}
  style:--ring={ring}
  onclick={onToggle}
>
  <svg viewBox="0 0 16 16" aria-hidden="true" class="mark">
    <path d="M3.5 8.5 6.5 11.5 12.5 4.5" />
  </svg>
</button>

<style>
  .checkbox {
    display: grid;
    flex: none;
    place-items: center;
    width: 18px;
    height: 18px;
    padding: 0;
    border: 1.5px solid
      color-mix(in srgb, var(--text-secondary) 75%, transparent);
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--on-ink);
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out),
      border-radius var(--motion-base) var(--ease-spring),
      transform var(--motion-fast) var(--ease-spring);
  }

  /* The level shows in the ring and a faint fill; the label names it for screen readers. */
  .checkbox.prioritized {
    border-color: var(--ring);
    background: color-mix(in srgb, var(--ring) 14%, transparent);
  }

  .checkbox:hover:not(:disabled) {
    border-color: var(--text-primary);
  }

  .checkbox:active:not(:disabled) {
    transform: scale(0.86);
    transition-duration: var(--motion-instant);
  }

  .checkbox:disabled {
    cursor: default;
    opacity: 0.4;
  }

  /* Done turns the ring into a filled ink square, as in the reference: the shape changes too,
     so it reads without relying on colour. */
  .checkbox.checked {
    border-color: var(--ink);
    border-radius: 5px;
    background: var(--ink);
    animation: check-pop var(--motion-spring) var(--ease-bounce);
  }

  /* The tick is drawn, not faded: the dash offset runs the length of the path (about 14). */
  .mark {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-dasharray: 14;
    stroke-dashoffset: 14;
  }

  .checked .mark {
    stroke-dashoffset: 0;
    transition: stroke-dashoffset var(--motion-base) var(--ease-out) 60ms;
  }

  @keyframes check-pop {
    from {
      transform: scale(0.8);
    }
  }
</style>
