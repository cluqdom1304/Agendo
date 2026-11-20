<script lang="ts">
  import { isSettledDate } from "../../dates";
  import { t } from "../../i18n";
  import { tagsStore } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";

  interface Props {
    store: TasksStore;
  }

  let { store }: Props = $props();

  const batch = $derived(store.batch);
</script>

{#if batch.count > 0}
  <div class="bar" role="toolbar" aria-label={t.batch.barLabel}>
    <span class="count">{t.batch.selected(batch.count)}</span>
    <button type="button" class="action" onclick={() => void batch.complete()}>
      {batch.allDone ? t.batch.reopen : t.batch.complete}
    </button>
    <select
      class="action"
      aria-label={t.batch.tagChoose}
      value=""
      onchange={(event) => {
        const tagId = event.currentTarget.value;
        event.currentTarget.value = "";
        if (tagId) void batch.addTag(tagId);
      }}
    >
      <option value="">{t.batch.tag}</option>
      {#each tagsStore.tags as tag (tag.id)}
        <option value={tag.id}>{tag.name}</option>
      {/each}
    </select>
    <input
      class="action date"
      type="date"
      aria-label={t.batch.moveLabel}
      title={t.batch.move}
      onchange={(event) => {
        const value = event.currentTarget.value;
        if (value && isSettledDate(value)) void batch.moveTo(value);
        event.currentTarget.value = "";
      }}
    />
    <button
      type="button"
      class="action danger"
      onclick={() => void batch.remove()}
    >
      {t.batch.delete}
    </button>
    <button
      type="button"
      class="action"
      aria-label={t.batch.clear}
      title={t.batch.clear}
      onclick={() => batch.clear()}
    >
      ✕
    </button>
  </div>
{/if}

<style>
  /* A floating capsule over the list, like the toast: the selection is the one thing on screen. */
  .bar {
    position: fixed;
    left: 50%;
    bottom: 72px;
    z-index: 30;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 6px 8px 6px 18px;
    transform: translateX(-50%);
    border-radius: var(--radius-chip);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    font-size: 13px;
    animation: bar-in var(--motion-spring) var(--ease-spring);
  }

  @keyframes bar-in {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(12px) scale(0.96);
    }
  }

  .count {
    padding-right: var(--space-2);
    color: var(--text-primary);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .action {
    box-sizing: border-box;
    min-height: 28px;
    padding: 0 14px;
    border: 0;
    border-radius: var(--radius-chip);
    background: var(--bg-control);
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      transform var(--motion-fast) var(--ease-spring);
  }

  .action:hover {
    background: var(--bg-control-hover);
  }

  .action:active {
    transform: scale(0.96);
  }

  .action:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .danger {
    color: var(--danger);
  }

  .date {
    width: 130px;
  }
</style>
