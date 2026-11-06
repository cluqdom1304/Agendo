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
    right: var(--space-4);
    bottom: var(--space-4);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-control);
    background: var(--bg-elevated);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
    animation: toast-in var(--motion-fast) var(--ease-apple);
  }

  .toast.error {
    border-color: var(--danger);
  }

  button {
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  .action {
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
      transform: translateY(8px);
    }
  }
</style>
