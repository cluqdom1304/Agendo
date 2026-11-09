<script lang="ts">
  interface Props {
    title: string;
    description: string;
    actionLabel: string;
    shortcut?: string;
    onAction: () => void;
  }

  let { title, description, actionLabel, shortcut, onAction }: Props = $props();
</script>

<div class="empty">
  <h2>{title}</h2>
  <p>{description}</p>
  <button type="button" onclick={onAction}>
    {actionLabel}
    {#if shortcut}<kbd>{shortcut}</kbd>{/if}
  </button>
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: 96px var(--space-6);
    text-align: center;
  }

  h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
  }

  p {
    margin: 0;
    color: var(--text-secondary);
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-3);
    padding: 6px var(--space-4);
    border: 0;
    border-radius: var(--radius-control);
    background: var(--accent);
    color: var(--on-accent);
    font: inherit;
    cursor: pointer;
  }

  kbd {
    padding: 0 6px;
    border-radius: 4px;
    background: color-mix(in srgb, currentColor 22%, transparent);
    font: inherit;
    font-size: 12px;
  }
</style>
