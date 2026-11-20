<script lang="ts">
  import {
    convertTaskToNote,
    createNoteForTask,
    showNote,
  } from "$lib/features/notes/navigation";
  import { eventToTask, taskToEvent } from "$lib/features/events/convert";
  import { eventsStore } from "$lib/stores/events.svelte";
  import { habitsStore } from "$lib/stores/habits.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { notesStore } from "$lib/stores/notes.svelte";
  import { tagsStore } from "$lib/stores/tags.svelte";
  import { tasksStore } from "$lib/stores/tasks.svelte";
  import { viewStore } from "$lib/stores/view.svelte";
  import type { Task } from "$lib/types";

  import type { DialogState } from "./dialogs.svelte";

  interface Props {
    dialogs: DialogState;
    /** Shows a task found by the palette in the agenda's list. */
    onShowTask: (taskId: string) => void;
    onShowInList: (task: Task) => void;
  }

  let { dialogs, onShowTask, onShowInList }: Props = $props();

  // Dialogs and the palette are only needed once opened, so they stay out of the first load.
  const loadMenu = () => import("$lib/features/context/ContextMenu.svelte");
  const loadPalette = () => import("$lib/features/palette/PaletteHost.svelte");
  const loadTaskDialog = () => import("$lib/features/todo/TaskDialog.svelte");
  const loadEventDialog = () =>
    import("$lib/features/events/EventDialog.svelte");
  const loadHabitDialog = () =>
    import("$lib/features/habits/HabitDialog.svelte");
  const loadTaskDetail = () =>
    import("$lib/features/todo/TaskDetailDialog.svelte");
  const loadSettings = () =>
    import("$lib/features/settings/SettingsDialog.svelte");

  function onRestored() {
    tagsStore.showAll();
    void tagsStore.load();
    void tasksStore.load();
    void viewStore.load();
    // Folders, notes and tags changed too; the notes view reads them again if it was loaded.
    if (notesStore.status !== "idle") void notesStore.load();
  }
</script>

{#if dialogs.palette}
  {#await loadPalette() then { default: PaletteHost }}
    <PaletteHost
      mode={dialogs.palette}
      onTask={(taskId) => onShowTask(taskId)}
      onNewTask={() => dialogs.openNewTask()}
      onNewEvent={() => dialogs.openNewEvent()}
      onNewHabit={() => dialogs.openNewHabit()}
      onClose={() => (dialogs.palette = null)}
    />
  {/await}
{/if}

{#await loadMenu() then { default: ContextMenu }}
  <ContextMenu
    deps={{
      view: viewStore,
      tasks: tasksStore,
      tags: tagsStore,
      notes: notesStore,
      newTask: (date) => dialogs.openNewTask(date),
      newEvent: (date) => dialogs.openNewEvent(date),
      openEvent: (eventId) => dialogs.openEvent(eventId),
      taskToEvent: (task) =>
        taskToEvent(task, eventsStore, tasksStore, viewStore.today()),
      eventToTask: async (eventId) => {
        const event = await eventsStore.get(eventId);
        return event ? eventToTask(event, eventsStore, tasksStore) : null;
      },
      deleteEvent: async (eventId) => {
        const event = await eventsStore.get(eventId);
        return event ? eventsStore.remove(event) : null;
      },
      editTask: (task) => (dialogs.editing = task),
      openTask: (taskId) => (dialogs.detailId = taskId),
      showNote,
      noteForTask: createNoteForTask,
      taskToNote: convertTaskToNote,
    }}
  />
{/await}

{#if dialogs.newTaskOpen}
  {#await loadTaskDialog() then { default: TaskDialog }}
    <TaskDialog
      store={tasksStore}
      defaultDate={dialogs.newTaskDate ??
        (viewStore.mode === "list" ? null : viewStore.anchor)}
      onClose={() => (dialogs.newTaskOpen = false)}
    />
  {/await}
{/if}

{#if dialogs.event}
  {#await loadEventDialog() then { default: EventDialog }}
    {#key dialogs.event}
      <EventDialog
        store={eventsStore}
        eventId={dialogs.event.id}
        defaultDate={dialogs.event.date ??
          (viewStore.mode === "list" ? viewStore.today() : viewStore.anchor)}
        defaultTime={dialogs.event.time}
        onClose={() => (dialogs.event = null)}
        onConvert={(event) => {
          dialogs.event = null;
          void eventToTask(event, eventsStore, tasksStore);
        }}
      />
    {/key}
  {/await}
{/if}

{#if dialogs.habit}
  {#await loadHabitDialog() then { default: HabitDialog }}
    {#key dialogs.habit}
      <HabitDialog
        store={habitsStore}
        habit={dialogs.habit.habit}
        onClose={() => (dialogs.habit = null)}
      />
    {/key}
  {/await}
{/if}

{#if dialogs.detailId && !dialogs.editing}
  {#await loadTaskDetail() then { default: TaskDetailDialog }}
    <TaskDetailDialog
      store={tasksStore}
      taskId={dialogs.detailId}
      onEdit={(task) => (dialogs.editing = task)}
      onShowInList={(task) => onShowInList(task)}
      onClose={() => (dialogs.detailId = null)}
    />
  {/await}
{/if}

{#if dialogs.editing}
  {#await loadTaskDialog() then { default: TaskDialog }}
    <TaskDialog
      store={tasksStore}
      task={dialogs.editing}
      onClose={() => (dialogs.editing = null)}
    />
  {/await}
{/if}

{#if dialogs.settingsOpen}
  {#await loadSettings() then { default: SettingsDialog }}
    <SettingsDialog
      store={settingsStore}
      onClose={() => {
        dialogs.settingsOpen = false;
        // The default notification time moves saved reminders, which tasks hold a copy of.
        void tasksStore.load();
      }}
      {onRestored}
    />
  {/await}
{/if}
