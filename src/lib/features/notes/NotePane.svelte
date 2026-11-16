<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";

  import { t } from "../../i18n";
  import Menu, { type MenuItem } from "../../shell/Menu.svelte";
  import { tagsStore } from "../../stores/tags.svelte";
  import TagChip from "../tags/TagChip.svelte";
  import TagMenu from "../tags/TagMenu.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ExportFormat, Note } from "../../types";
  import { attachmentAdd, attachmentRead } from "../../api";
  import { noteLinkIds, wordStats } from "./content";
  import ImagePreview from "./ImagePreview.svelte";
  import { ImageCache } from "./imageCache";
  import { exportSelection } from "./exportNotes";
  import { formatNoteStamp } from "./format";
  import NoteFind from "./NoteFind.svelte";
  import NoteBacklinks from "./NoteBacklinks.svelte";
  import NoteHistory from "./NoteHistory.svelte";
  import NoteReminderDialog from "./NoteReminderDialog.svelte";
  import NoteEditor from "./NoteEditor.svelte";
  import { NoteSaver } from "./noteSaver";
  import NoteTaskDialog from "./NoteTaskDialog.svelte";
  import NoteTasksPanel from "./NoteTasksPanel.svelte";
  import NoteToolbar from "./NoteToolbar.svelte";
  import { createTaskHost } from "./taskHost";
  import { IDLE_TOOLBAR } from "./toolbarState";

  interface Props {
    notes: NotesStore;
    /** The note to edit. The parent re-creates the pane when it changes. */
    note: Note;
    tasks: TasksStore;
    /** Shows the details of a task (the dialog of the agenda). */
    onPreviewTask: (taskId: string) => void;
  }

  let { notes, note, tasks, onPreviewTask }: Props = $props();

  type OpenMenu = "move" | "more" | "tags" | null;

  // The pane is created per note, so these read the opening values on purpose.
  // svelte-ignore state_referenced_locally
  const id = note.id;
  // svelte-ignore state_referenced_locally
  let title = $state(note.title);
  /** Only one popover at a time. */
  let menu = $state<OpenMenu>(null);
  let titleInput = $state<HTMLInputElement>();
  let editor = $state<NoteEditor>();
  /** Text to start the new task with while its dialog is open; `null` when it is closed. */
  let taskPrefill = $state<string | null>(null);

  // The store is the same for the life of the pane.
  // svelte-ignore state_referenced_locally
  const taskHost = createTaskHost(tasks, tagsStore, (taskId) =>
    onPreviewTask(taskId),
  );

  /** The picture shown large, while the preview is open. */
  let preview = $state<{ url: string } | null>(null);
  const images = new ImageCache({
    api: { add: attachmentAdd, read: attachmentRead },
    show: (_id, _mime, url) => (preview = { url }),
    failed: () =>
      (notes.notice = { kind: "error", message: t.notesApp.imageRefused }),
  });
  onDestroy(() => images.dispose());

  // Links to other notes: titles are read from the store each time, so a rename shows at once.
  const links = {
    host: {
      lookup: (noteId: string) => notes.linkTitle(noteId),
      open: (noteId: string) => void notes.openNote(noteId),
    },
    search: (text: string) => notes.suggest(text),
    create: async (linkTitle: string) =>
      (await notes.createLinkedNote(linkTitle, current.folderId))?.id ?? null,
    track: () => {
      void [...notes.titles.values()];
      void notes.missing.size;
    },
  };

  // The titles the note's links show, fetched for the ones not known yet.
  $effect(() => {
    const ids = noteLinkIds(current.contentJson);
    untrack(() => void notes.ensureTitles(ids));
  });

  const titleSaver = new NoteSaver((value) => notes.saveTitle(id, value));

  /** The note as the store has it now: saves, pins and moves land there. */
  const current = $derived(notes.open?.id === id ? notes.open : note);
  const inTrash = $derived(notes.openInTrash);
  const stamp = $derived(formatNoteStamp(current.updatedAt));
  const stats = $derived(wordStats(current.contentJson));

  /** The find bar (Ctrl+F) is open. */
  let finding = $state(false);
  let filePicker = $state<HTMLInputElement>();

  function closeFind() {
    finding = false;
    editor?.clearFind();
    editor?.focus();
  }

  function onPaneKeydown(event: KeyboardEvent) {
    if (
      !inTrash &&
      (event.ctrlKey || event.metaKey) &&
      !event.shiftKey &&
      !event.altKey &&
      event.key.toLowerCase() === "f"
    ) {
      event.preventDefault();
      finding = true;
    }
  }

  async function onImagesPicked(
    event: Event & { currentTarget: HTMLInputElement },
  ) {
    const files = Array.from(event.currentTarget.files ?? []);
    event.currentTarget.value = "";
    await editor?.insertImages(files);
  }
  const noteTags = $derived(
    current.tagIds
      .map((tagId) => tagsStore.byId.get(tagId))
      .filter((tag) => tag !== undefined),
  );

  const moveItems = $derived<MenuItem[]>([
    {
      id: "",
      label: t.notesApp.inbox,
      checked: current.folderId === null,
    },
    ...notes.tree.map((row, index) => ({
      id: row.folder.id,
      label: row.folder.name,
      depth: row.depth,
      checked: current.folderId === row.folder.id,
      separated: index === 0,
    })),
  ]);

  const MORE_ITEMS: MenuItem[] = [
    { id: "duplicate", label: t.notesApp.duplicate },
    { id: "convert", label: t.notesApp.convertChecklists },
    { id: "reminder", label: t.notesApp.reminder.open },
    { id: "history", label: t.notesApp.history.open },
    { id: "exportMarkdown", label: t.notesApp.exportMarkdown, separated: true },
    { id: "exportHtml", label: t.notesApp.exportHtml },
    { id: "delete", label: t.notesApp.delete, danger: true, separated: true },
  ];

  // A fresh note takes the cursor in its title.
  onMount(() => {
    if (note.title === "" && note.contentJson.length < 60) titleInput?.focus();
    return () => void titleSaver.flush();
  });

  function toggleMenu(which: Exclude<OpenMenu, null>) {
    menu = menu === which ? null : which;
  }

  function onTitleInput() {
    titleSaver.schedule(title);
  }

  // Enter and the down arrow carry on into the text, as if the title were its first line.
  function onTitleKey(event: KeyboardEvent) {
    if (event.key !== "Enter" && event.key !== "ArrowDown") return;
    if (event.isComposing) return;
    event.preventDefault();
    editor?.focus("start");
  }

  async function setTags(ids: string[]) {
    await notes.setTags(id, ids);
    void tagsStore.load();
  }

  // Choosing a suggestion from the `#` list adds the tag to the note.
  function addTag(tag: { id: string }) {
    if (!current.tagIds.includes(tag.id)) {
      void setTags([...current.tagIds, tag.id]);
    }
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
    const tagIds = current.tagIds;
    const converted = await editor.convertChecklists(async (items) => {
      const ids: string[] = [];
      for (const item of items) {
        const created = await tasks.create({
          title: item.text || t.notesApp.untitledTask,
          sourceNoteId: id,
          ...(tagIds.length > 0 ? { tagIds } : {}),
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

  let historyOpen = $state(false);
  let reminderOpen = $state(false);

  /** What is typed must be saved first, or the history would compare an older text. */
  async function openHistory() {
    await titleSaver.flush();
    await editor?.flush();
    historyOpen = true;
  }

  /** What is typed must be saved first, or the file would hold an older text. */
  async function exportNote(format: ExportFormat) {
    await titleSaver.flush();
    await editor?.flush();
    const notice = await exportSelection(format, "note", id);
    if (notice) notes.notice = notice;
  }

  function onMore(action: string) {
    if (action === "duplicate") void notes.duplicateNote(id);
    else if (action === "convert") void convertChecklists();
    else if (action === "history") void openHistory();
    else if (action === "reminder") reminderOpen = true;
    else if (action === "exportMarkdown") void exportNote("markdown");
    else if (action === "exportHtml") void exportNote("html");
    else if (action === "delete") void notes.trashNote(id);
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<article
  class="note-pane"
  aria-label={t.notesApp.noteLabel}
  onkeydown={onPaneKeydown}
>
  <header class="bar" data-tauri-drag-region>
    <div class="format" data-tauri-drag-region>
      {#if inTrash}
        <span class="trash-note" data-tauri-drag-region>
          {t.notesApp.inTrash}
        </span>
      {:else}
        <NoteToolbar
          floating
          advanced
          formats={editor?.formats() ?? IDLE_TOOLBAR}
          onInsertRule={() => editor?.insertRule()}
          onInsertTable={() => editor?.insertTable()}
          onInsertImage={() => filePicker?.click()}
          linkRequest={editor?.linkRequests() ?? 0}
          onToggle={(name) => editor?.toggle(name)}
          onSetLink={(href) => editor?.setLink(href) ?? false}
          onUndo={() => editor?.undo()}
          onRedo={() => editor?.redo()}
        />
      {/if}
    </div>
    <div class="actions">
      {#if inTrash}
        <button
          type="button"
          class="btn small primary"
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
          aria-haspopup="dialog"
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
          class:on={current.isPinned}
          aria-pressed={current.isPinned}
          aria-label={current.isPinned ? t.notesApp.unpin : t.notesApp.pin}
          title={current.isPinned ? t.notesApp.unpin : t.notesApp.pin}
          onclick={() => void notes.setPinned(id, !current.isPinned)}
        >
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
              d="M9.8 2.5l3.7 3.7-1.6.5-2.3 2.3.2 3-.9.9-2.7-2.7-3.5 3.5M6.4 5.2l.9-.9 3 .2"
              fill={current.isPinned ? "currentColor" : "none"}
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <div class="anchor">
          <button
            type="button"
            class="icon-btn"
            class:open={menu === "move"}
            data-menu-toggle
            aria-haspopup="menu"
            aria-expanded={menu === "move"}
            aria-label={t.notesApp.moveTo}
            title={t.notesApp.moveTo}
            onclick={() => toggleMenu("move")}
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
          {#if menu === "move"}
            <Menu
              label={t.notesApp.moveMenu}
              items={moveItems}
              onSelect={(folderId) =>
                void notes.moveNote(id, folderId === "" ? null : folderId)}
              onClose={() => (menu = null)}
            />
          {/if}
        </div>
        <div class="anchor">
          <button
            type="button"
            class="icon-btn"
            class:open={menu === "more"}
            data-menu-toggle
            aria-haspopup="menu"
            aria-expanded={menu === "more"}
            aria-label={t.notesApp.more}
            title={t.notesApp.more}
            onclick={() => toggleMenu("more")}
          >
            <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
              <path
                d="M3.5 8h.01M8 8h.01M12.5 8h.01"
                stroke="currentColor"
                stroke-width="2.4"
                stroke-linecap="round"
              />
            </svg>
          </button>
          {#if menu === "more"}
            <Menu
              label={t.notesApp.more}
              items={MORE_ITEMS}
              onSelect={onMore}
              onClose={() => (menu = null)}
            />
          {/if}
        </div>
      {/if}
    </div>
  </header>

  <div class="body">
    <div class="sheet">
      <p class="stamp" title={t.notesApp.editedAt(stamp)}>{stamp}</p>
      {#if finding}
        <NoteFind
          find={(query) => editor?.find(query) ?? { count: 0, current: -1 }}
          step={(direction) =>
            editor?.findStep(direction) ?? { count: 0, current: -1 }}
          onClose={closeFind}
        />
      {/if}
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
      {#if !inTrash || noteTags.length > 0}
        <div class="tag-row">
          {#each noteTags as tag (tag.id)}
            <TagChip name={tag.name} color={tag.colorHex} />
          {/each}
          {#if !inTrash}
            <span class="tag-slot">
              <button
                type="button"
                class="add-tags"
                class:open={menu === "tags"}
                class:compact={noteTags.length > 0}
                data-tag-toggle
                aria-haspopup="menu"
                aria-expanded={menu === "tags"}
                aria-label={t.notesApp.noteTags}
                title={t.notesApp.noteTags}
                onclick={() => toggleMenu("tags")}
              >
                <svg
                  viewBox="0 0 16 16"
                  width="13"
                  height="13"
                  aria-hidden="true"
                >
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
              {#if menu === "tags"}
                <TagMenu
                  values={current.tagIds}
                  onChange={(ids) => void setTags(ids)}
                  onClose={() => (menu = null)}
                />
              {/if}
            </span>
          {/if}
        </div>
      {/if}
      <NoteEditor
        bind:this={editor}
        contentJson={note.contentJson}
        save={saveContent}
        variant="page"
        readonly={inTrash}
        label={t.notesApp.bodyLabel}
        links={inTrash ? undefined : links}
        {images}
        advanced
        {taskHost}
        taskTrack={taskHost.track}
        onTaskRequest={inTrash ? undefined : requestTask}
        tagSuggest={inTrash
          ? undefined
          : { tags: tagsStore.tags, onPick: addTag }}
        placeholder={t.notesApp.bodyPlaceholder}
      />
      {#if !inTrash}
        <p class="words" aria-live="off">
          {t.notesApp.words(stats.words, stats.characters)}
        </p>
      {/if}
      <NoteBacklinks {notes} noteId={id} />
      {#if reminderOpen}
        <NoteReminderDialog
          {notes}
          noteId={id}
          onClose={() => (reminderOpen = false)}
        />
      {/if}
      {#if historyOpen}
        <NoteHistory
          {notes}
          note={current}
          onClose={() => (historyOpen = false)}
        />
      {/if}
      <NoteTasksPanel
        {tasks}
        noteId={id}
        contentJson={current.contentJson}
        onOpen={(task) => onPreviewTask(task.id)}
      />
    </div>
  </div>
</article>

<input
  bind:this={filePicker}
  type="file"
  class="file-picker"
  accept="image/png,image/jpeg,image/gif,image/webp"
  multiple
  tabindex="-1"
  aria-label={t.notes.image}
  onchange={onImagesPicked}
/>

{#if preview}
  <ImagePreview url={preview.url} onClose={() => (preview = null)} />
{/if}

{#if taskPrefill !== null}
  <NoteTaskDialog
    {tasks}
    noteId={id}
    prefill={taskPrefill}
    tagIds={current.tagIds}
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

  /* Format on the left, the note's own actions on the right: one quiet row of icons, as
     tall as the header of the list next to it. */
  .bar {
    position: relative;
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-2);
    /* Above the text, so the menus that hang from it are never covered. */
    z-index: 2;
    height: 44px;
    padding: 0 var(--space-3);
    container-type: inline-size;
  }

  .format {
    display: flex;
    flex: 1;
    align-items: center;
    min-width: 0;
    height: 100%;
    overflow: hidden;
    --note-toolbar-bg: transparent;
    --note-toolbar-margin: 0;
    --note-tool-size: 28px;
  }

  /* A narrow pane keeps the formats and lets go of undo and redo (Ctrl+Z still works). */
  @container (max-width: 430px) {
    .format :global(.history) {
      display: none;
    }
  }

  .trash-note {
    padding-left: var(--space-2);
    overflow: hidden;
    color: var(--text-secondary);
    font-size: 12px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    display: flex;
    flex: none;
    align-items: center;
    gap: 2px;
  }

  .anchor {
    position: relative;
  }

  .icon-btn.on {
    color: var(--accent);
  }

  .icon-btn.open {
    background: var(--bg-active);
    color: var(--text-primary);
  }

  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  /* A column of comfortable reading width, centred when the window is wide. */
  .sheet {
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    width: 100%;
    max-width: 760px;
    min-height: 100%;
    margin: 0 auto;
    padding: var(--space-1) var(--space-8) var(--space-6);
  }

  .file-picker {
    display: none;
  }

  .words {
    margin: var(--space-4) 0 0;
    color: var(--text-secondary);
    font-size: 11px;
    text-align: right;
    font-variant-numeric: tabular-nums;
    user-select: none;
  }

  .stamp {
    flex: none;
    margin: 0 0 var(--space-3);
    color: var(--text-secondary);
    font-size: 11px;
    text-align: center;
    user-select: none;
  }

  .title {
    flex: none;
    width: 100%;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: 700;
    letter-spacing: -0.022em;
    line-height: 1.2;
    text-overflow: ellipsis;
    outline: none;
  }

  .title::placeholder {
    color: var(--text-secondary);
    opacity: 0.55;
  }

  .tag-row {
    display: flex;
    flex: none;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1);
    min-height: 22px;
    margin-top: var(--space-2);
  }

  .tag-slot {
    position: relative;
  }

  /* The tag menu opens towards the text, not over the edge of the pane. */
  .tag-slot :global(.menu) {
    right: auto;
    left: 0;
    transform-origin: top left;
  }

  .add-tags {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 22px;
    margin-left: -6px;
    padding: 0 8px 0 6px;
    border: 0;
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      color var(--motion-instant) linear;
  }

  .add-tags.compact {
    margin-left: 0;
    padding: 0 6px;
  }

  .add-tags:hover,
  .add-tags.open {
    background: var(--bg-control-hover);
    color: var(--text-primary);
  }

  .sheet :global(.note) {
    margin-top: var(--space-3);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  @media (max-width: 860px) {
    .sheet {
      padding-right: var(--space-6);
      padding-left: var(--space-6);
    }
  }
</style>
