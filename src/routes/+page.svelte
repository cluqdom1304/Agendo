<script lang="ts">
  import { onMount, tick } from "svelte";

  import { onReminderFired, onReminderOpen, onTasksChanged } from "$lib/api";
  import ContextMenu from "$lib/features/context/ContextMenu.svelte";
  import FolderSidebar from "$lib/features/notes/FolderSidebar.svelte";
  import {
    convertTaskToNote,
    createNoteForTask,
    showNote,
  } from "$lib/features/notes/navigation";
  import NotesView from "$lib/features/notes/NotesView.svelte";
  import PaletteHost from "$lib/features/palette/PaletteHost.svelte";
  import type { PaletteMode } from "$lib/features/palette/palette";
  import TagDragGhost from "$lib/features/tags/TagDragGhost.svelte";
  import TagSidebar from "$lib/features/tags/TagSidebar.svelte";
  import TaskDialog from "$lib/features/todo/TaskDialog.svelte";
  import SettingsDialog from "$lib/features/settings/SettingsDialog.svelte";
  import TaskDetailDialog from "$lib/features/todo/TaskDetailDialog.svelte";
  import TodoView from "$lib/features/todo/TodoView.svelte";
  import { t } from "$lib/i18n";
  import { findShortcut, focusContextOf, ROW_ATTRIBUTE } from "$lib/keymap";
  import SegmentedControl from "$lib/shell/SegmentedControl.svelte";
  import Toast from "$lib/shell/Toast.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { notesStore } from "$lib/stores/notes.svelte";
  import { folderDrag, tagDrag } from "$lib/stores/tagDrag.svelte";
  import { tagsStore } from "$lib/stores/tags.svelte";
  import { tasksStore } from "$lib/stores/tasks.svelte";
  import { viewStore, type Section } from "$lib/stores/view.svelte";
  import type { Task, ViewMode } from "$lib/types";

  type ViewSection = "list" | "calendar";

  let settingsOpen = $state(false);
  let newTaskOpen = $state(false);
  /** The day the creation dialog starts on, when it was opened from a day of the calendar. */
  let newTaskDate = $state<string | null>(null);
  function openNewTask(date: string | null = null) {
    newTaskDate = date;
    newTaskOpen = true;
  }

  // Narrow windows start with the sidebar folded so the content keeps its room.
  let sidebarOpen = $state(
    typeof window === "undefined" || window.innerWidth >= 900,
  );
  /** The task whose view is open (opened from the calendar). */
  let detailId = $state<string | null>(null);
  /** The command palette (Ctrl+K) or "Ir a…" (Ctrl+P) while it is open. */
  let palette = $state<PaletteMode | null>(null);
  /** The task being edited in the creation dialog; the view comes back when it closes. */
  let editing = $state.raw<Task | null>(null);

  // The tag filter only hides tasks: in the notes there is nothing to warn about.
  const filterShown = $derived(
    tagsStore.filtered && viewStore.section === "agenda",
  );

  const sidebarLabel = $derived(
    viewStore.section === "notes"
      ? t.notesApp.toggleSidebar
      : t.tags.toggleSidebar,
  );

  // The list shows its own deletion notices; elsewhere they are shown here.
  const lastDeletion = $derived(tasksStore.undoStack.at(-1));

  const VIEW_OPTIONS: { value: ViewSection; label: string }[] = [
    { value: "list", label: t.views.list },
    { value: "calendar", label: t.views.calendar },
  ];

  const SECTION_OPTIONS: { value: Section; label: string }[] = [
    { value: "agenda", label: t.notesApp.agenda },
    { value: "notes", label: t.notesApp.notes },
  ];

  const SHORTCUT_VIEWS: Record<string, ViewMode> = {
    viewList: "list",
    viewDay: "day",
    viewWeek: "week",
    viewMonth: "month",
  };

  // The calendar is a separate chunk: it is only requested when a calendar view is shown, so
  // it does not weigh on startup (N-01).
  const loadCalendar = () =>
    import("$lib/features/calendar/CalendarView.svelte");

  // Tasks made from a note name it (F10-05): look the titles up as tasks come and go.
  $effect(() => {
    const ids = tasksStore.tasks.flatMap((task) =>
      task.sourceNoteId ? [task.sourceNoteId] : [],
    );
    if (ids.length > 0) void notesStore.ensureTitles(ids);
  });

  // The notes load the first time their section is shown, not at startup (N-01).
  $effect(() => {
    if (viewStore.section === "notes" && notesStore.status === "idle") {
      void notesStore.load();
    }
  });

  // The sidebar is the drop target, so it opens as soon as a drag starts.
  $effect(() => {
    if (tagDrag.active || folderDrag.active) sidebarOpen = true;
  });

  onMount(() => {
    tasksStore.filter = (task) => tagsStore.isVisible(task);
    void tagsStore.load();
    void tasksStore.load();
    void viewStore.load();
    const unlisten = onReminderFired((fired) => {
      if (fired.taskId) return tasksStore.onReminderFired(fired.taskId);
      if (fired.noteId) return notesStore.onReminderFired(fired.noteId);
    });
    const unlistenChanged = onTasksChanged(() => void tasksStore.refresh());
    const unlistenOpen = onReminderOpen((target) => {
      if (target.taskId) void showTask(target.taskId);
      else if (target.noteId) void showNote(target.noteId);
    });
    return () => {
      void unlisten.then((stop) => stop());
      void unlistenChanged.then((stop) => stop());
      void unlistenOpen.then((stop) => stop());
      tasksStore.dispose();
    };
  });

  function onRestored() {
    tagsStore.showAll();
    void tagsStore.load();
    void tasksStore.load();
    void viewStore.load();
  }

  async function openInList(task: Task) {
    detailId = null;
    if (!tagsStore.isVisible(task)) tagsStore.showAll();
    viewStore.setMode("list");
    if (!tasksStore.reveal(task.id)) return;
    await tick();
    const row = document.querySelector<HTMLElement>(
      `[${ROW_ATTRIBUTE}][data-task-id="${task.id}"]`,
    );
    row?.focus();
    row?.scrollIntoView?.({ block: "center" });
  }

  /** Shows a task found by the palette in the agenda's list. */
  async function showTask(taskId: string) {
    viewStore.setSection("agenda");
    await tick();
    const task = tasksStore.find(taskId);
    if (task) await openInList(task);
    else detailId = taskId;
  }

  function onKeydown(event: KeyboardEvent) {
    const shortcut = findShortcut(event, focusContextOf(event.target));
    if (shortcut === "openSearch" || shortcut === "quickOpen") {
      event.preventDefault();
      palette = shortcut === "openSearch" ? "all" : "notes";
      return;
    }
    if (shortcut === "goAgenda" || shortcut === "goNotes") {
      event.preventDefault();
      viewStore.setSection(shortcut === "goAgenda" ? "agenda" : "notes");
      return;
    }
    const mode = shortcut ? SHORTCUT_VIEWS[shortcut] : undefined;
    if (!mode || viewStore.section !== "agenda") return;
    event.preventDefault();
    viewStore.setMode(mode);
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="shell">
  {#if viewStore.section === "notes"}
    <FolderSidebar
      notes={notesStore}
      tags={tagsStore}
      pendingOf={(id) =>
        tasksStore.tasks.filter(
          (task) => !task.isCompleted && task.tagIds.includes(id),
        ).length}
      open={sidebarOpen}
    />
  {:else}
    <TagSidebar tags={tagsStore} tasks={tasksStore} open={sidebarOpen} />
  {/if}
  <div class="page">
    <div class="toolbar" data-tauri-drag-region>
      <div class="lead" data-tauri-drag-region>
        <button
          type="button"
          class="icon-btn sidebar-toggle"
          class:filtered={filterShown}
          aria-pressed={sidebarOpen}
          aria-label={filterShown
            ? `${sidebarLabel}. ${t.tags.filterActive}`
            : sidebarLabel}
          title={filterShown ? t.tags.filterActive : sidebarLabel}
          onclick={() => (sidebarOpen = !sidebarOpen)}
        >
          <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <rect
              x="2"
              y="3"
              width="12"
              height="10"
              rx="2"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
            />
            <path d="M6 3v10" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
        <SegmentedControl
          label={t.notesApp.sections}
          options={SECTION_OPTIONS}
          value={viewStore.section}
          onChange={(section) => viewStore.setSection(section)}
        />
      </div>
      {#if viewStore.section === "agenda"}
        <SegmentedControl
          label={t.views.label}
          options={VIEW_OPTIONS}
          value={viewStore.mode === "list" ? "list" : "calendar"}
          onChange={(section) =>
            viewStore.setMode(
              section === "list" ? "list" : viewStore.calendarMode,
            )}
        />
      {:else}
        <span></span>
      {/if}
      <div class="trail" data-tauri-drag-region>
        <button
          type="button"
          class="icon-btn"
          aria-label={t.settings.open}
          title={t.settings.open}
          aria-haspopup="dialog"
          onclick={() => (settingsOpen = true)}
        >
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
            <path
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              d="M4 7h10M18 7h2M4 17h2M10 17h10"
            />
            <circle
              cx="16"
              cy="7"
              r="2"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
            />
            <circle
              cx="8"
              cy="17"
              r="2"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
            />
          </svg>
        </button>
      </div>
    </div>

    {#if viewStore.ready}
      {#if viewStore.section === "notes"}
        <div class="content" data-menu-area="notes">
          <NotesView
            notes={notesStore}
            tags={tagsStore}
            tasks={tasksStore}
            onPreviewTask={(taskId) => (detailId = taskId)}
            onOpenTask={(task) => {
              viewStore.setSection("agenda");
              void openInList(task);
            }}
          />
        </div>
      {:else if viewStore.mode === "list"}
        <div class="content scrolls" data-menu-area="agenda">
          <TodoView store={tasksStore} />
        </div>
      {:else}
        <div class="content" data-menu-area="agenda">
          {#await loadCalendar()}
            <p class="status">{t.calendar.loading}</p>
          {:then module}
            <module.default
              store={tasksStore}
              view={viewStore}
              onOpenTask={(task: Task) => (detailId = task.id)}
            />
          {:catch}
            <p class="status">{t.calendar.loadFailed}</p>
          {/await}
        </div>
      {/if}
    {/if}
  </div>
</div>

{#if palette}
  <PaletteHost
    mode={palette}
    onTask={(taskId) => void showTask(taskId)}
    onClose={() => (palette = null)}
  />
{/if}

<ContextMenu
  deps={{
    view: viewStore,
    tasks: tasksStore,
    tags: tagsStore,
    notes: notesStore,
    newTask: openNewTask,
    editTask: (task) => (editing = task),
    openTask: (taskId) => (detailId = taskId),
    showNote,
    noteForTask: createNoteForTask,
    taskToNote: convertTaskToNote,
  }}
/>

{#if viewStore.section === "agenda"}
  <button
    type="button"
    class="fab"
    title={t.newTask.buttonHint}
    aria-label={t.newTask.button}
    aria-haspopup="dialog"
    onclick={() => openNewTask()}
  >
    <svg viewBox="0 0 16 16" width="20" height="20" aria-hidden="true">
      <path
        d="M8 3v10M3 8h10"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
    </svg>
  </button>
{/if}

<TagDragGhost drag={tagDrag} />
<TagDragGhost drag={folderDrag} hint={t.notesApp.dragHint} />

{#if newTaskOpen}
  <TaskDialog
    store={tasksStore}
    defaultDate={newTaskDate ??
      (viewStore.mode === "list" ? null : viewStore.anchor)}
    onClose={() => (newTaskOpen = false)}
  />
{/if}

{#if detailId && !editing}
  <TaskDetailDialog
    store={tasksStore}
    taskId={detailId}
    onEdit={(task) => (editing = task)}
    onShowInList={(task) => void openInList(task)}
    onClose={() => (detailId = null)}
  />
{/if}

{#if editing}
  <TaskDialog
    store={tasksStore}
    task={editing}
    onClose={() => (editing = null)}
  />
{/if}

{#if viewStore.section === "agenda" && viewStore.mode !== "list"}
  {#if lastDeletion}
    <Toast
      message={t.tasks.deleted}
      actionLabel={t.tasks.undo}
      closeLabel={t.tasks.dismiss}
      onAction={() => void tasksStore.undoDelete()}
      onClose={() => (tasksStore.undoStack = [])}
    />
  {:else if tasksStore.notice}
    <Toast
      kind={tasksStore.notice.kind}
      message={tasksStore.notice.message}
      closeLabel={t.tasks.dismiss}
      onClose={() => tasksStore.dismissNotice()}
    />
  {/if}
{/if}

{#if settingsOpen}
  <SettingsDialog
    store={settingsStore}
    onClose={() => {
      settingsOpen = false;
      // The default notification time moves saved reminders, which tasks hold a copy of.
      void tasksStore.load();
    }}
    {onRestored}
  />
{/if}

<style>
  .shell {
    display: flex;
    height: 100%;
  }

  .page {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    height: 100%;
  }

  .toolbar {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    flex: none;
    height: 48px;
    padding-left: var(--space-3);
  }

  .lead,
  .trail {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    height: 100%;
  }

  .trail {
    justify-content: flex-end;
    /* Room for the three 46 px caption buttons that float over this bar. */
    padding-right: 146px;
  }

  .sidebar-toggle {
    position: relative;
  }

  .sidebar-toggle.filtered::after {
    content: "";
    position: absolute;
    top: 4px;
    right: 4px;
    width: 7px;
    height: 7px;
    border-radius: var(--radius-chip);
    background: var(--accent);
  }

  .content {
    flex: 1;
    min-height: 0;
    animation: rise-in var(--motion-base) var(--ease-out);
  }

  /* The round button of the agenda: bottom right, above the content but below the dialogs and
     the undo notice. */
  .fab {
    position: fixed;
    right: var(--space-6);
    bottom: var(--space-6);
    z-index: 15;
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: var(--accent);
    color: var(--on-accent);
    box-shadow: var(--shadow-popover);
    cursor: pointer;
    transition:
      transform var(--motion-fast) var(--ease-out),
      filter var(--motion-fast) linear;
  }

  .fab:hover {
    filter: brightness(1.08);
  }

  .fab:active {
    transform: scale(0.94);
  }

  .fab:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 3px;
  }

  @media (prefers-reduced-motion: reduce) {
    .fab {
      transition: none;
    }
  }

  .scrolls {
    overflow: auto;
  }

  .status {
    padding: var(--space-6);
    color: var(--text-secondary);
  }
</style>
