<script lang="ts">
  import { t } from "../../i18n";
  import { sortDay } from "../../ordering";
  import type { Task } from "../../types";
  import { isSameMonth, monthGrid, type WeekDay } from "./grid";
  import { dayNumber, weekdayName } from "./labels";

  /** Chips shown in a day cell before the "more" link. */
  const VISIBLE_TASKS = 3;

  interface Props {
    anchor: string;
    firstDay: WeekDay;
    today: string;
    tasksByDate: Map<string, Task[]>;
    label: string;
    onShowDay: (date: string) => void;
    onOpenTask: (task: Task) => void;
  }

  let {
    anchor,
    firstDay,
    today,
    tasksByDate,
    label,
    onShowDay,
    onOpenTask,
  }: Props = $props();

  const weeks = $derived(
    monthGrid(anchor, firstDay).map((week) =>
      week.map((date) => {
        const tasks = sortDay(tasksByDate.get(date) ?? []);
        return {
          date,
          outside: !isSameMonth(date, anchor),
          isToday: date === today,
          shown: tasks.slice(0, VISIBLE_TASKS),
          hidden: Math.max(tasks.length - VISIBLE_TASKS, 0),
        };
      }),
    ),
  );
</script>

<div class="month" role="grid" aria-label={label} style:--rows={weeks.length}>
  <div class="row" role="row">
    {#each weeks[0] as day (day.date)}
      <div class="weekday" role="columnheader">{weekdayName(day.date)}</div>
    {/each}
  </div>
  {#each weeks as week (week[0].date)}
    <div class="row" role="row">
      {#each week as day (day.date)}
        <div
          class="cell"
          class:outside={day.outside}
          class:today={day.isToday}
          role="gridcell"
          data-date={day.date}
        >
          <button
            type="button"
            class="number"
            aria-label={t.calendar.goToDay(day.date)}
            onclick={() => onShowDay(day.date)}
          >
            {dayNumber(day.date)}
          </button>
          {#each day.shown as task (task.id)}
            <button
              type="button"
              class="chip"
              class:done={task.isCompleted}
              title={t.calendar.openTask(task.title || t.calendar.untitled)}
              onclick={() => onOpenTask(task)}
            >
              {#if task.dueTime}<span class="time">{task.dueTime}</span>{/if}
              {task.title || t.calendar.untitled}
            </button>
          {/each}
          {#if day.hidden > 0}
            <button
              type="button"
              class="more"
              onclick={() => onShowDay(day.date)}
            >
              {t.calendar.more(day.hidden)}
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/each}
</div>

<style>
  .month {
    display: grid;
    grid-template-rows: auto repeat(var(--rows), minmax(96px, 1fr));
    height: 100%;
    min-height: 0;
  }

  .row {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
  }

  .weekday {
    padding: var(--space-2) 0;
    color: var(--text-secondary);
    font-size: 12px;
    text-align: center;
  }

  .cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding: 4px;
    border-top: 1px solid var(--border-subtle);
    border-left: 1px solid var(--border-subtle);
  }

  .cell.outside {
    opacity: 0.5;
  }

  .number {
    align-self: flex-start;
    min-width: 22px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .number:hover {
    background: var(--bg-hover);
  }

  .cell.today .number {
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 600;
  }

  .chip {
    padding: 1px 6px;
    border: 0;
    border-radius: 4px;
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip.done {
    opacity: 0.55;
    text-decoration: line-through;
  }

  .time {
    color: var(--text-secondary);
    margin-right: 4px;
  }

  .more {
    align-self: flex-start;
    padding: 0 6px;
    border: 0;
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }

  .more:hover {
    color: var(--text-primary);
  }

  .number:focus-visible,
  .chip:focus-visible,
  .more:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
