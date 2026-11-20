<script lang="ts">
  import { onMount } from "svelte";

  import { toLocalIsoDate } from "../../dates";
  import { showDailyNote } from "../notes/navigation";
  import { t } from "../../i18n";
  import SegmentedControl from "../../shell/SegmentedControl.svelte";
  import type { EventsStore } from "../../stores/events.svelte";
  import { tagDrag } from "../../stores/tagDrag.svelte";
  import { NO_TAG, tagsStore } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import type { EventOccurrence, Task, ViewMode } from "../../types";
  import CalendarDay from "./CalendarDay.svelte";
  import CalendarMonth from "./CalendarMonth.svelte";
  import CalendarWeek from "./CalendarWeek.svelte";
  import { addDays, monthGrid, tasksByDate, weekDates } from "./grid";
  import { periodLabel } from "./labels";

  interface Props {
    store: TasksStore;
    eventStore: EventsStore;
    view: ViewStore;
    /** Leaves the calendar and shows the task in the list. */
    onOpenTask: (task: Task) => void;
    onOpenEvent: (eventId: string) => void;
    /** Opens the dialog of a new event on a day, at a time if one is given. */
    onNewEvent: (date: string, time: string | null) => void;
  }

  let { store, eventStore, view, onOpenTask, onOpenEvent, onNewEvent }: Props =
    $props();

  const SCALES: { value: ViewMode; label: string }[] = [
    { value: "day", label: t.views.day },
    { value: "week", label: t.views.week },
    { value: "month", label: t.views.month },
  ];

  const byDate = $derived(tasksByDate(store.visibleTasks));
  const heading = $derived(periodLabel(view.mode, view.anchor, view.firstDay));
  // The clock ticks every 30 s so the "now" line moves and midnight changes "today".
  let current = $state.raw(new Date());
  const today = $derived(toLocalIsoDate(current));
  const nowMinutes = $derived(current.getHours() * 60 + current.getMinutes());

  const minuteOf = (date: Date) => Math.floor(date.getTime() / 60_000);

  onMount(() => {
    const tick = () => {
      const next = view.now();
      // Same minute: keep the old reading so nothing derived from the clock recomputes.
      if (minuteOf(next) !== minuteOf(current)) {
        current = next;
      }
    };
    current = view.now();
    const timer = setInterval(tick, 30_000);
    return () => clearInterval(timer);
  });
  const week = $derived(weekDates(view.anchor, view.firstDay));

  // The events of the days on screen; the month also shows the days of its neighbours.
  const range = $derived.by(() => {
    if (view.mode === "day") return { from: view.anchor, to: view.anchor };
    if (view.mode === "week") return { from: week[0], to: week[6] };
    const grid = monthGrid(view.anchor, view.firstDay);
    return { from: grid[0][0], to: addDays(grid[grid.length - 1][0], 6) };
  });
  $effect(() => {
    void eventStore.show(range.from, range.to);
  });
  const moveEvent = (
    occurrence: EventOccurrence,
    date: string,
    start: number,
  ) => void eventStore.move(occurrence, date, start);
  const resizeEvent = (occurrence: EventOccurrence, end: number) =>
    void eventStore.resize(occurrence, end);

  const tagOf = (item: Pick<Task, "tagIds">) => tagsStore.tagOf(item);
  const moreTags = (task: Task) =>
    Math.max(0, tagsStore.tagsOf(task).length - 1);
  const dragTask = (event: PointerEvent, task: Task) =>
    tagDrag.begin(event, { id: task.id, title: task.title }, (key) =>
      key === NO_TAG ? store.setTags(task.id, []) : store.addTag(task.id, key),
    );
  const create = (
    title: string,
    date: string,
    time: string,
    duration?: number,
  ) => store.createScheduled(title, date, time, duration);
</script>

<section class="calendar">
  <div class="nav">
    <div class="title">
      <SegmentedControl
        label={t.views.label}
        options={SCALES}
        value={view.mode}
        onChange={(mode) => view.setMode(mode)}
      />
      <h2>{heading}</h2>
    </div>
    <div class="buttons">
      <button
        type="button"
        class="btn small"
        onclick={() => void showDailyNote(view.anchor)}
      >
        {t.calendar.dailyNote}
      </button>
      <button
        type="button"
        class="btn small"
        onclick={() => onNewEvent(view.anchor, null)}
      >
        {t.events.newEvent}
      </button>
      <button type="button" class="btn small" onclick={() => view.shift(-1)}>
        {t.calendar.previous}
      </button>
      <button type="button" class="btn small" onclick={() => view.goToToday()}>
        {t.calendar.today}
      </button>
      <button type="button" class="btn small" onclick={() => view.shift(1)}>
        {t.calendar.next}
      </button>
    </div>
  </div>

  <div class="surface">
    {#if view.mode === "month"}
      <CalendarMonth
        anchor={view.anchor}
        firstDay={view.firstDay}
        {today}
        tasksByDate={byDate}
        eventsByDate={eventStore.byDate}
        label={t.calendar.monthGrid}
        onShowDay={(date) => view.showDay(date)}
        {onOpenTask}
        {onOpenEvent}
        {tagOf}
        {moreTags}
        {dragTask}
      />
    {:else if view.mode === "week"}
      <CalendarWeek
        dates={week}
        {today}
        {nowMinutes}
        tasksByDate={byDate}
        eventsByDate={eventStore.byDate}
        label={t.calendar.weekGrid}
        onCreate={create}
        {onOpenTask}
        {onOpenEvent}
        onMoveEvent={moveEvent}
        onResizeEvent={resizeEvent}
        {tagOf}
        {moreTags}
        {dragTask}
      />
    {:else}
      <CalendarDay
        date={view.anchor}
        {today}
        {nowMinutes}
        tasksByDate={byDate}
        eventsByDate={eventStore.byDate}
        label={t.calendar.dayGrid}
        onCreate={create}
        {onOpenTask}
        {onOpenEvent}
        onMoveEvent={moveEvent}
        onResizeEvent={resizeEvent}
        {tagOf}
        {moreTags}
        {dragTask}
      />
    {/if}
  </div>
</section>

<style>
  .calendar {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding: 0 var(--space-4) var(--space-4);
  }

  .nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-1) var(--space-1) var(--space-3);
  }

  .title {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    min-width: 0;
  }

  h2 {
    margin: 0;
    overflow: hidden;
    color: var(--text-primary);
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  h2::first-letter {
    text-transform: uppercase;
  }

  .buttons {
    display: flex;
    flex: none;
    gap: var(--space-1);
  }

  /* The grid sits on its own card, clipped to the rounded corners. */
  .surface {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    border: 1px solid var(--separator);
    border-radius: var(--radius-card);
  }
</style>
