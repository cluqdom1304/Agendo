<script lang="ts">
  import { t } from "../../i18n";
  import { findShortcut } from "../../keymap";
  import EmptyState from "../../shell/EmptyState.svelte";
  import SegmentedControl from "../../shell/SegmentedControl.svelte";
  import Toast from "../../shell/Toast.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { TagsStore } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { Task } from "../../types";
  import NoteList from "./NoteList.svelte";
  import NotePane from "./NotePane.svelte";
  import TagTasks from "./TagTasks.svelte";

  interface Props {
    notes: NotesStore;
    tags: TagsStore;
    tasks: TasksStore;
    /** Shows a task in the agenda. */
    onOpenTask: (task: Task) => void;
  }

  let { notes, tags, tasks, onOpenTask }: Props = $props();

  type Tab = "notes" | "tasks";
  let tab = $state<Tab>("notes");

  const lastTrashed = $derived(notes.undoStack.at(-1));
  const tagId = $derived(notes.scopeTagId);

  // Pending tasks first, then the finished ones; each group keeps the store's order.
  const tagTasks = $derived.by(() => {
    if (!tagId) return [];
    const own = tasks.tasks.filter((task) => task.tagIds.includes(tagId));
    return [
      ...own.filter((task) => !task.isCompleted),
      ...own.filter((task) => task.isCompleted),
    ];
  });

  // Leaving a tag, or opening another, goes back to its notes.
  $effect(() => {
    void tagId;
    tab = "notes";
  });

  // The tag counters follow the notes: refreshed when a note appears, goes or changes tags.
  const tagSignature = $derived(
    notes.notes.map((note) => `${note.id}:${note.tagIds.join(",")}`).join("|"),
  );
  $effect(() => {
    void tagSignature;
    void tags.load();
  });

  function create() {
    void notes.createNote();
  }

  // Ctrl+N is the section's own "new": the agenda's handler is not mounted here.
  function onKeydown(event: KeyboardEvent) {
    if (findShortcut(event, "other") !== "newTask") return;
    event.preventDefault();
    create();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="notes">
  {#if notes.status === "error"}
    <p class="status" role="alert">{t.notesApp.loadFailed}</p>
  {:else}
    <div class="column">
      {#if tagId}
        <div class="tabs">
          <SegmentedControl
            label={t.notesApp.tagTabs}
            options={[
              {
                value: "notes",
                label: t.notesApp.tabNotes(notes.notes.length),
              },
              {
                value: "tasks",
                label: t.notesApp.tabTasks(tagTasks.length),
              },
            ]}
            value={tab}
            onChange={(next) => (tab = next as Tab)}
          />
        </div>
      {/if}
      {#if tagId && tab === "tasks"}
        <TagTasks tasks={tagTasks} onOpen={onOpenTask} />
      {:else}
        <NoteList {notes} {tags} onCreate={create} />
      {/if}
    </div>
    <div class="pane">
      {#if notes.open}
        {#key notes.open.id}
          <NotePane {notes} note={notes.open} />
        {/key}
      {:else}
        <EmptyState
          title={t.notesApp.noSelectionTitle}
          description={t.notesApp.noSelectionDescription}
        />
      {/if}
    </div>
  {/if}
</div>

{#if lastTrashed}
  <Toast
    message={t.notesApp.deleted}
    actionLabel={t.notesApp.undo}
    closeLabel={t.notesApp.dismiss}
    onAction={() => void notes.undoTrash()}
    onClose={() => (notes.undoStack = [])}
  />
{:else if notes.notice}
  <Toast
    kind={notes.notice.kind}
    message={notes.notice.message}
    closeLabel={t.notesApp.dismiss}
    onClose={() => notes.dismissNotice()}
  />
{/if}

<style>
  .notes {
    display: flex;
    height: 100%;
  }

  .column {
    display: flex;
    flex: none;
    flex-direction: column;
    width: 300px;
    min-height: 0;
    height: 100%;
    box-shadow: inset -1px 0 0 var(--separator);
  }

  .tabs {
    display: flex;
    flex: none;
    justify-content: center;
    padding: var(--space-2) var(--space-3) var(--space-1);
  }

  .pane {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .status {
    padding: var(--space-6);
    color: var(--text-secondary);
  }
</style>
