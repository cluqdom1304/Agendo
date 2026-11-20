<script lang="ts">
  import { t } from "../../i18n";
  import { contextMenu } from "../context/context.svelte";
  import { folderDrag } from "../../stores/tagDrag.svelte";
  import type { NotesStore, ScopeKey } from "../../stores/notes.svelte";
  import type { TagsStore } from "../../stores/tags.svelte";
  import type { FolderWithCount } from "../../types";

  interface Props {
    notes: NotesStore;
    tags: TagsStore;
    /** Pending tasks per tag, to show next to the number of notes. */
    pendingOf?: (tagId: string) => number;
    /** In the notes section: smart folders, tags of the notes and the trash too. */
    full: boolean;
    /** Called when a folder or a smart folder is chosen, before the list changes. */
    onNavigate?: () => void;
  }

  let { notes, tags, pendingOf, full, onNavigate }: Props = $props();

  /** `"new"` (top level), `"new:<parent>"`, `"rename:<id>"` or nothing. */
  let editing = $state<string | null>(null);
  let draft = $state("");
  let confirmingDelete = $state<string | null>(null);

  // Where the thing being dragged may land: a note goes to the inbox or a folder, a folder
  // to the top level or another folder. Anything else is not lit up.
  const drop = $derived.by(() => {
    const drag = folderDrag.active;
    if (!drag?.overKey) return null;
    const isNote = drag.id.startsWith("note:");
    if (drag.overKey === (isNote ? "root" : "inbox")) return null;
    if (drag.id === `folder:${drag.overKey}`) return null;
    return drag.overKey;
  });

  // The context menu (D-50) asks for the fields that live here.
  $effect(() => {
    const intent = contextMenu.take("folder");
    if (!intent) return;
    if (intent.mode === "delete") confirmingDelete = intent.id;
    else if (intent.mode === "rename") {
      begin(`rename:${intent.id}`, notes.byId.get(intent.id)?.name ?? "");
    } else {
      notes.expanded.add(intent.id);
      begin(`new:${intent.id}`);
    }
  });

  function select(scope: ScopeKey) {
    onNavigate?.();
    void notes.selectScope(scope);
  }

  function begin(mode: string, name = "") {
    editing = mode;
    draft = name;
    confirmingDelete = null;
  }

  async function commit() {
    const mode = editing;
    const name = draft.trim();
    editing = null;
    if (!mode || name === "") return;
    if (mode === "new") await notes.createFolder(name);
    else if (mode.startsWith("new:")) {
      await notes.createFolder(name, mode.slice(4));
    } else await notes.renameFolder(mode.slice(7), name);
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  function onEditKey(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      void commit();
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      editing = null;
    }
  }

  function press(event: PointerEvent, folder: FolderWithCount) {
    if (editing) return;
    folderDrag.begin(
      event,
      { id: `folder:${folder.id}`, title: folder.name },
      (key) => {
        // The inbox holds notes, not folders.
        if (key === "inbox") return;
        const target = key === "root" ? null : key;
        if (target !== folder.id) void notes.moveFolder(folder.id, target);
      },
    );
  }

  async function remove(id: string) {
    confirmingDelete = null;
    await notes.deleteFolder(id);
  }
</script>

<section class="group" aria-labelledby="sidebar-folders">
  <div class="inner">
    <header class="head">
      <h2
        id="sidebar-folders"
        data-folder-drop="root"
        class:drop-target={drop === "root"}
      >
        {t.notesApp.folders}
      </h2>
      <button
        type="button"
        class="icon"
        aria-label={t.notesApp.newFolder}
        title={t.notesApp.newFolder}
        onclick={() => begin("new")}
      >
        <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true">
          <path
            d="M2 5.2c0-.7.5-1.2 1.2-1.2h2.6l1.4 1.5h5.6c.7 0 1.2.5 1.2 1.2v5.1c0 .7-.5 1.2-1.2 1.2H3.2c-.7 0-1.2-.5-1.2-1.2z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linejoin="round"
          />
          <path
            d="M8 7.6v3M6.5 9.1h3"
            stroke="currentColor"
            stroke-width="1.3"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </header>

    {#if full}
      <nav class="smart" aria-label={t.notesApp.smartFolders}>
        {@render smart("all", t.notesApp.all, "M3 4h10M3 8h10M3 12h6")}
        <div
          class="row-wrap"
          class:drop-target={drop === "inbox"}
          data-folder-drop="inbox"
        >
          {@render smart(
            "inbox",
            t.notesApp.inbox,
            "M2.5 9.5l1.7-5.2c.2-.5.6-.8 1.2-.8h5.2c.6 0 1 .3 1.2.8l1.7 5.2v2.3c0 .6-.5 1.2-1.2 1.2H3.7c-.7 0-1.2-.6-1.2-1.2zM2.5 9.5h3.2l.8 1.5h3l.8-1.5h3.2",
          )}
        </div>
        {@render smart(
          "pinned",
          t.notesApp.pinned,
          "M9.8 2.5l3.7 3.7-1.6.5-2.3 2.3.2 3-.9.9-2.7-2.7-3.5 3.5M6.4 5.2l.9-.9 3 .2",
        )}
        {@render smart(
          "orphans",
          t.notesApp.orphans,
          "M6.5 9.5l3-3M5 8L3.5 9.5a2.1 2.1 0 003 3L8 11M11 8l1.5-1.5a2.1 2.1 0 00-3-3L8 5",
        )}
      </nav>
    {/if}

    {#if editing === "new"}
      <div class="edit" style:padding-left="12px">
        {@render input()}
      </div>
    {/if}

    <ul class="tree" role="tree">
      {#each notes.rows as row (row.folder.id)}
        {@const folder = row.folder}
        {@const key = `folder:${folder.id}` as ScopeKey}
        <li
          role="treeitem"
          aria-selected={notes.scope === key}
          aria-expanded={row.hasChildren ? row.expanded : undefined}
        >
          {#if editing === `rename:${folder.id}`}
            <div class="edit" style:padding-left="{12 + row.depth * 14}px">
              {@render input()}
            </div>
          {:else if confirmingDelete === folder.id}
            <div class="confirm" style:padding-left="{12 + row.depth * 14}px">
              <span>{t.notesApp.confirmDeleteFolder}</span>
              <span class="confirm-actions">
                <button
                  type="button"
                  class="btn small"
                  onclick={() => (confirmingDelete = null)}
                >
                  {t.notesApp.cancel}
                </button>
                <button
                  type="button"
                  class="btn small danger-fill"
                  aria-label={t.notesApp.deleteFolder(folder.name)}
                  onclick={() => void remove(folder.id)}
                >
                  {t.notesApp.delete}
                </button>
              </span>
            </div>
          {:else}
            <div
              class="row has-actions"
              class:active={notes.scope === key}
              class:drop-target={drop === folder.id}
              data-folder-drop={folder.id}
              data-folder-id={folder.id}
              style:padding-left="{4 + row.depth * 14}px"
            >
              {#if row.hasChildren}
                <button
                  type="button"
                  class="chevron"
                  class:turned={row.expanded}
                  aria-label={row.expanded
                    ? t.notesApp.collapse(folder.name)
                    : t.notesApp.expand(folder.name)}
                  onclick={() => notes.toggleExpanded(folder.id)}
                >
                  <svg
                    viewBox="0 0 12 12"
                    width="10"
                    height="10"
                    aria-hidden="true"
                  >
                    <path
                      d="M4 2.5l3.5 3.5L4 9.5"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.6"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </button>
              {:else}
                <span class="chevron-space"></span>
              {/if}
              <button
                type="button"
                class="name"
                onclick={() => select(key)}
                onpointerdown={(event) => press(event, folder)}
              >
                <svg
                  viewBox="0 0 16 16"
                  width="15"
                  height="15"
                  aria-hidden="true"
                >
                  <path
                    d="M2 5.2c0-.7.5-1.2 1.2-1.2h2.6l1.4 1.5h5.6c.7 0 1.2.5 1.2 1.2v5.1c0 .7-.5 1.2-1.2 1.2H3.2c-.7 0-1.2-.5-1.2-1.2z"
                    fill="currentColor"
                    fill-opacity="0.18"
                    stroke="currentColor"
                    stroke-width="1.3"
                    stroke-linejoin="round"
                  />
                </svg>
                <span class="label">{folder.name}</span>
              </button>
              <span class="count" class:zero={folder.noteCount === 0}>
                {folder.noteCount}
              </span>
              <span class="actions">
                <button
                  type="button"
                  aria-label={t.notesApp.newSubfolder(folder.name)}
                  title={t.notesApp.newSubfolder(folder.name)}
                  onclick={() => {
                    notes.expanded.add(folder.id);
                    begin(`new:${folder.id}`);
                  }}
                >
                  <svg
                    viewBox="0 0 16 16"
                    width="11"
                    height="11"
                    aria-hidden="true"
                  >
                    <path
                      d="M8 3v10M3 8h10"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.7"
                      stroke-linecap="round"
                    />
                  </svg>
                </button>
                <button
                  type="button"
                  aria-label={t.notesApp.renameFolder(folder.name)}
                  title={t.notesApp.renameFolder(folder.name)}
                  onclick={() => begin(`rename:${folder.id}`, folder.name)}
                >
                  <svg
                    viewBox="0 0 16 16"
                    width="11"
                    height="11"
                    aria-hidden="true"
                  >
                    <path
                      d="M11 2.5l2.5 2.5L5.5 13H3v-2.5z"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.5"
                      stroke-linejoin="round"
                    />
                  </svg>
                </button>
                <button
                  type="button"
                  aria-label={t.notesApp.deleteFolder(folder.name)}
                  title={t.notesApp.deleteFolder(folder.name)}
                  onclick={() => (confirmingDelete = folder.id)}
                >
                  <svg
                    viewBox="0 0 16 16"
                    width="11"
                    height="11"
                    aria-hidden="true"
                  >
                    <path
                      d="M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 8h5.8l.6-8"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="1.4"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </button>
              </span>
            </div>
          {/if}
          {#if editing === `new:${folder.id}`}
            <div
              class="edit"
              style:padding-left="{12 + (row.depth + 1) * 14}px"
            >
              {@render input()}
            </div>
          {/if}
        </li>
      {/each}
    </ul>

    {#if full && tags.tags.length > 0}
      <div class="section">
        <span class="section-title">{t.notesApp.tags}</span>
      </div>
      <ul class="tag-list">
        {#each tags.ordered as tag (tag.id)}
          {@const key = `tag:${tag.id}` as ScopeKey}
          <li class:child={!!tag.parentId}>
            <button
              type="button"
              class="row smart-row"
              data-note-tag-id={tag.id}
              class:active={notes.scope === key}
              aria-current={notes.scope === key ? "page" : undefined}
              title={t.notesApp.tagTotals(
                pendingOf?.(tag.id) ?? 0,
                tag.noteCount,
              )}
              onclick={() => select(key)}
            >
              <span class="dot" style:background={tag.colorHex}></span>
              <span class="label">{tag.name}</span>
              <span class="count" class:zero={tag.noteCount === 0}>
                {tag.noteCount}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}

    {#if full}
      <div class="bottom">
        {@render smart(
          "trash",
          t.notesApp.trash,
          "M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 8h5.8l.6-8",
        )}
      </div>
    {/if}
  </div>
</section>

{#snippet input()}
  <input
    class="folder-input"
    aria-label={t.notesApp.folderNameLabel}
    placeholder={t.notesApp.folderNamePlaceholder}
    maxlength="100"
    bind:value={draft}
    use:focusOnMount
    onkeydown={onEditKey}
    onblur={() => void commit()}
  />
{/snippet}

{#snippet smart(scope: ScopeKey, label: string, path: string)}
  <button
    type="button"
    class="row smart-row"
    class:active={notes.scope === scope}
    aria-current={notes.scope === scope ? "page" : undefined}
    onclick={() => select(scope)}
  >
    <svg viewBox="0 0 16 16" width="15" height="15" aria-hidden="true">
      <path
        d={path}
        fill="none"
        stroke="currentColor"
        stroke-width="1.4"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <span class="label">{label}</span>
  </button>
{/snippet}

<style>
  .inner {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 2px 0 0;
  }

  /* The heading is also where a folder is dropped to take it to the top level. */
  h2 {
    flex: 1;
    margin: 0;
    padding: 3px var(--space-2);
    border-radius: var(--radius-small);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
  }

  .icon,
  .actions button {
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

  /* The icons of the rows are tinted; the ones of their buttons are not. */
  .row .actions svg {
    color: inherit;
  }

  .icon:hover,
  .actions button:hover {
    background: var(--bg-control-hover);
    color: var(--text-primary);
  }

  .icon:active,
  .actions button:active {
    transform: scale(0.9);
  }

  .smart {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .section {
    margin-top: var(--space-3);
    padding: 0 var(--space-2) 2px;
  }

  .section-title {
    display: block;
    margin: 0 calc(var(--space-2) * -1);
    padding: 3px var(--space-2);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    border-radius: var(--radius-small);
  }

  .tree {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row,
  .row-wrap {
    border-radius: var(--radius-small);
  }

  .row {
    display: flex;
    align-items: center;
    min-height: 30px;
    transition: background-color var(--motion-instant) linear;
  }

  .row:hover,
  .row:focus-within {
    background: var(--bg-hover);
  }

  .row.active {
    background: var(--bg-active);
  }

  .row.active .label {
    font-weight: 600;
  }

  .drop-target {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }

  .smart-row,
  .name {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    padding: 5px var(--space-2);
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .smart-row {
    width: 100%;
  }

  .row svg {
    flex: none;
    color: var(--accent);
  }

  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chevron,
  .chevron-space {
    flex: none;
    width: 16px;
    height: 20px;
  }

  .chevron {
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .chevron svg {
    color: inherit;
    transition: transform var(--motion-fast) var(--ease-out);
  }

  .chevron.turned svg {
    transform: rotate(90deg);
  }

  .count {
    flex: none;
    min-width: 16px;
    margin-right: var(--space-2);
    color: var(--text-secondary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .count.zero {
    opacity: 0.5;
  }

  /* Inside a button the padding of the button already keeps the number off the edge. */
  .smart-row .count {
    margin-right: 0;
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

  /* The number only makes room where there are buttons to show instead. */
  .has-actions:hover .count,
  .has-actions:focus-within .count {
    display: none;
  }

  .edit {
    padding-right: 4px;
  }

  .folder-input {
    box-sizing: border-box;
    width: 100%;
    height: 28px;
    padding: 0 var(--space-2);
    border: 0;
    border-radius: var(--radius-small);
    background: var(--bg-control);
    box-shadow:
      inset 0 0 0 1px var(--accent),
      0 0 0 3px color-mix(in srgb, var(--accent) 30%, transparent);
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    outline: none;
  }

  .confirm {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 4px 8px 12px;
    color: var(--text-secondary);
    font-size: 12px;
    animation: rise-in var(--motion-fast) var(--ease-out);
  }

  .confirm-actions {
    display: flex;
    gap: var(--space-1);
  }

  .tag-list {
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

  .dot {
    flex: none;
    width: 10px;
    height: 10px;
    margin: 0 2px;
    border-radius: var(--radius-chip);
  }

  .bottom {
    margin-top: auto;
    padding-top: var(--space-3);
  }

  .bottom::before {
    content: "";
    display: block;
    height: 1px;
    margin: 0 var(--space-2) var(--space-2);
    background: var(--separator);
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @media (prefers-reduced-motion: reduce) {
    .chevron svg {
      transition: none;
    }
  }
</style>
