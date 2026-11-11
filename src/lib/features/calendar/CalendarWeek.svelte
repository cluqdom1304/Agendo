<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import { timeRange } from "../../schedule";
  import type { Tag, Task } from "../../types";
  import {
    BLOCK_MINUTES,
    SNAP_MINUTES,
    layoutDay,
    minutesToTime,
    snapSpan,
  } from "./grid";
  import { dayNumber, weekdayName } from "./labels";

  /** Pixels per hour. The whole day is 24 of them; the grid scrolls. */
  const HOUR_PX = 48;
  const FIRST_VISIBLE_HOUR = 7;
  /** Where a task made from the keyboard on a day other than today starts. */
  const DEFAULT_START_MINUTES = 9 * 60;

  interface Props {
    /** The columns: seven dates for a week, one for a day. */
    dates: string[];
    today: string;
    /** Minutes since midnight of the current time, for the "now" line. */
    nowMinutes: number;
    tasksByDate: Map<string, Task[]>;
    label: string;
    /** `duration` is only given when the span was dragged out (D-33). */
    onCreate: (
      title: string,
      date: string,
      time: string,
      duration?: number,
    ) => Promise<boolean>;
    onOpenTask: (task: Task) => void;
    /** The task's tag, which gives its colour and the small name on the event. */
    tagOf?: (task: Task) => Tag | undefined;
    /** Pressed on an event: the parent starts dragging it towards a sidebar tag. */
    dragTask?: (event: PointerEvent, task: Task) => void;
  }

  let {
    dates,
    today,
    nowMinutes,
    tasksByDate,
    label,
    tagOf,
    dragTask,
    onCreate,
    onOpenTask,
  }: Props = $props();

  interface Drag {
    date: string;
    anchor: number;
    current: number;
  }
  interface Compose {
    date: string;
    start: number;
    end: number;
    /** Dragged over more than one step: the length is the task's duration. */
    sized: boolean;
  }

  let scroller: HTMLElement | undefined = $state();
  // The body scrolls and its scrollbar narrows the columns; the fixed rows above get the same
  // width taken away so every column line stays on the same x.
  let scrollbarWidth = $state(0);

  function measureScrollbar() {
    if (scroller) {
      scrollbarWidth = Math.max(0, scroller.offsetWidth - scroller.clientWidth);
    }
  }
  let drag = $state.raw<Drag | null>(null);
  let compose = $state.raw<Compose | null>(null);
  let title = $state("");
  let dragColumn: HTMLElement | null = null;
  let committing = false;

  const showNow = $derived(dates.includes(today));
  const dragSpan = $derived(
    drag ? { date: drag.date, ...snapSpan(drag.anchor, drag.current) } : null,
  );
  const selection = $derived(dragSpan ?? compose);

  const columns = $derived(
    dates.map((date) => {
      const tasks = tasksByDate.get(date) ?? [];
      return {
        date,
        isToday: date === today,
        // The line starts on today's column and runs to the end of the week; the days
        // before it only get a faint trace.
        nowLine: today < date ? "full" : date === today ? "start" : "faint",
        allDay: tasks.filter((task) => task.dueTime === null),
        placed: layoutDay(tasks),
      };
    }),
  );

  onMount(() => {
    if (scroller) scroller.scrollTop = FIRST_VISIBLE_HOUR * HOUR_PX;
    measureScrollbar();
  });

  const minutesAt = (event: MouseEvent, element: HTMLElement) =>
    ((event.clientY - element.getBoundingClientRect().top) / HOUR_PX) * 60;

  const top = (minutes: number) => (minutes / 60) * HOUR_PX;

  function onPointerDown(event: PointerEvent, date: string) {
    if (event.button !== 0) return;
    const target = event.target;
    if (
      target instanceof Element &&
      target.closest("[data-block], [data-quick-create]")
    ) {
      return;
    }
    const element = event.currentTarget;
    if (!(element instanceof HTMLElement)) return;
    dragColumn = element;
    const minutes = minutesAt(event, element);
    compose = null;
    drag = { date, anchor: minutes, current: minutes };
    event.preventDefault();
  }

  function onPointerMove(event: PointerEvent) {
    if (!drag || !dragColumn) return;
    drag = { ...drag, current: minutesAt(event, dragColumn) };
  }

  function onPointerUp(event: PointerEvent) {
    if (!drag || !dragColumn) return;
    const finished = { ...drag, current: minutesAt(event, dragColumn) };
    drag = null;
    dragColumn = null;
    const span = snapSpan(finished.anchor, finished.current);
    compose = {
      date: finished.date,
      ...span,
      sized: span.end - span.start > SNAP_MINUTES,
    };
    title = "";
  }

  /** Keyboard route to what a drag does: Enter on a day column opens the quick title. */
  function onColumnKeydown(event: KeyboardEvent, date: string) {
    if (event.target !== event.currentTarget) return;
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    const start =
      date === today
        ? Math.min(
            Math.ceil(nowMinutes / BLOCK_MINUTES) * BLOCK_MINUTES,
            24 * 60 - BLOCK_MINUTES,
          )
        : DEFAULT_START_MINUTES;
    drag = null;
    compose = { date, start, end: start + BLOCK_MINUTES, sized: false };
    title = "";
  }

  function cancelCompose() {
    compose = null;
    title = "";
  }

  async function commit() {
    const pending = compose;
    const text = title.trim();
    if (!pending || committing) return;
    if (!text) {
      cancelCompose();
      return;
    }
    committing = true;
    try {
      // A dragged span becomes the duration; a plain click only picks the start (B-04, D-33).
      if (
        await onCreate(
          text,
          pending.date,
          minutesToTime(pending.start),
          pending.sized ? pending.end - pending.start : undefined,
        )
      ) {
        cancelCompose();
      }
    } finally {
      committing = false;
    }
  }

  function onQuickKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      void commit();
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      cancelCompose();
    }
  }

  function focusOnMount(node: HTMLElement) {
    node.focus();
  }

  const hours = Array.from({ length: 24 }, (_, hour) => hour);
</script>

<svelte:window
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onresize={measureScrollbar}
/>

<div
  class="grid"
  role="grid"
  aria-label={label}
  style:--columns={dates.length}
  style:--scrollbar-gap="{scrollbarWidth}px"
>
  <div class="row head" role="row">
    <div class="gutter"></div>
    {#each columns as column (column.date)}
      <div class="head-cell" class:today={column.isToday} role="columnheader">
        <span class="weekday">{weekdayName(column.date)}</span>
        <span class="number">{dayNumber(column.date)}</span>
      </div>
    {/each}
  </div>

  <div class="row allday" role="row">
    <div class="gutter">{t.calendar.allDay}</div>
    {#each columns as column (column.date)}
      <div class="allday-cell" role="gridcell">
        {#each column.allDay as task (task.id)}
          {@const tag = tagOf?.(task)}
          <button
            type="button"
            class="chip"
            class:done={task.isCompleted}
            style:--tag={tag?.colorHex}
            onpointerdown={(event) => dragTask?.(event, task)}
            title={t.calendar.openTask(task.title || t.calendar.untitled)}
            onclick={() => onOpenTask(task)}
          >
            {task.title || t.calendar.untitled}
            {#if tag}<span class="tag-name">{tag.name}</span>{/if}
          </button>
        {/each}
      </div>
    {/each}
  </div>

  <div class="scroller" bind:this={scroller}>
    <div class="body" style:height="{24 * HOUR_PX}px">
      <div class="gutter hours">
        {#each hours as hour (hour)}
          <span style:top="{top(hour * 60)}px">{minutesToTime(hour * 60)}</span>
        {/each}
      </div>
      {#each columns as column (column.date)}
        <div
          class="column"
          class:today={column.isToday}
          role="gridcell"
          tabindex="0"
          aria-label={t.calendar.createAt(column.date)}
          data-date={column.date}
          style:--hour-px="{HOUR_PX}px"
          onpointerdown={(event) => onPointerDown(event, column.date)}
          onkeydown={(event) => onColumnKeydown(event, column.date)}
        >
          {#each column.placed as item (item.task.id)}
            {@const tag = tagOf?.(item.task)}
            <button
              type="button"
              class="block"
              class:done={item.task.isCompleted}
              style:--tag={tag?.colorHex}
              onpointerdown={(event) => dragTask?.(event, item.task)}
              data-block
              style:top="{top(item.start)}px"
              style:height="{top(item.span) - 2}px"
              style:left="{(item.lane / item.lanes) * 100}%"
              style:width="calc({100 / item.lanes}% - 2px)"
              title={t.calendar.openTask(
                item.task.title || t.calendar.untitled,
              )}
              onclick={() => onOpenTask(item.task)}
            >
              <span class="time">
                {timeRange(item.task.dueTime ?? "", item.task.durationMinutes)}
              </span>
              {item.task.title || t.calendar.untitled}
              {#if tag}<span class="tag-name">{tag.name}</span>{/if}
            </button>
          {/each}

          {#if showNow}
            <div
              class="now {column.nowLine}"
              data-now-line={column.nowLine}
              style:top="{top(nowMinutes)}px"
            ></div>
          {/if}

          {#if selection && selection.date === column.date}
            <div
              class="selection"
              data-selection
              style:top="{top(selection.start)}px"
              style:height="{top(selection.end - selection.start)}px"
            >
              <span class="span-label">
                {t.calendar.span(
                  minutesToTime(selection.start),
                  minutesToTime(selection.end),
                )}
              </span>
              {#if compose && !drag}
                <input
                  class="quick"
                  data-quick-create
                  type="text"
                  autocomplete="off"
                  aria-label={t.calendar.quickTitleLabel}
                  placeholder={t.calendar.quickTitlePlaceholder}
                  bind:value={title}
                  use:focusOnMount
                  onkeydown={onQuickKeydown}
                  onblur={() => void commit()}
                />
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .grid {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    user-select: none;
  }

  .row {
    display: grid;
    grid-template-columns: 52px repeat(var(--columns, 1), minmax(0, 1fr));
    padding-right: var(--scrollbar-gap, 0px);
    flex: none;
  }

  .gutter {
    color: var(--text-secondary);
    font-size: 11px;
    padding: 4px 6px 0 0;
    text-align: right;
  }

  .head-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 40px;
    border-left: 1px solid var(--separator);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .head-cell .number {
    display: inline-grid;
    place-items: center;
    min-width: 26px;
    height: 26px;
    font-size: 15px;
    font-weight: 600;
    letter-spacing: 0;
    color: var(--text-primary);
  }

  .head-cell.today .number {
    padding: 0 6px;
    border-radius: var(--radius-chip);
    background: var(--accent);
    color: var(--on-accent);
  }

  .allday {
    border-top: 1px solid var(--separator);
    border-bottom: 1px solid var(--separator);
    min-height: 28px;
  }

  .allday-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 2px;
    border-left: 1px solid var(--separator);
    min-width: 0;
  }

  .chip,
  .block {
    border: 0;
    border-radius: 5px;
    background: color-mix(in srgb, var(--tag, var(--accent)) 22%, transparent);
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition:
      background-color var(--motion-instant) linear,
      transform var(--motion-instant) var(--ease-out);
  }

  .chip:hover,
  .block:hover {
    background: color-mix(in srgb, var(--tag, var(--accent)) 34%, transparent);
  }

  .chip:active,
  .block:active {
    transform: scale(0.98);
  }

  .chip {
    padding: 2px 6px;
  }

  .chip.done,
  .block.done {
    opacity: 0.55;
    text-decoration: line-through;
  }

  .scroller {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .body {
    position: relative;
    display: grid;
    grid-template-columns: 52px repeat(var(--columns, 1), minmax(0, 1fr));
  }

  .hours {
    position: relative;
    padding: 0;
    font-variant-numeric: tabular-nums;
  }

  .hours span {
    position: absolute;
    right: 6px;
    transform: translateY(-50%);
  }

  .hours span:first-child {
    transform: none;
  }

  .column {
    position: relative;
    border-left: 1px solid var(--separator);
    background-image: repeating-linear-gradient(
      to bottom,
      var(--separator) 0,
      var(--separator) 1px,
      transparent 1px,
      transparent var(--hour-px)
    );
    cursor: crosshair;
  }

  .column.today {
    background-color: color-mix(in srgb, var(--accent) 5%, transparent);
  }

  .block {
    position: absolute;
    min-height: 14px;
    padding: 1px 6px;
    line-height: 1.2;
    border-left: 3px solid var(--tag, var(--accent));
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

  .block .time {
    color: var(--text-secondary);
    margin-right: 4px;
    font-variant-numeric: tabular-nums;
  }

  .now {
    position: absolute;
    inset-inline: 0;
    z-index: 3;
    height: 1px;
    margin-top: -0.5px;
    background: var(--danger);
    pointer-events: none;
  }

  .now.faint {
    opacity: 0.3;
  }

  .now.start::before {
    content: "";
    position: absolute;
    left: -4px;
    top: -3.5px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--danger);
    box-shadow: 0 0 0 2px var(--bg-window-solid);
  }

  .selection {
    position: absolute;
    inset-inline: 2px;
    z-index: 2;
    padding: 2px 6px;
    border-radius: 5px;
    background: color-mix(in srgb, var(--accent) 30%, transparent);
    box-shadow: inset 0 0 0 1px var(--accent);
    font-size: 11px;
    pointer-events: none;
  }

  .selection .span-label {
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .quick {
    display: block;
    width: 100%;
    margin-top: 2px;
    padding: 2px 6px;
    border: 0;
    border-radius: 4px;
    background: var(--bg-popover);
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
    pointer-events: auto;
    user-select: text;
  }

  .quick:focus-visible {
    outline: 2px solid var(--accent);
  }
</style>
