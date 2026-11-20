<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import type { EventsStore } from "../../stores/events.svelte";
  import type { HabitsStore } from "../../stores/habits.svelte";
  import type { NotesStore } from "../../stores/notes.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import type { NoteSummary } from "../../types";
  import { addDays } from "../calendar/grid";
  import EventRow from "../events/EventRow.svelte";
  import { isScheduled } from "../habits/habits";
  import Checkbox from "../todo/Checkbox.svelte";
  import TaskLine from "../todo/TaskLine.svelte";
  import {
    pendingByDate,
    STRIP_DAYS,
    stripDates,
    summarizeDay,
  } from "./summary";

  // «Hoy» (F13-07, F20-01): what a day holds in one place: a summary, its tasks, its daily
  // note and the notes pinned in the notes section. The strip picks the day, today by default.
  interface Props {
    store: TasksStore;
    /** The events of the days of the strip; without it «Hoy» shows tasks only. */
    eventStore?: EventsStore;
    /** The habits of the day; without it «Hoy» shows no habits. */
    habitStore?: HabitsStore;
    notes: NotesStore;
    view: ViewStore;
    onOpenTask: (taskId: string) => void;
    onOpenEvent?: (eventId: string) => void;
    onOpenNote: (noteId: string) => void;
    onOpenDaily: (date: string) => void;
    /** Opens «Planificar mi día». */
    onPlan?: () => void;
  }

  let {
    store,
    eventStore,
    habitStore,
    notes,
    view,
    onOpenTask,
    onOpenEvent,
    onOpenNote,
    onOpenDaily,
    onPlan,
  }: Props = $props();
  const l = t.today;

  // A window left open overnight must not keep showing yesterday.
  // svelte-ignore state_referenced_locally
  let today = $state(view.today());
  // svelte-ignore state_referenced_locally
  let selected = $state(today);
  let pinned = $state.raw<NoteSummary[]>([]);
  let pinnedLoaded = $state(false);
  let stripEl = $state<HTMLElement>();

  const longDate = new Intl.DateTimeFormat("es-ES", {
    weekday: "long",
    day: "numeric",
    month: "long",
  });
  const shortWeekday = new Intl.DateTimeFormat("es-ES", { weekday: "short" });
  const longWeekday = new Intl.DateTimeFormat("es-ES", { weekday: "long" });
  const asDate = (iso: string) => new Date(`${iso}T12:00:00`);
  const capitalize = (text: string) =>
    text.charAt(0).toUpperCase() + text.slice(1);

  const isToday = $derived(selected === today);
  const heading = $derived(longDate.format(asDate(selected)));
  const title = $derived(
    isToday
      ? l.title
      : selected === addDays(today, 1)
        ? l.tomorrow
        : capitalize(longWeekday.format(asDate(selected))),
  );

  $effect(() => {
    void eventStore?.show(today, addDays(today, STRIP_DAYS - 1));
  });
  // The habits are only ticked for today: a later day has nothing to mark yet.
  $effect(() => {
    void habitStore?.show(addDays(today, -6), today);
  });
  const dayHabits = $derived(
    isToday
      ? (habitStore?.active.filter((habit) => isScheduled(habit.days, today)) ??
          [])
      : [],
  );
  const habitsDone = $derived(
    dayHabits.filter((habit) => habitStore?.isDone(habit.id, today)).length,
  );
  const dayEvents = $derived(eventStore?.byDate.get(selected) ?? []);

  const pendingCounts = $derived(pendingByDate(store.visibleTasks));
  const strip = $derived(
    stripDates(today).map((date) => {
      const pending = pendingCounts.get(date) ?? 0;
      return {
        date,
        weekday: shortWeekday.format(asDate(date)).replace(".", ""),
        day: Number(date.slice(8)),
        pending,
        marked: pending > 0 || (eventStore?.byDate.has(date) ?? false),
        label: l.stripDay(longDate.format(asDate(date)), pending),
      };
    }),
  );

  const summary = $derived(summarizeDay(store.visibleTasks, selected, today));
  const summaryText = $derived(
    isToday
      ? l.summaryToday(summary.pending, summary.total)
      : l.summaryDay(summary.pending, summary.total),
  );
  const progress = $derived(
    summary.total === 0 ? 0 : (summary.done / summary.total) * 100,
  );

  /** The tasks of the day: open ones first, by time (all-day ones before timed) and priority, then done. */
  const dayTasks = $derived(
    store.visibleTasks
      .filter((task) => task.dueDate === selected)
      .toSorted(
        (a, b) =>
          Number(a.isCompleted) - Number(b.isCompleted) ||
          (a.dueTime ?? "").localeCompare(b.dueTime ?? "") ||
          b.priority - a.priority ||
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
    const next = view.today();
    // A day that has gone by drops off the strip: the selection follows today.
    if (selected === today || selected < next) selected = next;
    today = next;
    void loadPinned();
  }

  /** Arrow keys move along the strip, which keeps a single tab stop (the chosen day). */
  function onStripKey(event: KeyboardEvent) {
    const index = strip.findIndex((day) => day.date === selected);
    const target =
      event.key === "ArrowRight"
        ? index + 1
        : event.key === "ArrowLeft"
          ? index - 1
          : event.key === "Home"
            ? 0
            : event.key === "End"
              ? strip.length - 1
              : null;
    if (target === null || target < 0 || target >= strip.length) return;
    event.preventDefault();
    selected = strip[target].date;
    stripEl?.querySelector<HTMLElement>(`[data-date="${selected}"]`)?.focus();
  }
</script>

<svelte:window onfocus={refresh} />

<div class="today view">
  <header class="view-head">
    <div class="view-heading">
      <h2 class="view-title">{title}</h2>
      <p class="view-sub">{heading}</p>
    </div>
    <div class="view-actions">
      {#if onPlan}
        <button type="button" class="btn small" onclick={onPlan}>
          {t.plan.open}
        </button>
      {/if}
    </div>
  </header>

  <div class="strip" role="group" aria-label={l.strip} bind:this={stripEl}>
    {#each strip as day (day.date)}
      <button
        type="button"
        class="day"
        class:selected={day.date === selected}
        class:is-today={day.date === today}
        data-date={day.date}
        aria-pressed={day.date === selected}
        aria-label={day.label}
        tabindex={day.date === selected ? 0 : -1}
        onclick={() => (selected = day.date)}
        onkeydown={onStripKey}
      >
        <span class="number">{day.day}</span>
        <span class="weekday">{day.weekday}</span>
        <span class="mark" class:shown={day.marked}></span>
      </button>
    {/each}
  </div>

  <section class="banner" aria-label={l.summary}>
    <p class="headline">{summaryText}</p>
    {#if dayEvents.length > 0 || dayHabits.length > 0 || summary.overdue > 0}
      <p class="pills">
        {#if dayEvents.length > 0}
          <span class="pill event">{t.events.summary(dayEvents.length)}</span>
        {/if}
        {#if dayHabits.length > 0}
          <span class="pill habit">
            {t.habits.todayCount(habitsDone, dayHabits.length)}
          </span>
        {/if}
        {#if summary.overdue > 0}
          <span class="pill overdue">{l.overdue(summary.overdue)}</span>
        {/if}
      </p>
    {/if}
    {#if summary.total > 0}
      <div class="progress">
        <div
          class="track"
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={summary.total}
          aria-valuenow={summary.done}
          aria-label={l.progress(summary.done, summary.total)}
        >
          <span class="fill" style:width="{progress}%"></span>
        </div>
        <span class="ratio">{l.progress(summary.done, summary.total)}</span>
      </div>
    {/if}
  </section>

  {#if dayEvents.length > 0}
    <section class="block" aria-labelledby="today-events">
      <h3 id="today-events" class="group-label">
        {t.events.dayEvents}
        <span class="count">{dayEvents.length}</span>
      </h3>
      <ul class="list">
        {#each dayEvents as occurrence (occurrence.eventId + occurrence.startDate)}
          <EventRow {occurrence} onOpen={(eventId) => onOpenEvent?.(eventId)} />
        {/each}
      </ul>
    </section>
  {/if}

  {#if dayHabits.length > 0 && habitStore}
    <section class="block" aria-labelledby="today-habits">
      <h3 id="today-habits" class="group-label">
        {t.habits.today}
        <span class="count">
          {t.habits.todayCount(habitsDone, dayHabits.length)}
        </span>
      </h3>
      <ul class="list">
        {#each dayHabits as habit (habit.id)}
          {@const done = habitStore.isDone(habit.id, today)}
          <li class="row" class:done data-habit-id={habit.id}>
            <Checkbox
              checked={done}
              label={done
                ? t.habits.unmarkToday(habit.title)
                : t.habits.markToday(habit.title)}
              onToggle={() =>
                void habitStore.setDone(habit, today, !done, !done)}
            />
            <span class="open habit">
              <span class="title">{habit.title}</span>
              {#if habit.currentStreak > 0}
                <span class="time">{t.habits.streak(habit.currentStreak)}</span>
              {/if}
              {#if habit.time}<span class="time">{habit.time}</span>{/if}
            </span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <section class="block" aria-labelledby="today-tasks">
    <h3 id="today-tasks" class="group-label">
      {isToday ? l.tasks : l.dayTasks}
      <span class="count">{l.pending(pending)}</span>
    </h3>
    {#if dayTasks.length === 0}
      <div class="card none">
        <strong>{isToday ? l.noTasksTitle : l.noDayTitle}</strong>
        <span>{isToday ? l.noTasksDescription : l.noDayTasks}</span>
      </div>
    {:else}
      <ul class="list">
        {#each dayTasks as task (task.id)}
          <li class="row task">
            <TaskLine
              title={task.title}
              priority={task.priority}
              completed={task.isCompleted}
              checklistDone={task.checklistDone}
              checklistTotal={task.checklistTotal}
              time={task.dueTime ?? l.allDay}
              overdue={!task.isCompleted && selected < today}
              tagIds={task.tagIds}
              checkLabel={(task.isCompleted
                ? l.markOpen(task.title || t.reminders.untitled)
                : l.markDone(task.title || t.reminders.untitled)) +
                t.priority.suffix(task.priority)}
              onToggle={() =>
                void store.setCompleted(task.id, !task.isCompleted)}
              onOpen={() => onOpenTask(task.id)}
            />
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="block" aria-labelledby="today-daily">
    <h3 id="today-daily" class="group-label">{l.daily}</h3>
    <div class="card">
      <button
        type="button"
        class="open solo"
        onclick={() => onOpenDaily(selected)}
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
        <span class="title">{isToday ? l.openDaily : l.openDayNote}</span>
        {@render chevron()}
      </button>
    </div>
  </section>

  <section class="block" aria-labelledby="today-pinned">
    <h3 id="today-pinned" class="group-label">{l.pinned}</h3>
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
    gap: 22px;
  }

  /* Fourteen equal columns that shrink with the panel; it scrolls only below their minimum. */
  .strip {
    display: grid;
    grid-template-columns: repeat(14, minmax(34px, 1fr));
    gap: 2px;
    overflow-x: auto;
    margin: calc(var(--space-2) * -1) -6px 0;
    padding: 2px;
  }

  .day {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    padding: 6px 0 5px;
    border: 0;
    border-radius: 12px;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    cursor: pointer;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      box-shadow var(--motion-base) var(--ease-out),
      transform var(--motion-fast) var(--ease-spring);
  }

  .day:hover {
    background: var(--bg-hover);
  }

  .day:active {
    transform: scale(0.94);
  }

  .day:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .weekday {
    color: var(--text-secondary);
    font-size: 11px;
    text-transform: capitalize;
  }

  .number {
    font-family: var(--font-display);
    font-size: 17px;
    font-weight: 500;
    line-height: 1.25;
    font-variant-numeric: tabular-nums;
  }

  .day.is-today .number,
  .day.is-today .weekday {
    color: var(--accent);
    font-weight: 600;
  }

  .mark {
    width: 4px;
    height: 4px;
    margin-top: 1px;
    border-radius: var(--radius-chip);
  }

  .mark.shown {
    background: var(--text-secondary);
  }

  /* The chosen day sits in a raised, outlined tile, as in the reference. */
  .day.selected {
    background: var(--bg-panel);
    box-shadow:
      0 0 0 1px var(--border-subtle),
      0 1px 3px rgba(0, 0, 0, 0.06);
  }

  .day.selected .number {
    font-weight: 700;
  }

  .banner {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 10px;
    padding: 12px 14px;
    border-radius: var(--radius-panel);
    background: var(--bg-card);
    box-shadow: inset 0 0 0 1px var(--panel-border);
  }

  .banner p {
    margin: 0;
  }

  .headline {
    font-size: 14px;
    font-weight: 500;
  }

  .pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    padding: 2px 10px;
    border-radius: var(--radius-chip);
    font-size: 12px;
    font-weight: 600;
  }

  .pill.event {
    background: color-mix(in srgb, var(--danger) 13%, transparent);
    color: var(--danger);
  }

  .pill.habit {
    background: var(--bg-control-hover);
    color: var(--text-primary);
  }

  .pill.overdue {
    background: color-mix(in srgb, var(--priority-2) 15%, transparent);
    color: var(--priority-2);
  }

  .progress {
    display: flex;
    flex-basis: 100%;
    align-items: center;
    gap: var(--space-3);
  }

  .track {
    flex: 1;
    height: 4px;
    overflow: hidden;
    border-radius: var(--radius-chip);
    background: var(--bg-control-hover);
  }

  .fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--ink);
    transition: width var(--motion-spring) var(--ease-spring);
  }

  .ratio {
    flex: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .block {
    display: grid;
    align-content: start;
    gap: 4px;
  }

  /* Flat rows, as in the reference: the hover lights a rounded tile, no hairlines. */
  .list,
  .card {
    display: grid;
    gap: 1px;
    margin: 0 -10px;
    padding: 0;
    list-style: none;
  }

  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 36px;
    padding: 0 10px;
    border-radius: var(--radius-card);
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  .row.task {
    align-items: flex-start;
    padding-top: 7px;
    padding-bottom: 7px;
  }

  .row:hover,
  .open.solo:hover {
    background: var(--bg-hover);
  }

  .open {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
    min-height: 36px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: var(--text-body);
    text-align: left;
    cursor: pointer;
  }

  .open.solo {
    width: 100%;
    padding: 0 10px;
    border-radius: var(--radius-card);
    transition: background-color var(--motion-fast) var(--ease-out);
  }

  .open.stacked {
    min-height: 44px;
  }

  .open.solo:active {
    background: var(--bg-active);
  }

  .open:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    border-radius: var(--radius-card);
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
    padding: var(--space-2) 10px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .none strong {
    color: var(--text-primary);
    font-weight: 600;
  }
</style>
