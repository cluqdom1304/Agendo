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
    width: 18px;
    height: 18px;
    padding: 0;
    border: 1.5px solid var(--text-secondary);
    border-radius: var(--radius-chip);
    background: transparent;
    color: white;
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-apple),
      border-color var(--motion-fast) var(--ease-apple);
  }

  .checkbox:disabled {
    cursor: default;
    opacity: 0.4;
  }

  .checkbox.checked {
    border-color: var(--accent);
    background: var(--accent);
    animation: check-pop var(--motion-fast) var(--ease-elastic);
  }

  .mark {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
    opacity: 0;
  }

  .checked .mark {
    opacity: 1;
  }

  @keyframes check-pop {
    0% {
      transform: scale(0.85);
    }
    60% {
      transform: scale(1.1);
    }
    100% {
      transform: scale(1);
    }
  }
</style>
