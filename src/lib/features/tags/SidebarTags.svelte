<script lang="ts">
  import { t } from "../../i18n";
  import { contextMenu } from "../context/context.svelte";
  import { tagDrag } from "../../stores/tagDrag.svelte";
  import type { TagsStore } from "../../stores/tags.svelte";
  import { NO_TAG } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Tag } from "../../types";
  import TagEditor from "./TagEditor.svelte";

  interface Props {
    tags: TagsStore;
    tasks: TasksStore;
  }

  let { tags, tasks }: Props = $props();

  /** `"new"`, the id of the tag being edited, or nothing. */
  let editing = $state<string | null>(null);

  // Pending tasks per filter key. A task without a (known) tag counts under NO_TAG.
  const counts = $derived.by(() => {
    // Built fresh on each run and never mutated afterwards.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const result = new Map<string, number>();
    for (const task of tasks.tasks) {
      if (task.isCompleted) continue;
      // A task with several tags counts under each of them.
      const own = tags.tagsOf(task);
      // A task with a subtag also counts under its parent, once.
      const keys = new Set<string>(); // eslint-disable-line svelte/prefer-svelte-reactivity
      for (const tag of own) {
        keys.add(tag.id);
        if (tag.parentId) keys.add(tag.parentId);
      }
      for (const key of keys.size > 0 ? keys : [NO_TAG]) {
        result.set(key, (result.get(key) ?? 0) + 1);
      }
    }
    return result;
  });

  // The context menu (D-50) opens the editor of a tag.
  $effect(() => {
    const intent = contextMenu.take("tag-edit");
    if (intent) editing = intent.id;
  });

  const editedTag = $derived(
    editing && editing !== "new" ? tags.byId.get(editing) : undefined,
  );

  async function save(
    id: string | undefined,
    name: string,
    colorHex: string,
    parentId: string | null,
  ): Promise<"duplicate" | "failed" | null> {
    const failure = await tags.save({
      ...(id ? { id } : {}),
      name,
      colorHex,
      ...(parentId ? { parentId } : {}),
    });
    if (!failure) editing = null;
    return failure;
  }

  async function merge(source: Tag, targetId: string): Promise<boolean> {
    const merged = await tags.merge(source.id, targetId);
    if (merged) {
      editing = null;
      await tasks.load();
    }
    return merged;
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

<section class="group" aria-labelledby="sidebar-tags">
  <div class="inner">
    <header class="head">
      <h2 id="sidebar-tags">{t.tags.sidebar}</h2>
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
        onSave={(name, color, parent) => save(undefined, name, color, parent)}
        parentOptions={tags.tags.filter((tag) => !tag.parentId)}
        onCancel={() => (editing = null)}
      />
    {/if}

    <ul class="list">
      {#each tags.ordered as tag (tag.id)}
        <li class:child={!!tag.parentId}>
          {#if editedTag && editedTag.id === tag.id}
            <TagEditor
              name={tag.name}
              color={tag.colorHex}
              parentId={tag.parentId}
              parentOptions={tags.tags.filter(
                (other) => !other.parentId && other.id !== tag.id,
              )}
              hasChildren={tags.tags.some((other) => other.parentId === tag.id)}
              mergeTargets={tags.tags.filter((other) => other.id !== tag.id)}
              onSave={(name, color, parent) =>
                save(tag.id, name, color, parent)}
              onMerge={(targetId) => merge(tag, targetId)}
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
</section>

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
    data-tag-id={tag?.id}
  >
    <label class="check" title={t.tags.pending(count)}>
      <input
        type="checkbox"
        style:--tag={color || "var(--text-secondary)"}
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
  .inner {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 2px 0 var(--space-2);
  }

  h2 {
    margin: 0;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
  }

  .add,
  .edit {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .add:hover,
  .edit:hover {
    background: var(--bg-control-hover);
    color: var(--text-primary);
  }

  .add:active,
  .edit:active {
    transform: scale(0.9);
  }

  .bulk {
    display: flex;
    gap: var(--space-1);
    padding: 0 2px;
  }

  .bulk button,
  .only {
    padding: 1px 6px;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--accent);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  .bulk button:hover,
  .only:hover {
    background: var(--bg-control-hover);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .child {
    padding-left: 14px;
  }

  .separator {
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--separator);
  }

  .row {
    display: flex;
    align-items: center;
    min-height: 30px;
    border-radius: var(--radius-small);
    transition: background-color var(--motion-instant) linear;
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

  /* A dot of the tag's own colour: filled when its tasks are shown, hollow when hidden. */
  .check input {
    flex: none;
    appearance: none;
    width: 14px;
    height: 14px;
    margin: 0;
    border: 1.5px solid var(--tag);
    border-radius: var(--radius-chip);
    background: transparent;
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      transform var(--motion-base) var(--ease-bounce);
  }

  .check input:checked {
    background: var(--tag);
  }

  .check input:active {
    transform: scale(0.8);
  }

  .name {
    flex: 1;
    min-width: 0;
    color: var(--text-primary);
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: opacity var(--motion-fast) var(--ease-out);
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
    padding-right: 3px;
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
</style>
