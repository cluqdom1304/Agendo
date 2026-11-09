<script lang="ts">
  import { onMount, tick } from "svelte";

  import { onReminderFired } from "$lib/api";
  import TodoView from "$lib/features/todo/TodoView.svelte";
  import { t } from "$lib/i18n";
  import { findShortcut, focusContextOf, ROW_ATTRIBUTE } from "$lib/keymap";
  import SegmentedControl from "$lib/shell/SegmentedControl.svelte";
  import { tasksStore } from "$lib/stores/tasks.svelte";
  import { viewStore } from "$lib/stores/view.svelte";
  import type { Task, ViewMode } from "$lib/types";

  type Section = "list" | "calendar";

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

  async function openInList(task: Task) {
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
    <SegmentedControl
      label={t.views.label}
      options={SECTION_OPTIONS}
      value={viewStore.mode === "list" ? "list" : "calendar"}
      onChange={(section) =>
        viewStore.setMode(section === "list" ? "list" : viewStore.calendarMode)}
    />
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
            onOpenTask={openInList}
          />
        {:catch}
          <p class="status">{t.calendar.loadFailed}</p>
        {/await}
      </div>
    {/if}
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .toolbar {
    display: flex;
    justify-content: center;
    flex: none;
    padding: var(--space-2) var(--space-4) 0;
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
