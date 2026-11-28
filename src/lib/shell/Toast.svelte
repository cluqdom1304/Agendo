<script lang="ts">
  interface Props {
    message: string;
    kind?: "info" | "error";
    actionLabel?: string;
    closeLabel: string;
    onAction?: () => void;
    onClose: () => void;
  }

  let {
    message,
    kind = "info",
    actionLabel,
    closeLabel,
    onAction,
    onClose,
  }: Props = $props();
</script>

<div class="toast" class:error={kind === "error"} role="status">
  <span>{message}</span>
  {#if actionLabel && onAction}
    <button type="button" class="action" onclick={onAction}>
      {actionLabel}
    </button>
  {/if}
  <button type="button" class="close" aria-label={closeLabel} onclick={onClose}>
    <svg viewBox="0 0 10 10" width="9" height="9" aria-hidden="true">
      <path
        d="M1.5 1.5l7 7M8.5 1.5l-7 7"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
      />
    </svg>
  </button>
</div>

<style>
  .toast {
    position: fixed;
    inset-inline: 0;
    bottom: 20px;
    z-index: 25;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: fit-content;
    max-width: calc(100vw - 2 * var(--space-4));
    margin-inline: auto;
    min-height: 38px;
    box-sizing: border-box;
    padding: 5px 6px 5px 18px;
    border-radius: var(--radius-chip);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    font-size: 13px;
    font-weight: 500;
    animation: toast-in var(--motion-spring) var(--ease-bounce);
  }

  /* A long message wraps inside the pill instead of pushing the buttons out. */
  .toast > span {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .toast.error {
    box-shadow:
      inset 0 0 0 1px var(--danger),
      var(--shadow-popover);
  }

  button {
    display: grid;
    flex: none;
    place-items: center;
    min-width: 24px;
    height: 24px;
    border: 0;
    border-radius: var(--radius-chip);
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  button:hover {
    background: var(--bg-hover);
  }

  button:active {
    transform: scale(0.94);
  }

  .action {
    height: 28px;
    padding: 0 12px;
    background: var(--ink);
    color: var(--on-ink);
    font-weight: 600;
  }

  .action:hover {
    background: color-mix(in srgb, var(--ink) 84%, var(--bg-popover));
  }

  .close {
    padding: 0;
    color: var(--text-secondary);
  }

  .close:hover {
    color: var(--text-primary);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(14px) scale(0.94);
    }
  }
</style>
