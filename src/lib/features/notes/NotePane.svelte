<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import { tagsStore } from "../../stores/tags.svelte";
  import TagChip from "../tags/TagChip.svelte";
  import TagMenu from "../tags/TagMenu.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Note } from "../../types";
  import { formatNoteDate } from "./format";
  import NoteEditor from "./NoteEditor.svelte";
  import { NoteSaver } from "./noteSaver";
  import NoteTaskDialog from "./NoteTaskDialog.svelte";
  import NoteTasksPanel from "./NoteTasksPanel.svelte";
  import { createTaskHost } from "./taskHost";

  interface Props {
    notes: NotesStore;
    /** The note to edit. The parent re-creates the pane when it changes. */
    note: Note;
    tasks: TasksStore;
    /** Shows the details of a task (the dialog of the agenda). */
    onPreviewTask: (taskId: string) => void;
    now?: () => Date;
  }

  let {
    notes,
    note,
    tasks,
    onPreviewTask,
    now = () => new Date(),
  }: Props = $props();

  // The pane is created per note, so these read the opening values on purpose.
  // svelte-ignore state_referenced_locally
  const id = note.id;
  // svelte-ignore state_referenced_locally
  let title = $state(note.title);
  let moveOpen = $state(false);
  let tagsOpen = $state(false);
  let titleInput = $state<HTMLInputElement>();
  let editor = $state<NoteEditor>();
  /** Text to start the new task with while its dialog is open; `null` when it is closed. */
  let taskPrefill = $state<string | null>(null);

  // The store is the same for the life of the pane.
  // svelte-ignore state_referenced_locally
  const taskHost = createTaskHost(tasks, tagsStore, (taskId) =>
    onPreviewTask(taskId),
  );

  const titleSaver = new NoteSaver((value) => notes.saveTitle(id, value));
  const inTrash = $derived(notes.scope === "trash");

  // Title and body are kept in step with the list while typing; a fresh note takes the cursor.
  onMount(() => {
    if (note.title === "" && note.contentJson.length < 60) titleInput?.focus();
    return () => void titleSaver.flush();
  });

  function onTitleInput() {
    titleSaver.schedule(title);
  }

  function onTitleKey(event: KeyboardEvent) {
    if (event.key !== "Enter") return;
    event.preventDefault();
    document.querySelector<HTMLElement>(".note-pane .tiptap")?.focus();
  }

  const noteTags = $derived(
    (notes.open?.tagIds ?? [])
      .map((tagId) => tagsStore.byId.get(tagId))
      .filter((tag) => tag !== undefined),
  );

  async function setTags(ids: string[]) {
    await notes.setTags(id, ids);
    void tagsStore.load();
  }

  // Choosing a suggestion from the `#` list adds the tag to the note.
  function addTag(tag: { id: string }) {
    const current = notes.open?.tagIds ?? [];
    if (!current.includes(tag.id)) void setTags([...current, tag.id]);
  }

  function requestTask(prefill = editor?.selectedText() ?? "") {
    taskPrefill = prefill;
  }

  /** D-37 checklist items become real tasks, in place (F10-03). */
  async function convertChecklists() {
    if (!editor?.hasChecklists()) {
      notes.notice = { kind: "info", message: t.notesApp.noChecklists };
      return;
    }
    const converted = await editor.convertChecklists(async (items) => {
      const ids: string[] = [];
      for (const item of items) {
        const created = await tasks.create({
          title: item.text || t.notesApp.untitledTask,
          sourceNoteId: id,
          ...((notes.open?.tagIds.length ?? 0) > 0
            ? { tagIds: notes.open?.tagIds ?? [] }
            : {}),
        });
        if (!created) {
          notes.notice = { kind: "error", message: t.notesApp.convertFailed };
          return null;
        }
        ids.push(created.id);
      }
      // A ticked box becomes a task that is already done.
      await Promise.all(
        items.flatMap((item, at) => {
          const taskId = ids[at];
          return item.checked && taskId
            ? [tasks.setCompleted(taskId, true)]
            : [];
        }),
      );
      return ids;
    });
    if (converted > 0) {
      notes.notice = {
        kind: "info",
        message: t.notesApp.converted(converted),
      };
    }
  }

  const saveContent = (contentJson: string) =>
    notes.saveContent(id, contentJson);

  async function moveTo(folderId: string | null) {
    moveOpen = false;
    await notes.moveNote(id, folderId);
  }

  function onMoveKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.stopPropagation();
      moveOpen = false;
    }
  }
</script>

<article class="note-pane" aria-label={t.notesApp.titleLabel}>
  <header class="bar" data-tauri-drag-region>
    <span class="edited" data-tauri-drag-region>
      {t.notesApp.editedAt(
        formatNoteDate(notes.open?.updatedAt ?? note.updatedAt, now()),
      )}
    </span>
    <div class="actions">
      {#if inTrash}
        <button
          type="button"
          class="text-btn"
          onclick={() => void notes.restoreNote(id)}
        >
          {t.notesApp.restore}
        </button>
      {:else}
        <button
          type="button"
          class="icon-btn"
          aria-label={t.notesApp.newTaskFromNote}
          title={t.notesApp.newTaskFromNote}
          onclick={() => requestTask()}
        >
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <circle
              cx="8"
              cy="8"
              r="5.6"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
            />
            <path
              d="M8 5.6v4.8M5.6 8h4.8"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
          </svg>
        </button>
        <button
          type="button"
          class="icon-btn"
          aria-label={t.notesApp.convertChecklists}
          title={t.notesApp.convertChecklists}
          onclick={() => void convertChecklists()}
        >
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
              d="M2.5 4.2l1.2 1.2 2-2.2M2.5 10.2l1.2 1.2 2-2.2M8 4.5h5.5M8 10.5h5.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button
          type="button"
          class="icon-btn"
          class:on={notes.open?.isPinned}
          aria-pressed={notes.open?.isPinned ?? false}
          aria-label={notes.open?.isPinned ? t.notesApp.unpin : t.notesApp.pin}
          title={notes.open?.isPinned ? t.notesApp.unpin : t.notesApp.pin}
          onclick={() => void notes.setPinned(id, !notes.open?.isPinned)}
        >
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
              d="M9.8 2.5l3.7 3.7-1.6.5-2.3 2.3.2 3-.9.9-2.7-2.7-3.5 3.5M6.4 5.2l.9-.9 3 .2"
              fill={notes.open?.isPinned ? "currentColor" : "none"}
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button
          type="button"
          class="icon-btn"
          aria-label={t.notesApp.duplicate}
          title={t.notesApp.duplicate}
          onclick={() => void notes.duplicateNote(id)}
        >
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <rect
              x="5.5"
              y="5.5"
              width="8"
              height="8"
              rx="1.6"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
            />
            <path
              d="M10.5 5.5v-1c0-.8-.7-1.5-1.5-1.5H4.5C3.7 3 3 3.7 3 4.5V9c0 .8.7 1.5 1.5 1.5h1"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
          </svg>
        </button>
        <div class="move" onkeydown={onMoveKey} role="presentation">
          <button
            type="button"
            class="icon-btn"
            aria-haspopup="menu"
            aria-expanded={moveOpen}
            aria-label={t.notesApp.moveTo}
            title={t.notesApp.moveTo}
            onclick={() => (moveOpen = !moveOpen)}
          >
            <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
              <path
                d="M2 5.2c0-.7.5-1.2 1.2-1.2h2.6l1.4 1.5h5.6c.7 0 1.2.5 1.2 1.2v5.1c0 .7-.5 1.2-1.2 1.2H3.2c-.7 0-1.2-.5-1.2-1.2z"
                fill="none"
                stroke="currentColor"
                stroke-width="1.4"
                stroke-linejoin="round"
              />
            </svg>
          </button>
          {#if moveOpen}
            <button
              type="button"
              class="scrim"
              tabindex="-1"
              aria-hidden="true"
              onclick={() => (moveOpen = false)}
            ></button>
            <ul class="menu" role="menu" aria-label={t.notesApp.moveMenu}>
              <li role="none">
                <button
                  type="button"
                  role="menuitemradio"
                  aria-checked={notes.open?.folderId === null}
                  onclick={() => void moveTo(null)}
                >
                  {t.notesApp.inbox}
                </button>
              </li>
              {#each notes.folders as folder (folder.id)}
                <li role="none">
                  <button
                    type="button"
                    role="menuitemradio"
                    aria-checked={notes.open?.folderId === folder.id}
                    onclick={() => void moveTo(folder.id)}
                  >
                    {folder.name}
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
        <button
          type="button"
          class="icon-btn danger"
          aria-label={t.notesApp.delete}
          title={t.notesApp.delete}
          onclick={() => void notes.trashNote(id)}
        >
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
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
      {/if}
    </div>
  </header>

  <div class="body">
    <input
      class="title"
      bind:this={titleInput}
      bind:value={title}
      aria-label={t.notesApp.titleLabel}
      placeholder={t.notesApp.titlePlaceholder}
      maxlength="200"
      readonly={inTrash}
      oninput={onTitleInput}
      onkeydown={onTitleKey}
    />
    <div class="tag-row">
      {#each noteTags as tag (tag.id)}
        <TagChip name={tag.name} color={tag.colorHex} />
      {/each}
      {#if !inTrash}
        <span class="tag-slot">
          <button
            type="button"
            class="add-tags"
            data-tag-toggle
            aria-haspopup="menu"
            aria-expanded={tagsOpen}
            aria-label={t.notesApp.noteTags}
            title={t.notesApp.noteTags}
            onclick={() => (tagsOpen = !tagsOpen)}
          >
            <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
              <path
                d="M2.5 8.5V3h5.5l5.5 5.5-5.5 5.5z"
                fill="none"
                stroke="currentColor"
                stroke-width="1.4"
                stroke-linejoin="round"
              />
              <circle cx="5.5" cy="6" r="0.9" fill="currentColor" />
            </svg>
            {#if noteTags.length === 0}{t.notesApp.addTags}{/if}
          </button>
          {#if tagsOpen}
            <TagMenu
              values={notes.open?.tagIds ?? []}
              onChange={(ids) => void setTags(ids)}
              onClose={() => (tagsOpen = false)}
            />
          {/if}
        </span>
      {/if}
    </div>
    {#if inTrash}
      <p class="trash-note">{t.notesApp.inTrash}</p>
    {/if}
    <NoteEditor
      bind:this={editor}
      contentJson={note.contentJson}
      save={saveContent}
      variant="page"
      {taskHost}
      taskTrack={taskHost.track}
      onTaskRequest={inTrash ? undefined : requestTask}
      tagSuggest={{ tags: tagsStore.tags, onPick: addTag }}
      placeholder={t.notesApp.bodyPlaceholder}
    />
    <NoteTasksPanel
      {tasks}
      noteId={id}
      contentJson={notes.open?.contentJson ?? note.contentJson}
      onOpen={(task) => onPreviewTask(task.id)}
    />
  </div>
</article>

{#if taskPrefill !== null}
  <NoteTaskDialog
    {tasks}
    noteId={id}
    prefill={taskPrefill}
    tagIds={notes.open?.tagIds ?? []}
    onCreated={(task) => editor?.insertTaskRef(task.id)}
    onClose={() => (taskPrefill = null)}
  />
{/if}

<style>
  .note-pane {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    animation: fade-in var(--motion-base) var(--ease-out);
  }

  .bar {
    display: flex;
    flex: none;
    align-items: center;
    justify-content: space-between;
    height: 44px;
    padding: 0 var(--space-4);
  }

  .edited {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .icon-btn.on {
    color: var(--accent);
  }

  .icon-btn.danger:hover {
    color: var(--danger);
  }

  .text-btn {
    padding: 3px 12px;
    border: 0;
    border-radius: var(--radius-chip);
    background: var(--accent);
    color: var(--on-accent);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .move {
    position: relative;
  }

  .scrim {
    position: fixed;
    inset: 0;
    z-index: 9;
    border: 0;
    background: transparent;
    cursor: default;
  }

  .menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 10;
    min-width: 180px;
    max-height: 280px;
    margin: 0;
    padding: 4px;
    overflow-y: auto;
    border-radius: var(--radius-card);
    background: var(--bg-popover);
    box-shadow: var(--shadow-popover);
    list-style: none;
    transform-origin: top right;
    animation: pop-in var(--motion-fast) var(--ease-out);
  }

  .menu button {
    display: block;
    width: 100%;
    padding: 6px var(--space-3);
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .menu button:hover {
    background: var(--bg-hover);
  }

  .menu button[aria-checked="true"] {
    color: var(--accent);
    font-weight: 600;
  }

  .body {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
    padding: 0 var(--space-6) var(--space-4);
    overflow-y: auto;
  }

  .title {
    flex: none;
    width: 100%;
    padding: 0 0 var(--space-2);
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font-family: var(--font-display);
    font-size: 26px;
    font-weight: 700;
    letter-spacing: -0.02em;
    outline: none;
  }

  .title::placeholder {
    color: var(--text-secondary);
    opacity: 0.6;
  }

  .tag-row {
    display: flex;
    flex: none;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1);
    min-height: 26px;
    margin-bottom: var(--space-2);
  }

  .tag-slot {
    position: relative;
  }

  .add-tags {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 22px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  .add-tags:hover,
  .add-tags[aria-expanded="true"] {
    background: var(--bg-control-hover);
    color: var(--text-primary);
  }

  .trash-note {
    margin: 0 0 var(--space-2);
    color: var(--text-secondary);
    font-size: 12px;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
