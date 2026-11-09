<script lang="ts">
  import { t } from "../../i18n";
  import { tagDrag } from "../../stores/tagDrag.svelte";
  import type { TagsStore } from "../../stores/tags.svelte";
  import { NO_TAG } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Tag } from "../../types";
  import TagEditor from "./TagEditor.svelte";

  interface Props {
    tags: TagsStore;
    tasks: TasksStore;
    open: boolean;
  }

  let { tags, tasks, open }: Props = $props();

  /** `"new"`, the id of the tag being edited, or nothing. */
  let editing = $state<string | null>(null);

  // Pending tasks per filter key. A task without a (known) tag counts under NO_TAG.
  const counts = $derived.by(() => {
    // Built fresh on each run and never mutated afterwards.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const result = new Map<string, number>();
    for (const task of tasks.tasks) {
      if (task.isCompleted) continue;
      const key = tags.tagOf(task)?.id ?? NO_TAG;
      result.set(key, (result.get(key) ?? 0) + 1);
    }
    return result;
  });

  const editedTag = $derived(
    editing && editing !== "new" ? tags.byId.get(editing) : undefined,
  );

  async function save(
    id: string | undefined,
    name: string,
    colorHex: string,
  ): Promise<"duplicate" | "failed" | null> {
    const failure = await tags.save({ ...(id ? { id } : {}), name, colorHex });
    if (!failure) editing = null;
    return failure;
  }

  async function remove(tag: Tag): Promise<boolean> {
    const removed = await tags.remove(tag.id);
    if (removed) {
      editing = null;
      await tasks.load();
    }
    return removed;
  }
</script>

<aside class="sidebar" class:open aria-label={t.tags.sidebar} inert={!open}>
  <div class="inner">
    <header class="head">
      <h2>{t.tags.sidebar}</h2>
      <button
        type="button"
        class="add"
        aria-label={t.tags.newTag}
        title={t.tags.newTag}
        onclick={() => (editing = "new")}
      >
        <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
          <path
            d="M8 3v10M3 8h10"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </header>

    <div class="bulk">
      <button type="button" onclick={() => tags.showAll()}>{t.tags.all}</button>
      <button type="button" onclick={() => tags.hideAll()}>
        {t.tags.none}
      </button>
    </div>

    {#if editing === "new"}
      <TagEditor
        color={tags.nextColor()}
        onSave={(name, color) => save(undefined, name, color)}
        onCancel={() => (editing = null)}
      />
    {/if}

    <ul class="list">
      {#each tags.tags as tag (tag.id)}
        <li>
          {#if editedTag && editedTag.id === tag.id}
            <TagEditor
              name={tag.name}
              color={tag.colorHex}
              onSave={(name, color) => save(tag.id, name, color)}
              onDelete={() => remove(tag)}
              onCancel={() => (editing = null)}
            />
          {:else}
            {@render row(
              tag.id,
              tag.name,
              tag.colorHex,
              counts.get(tag.id) ?? 0,
              tag,
            )}
          {/if}
        </li>
      {/each}
      <li class="separator">
        {@render row(NO_TAG, t.tags.noTag, "", counts.get(NO_TAG) ?? 0)}
      </li>
    </ul>
  </div>
</aside>

{#snippet row(
  key: string,
  name: string,
  color: string,
  count: number,
  tag?: Tag,
)}
  <div
    class="row"
    class:unchecked={!tags.isChecked(key)}
    class:drop-target={tagDrag.active?.overKey === key}
    data-tag-drop={key}
  >
    <label class="check" title={t.tags.pending(count)}>
      <input
        type="checkbox"
        style:accent-color={color || "var(--text-secondary)"}
        checked={tags.isChecked(key)}
        aria-label={t.tags.toggleLabel(name)}
        onchange={() => tags.toggle(key)}
      />
      <span class="name" class:plain={!color}>{name}</span>
      <span class="count" class:zero={count === 0}>{count}</span>
    </label>
    <span class="actions">
      <button
        type="button"
        class="only"
        aria-label={t.tags.onlyLabel(name)}
        title={t.tags.onlyLabel(name)}
        onclick={() => tags.only(key)}
      >
        {t.tags.only}
      </button>
      {#if tag}
        <button
          type="button"
          class="edit"
          aria-label={t.tags.editTag(name)}
          title={t.tags.editTag(name)}
          onclick={() => (editing = tag.id)}
        >
          <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
            <path
              d="M11 2.5l2.5 2.5L5.5 13H3v-2.5z"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linejoin="round"
            />
          </svg>
        </button>
      {/if}
    </span>
  </div>
{/snippet}

<style>
  .sidebar {
    flex: none;
    width: 0;
    overflow: hidden;
    border-right: 0 solid var(--border-subtle);
    transition:
      width 200ms var(--ease-apple),
      border-width 200ms var(--ease-apple);
  }

  .sidebar.open {
    width: 224px;
    border-right-width: 1px;
  }

  .inner {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    width: 224px;
    height: 100%;
    padding: var(--space-3) var(--space-2);
    overflow-y: auto;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-2);
  }

  h2 {
    margin: 0;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .add,
  .edit {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .add:hover,
  .edit:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .bulk {
    display: flex;
    gap: var(--space-1);
    padding: 0 var(--space-2);
  }

  .bulk button,
  .only {
    padding: 1px 6px;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--accent);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }

  .bulk button:hover,
  .only:hover {
    background: var(--bg-hover);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .separator {
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--border-subtle);
  }

  .row {
    display: flex;
    align-items: center;
    border-radius: 6px;
  }

  .row:hover,
  .row:focus-within {
    background: var(--bg-hover);
  }

  .row.drop-target {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }

  .check {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    padding: 5px var(--space-2);
    cursor: pointer;
  }

  .check input {
    flex: none;
    width: 14px;
    height: 14px;
    margin: 0;
    cursor: pointer;
  }

  .name {
    flex: 1;
    min-width: 0;
    color: var(--text-primary);
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .name.plain {
    color: var(--text-secondary);
  }

  .unchecked .name {
    opacity: 0.5;
  }

  .count {
    flex: none;
    min-width: 16px;
    color: var(--text-secondary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .count.zero {
    opacity: 0.5;
  }

  .actions {
    display: none;
    flex: none;
    align-items: center;
    padding-right: 2px;
  }

  .row:hover .actions,
  .row:focus-within .actions {
    display: flex;
  }

  .row:hover .count,
  .row:focus-within .count {
    display: none;
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @media (prefers-reduced-motion: reduce) {
    .sidebar {
      transition: none;
    }
  }
</style>
