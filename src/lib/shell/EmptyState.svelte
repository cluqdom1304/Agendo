<script lang="ts">
  interface Props {
    title: string;
    description?: string;
    /** `loading` and `error` are announced to assistive technology; `empty` is plain content. */
    variant?: "empty" | "loading" | "error";
    actionLabel?: string;
    shortcut?: string;
    onAction?: () => void;
  }

  let {
    title,
    description,
    variant = "empty",
    actionLabel,
    shortcut,
    onAction,
  }: Props = $props();
</script>

<div
  class="empty"
  class:error={variant === "error"}
  role={variant === "error"
    ? "alert"
    : variant === "loading"
      ? "status"
      : undefined}
  aria-busy={variant === "loading" ? true : undefined}
>
  <h2>{title}</h2>
  {#if description}<p>{description}</p>{/if}
  {#if actionLabel && onAction}
    <button type="button" class="btn primary" onclick={onAction}>
      {actionLabel}
      {#if shortcut}<kbd>{shortcut}</kbd>{/if}
    </button>
  {/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 88px var(--space-6);
    text-align: center;
    animation: view-in var(--motion-spring) var(--ease-spring);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: var(--text-title);
    font-weight: 700;
    letter-spacing: -0.015em;
  }

  .error h2 {
    color: var(--danger);
  }

  p {
    max-width: 36ch;
    margin: 0;
    color: var(--text-secondary);
    line-height: 1.45;
  }

  button {
    margin-top: var(--space-4);
  }

  kbd {
    padding: 2px 6px;
    border-radius: var(--radius-chip);
    background: color-mix(in srgb, currentColor 22%, transparent);
    font: inherit;
    font-size: 11px;
  }
</style>
