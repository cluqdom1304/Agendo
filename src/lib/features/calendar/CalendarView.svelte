<script lang="ts">
  import { onMount } from "svelte";

  import { toLocalIsoDate } from "../../dates";
  import { t } from "../../i18n";
  import SegmentedControl from "../../shell/SegmentedControl.svelte";
  import { tagDrag } from "../../stores/tagDrag.svelte";
  import { NO_TAG, tagsStore } from "../../stores/tags.svelte";
  import type { TasksStore } from "../../stores/tasks.svelte";
  import type { ViewStore } from "../../stores/view.svelte";
  import type { Task, ViewMode } from "../../types";
  import CalendarDay from "./CalendarDay.svelte";
  import CalendarMonth from "./CalendarMonth.svelte";
  import CalendarWeek from "./CalendarWeek.svelte";
  import { tasksByDate, weekDates } from "./grid";
  import { periodLabel } from "./labels";

  interface Props {
    store: TasksStore;
    view: ViewStore;
    /** Leaves the calendar and shows the task in the list. */
    onOpenTask: (task: Task) => void;
  }

  let { store, view, onOpenTask }: Props = $props();

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

  onMount(() => {
    current = view.now();
    const timer = setInterval(() => (current = view.now()), 30_000);
    return () => clearInterval(timer);
  });
  const week = $derived(weekDates(view.anchor, view.firstDay));

  const tagOf = (task: Task) => tagsStore.tagOf(task);
  const dragTask = (event: PointerEvent, task: Task) =>
    tagDrag.begin(event, { id: task.id, title: task.title }, (key) =>
      store.setTag(task.id, key === NO_TAG ? null : key),
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
        label={t.calendar.monthGrid}
        onShowDay={(date) => view.showDay(date)}
        {onOpenTask}
        {tagOf}
        {dragTask}
      />
    {:else if view.mode === "week"}
      <CalendarWeek
        dates={week}
        {today}
        {nowMinutes}
        tasksByDate={byDate}
        label={t.calendar.weekGrid}
        onCreate={create}
        {onOpenTask}
        {tagOf}
        {dragTask}
      />
    {:else}
      <CalendarDay
        date={view.anchor}
        {today}
        {nowMinutes}
        tasksByDate={byDate}
        label={t.calendar.dayGrid}
        onCreate={create}
        {onOpenTask}
        {tagOf}
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
