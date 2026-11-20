<script lang="ts">
  import { t } from "../../i18n";
  import EmptyState from "../../shell/EmptyState.svelte";
  import type { EventsStore } from "../../stores/events.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import type { EventOccurrence, Task } from "../../types";
  import { addDays } from "../calendar/grid";
  import EventRow from "../events/EventRow.svelte";
  import TaskLine from "../todo/TaskLine.svelte";
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
  const subtitle = $derived(l.horizon(UPCOMING_DAYS));
  const empty = $derived(
    groups.overdue.length + dayList.length + groups.later.length === 0,
  );
  const days = $derived(
    dayList.map((day) => ({
      ...day,
      sub:
        day.date === today || day.date === addDays(today, 1)
          ? shortDate.format(asDate(day.date))
          : "",
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

<div class="upcoming view">
  <header class="view-head">
    <div class="view-heading">
      <h2 class="view-title">{l.title}</h2>
      <p class="view-sub">{subtitle}</p>
    </div>
  </header>

  {#if empty}
    <EmptyState title={l.emptyTitle} description={l.emptyDescription} />
  {:else}
    {#if groups.overdue.length > 0}
      {@render group(
        "upcoming-overdue",
        l.overdue,
        String(groups.overdue.length),
        groups.overdue,
        true,
      )}
    {/if}
    {#each days as day (day.date)}
      {@render group(
        `upcoming-${day.date}`,
        day.heading,
        day.sub,
        day.tasks,
        false,
        day.occurrences,
        day.date === today,
      )}
    {/each}
    {#if groups.later.length > 0}
      {@render group(
        "upcoming-later",
        l.later,
        String(groups.later.length),
        groups.later,
        true,
      )}
    {/if}
  {/if}
</div>

{#snippet group(
  id: string,
  heading: string,
  sub: string,
  tasks: Task[],
  dated: boolean,
  occurrences: EventOccurrence[] = [],
  current = false,
)}
  <section class="block" aria-labelledby={id}>
    <h3 {id} class:current>
      {heading}
      {#if sub}<span class="sub">{sub}</span>{/if}
    </h3>
    <ul class="cards">
      {#each occurrences as occurrence (occurrence.eventId + occurrence.startDate)}
        <EventRow {occurrence} onOpen={(eventId) => onOpenEvent?.(eventId)} />
      {/each}
      {#each tasks as task (task.id)}
        <li class="row" data-task-id={task.id}>
          <TaskLine
            title={task.title}
            priority={task.priority}
            checklistDone={task.checklistDone}
            checklistTotal={task.checklistTotal}
            dateLabel={dated && task.dueDate
              ? shortDate.format(asDate(task.dueDate))
              : ""}
            time={task.dueTime ?? (dated ? null : l.allDay)}
            overdue={dated && !!task.dueDate && task.dueDate < today}
            tagIds={task.tagIds}
            checkLabel={t.today.markDone(task.title || t.reminders.untitled) +
              t.priority.suffix(task.priority)}
            onToggle={() => void store.setCompleted(task.id, true)}
            onOpen={() => onOpenTask(task.id)}
          />
        </li>
      {/each}
    </ul>
  </section>
{/snippet}

<style>
  /* Each day is a row of two columns, as in the reference: its name on the left and its items,
     one card each, on the right. Below 560 px the name goes on top. */
  .upcoming {
    gap: 28px;
    container-type: inline-size;
  }

  .block {
    display: grid;
    grid-template-columns: 132px minmax(0, 1fr);
    align-items: start;
    gap: var(--space-3);
  }

  @container (max-width: 560px) {
    .block {
      grid-template-columns: minmax(0, 1fr);
      gap: 6px;
    }
  }

  h3 {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
    padding-top: 6px;
    font-family: var(--font-display);
    font-size: 17px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  h3.current {
    color: var(--accent);
  }

  .sub {
    color: var(--text-secondary);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 400;
    letter-spacing: 0;
    font-variant-numeric: tabular-nums;
  }

  .sub::first-letter {
    text-transform: uppercase;
  }

  .cards {
    display: grid;
    gap: 6px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .cards > :global(li) {
    border-radius: var(--radius-card);
    background: var(--bg-card);
    transition:
      background-color var(--motion-fast) var(--ease-out),
      transform var(--motion-fast) var(--ease-spring);
  }

  .cards > :global(li:hover) {
    background: var(--bg-card-hover);
  }

  .row {
    display: flex;
    padding: 8px 10px;
  }
</style>
