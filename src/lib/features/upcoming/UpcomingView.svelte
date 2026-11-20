<script lang="ts">
  import { t } from "../../i18n";
  import EmptyState from "../../shell/EmptyState.svelte";
  import type { EventsStore } from "../../stores/events.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import type { EventOccurrence, Task } from "../../types";
  import { addDays } from "../calendar/grid";
  import EventRow from "../events/EventRow.svelte";
  import Checkbox from "../todo/Checkbox.svelte";
  import ChecklistProgress from "../todo/ChecklistProgress.svelte";
  import { UPCOMING_DAYS, upcomingGroups } from "./upcoming";

  // «Próximas» (F20-05): what is still to do, overdue first, then day by day for a month and
  // the rest under «Más adelante».
  interface Props {
    store: TasksStore;
    /** The events of the coming days; without it «Próximas» shows tasks only. */
    eventStore?: EventsStore;
    view: ViewStore;
    onOpenTask: (taskId: string) => void;
    onOpenEvent?: (eventId: string) => void;
  }

  let { store, eventStore, view, onOpenTask, onOpenEvent }: Props = $props();
  const l = t.upcoming;

  // A window left open overnight must not keep yesterday as today.
  // svelte-ignore state_referenced_locally
  let today = $state(view.today());

  const longDate = new Intl.DateTimeFormat("es-ES", {
    weekday: "long",
    day: "numeric",
    month: "long",
  });
  const shortDate = new Intl.DateTimeFormat("es-ES", {
    day: "numeric",
    month: "short",
  });
  const asDate = (iso: string) => new Date(`${iso}T12:00:00`);
  const capitalize = (text: string) =>
    text.charAt(0).toUpperCase() + text.slice(1);

  $effect(() => {
    void eventStore?.show(today, addDays(today, UPCOMING_DAYS - 1));
  });

  const groups = $derived(upcomingGroups(store.visibleTasks, today));
  // A day with events but no tasks gets a heading too.
  const dayList = $derived.by(() => {
    const withTasks = groups.days.map((day) => day.date);
    const onlyEvents = [...(eventStore?.byDate.keys() ?? [])].filter(
      (date) => !withTasks.includes(date),
    );
    return [...withTasks, ...onlyEvents]
      .toSorted((x, y) => (x < y ? -1 : 1))
      .map((date) => ({
        date,
        tasks: groups.days.find((day) => day.date === date)?.tasks ?? [],
        occurrences: eventStore?.byDate.get(date) ?? [],
      }));
  });
  const empty = $derived(
    groups.overdue.length + dayList.length + groups.later.length === 0,
  );
  const days = $derived(
    dayList.map((day) => ({
      ...day,
      heading:
        day.date === today
          ? l.today
          : day.date === addDays(today, 1)
            ? l.tomorrow
            : capitalize(longDate.format(asDate(day.date))),
    })),
  );
</script>

<svelte:window onfocus={() => (today = view.today())} />

<div class="upcoming">
  <h2>{l.title}</h2>

  {#if empty}
    <EmptyState title={l.emptyTitle} description={l.emptyDescription} />
  {:else}
    {#if groups.overdue.length > 0}
      {@render group("upcoming-overdue", l.overdue, groups.overdue, true)}
    {/if}
    {#each days as day (day.date)}
      {@render group(
        `upcoming-${day.date}`,
        day.heading,
        day.tasks,
        false,
        day.occurrences,
      )}
    {/each}
    {#if groups.later.length > 0}
      {@render group("upcoming-later", l.later, groups.later, true)}
    {/if}
  {/if}
</div>

{#snippet group(
  id: string,
  heading: string,
  tasks: Task[],
  dated: boolean,
  occurrences: EventOccurrence[] = [],
)}
  <section class="block" aria-labelledby={id}>
    <h3 {id}>
      {heading}
      <span class="count">{tasks.length + occurrences.length}</span>
    </h3>
    <ul class="card">
      {#each occurrences as occurrence (occurrence.eventId + occurrence.startDate)}
        <EventRow {occurrence} onOpen={(eventId) => onOpenEvent?.(eventId)} />
      {/each}
      {#each tasks as task (task.id)}
        <li class="row" data-task-id={task.id}>
          <Checkbox
            checked={false}
            label={t.today.markDone(task.title || t.reminders.untitled) +
              t.priority.suffix(task.priority)}
            priority={task.priority}
            onToggle={() => void store.setCompleted(task.id, true)}
          />
          <button
            type="button"
            class="open"
            onclick={() => onOpenTask(task.id)}
          >
            <span class="title">{task.title || t.reminders.untitled}</span>
            {#if task.checklistTotal > 0}
              <ChecklistProgress
                done={task.checklistDone}
                total={task.checklistTotal}
              />
            {/if}
            <span class="when">
              {#if dated && task.dueDate}
                {shortDate.format(asDate(task.dueDate))}
              {/if}
              {task.dueTime ?? (dated ? "" : l.allDay)}
            </span>
          </button>
        </li>
      {/each}
    </ul>
  </section>
{/snippet}

<style>
  .upcoming {
    display: grid;
    align-content: start;
    gap: var(--space-6);
    box-sizing: border-box;
    width: 100%;
    max-width: 680px;
    margin: 0 auto;
    padding: var(--space-4) var(--space-6) var(--space-8);
  }

  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 34px;
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: -0.025em;
  }

  .block {
    display: grid;
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
    transition: background-color var(--motion-instant) linear;
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

  .row:hover {
    background: var(--bg-hover);
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
  }

  .open:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    border-radius: var(--radius-control);
  }

  .title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    font-weight: 500;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .when {
    flex: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }
</style>
