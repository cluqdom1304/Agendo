<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import EmptyState from "../../shell/EmptyState.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import type { NoteSummary } from "../../types";
  import Checkbox from "../todo/Checkbox.svelte";

  // «Hoy» (F13-07): what the day holds in one place: its tasks, its daily note and the notes
  // pinned in the notes section.
  interface Props {
    store: TasksStore;
    notes: NotesStore;
    view: ViewStore;
    onOpenTask: (taskId: string) => void;
    onOpenNote: (noteId: string) => void;
    onOpenDaily: (date: string) => void;
  }

  let { store, notes, view, onOpenTask, onOpenNote, onOpenDaily }: Props =
    $props();
  const l = t.today;

  // A window left open overnight must not keep showing yesterday.
  // svelte-ignore state_referenced_locally
  let today = $state(view.today());
  let pinned = $state.raw<NoteSummary[]>([]);
  let pinnedLoaded = $state(false);

  const heading = $derived(
    new Intl.DateTimeFormat("es-ES", {
      weekday: "long",
      day: "numeric",
      month: "long",
    }).format(new Date(`${today}T12:00:00`)),
  );

  /** The tasks of the day: open ones first, by time (all-day ones before timed), then done. */
  const dayTasks = $derived(
    store.visibleTasks
      .filter((task) => task.dueDate === today)
      .toSorted(
        (a, b) =>
          Number(a.isCompleted) - Number(b.isCompleted) ||
          (a.dueTime ?? "").localeCompare(b.dueTime ?? "") ||
          a.position - b.position,
      ),
  );
  const pending = $derived(dayTasks.filter((task) => !task.isCompleted).length);

  async function loadPinned() {
    pinned = await notes.pinnedNotes();
    pinnedLoaded = true;
  }

  onMount(() => void loadPinned());

  function refresh() {
    today = view.today();
    void loadPinned();
  }
</script>

<svelte:window onfocus={refresh} />

<div class="today">
  <header class="head">
    <h2>{l.title}</h2>
    <p class="date">{heading}</p>
  </header>

  <section class="block" aria-labelledby="today-tasks">
    <h3 id="today-tasks">
      {l.tasks}
      <span class="count">{l.pending(pending)}</span>
    </h3>
    {#if dayTasks.length === 0}
      <EmptyState title={l.noTasksTitle} description={l.noTasksDescription} />
    {:else}
      <ul class="rows">
        {#each dayTasks as task (task.id)}
          <li class="row" class:done={task.isCompleted}>
            <Checkbox
              checked={task.isCompleted}
              label={task.isCompleted
                ? l.markOpen(task.title || t.reminders.untitled)
                : l.markDone(task.title || t.reminders.untitled)}
              onToggle={() =>
                void store.setCompleted(task.id, !task.isCompleted)}
            />
            <button
              type="button"
              class="open"
              onclick={() => onOpenTask(task.id)}
            >
              <span class="time">{task.dueTime ?? l.allDay}</span>
              <span class="title">{task.title || t.reminders.untitled}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="block" aria-labelledby="today-daily">
    <h3 id="today-daily">{l.daily}</h3>
    <button type="button" class="btn" onclick={() => onOpenDaily(today)}>
      {l.openDaily}
    </button>
  </section>

  <section class="block" aria-labelledby="today-pinned">
    <h3 id="today-pinned">{l.pinned}</h3>
    {#if pinnedLoaded && pinned.length === 0}
      <p class="muted">{l.noPinned}</p>
    {:else}
      <ul class="rows">
        {#each pinned as note (note.id)}
          <li class="row">
            <button
              type="button"
              class="open"
              onclick={() => onOpenNote(note.id)}
            >
              <span class="title">{note.title || t.notesApp.untitled}</span>
              {#if note.excerpt}
                <span class="excerpt">{note.excerpt}</span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .today {
    display: grid;
    align-content: start;
    gap: var(--space-5, 24px);
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-5, 24px) var(--space-4);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 24px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  h3 {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .date {
    margin: 2px 0 0;
    color: var(--text-secondary);
    text-transform: capitalize;
  }

  .count,
  .muted,
  .time,
  .excerpt {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 400;
  }

  .muted {
    margin: 0;
  }

  .block {
    display: grid;
    align-content: start;
    gap: var(--space-2);
  }

  .block > .btn {
    justify-self: start;
  }

  .rows {
    display: grid;
    gap: 2px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .open {
    display: grid;
    flex: 1;
    min-width: 0;
    gap: 2px;
    padding: var(--space-2);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }

  .open:hover {
    background: var(--surface-hover, rgb(127 127 127 / 12%));
  }

  .title,
  .excerpt {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .done .title {
    color: var(--text-secondary);
    text-decoration: line-through;
  }
</style>
