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
  .bar {
    position: fixed;
    left: 50%;
    bottom: 72px;
    z-index: 30;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    transform: translateX(-50%);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    background: var(--bg-elevated, var(--bg-hover));
    box-shadow: 0 6px 24px rgb(0 0 0 / 0.25);
    font-size: 13px;
  }

  .count {
    padding-right: var(--space-2);
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .action {
    padding: 4px var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    cursor: pointer;
  }

  .action:hover {
    background: var(--bg-hover);
  }

  .danger {
    color: var(--danger);
  }

  .date {
    width: 130px;
  }
</style>
