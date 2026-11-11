<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    disabled?: boolean;
    onToggle: () => void;
  }

  let { checked, label, disabled = false, onToggle }: Props = $props();
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
    width: 20px;
    height: 20px;
    padding: 0;
    border: 1.5px solid var(--text-secondary);
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--on-accent);
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out),
      transform var(--motion-instant) var(--ease-out);
  }

  .checkbox:hover:not(:disabled) {
    border-color: var(--accent);
  }

  .checkbox:active:not(:disabled) {
    transform: scale(0.88);
  }

  .checkbox:disabled {
    cursor: default;
    opacity: 0.4;
  }

  .checkbox.checked {
    border-color: var(--accent);
    background: var(--accent);
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
