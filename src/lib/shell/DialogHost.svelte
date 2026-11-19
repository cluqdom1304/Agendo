<script lang="ts">
  import {
    convertTaskToNote,
    createNoteForTask,
    showNote,
  } from "$lib/features/notes/navigation";
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
