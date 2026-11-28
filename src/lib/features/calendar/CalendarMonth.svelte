<script lang="ts">
  import { t } from "../../i18n";
  import { sortDay } from "../../ordering";
  import { spentLabel } from "../../schedule";
  import type { EventOccurrence, Tag, Task } from "../../types";
  import { timeRangeOf } from "../events/occurrences";
  import { isSameMonth, monthGrid, type WeekDay } from "./grid";
  import { dayNumber, weekdayName } from "./labels";

  /** Chips shown in a day cell before the "more" link. */
  const VISIBLE_TASKS = 3;

  interface Props {
    anchor: string;
    firstDay: WeekDay;
    today: string;
    tasksByDate: Map<string, Task[]>;
    /** The events of each day, shown before its tasks. */
    eventsByDate?: Map<string, EventOccurrence[]>;
    /** Seconds of focus of each day (D-102). */
    focusSeconds?: Map<string, number>;
    label: string;
    onShowDay: (date: string) => void;
    onOpenTask: (task: Task) => void;
    onOpenEvent?: (eventId: string) => void;
    /** The task's tag, which gives its colour and the small name on the event. */
    tagOf?: (item: Pick<Task, "tagIds">) => Tag | undefined;
    /** How many more tags the task has besides the one shown. */
    moreTags?: (task: Task) => number;
    /** Pressed on an event: the parent starts dragging it towards a sidebar tag. */
    dragTask?: (event: PointerEvent, task: Task) => void;
  }

  let {
    anchor,
    firstDay,
    today,
    tasksByDate,
    eventsByDate,
    focusSeconds,
    label,
    onShowDay,
    onOpenTask,
    onOpenEvent,
    tagOf,
    moreTags,
    dragTask,
  }: Props = $props();

  const weeks = $derived(
    monthGrid(anchor, firstDay).map((week) =>
      week.map((date) => {
        const entries = [
          ...(eventsByDate?.get(date) ?? []).map((occurrence) => ({
            key: `e:${occurrence.eventId}:${occurrence.startDate}`,
            occurrence,
            task: null,
          })),
          ...sortDay(tasksByDate.get(date) ?? []).map((task) => ({
            key: `t:${task.id}`,
            occurrence: null,
            task,
          })),
        ];
        return {
          date,
          outside: !isSameMonth(date, anchor),
          isToday: date === today,
          shown: entries.slice(0, VISIBLE_TASKS),
          hidden: Math.max(entries.length - VISIBLE_TASKS, 0),
          focus: focusSeconds?.get(date) ?? 0,
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
          {#each day.shown as entry (entry.key)}
            {#if entry.occurrence}
              {@const occurrence = entry.occurrence}
              {@const tag = tagOf?.(occurrence)}
              {@const name = occurrence.title || t.calendar.untitled}
              <button
                type="button"
                class="chip event"
                data-event-id={occurrence.eventId}
                style:--tag={tag?.colorHex}
                title={t.calendar.openEvent(name)}
                onclick={() => onOpenEvent?.(occurrence.eventId)}
              >
                {#if timeRangeOf(occurrence)}<span class="time"
                    >{occurrence.startTime}</span
                  >{/if}
                <span class="title">{name}</span>
              </button>
            {:else if entry.task}
              {@const task = entry.task}
              {@const tag = tagOf?.(task)}
              <button
                type="button"
                class="chip"
                data-task-id={task.id}
                class:done={task.isCompleted}
                style:--tag={tag?.colorHex}
                onpointerdown={(event) => dragTask?.(event, task)}
                title={t.calendar.openTask(task.title || t.calendar.untitled)}
                onclick={() => onOpenTask(task)}
              >
                {#if task.dueTime}<span class="time">{task.dueTime}</span>{/if}
                <span class="title">{task.title || t.calendar.untitled}</span>
                {#if tag}<span class="tag-name"
                    >{tag.name}{#if moreTags?.(task)}
                      +{moreTags(task)}{/if}</span
                  >{/if}
              </button>
            {/if}
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
          {#if day.focus > 0}
            <button
              type="button"
              class="focus-total"
              data-focus-total
              onclick={() => onShowDay(day.date)}
            >
              {t.calendar.focusDay(spentLabel(day.focus))}
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
    /* Plain 1fr rows grow to their content (the cell's own min-height sets the floor), so a
       full day never spills over the week below; a short window scrolls the month instead. */
    grid-template-rows: auto repeat(var(--rows), 1fr);
    height: 100%;
    min-height: 0;
    overflow-y: auto;
    --tag-fallback: var(--text-secondary);
  }

  .row {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
  }

  .weekday {
    min-width: 0;
    padding: var(--space-2) var(--space-1);
    overflow: hidden;
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-align: center;
    text-overflow: ellipsis;
    text-transform: uppercase;
    white-space: nowrap;
  }

  .cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    min-height: 96px;
    box-sizing: border-box;
    padding: var(--space-1);
    border-top: 1px solid var(--separator);
    border-left: 1px solid var(--separator);
  }

  .cell:first-child {
    border-left: 0;
  }

  .cell.outside {
    opacity: 0.62;
  }

  .number {
    display: grid;
    place-items: center;
    align-self: flex-end;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    cursor: pointer;
    transition:
      background-color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .number:hover {
    background: var(--bg-control-hover);
  }

  .number:active {
    transform: scale(0.9);
  }

  .cell.today .number {
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 600;
  }

  .chip {
    flex: none;
    padding: 2px 6px;
    border: 0;
    border-radius: var(--radius-small);
    border-left: 3px solid var(--tag, var(--tag-fallback));
    background: color-mix(
      in srgb,
      var(--tag, var(--tag-fallback)) 14%,
      var(--bg-card)
    );
    color: var(--text-primary);
    font: inherit;
    font-size: var(--text-caption);
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: flex;
    align-items: baseline;
    min-width: 0;
    transition:
      background-color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .chip .title {
    flex: 1 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .chip .tag-name {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .chip:hover {
    background: color-mix(
      in srgb,
      var(--tag, var(--tag-fallback)) 24%,
      var(--bg-card)
    );
  }

  .chip:active {
    transform: scale(0.98);
  }

  .chip.event {
    background: color-mix(
      in srgb,
      var(--tag, var(--tag-fallback)) 22%,
      var(--bg-card)
    );
    font-weight: 600;
  }

  .chip.done {
    opacity: 0.72;
    text-decoration: line-through;
  }

  .time {
    flex: none;
    color: var(--text-secondary);
    margin-right: 4px;
    font-variant-numeric: tabular-nums;
  }

  .tag-name {
    margin-left: 4px;
    padding: 0 5px;
    border-radius: var(--radius-chip);
    background: color-mix(in srgb, var(--tag) 28%, transparent);
    color: color-mix(in srgb, var(--tag) 45%, var(--text-primary));
    font-size: 10px;
    font-weight: 600;
  }

  .more {
    align-self: flex-start;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-small);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    cursor: pointer;
    transition: background-color var(--motion-instant) linear;
  }

  .more:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .focus-total {
    flex: none;
    align-self: flex-start;
    max-width: 100%;
    margin-top: auto;
    padding: 0 6px;
    overflow: hidden;
    border: 1px solid color-mix(in srgb, var(--tag-fallback) 45%, transparent);
    border-radius: var(--radius-small);
    background: repeating-linear-gradient(
      135deg,
      color-mix(in srgb, var(--tag-fallback) 24%, var(--bg-card)) 0 4px,
      color-mix(in srgb, var(--tag-fallback) 5%, var(--bg-card)) 4px 10px
    );
    color: var(--text-primary);
    font: inherit;
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: pointer;
  }

  .number:focus-visible,
  .focus-total:focus-visible,
  .chip:focus-visible,
  .more:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
