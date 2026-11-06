<script lang="ts">
  interface Props {
    title: string;
    /** Secondary text, e.g. how many tasks are pending. */
    detail?: string;
  }

  let { title, detail }: Props = $props();
</script>

<header class="section">
  <h2>{title}</h2>
  {#if detail}<span class="detail">{detail}</span>{/if}
</header>

<style>
  .section {
    display: flex;
    align-items: baseline;
    gap: var(--space-3);
    padding: var(--space-6) var(--space-3) var(--space-2);
    border-bottom: 1px solid var(--border-subtle);
  }

  h2 {
    margin: 0;
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
  }

  .detail {
    color: var(--text-secondary);
    font-size: 12px;
  }
</style>
