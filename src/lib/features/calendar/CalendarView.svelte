<script lang="ts">
  import { onMount } from "svelte";

  import { toLocalIsoDate } from "../../dates";
  import { taskFieldsOf, type Capture } from "../../naturalInput";
  import { showDailyNote } from "../notes/navigation";
  import { t } from "../../i18n";
  import SegmentedControl from "../../shell/SegmentedControl.svelte";
  import type { EventsStore } from "../../stores/events.svelte";
  import { focusStore, type FocusStore } from "../../stores/focus.svelte";
  import { tagDrag } from "../../stores/tagDrag.svelte";
  import { NO_TAG, tagsStore } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import type { EventOccurrence, Task, ViewMode } from "../../types";
  import CalendarDay from "./CalendarDay.svelte";
  import CalendarMonth from "./CalendarMonth.svelte";
  import CalendarWeek from "./CalendarWeek.svelte";
  import { FocusHistory } from "./focusHistory.svelte";
  import { focusDays } from "./focusSlots";
  import {
    addDays,
    BLOCK_MINUTES,
    minutesToTime,
    monthGrid,
    tasksByDate,
    timeToMinutes,
    weekDates,
  } from "./grid";
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
    /** The session in progress: a change to it or to the totals reloads the history. */
    focus?: FocusStore;
    history?: FocusHistory;
  }

  let {
    store,
    eventStore,
    view,
    onOpenTask,
    onOpenEvent,
    onNewEvent,
    focus = focusStore,
    history = new FocusHistory({ onError: (error) => focus.report(error) }),
  }: Props = $props();

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
  $effect(() => {
    // Read only to depend on them: starting, pausing, ending or editing time changes one.
    void focus.current;
    void focus.totals;
    void history.show(range.from, range.to);
  });
  // The tag filter also hides the time of the tasks it hides; trashed tasks have none.
  const focused = $derived(
    focusDays(history.entries, store.visibleTasks, current),
  );
  const moveEvent = (
    occurrence: EventOccurrence,
    date: string,
    start: number,
  ) => void eventStore.move(occurrence, date, start);
  const resizeEvent = (occurrence: EventOccurrence, end: number) =>
    void eventStore.resize(occurrence, end);

  const moveTask = (task: Task, date: string, start: number) =>
    void store.schedule(
      task.id,
      date,
      minutesToTime(start),
      task.durationMinutes ?? BLOCK_MINUTES,
    );
  const resizeTask = (task: Task, end: number) =>
    void store.setDuration(
      task.id,
      end - timeToMinutes(task.dueTime ?? "00:00"),
    );

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
    capture?: Capture,
  ) =>
    capture
      ? store.createDetailed({
          title,
          ...taskFieldsOf(capture),
          dueDate: date,
          dueTime: time,
          ...(duration ? { durationMinutes: duration } : {}),
        })
      : store.createScheduled(title, date, time, duration);
</script>

<section class="calendar">
  <header class="nav view-head">
    <div class="view-heading">
      <h2 class="view-title">{heading}</h2>
    </div>
    <div class="view-actions">
      <SegmentedControl
        label={t.views.label}
        options={SCALES}
        value={view.mode}
        onChange={(mode) => view.setMode(mode)}
      />
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
      <span class="gap" aria-hidden="true"></span>
      <button type="button" class="btn small" onclick={() => view.goToToday()}>
        {t.calendar.today}
      </button>
      <button
        type="button"
        class="icon-btn filled"
        aria-label={t.calendar.previous}
        title={t.calendar.previous}
        onclick={() => view.shift(-1)}
        ><svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"
          ><path
            d="M10 3.5L5.5 8l4.5 4.5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          /></svg
        ></button
      >
      <button
        type="button"
        class="icon-btn filled"
        aria-label={t.calendar.next}
        title={t.calendar.next}
        onclick={() => view.shift(1)}
        ><svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"
          ><path
            d="M6 3.5L10.5 8 6 12.5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          /></svg
        ></button
      >
    </div>
  </header>

  <div class="surface">
    {#if view.mode === "month"}
      <CalendarMonth
        anchor={view.anchor}
        firstDay={view.firstDay}
        {today}
        tasksByDate={byDate}
        eventsByDate={eventStore.byDate}
        focusSeconds={focused.seconds}
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
        focusByDate={focused.slots}
        label={t.calendar.weekGrid}
        onCreate={create}
        {onOpenTask}
        {onOpenEvent}
        onMoveEvent={moveEvent}
        onResizeEvent={resizeEvent}
        onMoveTask={moveTask}
        onResizeTask={resizeTask}
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
        focusByDate={focused.slots}
        label={t.calendar.dayGrid}
        onCreate={create}
        {onOpenTask}
        {onOpenEvent}
        onMoveEvent={moveEvent}
        onResizeEvent={resizeEvent}
        onMoveTask={moveTask}
        onResizeTask={resizeTask}
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
    align-items: center;
    padding: var(--space-1) var(--space-2) var(--space-4) 12px;
  }

  .nav .view-actions {
    flex-wrap: wrap;
    justify-content: flex-end;
    padding-top: 0;
  }

  .view-title::first-letter {
    text-transform: uppercase;
  }

  .gap {
    width: 4px;
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
