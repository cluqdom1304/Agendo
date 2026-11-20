<script lang="ts">
  import { formatDisplayDate } from "../../dates";
  import { captureOptions } from "../capture/capture.svelte";
  import { t } from "../../i18n";
  import type { TaskBlock, TasksStore } from "../../stores/tasks.svelte";
  import { pending } from "../../reminders";
  import NoteEditor from "../notes/NoteEditor.svelte";
  import { hasNotes } from "../notes/content";
  import { showNote } from "../notes/navigation";
  import { focusStore } from "../../stores/focus.svelte";
  import { notesStore } from "../../stores/notes.svelte";
  import type { Task } from "../../types";
  import ReminderPopover from "./ReminderPopover.svelte";
  import { tagDrag } from "../../stores/tagDrag.svelte";
  import { NO_TAG, tagsStore } from "../../stores/tags.svelte";
  import {
    chunkRows,
    EAGER_CHUNKS,
    ESTIMATED_ROW_HEIGHT,
    WINDOW_THRESHOLD,
  } from "./chunks";
  import TaskRow from "./TaskRow.svelte";
  import WindowedChunk from "./WindowedChunk.svelte";

  // One block of tasks: a day of AGENDA, or POR HACER when `block.date` is null.
  interface Props {
    block: TaskBlock;
    store: TasksStore;
  }

  let { block, store }: Props = $props();

  const target = $derived(
    block.date === null
      ? ({ section: "todo" } as const)
      : ({ section: "agenda", date: block.date } as const),
  );
  /** The note a task comes from, once its title is known. */
  function sourceOf(task: Task): { id: string; title: string } | null {
    if (!task.sourceNoteId) return null;
    const title = notesStore.titleOf(task.sourceNoteId);
    return title === undefined ? null : { id: task.sourceNoteId, title };
  }

  const openChunks = $derived(
    block.open.length > WINDOW_THRESHOLD ? chunkRows(block.open) : null,
  );
  const completedChunks = $derived(
    block.completed.length > WINDOW_THRESHOLD
      ? chunkRows(block.completed)
      : null,
  );

  const draft = $derived(store.isDraftFor(target) ? store.draft : null);
  const heading = $derived(
    block.date === null ? t.tasks.sectionTodo : formatDisplayDate(block.date),
  );
</script>

{#snippet taskRow(task: TaskBlock["open"][number])}
  <TaskRow
    id={task.id}
    title={task.title}
    isCompleted={task.isCompleted}
    dueTime={task.dueTime}
    durationMinutes={task.durationMinutes}
    dueDate={task.dueDate}
    priority={task.priority}
    checklistDone={task.checklistDone}
    checklistTotal={task.checklistTotal}
    focusMinutes={focusStore.minutesOf(task.id)}
    tags={tagsStore.tagsOf(task)}
    onTagsChange={(tagIds) => store.setTags(task.id, tagIds)}
    sourceNote={sourceOf(task)}
    onOpenSource={(noteId) => void showNote(noteId)}
    onTagDragStart={(event) =>
      tagDrag.begin(event, { id: task.id, title: task.title }, (key) =>
        key === NO_TAG
          ? store.setTags(task.id, [])
          : store.addTag(task.id, key),
      )}
    onDateChange={(date) => store.setDueDate(task.id, date)}
    onDelete={() => store.remove(task.id)}
    hasNotes={hasNotes(task.contentJson)}
    notesOpen={store.isNotesOpen(task.id)}
    onToggleNotes={() => store.toggleNotes(task.id)}
    reminderCount={pending(task.reminders).length}
    remindersOpen={store.remindersOpenId === task.id}
    onToggleReminders={() => store.toggleReminders(task.id)}
    settling={store.settling.has(task.id)}
    onToggle={() => store.setCompleted(task.id, !task.isCompleted)}
    onSelect={() => (store.selectedId = task.id)}
    picked={store.batch.isSelected(task.id)}
    onPick={(mode) =>
      mode === "range"
        ? store.batch.extendTo(task.id)
        : store.batch.toggle(task.id)}
    onCommit={(text, trigger) => {
      void store.rename(task.id, text);
      if (trigger === "enter") store.startDraft(target);
    }}
  />
  {#if store.remindersOpenId === task.id}
    <ReminderPopover
      title={task.title}
      dueDate={task.dueDate}
      dueTime={task.dueTime}
      reminders={task.reminders}
      onSetTime={(time) => void store.setDueTime(task.id, time)}
      onChange={(inputs) => void store.setReminders(task.id, inputs)}
      onClose={() => (store.remindersOpenId = null)}
    />
  {/if}
  {#if store.isNotesOpen(task.id)}
    <NoteEditor
      variant="inline"
      contentJson={task.contentJson}
      save={(json) => store.saveNotes(task.id, json)}
      autofocus={store.notesFocusId === task.id}
      onAutofocus={() => (store.notesFocusId = null)}
      onEmptyBlur={(next) => {
        // Its own row keeps it open: the notes button there already toggles it.
        if (!next?.closest(`[data-task-id="${task.id}"]`))
          store.closeEmptyNotes(task.id);
      }}
    />
  {/if}
{/snippet}

{#snippet rowList(tasks: TaskBlock["open"], chunks: TaskBlock["open"][] | null)}
  {#if chunks}
    {#each chunks as rows, index (index)}
      <WindowedChunk
        eager={index < EAGER_CHUNKS}
        forced={rows.some((task) => task.id === store.selectedId)}
        estimate={rows.length * ESTIMATED_ROW_HEIGHT}
      >
        {#each rows as task (task.id)}
          {@render taskRow(task)}
        {/each}
      </WindowedChunk>
    {/each}
  {:else}
    {#each tasks as task (task.id)}
      {@render taskRow(task)}
    {/each}
  {/if}
{/snippet}

<section class="block" aria-label={heading}>
  {#if block.date !== null}
    <h3 class="day">{heading}</h3>
  {/if}

  <div class="open">
    {@render rowList(block.open, openChunks)}
  </div>

  <!-- Keyed by draft: each draft gets a fresh input, and the handlers keep the key of the
       draft they were created for even when a late blur arrives after it was closed. -->
  {#each draft ? [draft] : [] as current (current.key)}
    <TaskRow
      id=""
      title=""
      isCompleted={false}
      dueTime={null}
      draft
      capture={() =>
        captureOptions(
          store.today(),
          current.target.section === "agenda" ? current.target.date : undefined,
        )}
      onCommit={(text, trigger, capture) =>
        void store.commitDraft(current.key, text, trigger === "enter", capture)}
      onCancel={() => store.cancelDraft(current.key)}
    />
  {/each}

  {#if block.completed.length > 0}
    <div class="completed">
      <h4 class="completed-heading">
        {t.tasks.completed}
        <span class="completed-count">{block.completed.length}</span>
      </h4>
      {@render rowList(block.completed, completedChunks)}
    </div>
  {/if}
</section>

<style>
  .block {
    padding-top: var(--space-1);
  }

  .day {
    margin: 14px var(--space-3) var(--space-1);
    padding-top: var(--space-3);
    border-top: 1px solid var(--separator);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .day::first-letter {
    text-transform: uppercase;
  }

  .completed {
    margin-top: var(--space-2);
  }

  .completed-heading {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    padding: var(--space-1) var(--space-3);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .completed-count {
    font-variant-numeric: tabular-nums;
  }
</style>
