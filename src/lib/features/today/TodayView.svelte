<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
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
    <p class="date">{heading}</p>
    <h2>{l.title}</h2>
  </header>

  <section class="block" aria-labelledby="today-tasks">
    <h3 id="today-tasks">
      {l.tasks}
      <span class="count">{l.pending(pending)}</span>
    </h3>
    {#if dayTasks.length === 0}
      <div class="card none">
        <strong>{l.noTasksTitle}</strong>
        <span>{l.noTasksDescription}</span>
      </div>
    {:else}
      <ul class="card">
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
              <span class="title">{task.title || t.reminders.untitled}</span>
              <span class="time">{task.dueTime ?? l.allDay}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="block" aria-labelledby="today-daily">
    <h3 id="today-daily">{l.daily}</h3>
    <div class="card">
      <button
        type="button"
        class="open solo"
        onclick={() => onOpenDaily(today)}
      >
        <svg
          class="lead"
          viewBox="0 0 16 16"
          width="15"
          height="15"
          aria-hidden="true"
        >
          <path
            d="M4 2.5h5.5L12.5 5.5v8H4zM9.5 2.5v3h3M6 8.5h4M6 11h3"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span class="title">{l.openDaily}</span>
        {@render chevron()}
      </button>
    </div>
  </section>

  <section class="block" aria-labelledby="today-pinned">
    <h3 id="today-pinned">{l.pinned}</h3>
    {#if pinnedLoaded && pinned.length === 0}
      <p class="card none"><span>{l.noPinned}</span></p>
    {:else if pinned.length > 0}
      <ul class="card">
        {#each pinned as note (note.id)}
          <li class="row">
            <button
              type="button"
              class="open stacked"
              onclick={() => onOpenNote(note.id)}
            >
              <span class="text">
                <span class="title">{note.title || t.notesApp.untitled}</span>
                {#if note.excerpt}
                  <span class="excerpt">{note.excerpt}</span>
                {/if}
              </span>
              {@render chevron()}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

{#snippet chevron()}
  <svg
    class="chevron"
    viewBox="0 0 12 12"
    width="10"
    height="10"
    aria-hidden="true"
  >
    <path
      d="M4 2.5l3.5 3.5L4 9.5"
      fill="none"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
{/snippet}

<style>
  .today {
    display: grid;
    align-content: start;
    gap: var(--space-6);
    box-sizing: border-box;
    width: 100%;
    max-width: 680px;
    margin: 0 auto;
    padding: var(--space-4) var(--space-6) var(--space-8);
  }

  /* The day over a large title, as the sections of the agenda are headed. */
  .date {
    margin: 0;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .date::first-letter {
    text-transform: uppercase;
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: 700;
    line-height: 1.2;
    letter-spacing: -0.022em;
  }

  .block {
    display: grid;
    align-content: start;
    gap: 6px;
  }

  h3 {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    margin: 0;
    padding: 0 2px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .count {
    font-weight: 400;
    font-variant-numeric: tabular-nums;
  }

  /* One grouped card per block, its rows parted by a hairline (as the settings are). */
  .card {
    margin: 0;
    padding: 0;
    overflow: hidden;
    border-radius: var(--radius-card);
    background: var(--bg-control);
    list-style: none;
  }

  .row {
    position: relative;
    display: flex;
    align-items: center;
    padding-left: var(--space-3);
  }

  .row + .row::before {
    content: "";
    position: absolute;
    top: 0;
    right: 0;
    left: var(--space-3);
    height: 1px;
    background: var(--separator);
  }

  .open {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
    min-height: 40px;
    padding: 6px var(--space-3) 6px 10px;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  .open.solo {
    width: 100%;
    padding-left: var(--space-3);
  }

  .open.stacked {
    min-height: 48px;
    padding-left: 0;
  }

  .row {
    transition: background-color var(--motion-instant) linear;
  }

  /* The whole row lights up, checkbox included, not only the part that opens. */
  .row:hover,
  .open.solo:hover {
    background: var(--bg-hover);
  }

  .open.solo:active {
    background: var(--bg-active);
  }

  .open:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    border-radius: var(--radius-control);
  }

  .text {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    font-weight: 500;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .text .title {
    flex: none;
  }

  .time,
  .excerpt {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .time {
    flex: none;
    font-variant-numeric: tabular-nums;
  }

  .excerpt {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .done .title {
    color: var(--text-secondary);
    font-weight: 400;
    text-decoration: line-through;
  }

  .lead {
    flex: none;
    color: var(--accent);
  }

  .chevron {
    flex: none;
    color: var(--text-secondary);
    opacity: 0.6;
  }

  .none {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-3);
    color: var(--text-secondary);
    font-size: 13px;
  }

  .none strong {
    color: var(--text-primary);
    font-weight: 600;
  }
</style>
