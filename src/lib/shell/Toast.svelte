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
    &times;
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
    padding: 5px 6px 5px var(--space-4);
    border-radius: var(--radius-chip);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    font-size: 13px;
    animation: toast-in var(--motion-spring) var(--ease-bounce);
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
    padding: 0 10px;
    color: var(--accent);
    font-weight: 600;
  }

  .close {
    color: var(--text-secondary);
    font-size: 16px;
    line-height: 1;
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(14px) scale(0.94);
    }
  }
</style>
