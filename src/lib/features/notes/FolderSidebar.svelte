<script lang="ts">
  import { t } from "../../i18n";
  import { folderDrag } from "../../stores/tagDrag.svelte";
  import type { NotesStore, ScopeKey } from "../../stores/notes.svelte";
  import type { FolderWithCount } from "../../types";

  interface Props {
    notes: NotesStore;
    open: boolean;
  }

  let { notes, open }: Props = $props();

  /** `"new"` (top level), `"new:<parent>"`, `"rename:<id>"` or nothing. */
  let editing = $state<string | null>(null);
  let draft = $state("");
  let confirmingDelete = $state<string | null>(null);

  const drop = $derived(folderDrag.active?.overKey ?? null);

  function select(scope: ScopeKey) {
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

<aside
  class="sidebar"
  class:open
  aria-label={t.notesApp.foldersHeading}
  inert={!open}
>
  <div class="inner">
    <header class="head" data-tauri-drag-region>
      <h2 data-tauri-drag-region>{t.notesApp.foldersHeading}</h2>
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

    <nav class="smart" aria-label={t.notesApp.foldersHeading}>
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
    </nav>

    <div class="section">
      <span
        class="section-title"
        data-folder-drop="root"
        class:drop-target={drop === "root"}
      >
        {t.notesApp.folders}
      </span>
    </div>

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
                  class="danger"
                  onclick={() => void remove(folder.id)}
                >
                  {t.notesApp.deleteFolder(folder.name)}
                </button>
                <button type="button" onclick={() => (confirmingDelete = null)}>
                  {t.notesApp.cancel}
                </button>
              </span>
            </div>
          {:else}
            <div
              class="row"
              class:active={notes.scope === key}
              class:drop-target={drop === folder.id}
              data-folder-drop={folder.id}
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
              <span class="count">{folder.noteCount}</span>
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
                  +
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

    <div class="bottom">
      {@render smart(
        "trash",
        t.notesApp.trash,
        "M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 8h5.8l.6-8",
      )}
    </div>
  </div>
</aside>

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
  .sidebar {
    flex: none;
    width: 0;
    overflow: hidden;
    background: var(--bg-sidebar);
    box-shadow: inset -1px 0 0 var(--separator);
    transition: width var(--motion-base) var(--ease-out);
  }

  .sidebar.open {
    width: 224px;
  }

  .inner {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 224px;
    height: 100%;
    padding: 0 var(--space-2) var(--space-3);
    overflow-y: auto;
    opacity: 0;
    transform: translateX(-16px);
    transition:
      opacity var(--motion-base) var(--ease-out),
      transform var(--motion-base) var(--ease-out);
  }

  .open .inner {
    opacity: 1;
    transform: none;
  }

  .head {
    display: flex;
    flex: none;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 2px 0 var(--space-2);
  }

  h2 {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
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
    font: inherit;
    font-size: 15px;
    line-height: 1;
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
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
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.02em;
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

  .active svg {
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
    gap: var(--space-1);
    padding: 6px 4px 6px 12px;
    color: var(--text-secondary);
    font-size: 12px;
    animation: rise-in var(--motion-fast) var(--ease-out);
  }

  .confirm-actions {
    display: flex;
    gap: var(--space-1);
  }

  .confirm button {
    padding: 2px 8px;
    border: 0;
    border-radius: var(--radius-small);
    background: var(--bg-control);
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .confirm button.danger {
    background: var(--danger);
    color: var(--on-danger);
  }

  .bottom {
    margin-top: auto;
    padding-top: var(--space-2);
    border-top: 1px solid var(--separator);
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @media (prefers-reduced-motion: reduce) {
    .sidebar,
    .inner,
    .chevron svg {
      transition: none;
    }
  }
</style>
