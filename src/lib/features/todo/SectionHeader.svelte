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
    gap: 10px;
    padding: 20px var(--space-3) var(--space-1);
  }

  /* The label is stored in capitals; a large title reads better in sentence case. */
  h2 {
    margin: 0;
    color: var(--text-primary);
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 700;
    line-height: 28px;
    letter-spacing: -0.02em;
    text-transform: lowercase;
  }

  h2::first-letter {
    text-transform: uppercase;
  }

  .detail {
    color: var(--text-secondary);
    font-size: 13px;
  }
</style>
