<script lang="ts">
  import { onMount, tick } from "svelte";

  import { showDailyNote, showNote } from "$lib/features/notes/navigation";
  import TagDragGhost from "$lib/features/tags/TagDragGhost.svelte";
  import TagSidebar from "$lib/features/tags/TagSidebar.svelte";
  import TodoView from "$lib/features/todo/TodoView.svelte";
  import { t } from "$lib/i18n";
  import { ROW_ATTRIBUTE } from "$lib/keymap";
  import { listenBackendEvents } from "$lib/shell/backendEvents";
  import DialogHost from "$lib/shell/DialogHost.svelte";
  import { DialogState } from "$lib/shell/dialogs.svelte";
  import EmptyState from "$lib/shell/EmptyState.svelte";
  import SegmentedControl from "$lib/shell/SegmentedControl.svelte";
  import { dispatchShortcut } from "$lib/shell/shortcuts";
  import Toast from "$lib/shell/Toast.svelte";
  import { notesStore } from "$lib/stores/notes.svelte";
  import { folderDrag, tagDrag } from "$lib/stores/tagDrag.svelte";
  import { tagsStore } from "$lib/stores/tags.svelte";
  import { tasksStore } from "$lib/stores/tasks.svelte";
  import { viewStore, type Section } from "$lib/stores/view.svelte";
  import type { Task } from "$lib/types";

  const dialogs = new DialogState();

  // Narrow windows start with the sidebar folded so the content keeps its room.
  let sidebarOpen = $state(
    typeof window === "undefined" || window.innerWidth >= 900,
  );

  // The tag filter only hides tasks: in the notes there is nothing to warn about.
  const filterShown = $derived(tagsStore.filtered && viewStore.inAgenda);

  const sidebarLabel = $derived(
    viewStore.section === "notes"
      ? t.notesApp.toggleSidebar
      : t.tags.toggleSidebar,
  );

  // The list shows its own deletion notices; elsewhere they are shown here.
  const lastDeletion = $derived(tasksStore.undoStack.at(-1));

  const SECTION_OPTIONS: { value: Section; label: string }[] = [
    { value: "today", label: t.notesApp.today },
    { value: "tasks", label: t.views.list },
    { value: "calendar", label: t.views.calendar },
    { value: "notes", label: t.notesApp.notes },
  ];

  // The calendar is a separate chunk: it is only requested when a calendar view is shown, so
  // it does not weigh on startup (N-01).
  const loadCalendar = () =>
    import("$lib/features/calendar/CalendarView.svelte");

  // Dialogs and the palette load from `DialogHost`; views are only requested once shown.
  // They are fetched once the shell is idle and then rendered synchronously: awaiting a promise
  // inside the template leaves a frame without sidebar, which made the toolbar jump when
  // switching section.
  let FolderSidebar = $state<
    typeof import("$lib/features/notes/FolderSidebar.svelte").default | null
  >(null);
  let NotesView = $state<
    typeof import("$lib/features/notes/NotesView.svelte").default | null
  >(null);
  let TodayView = $state<
    typeof import("$lib/features/today/TodayView.svelte").default | null
  >(null);

  async function preloadViews() {
    const [folders, notes, today] = await Promise.all([
      import("$lib/features/notes/FolderSidebar.svelte"),
      import("$lib/features/notes/NotesView.svelte"),
      import("$lib/features/today/TodayView.svelte"),
    ]);
    FolderSidebar = folders.default;
    NotesView = notes.default;
    TodayView = today.default;
  }
  $effect(() => {
    const timer = setTimeout(() => void preloadViews(), 400);
    return () => clearTimeout(timer);
  });
  $effect(() => {
    if (!viewStore.inAgenda) void preloadViews();
  });

  // Tasks made from a note name it (F10-05): look the titles up as tasks come and go.
  // The key is a string so the effect only reruns when the set of origin notes changes, not on
  // every task edit.
  const sourceNoteKey = $derived(
    [
      ...new Set(
        tasksStore.tasks.flatMap((task) =>
          task.sourceNoteId ? [task.sourceNoteId] : [],
        ),
      ),
    ].join(","),
  );
  $effect(() => {
    if (sourceNoteKey) void notesStore.ensureTitles(sourceNoteKey.split(","));
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
    const stop = listenBackendEvents({
      taskReminderFired: (taskId) => tasksStore.onReminderFired(taskId),
      noteReminderFired: (noteId) => notesStore.onReminderFired(noteId),
      tasksChanged: () => void tasksStore.refresh(),
      openTask: (taskId) => void showTask(taskId),
      openNote: (noteId) => void showNote(noteId),
    });
    return () => {
      stop();
      tasksStore.dispose();
    };
  });

  async function openInList(task: Task) {
    dialogs.detailId = null;
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
    viewStore.showAgenda();
    await tick();
    const task = tasksStore.find(taskId);
    if (task) await openInList(task);
    else dialogs.detailId = taskId;
  }

  function onKeydown(event: KeyboardEvent) {
    dispatchShortcut(event, {
      section: () => viewStore.section,
      mode: () => viewStore.mode,
      setSection: (section) => viewStore.setSection(section),
      setMode: (mode) => viewStore.setMode(mode),
      openPalette: (mode) => (dialogs.palette = mode),
      newTaskDate: () => {
        const target = tasksStore.defaultTarget();
        return target.section === "agenda" ? target.date : null;
      },
      openNewTask: (date) => dialogs.openNewTask(date),
      modalOpen: () => document.querySelector('[aria-modal="true"]') !== null,
    });
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="shell">
  {#if viewStore.section === "notes"}
    {#if FolderSidebar}
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
      <div
        class="sidebar-placeholder"
        style:width={sidebarOpen ? "224px" : "0"}
        aria-hidden="true"
      ></div>
    {/if}
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
      <span></span>
      <div class="trail" data-tauri-drag-region>
        <button
          type="button"
          class="icon-btn"
          aria-label={t.settings.open}
          title={t.settings.open}
          aria-haspopup="dialog"
          onclick={() => (dialogs.settingsOpen = true)}
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
          {#if NotesView}
            <NotesView
              notes={notesStore}
              tags={tagsStore}
              tasks={tasksStore}
              onPreviewTask={(taskId) => (dialogs.detailId = taskId)}
              onOpenTask={(task) => {
                void openInList(task);
              }}
            />
          {/if}
        </div>
      {:else if viewStore.section === "today"}
        <div class="content scrolls" data-menu-area="agenda">
          {#if TodayView}
            <TodayView
              store={tasksStore}
              notes={notesStore}
              view={viewStore}
              onOpenTask={(taskId) => (dialogs.detailId = taskId)}
              onOpenNote={(noteId) => void showNote(noteId)}
              onOpenDaily={(date) => void showDailyNote(date)}
            />
          {/if}
        </div>
      {:else if viewStore.mode === "list"}
        <div class="content scrolls" data-menu-area="agenda">
          <TodoView store={tasksStore} />
        </div>
      {:else}
        <div class="content" data-menu-area="agenda">
          {#await loadCalendar()}
            <EmptyState variant="loading" title={t.calendar.loading} />
          {:then module}
            <module.default
              store={tasksStore}
              view={viewStore}
              onOpenTask={(task: Task) => (dialogs.detailId = task.id)}
            />
          {:catch}
            <EmptyState variant="error" title={t.calendar.loadFailed} />
          {/await}
        </div>
      {/if}
    {/if}
  </div>
</div>

{#if viewStore.inAgenda}
  <button
    type="button"
    class="fab"
    title={t.newTask.buttonHint}
    aria-label={t.newTask.button}
    aria-haspopup="dialog"
    onclick={() => dialogs.openNewTask()}
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

{#if viewStore.section === "calendar"}
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

<DialogHost
  {dialogs}
  onShowTask={(taskId) => void showTask(taskId)}
  onShowInList={(task) => void openInList(task)}
/>

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
</style>
