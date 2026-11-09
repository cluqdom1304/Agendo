<script lang="ts">
  import { onMount, tick } from "svelte";

  import { onReminderFired } from "$lib/api";
  import TaskDialog from "$lib/features/todo/TaskDialog.svelte";
  import SettingsDialog from "$lib/features/settings/SettingsDialog.svelte";
  import TaskDetailDialog from "$lib/features/todo/TaskDetailDialog.svelte";
  import TodoView from "$lib/features/todo/TodoView.svelte";
  import { t } from "$lib/i18n";
  import { findShortcut, focusContextOf, ROW_ATTRIBUTE } from "$lib/keymap";
  import SegmentedControl from "$lib/shell/SegmentedControl.svelte";
  import Toast from "$lib/shell/Toast.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { tasksStore } from "$lib/stores/tasks.svelte";
  import { viewStore } from "$lib/stores/view.svelte";
  import type { Task, ViewMode } from "$lib/types";

  type Section = "list" | "calendar";

  let settingsOpen = $state(false);
  let newTaskOpen = $state(false);
  /** The task whose view is open (opened from the calendar). */
  let detailId = $state<string | null>(null);
  /** The task being edited in the creation dialog; the view comes back when it closes. */
  let editing = $state.raw<Task | null>(null);

  // The list shows its own deletion notices; elsewhere they are shown here.
  const lastDeletion = $derived(tasksStore.undoStack.at(-1));

  const SECTION_OPTIONS: { value: Section; label: string }[] = [
    { value: "list", label: t.views.list },
    { value: "calendar", label: t.views.calendar },
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

  onMount(() => {
    void tasksStore.load();
    void viewStore.load();
    const unlisten = onReminderFired((fired) =>
      tasksStore.onReminderFired(fired.taskId),
    );
    return () => {
      void unlisten.then((stop) => stop());
      tasksStore.dispose();
    };
  });

  function onRestored() {
    void tasksStore.load();
    void viewStore.load();
  }

  async function openInList(task: Task) {
    detailId = null;
    viewStore.setMode("list");
    if (!tasksStore.reveal(task.id)) return;
    await tick();
    const row = document.querySelector<HTMLElement>(
      `[${ROW_ATTRIBUTE}][data-task-id="${task.id}"]`,
    );
    row?.focus();
    row?.scrollIntoView?.({ block: "center" });
  }

  function onKeydown(event: KeyboardEvent) {
    const shortcut = findShortcut(event, focusContextOf(event.target));
    const mode = shortcut ? SHORTCUT_VIEWS[shortcut] : undefined;
    if (!mode) return;
    event.preventDefault();
    viewStore.setMode(mode);
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="page">
  <div class="toolbar">
    <button
      type="button"
      class="new-task"
      title={t.newTask.buttonHint}
      aria-haspopup="dialog"
      onclick={() => (newTaskOpen = true)}
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
      {t.newTask.button}
    </button>
    <SegmentedControl
      label={t.views.label}
      options={SECTION_OPTIONS}
      value={viewStore.mode === "list" ? "list" : "calendar"}
      onChange={(section) =>
        viewStore.setMode(section === "list" ? "list" : viewStore.calendarMode)}
    />
    <button
      type="button"
      class="settings-button"
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

  {#if viewStore.ready}
    {#if viewStore.mode === "list"}
      <div class="content scrolls">
        <TodoView store={tasksStore} />
      </div>
    {:else}
      <div class="content">
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

{#if newTaskOpen}
  <TaskDialog
    store={tasksStore}
    defaultDate={viewStore.mode === "list" ? null : viewStore.anchor}
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

{#if viewStore.mode !== "list"}
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
    onClose={() => (settingsOpen = false)}
    {onRestored}
  />
{/if}

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .toolbar {
    position: relative;
    display: flex;
    justify-content: center;
    flex: none;
    padding: var(--space-2) var(--space-4) 0;
  }

  .new-task {
    position: absolute;
    top: var(--space-2);
    left: var(--space-4);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    border: 0;
    border-radius: var(--radius-control);
    background: var(--accent);
    color: var(--on-accent);
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .new-task:hover {
    filter: brightness(1.08);
  }

  .new-task:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .settings-button {
    position: absolute;
    top: var(--space-2);
    right: var(--space-4);
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .settings-button:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .settings-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .content {
    flex: 1;
    min-height: 0;
  }

  .scrolls {
    overflow: auto;
  }

  .status {
    padding: var(--space-6);
    color: var(--text-secondary);
  }
</style>
