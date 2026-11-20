<script lang="ts">
  import { onMount, tick } from "svelte";

  import { showDailyNote, showNote } from "$lib/features/notes/navigation";
  import TagDragGhost from "$lib/features/tags/TagDragGhost.svelte";
  import TodoView from "$lib/features/todo/TodoView.svelte";
  import { t } from "$lib/i18n";
  import { ROW_ATTRIBUTE } from "$lib/keymap";
  import AppSidebar from "$lib/shell/AppSidebar.svelte";
  import { listenBackendEvents } from "$lib/shell/backendEvents";
  import DialogHost from "$lib/shell/DialogHost.svelte";
  import { DialogState } from "$lib/shell/dialogs.svelte";
  import EmptyState from "$lib/shell/EmptyState.svelte";
  import { dispatchShortcut } from "$lib/shell/shortcuts";
  import Toast from "$lib/shell/Toast.svelte";
  import TopBar from "$lib/shell/TopBar.svelte";
  import { notesStore } from "$lib/stores/notes.svelte";
  import { folderDrag, tagDrag } from "$lib/stores/tagDrag.svelte";
  import { tagsStore } from "$lib/stores/tags.svelte";
  import { tasksStore } from "$lib/stores/tasks.svelte";
  import { viewStore } from "$lib/stores/view.svelte";
  import type { Task } from "$lib/types";

  const dialogs = new DialogState();

  // Narrow windows start with the sidebar folded so the content keeps its room.
  let sidebarOpen = $state(
    typeof window === "undefined" || window.innerWidth >= 900,
  );

  // The tag filter only hides tasks: in the notes there is nothing to warn about.
  const filterShown = $derived(tagsStore.filtered && viewStore.inAgenda);

  // The list shows its own deletion notices; elsewhere they are shown here.
  const lastDeletion = $derived(tasksStore.undoStack.at(-1));

  // The calendar is a separate chunk: it is only requested when a calendar view is shown, so
  // it does not weigh on startup (N-01).
  const loadCalendar = () =>
    import("$lib/features/calendar/CalendarView.svelte");

  // Dialogs and the palette load from `DialogHost`; views are only requested once shown.
  // They are fetched once the shell is idle and then rendered synchronously: awaiting a promise
  // inside the template leaves a frame without sidebar, which made the toolbar jump when
  // switching section.
  let NotesView = $state<
    typeof import("$lib/features/notes/NotesView.svelte").default | null
  >(null);
  let TodayView = $state<
    typeof import("$lib/features/today/TodayView.svelte").default | null
  >(null);

  async function preloadViews() {
    const [notes, today] = await Promise.all([
      import("$lib/features/notes/NotesView.svelte"),
      import("$lib/features/today/TodayView.svelte"),
    ]);
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

  /** The «new» button of the top bar: a note in the notes, a task everywhere else. */
  function createNew() {
    if (viewStore.section === "notes") {
      void notesStore.ensureLoaded().then(() => notesStore.createNote());
    } else {
      dialogs.openNewTask();
    }
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
  <TopBar
    {sidebarOpen}
    filtered={filterShown}
    canGoBack={viewStore.canGoBack}
    canGoForward={viewStore.canGoForward}
    newLabel={viewStore.section === "notes"
      ? t.topBar.newNote
      : t.newTask.button}
    onToggleSidebar={() => (sidebarOpen = !sidebarOpen)}
    onNew={createNew}
    onBack={() => viewStore.goBack()}
    onForward={() => viewStore.goForward()}
    onSearch={() => (dialogs.palette = "all")}
    onSettings={() => (dialogs.settingsOpen = true)}
  />
  <div class="body">
    <AppSidebar
      open={sidebarOpen}
      section={viewStore.section}
      notes={notesStore}
      tags={tagsStore}
      tasks={tasksStore}
      onSection={(section) => viewStore.setSection(section)}
      onOpenNote={(id) => void showNote(id)}
    />
    <div class="page">
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
</div>

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
    flex-direction: column;
    height: 100%;
    background: var(--sidebar-bg);
  }

  :global(.native-effect) .shell {
    background: var(--sidebar-bg-native);
  }

  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .page {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    background: var(--bg-window-solid);
  }

  .content {
    flex: 1;
    min-height: 0;
    animation: rise-in var(--motion-base) var(--ease-out);
  }

  .scrolls {
    overflow: auto;
  }
</style>
