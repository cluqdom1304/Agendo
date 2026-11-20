<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "../../i18n";
  import { timeRange } from "../../schedule";
  import type { EventOccurrence, Tag, Task } from "../../types";
  import { blockOf, isTimedBlock, timeRangeOf } from "../events/occurrences";
  import { columnDateAt, EventDrag } from "./eventDrag.svelte";
  import {
    HOUR_PX,
    layoutBlocks,
    minutesToTime,
    spanOf,
    timeToMinutes,
  } from "./grid";
  import { dayNumber, weekdayName } from "./labels";
  import { WeekCompose } from "./weekCompose.svelte";
  import type { Capture } from "../../naturalInput";
  import { captureOptions } from "../capture/capture.svelte";
  import CapturePreview from "../capture/CapturePreview.svelte";

  const FIRST_VISIBLE_HOUR = 7;

  interface Props {
    /** The columns: seven dates for a week, one for a day. */
    dates: string[];
    today: string;
    /** Minutes since midnight of the current time, for the "now" line. */
    nowMinutes: number;
    tasksByDate: Map<string, Task[]>;
    /** The events of each day: timed ones are blocks of the grid, the rest sit in the all-day row. */
    eventsByDate?: Map<string, EventOccurrence[]>;
    label: string;
    /** `duration` is only given when the span was dragged out (D-33). */
    onCreate: (
      title: string,
      date: string,
      time: string,
      duration?: number,
      capture?: Capture,
    ) => Promise<boolean>;
    onOpenTask: (task: Task) => void;
    onOpenEvent?: (eventId: string) => void;
    /** An event was dragged to `date` at `start` (minutes since midnight). */
    onMoveEvent?: (
      occurrence: EventOccurrence,
      date: string,
      start: number,
    ) => void;
    /** The lower edge of an event was dragged to `end`. */
    onResizeEvent?: (occurrence: EventOccurrence, end: number) => void;
    /** A timed task was dragged to `date` at `start` (minutes since midnight). */
    onMoveTask?: (task: Task, date: string, start: number) => void;
    /** The lower edge of a timed task was dragged to `end`. */
    onResizeTask?: (task: Task, end: number) => void;
    /** The tag of the task or event, which gives its colour and the small name on it. */
    tagOf?: (item: Pick<Task, "tagIds">) => Tag | undefined;
    /** How many more tags the task has besides the one shown. */
    moreTags?: (task: Task) => number;
    /** Pressed on an event: the parent starts dragging it towards a sidebar tag. */
    dragTask?: (event: PointerEvent, task: Task) => void;
  }

  let {
    dates,
    today,
    nowMinutes,
    tasksByDate,
    eventsByDate,
    label,
    tagOf,
    moreTags,
    dragTask,
    onCreate,
    onOpenTask,
    onOpenEvent,
    onMoveEvent,
    onResizeEvent,
    onMoveTask,
    onResizeTask,
  }: Props = $props();

  type Item =
    | { kind: "task"; task: Task }
    | { kind: "event"; occurrence: EventOccurrence };

  let scroller: HTMLElement | undefined = $state();
  // The body scrolls and its scrollbar narrows the columns; the fixed rows above get the same
  // width taken away so every column line stays on the same x.
  let scrollbarWidth = $state(0);

  function measureScrollbar() {
    if (scroller) {
      scrollbarWidth = Math.max(0, scroller.offsetWidth - scroller.clientWidth);
    }
  }
  const week = new WeekCompose({
    hourPx: HOUR_PX,
    today: () => today,
    nowMinutes: () => nowMinutes,
    capture: (date) => captureOptions(today, date),
    onCreate: (...args) => onCreate(...args),
  });

  const eventDrag = new EventDrag<EventOccurrence>({
    hourPx: HOUR_PX,
    dateAt: columnDateAt,
    onMove: (...args) => onMoveEvent?.(...args),
    onResize: (...args) => onResizeEvent?.(...args),
  });

  const taskDrag = new EventDrag<Task>({
    hourPx: HOUR_PX,
    dateAt: columnDateAt,
    onMove: (...args) => onMoveTask?.(...args),
    onResize: (...args) => onResizeTask?.(...args),
  });

  // A press on a task block may become a move in the grid or, once the pointer reaches the
  // sidebar, a drag towards a tag; the tag drag only starts there so the two never overlap.
  let tagPress: { event: PointerEvent; task: Task } | null = null;

  function followPointer(event: PointerEvent) {
    if (!tagPress) return;
    const target = document.elementFromPoint?.(event.clientX, event.clientY);
    if (!target?.closest("[data-tag-drop]")) return;
    const { event: press, task } = tagPress;
    tagPress = null;
    taskDrag.cancel();
    dragTask?.(press, task);
  }

  const showNow = $derived(dates.includes(today));
  const columns = $derived(
    dates.map((date) => {
      const tasks = tasksByDate.get(date) ?? [];
      const occurrences = eventsByDate?.get(date) ?? [];
      return {
        date,
        isToday: date === today,
        // The line starts on today's column and runs to the end of the week; the days
        // before it only get a faint trace.
        nowLine: today < date ? "full" : date === today ? "start" : "faint",
        allDay: tasks.filter((task) => task.dueTime === null),
        allDayEvents: occurrences.filter((item) => !isTimedBlock(item)),
        placed: layoutBlocks<Item>([
          ...tasks
            .filter((task) => task.dueTime !== null)
            .map((task) => ({
              item: { kind: "task" as const, task },
              key: task.id,
              start: timeToMinutes(task.dueTime ?? "00:00"),
              span: spanOf(task),
            })),
          ...occurrences.filter(isTimedBlock).map((occurrence) => ({
            item: { kind: "event" as const, occurrence },
            key: occurrence.eventId,
            ...blockOf(occurrence),
          })),
        ]),
      };
    }),
  );

  onMount(() => {
    if (scroller) scroller.scrollTop = FIRST_VISIBLE_HOUR * HOUR_PX;
    measureScrollbar();
  });

  const top = (minutes: number) => (minutes / 60) * HOUR_PX;

  function focusOnMount(node: HTMLElement) {
    node.focus();
  }

  const hours = Array.from({ length: 24 }, (_, hour) => hour);
</script>

<svelte:window
  onpointermove={(event) => {
    week.pointerMove(event);
    eventDrag.pointerMove(event);
    taskDrag.pointerMove(event);
    followPointer(event);
  }}
  onpointerup={(event) => {
    week.pointerUp(event);
    eventDrag.pointerUp(event);
    taskDrag.pointerUp(event);
    tagPress = null;
  }}
  onkeydown={(event) => {
    if (event.key === "Escape") {
      eventDrag.cancel();
      taskDrag.cancel();
    }
  }}
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
        {#each column.allDayEvents as occurrence (occurrence.eventId + occurrence.startDate)}
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
            {name}
          </button>
        {/each}
        {#each column.allDay as task (task.id)}
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
            {task.title || t.calendar.untitled}
            {#if tag}<span class="tag-name"
                >{tag.name}{#if moreTags?.(task)}
                  +{moreTags(task)}{/if}</span
              >{/if}
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
          data-slot-drop={column.date}
          style:--hour-px="{HOUR_PX}px"
          onpointerdown={(event) => week.pointerDown(event, column.date)}
          onkeydown={(event) => week.columnKeydown(event, column.date)}
        >
          {#each column.placed as block (block.item.kind === "task" ? `t:${block.item.task.id}` : `e:${block.item.occurrence.eventId}:${block.item.occurrence.startDate}`)}
            {#if block.item.kind === "task"}
              {@const item = { ...block, task: block.item.task }}
              {@const tag = tagOf?.(item.task)}
              <button
                type="button"
                class="block"
                class:done={item.task.isCompleted}
                class:dragging={taskDrag.preview?.subject === item.task}
                style:--tag={tag?.colorHex}
                onpointerdown={(event) => {
                  tagPress = { event, task: item.task };
                  taskDrag.begin(
                    event,
                    item.task,
                    "move",
                    column.date,
                    item.start,
                    item.span,
                  );
                }}
                data-block
                data-task-id={item.task.id}
                style:top="{top(item.start)}px"
                style:height="{top(item.span) - 2}px"
                style:left="{(item.lane / item.lanes) * 100}%"
                style:width="calc({100 / item.lanes}% - 2px)"
                title={t.calendar.openTask(
                  item.task.title || t.calendar.untitled,
                )}
                onclick={() => {
                  if (!taskDrag.consumeClick()) onOpenTask(item.task);
                }}
              >
                <span class="time">
                  {timeRange(
                    item.task.dueTime ?? "",
                    item.task.durationMinutes,
                  )}
                </span>
                {item.task.title || t.calendar.untitled}
                {#if tag}<span class="tag-name"
                    >{tag.name}{#if moreTags?.(item.task)}
                      +{moreTags(item.task)}{/if}</span
                  >{/if}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                  class="handle"
                  data-resize
                  title={t.calendar.resizeTask(
                    item.task.title || t.calendar.untitled,
                  )}
                  onpointerdown={(event) => {
                    event.stopPropagation();
                    taskDrag.begin(
                      event,
                      item.task,
                      "resize",
                      column.date,
                      item.start,
                      item.span,
                    );
                  }}
                ></span>
              </button>
            {:else}
              {@const occurrence = block.item.occurrence}
              {@const tag = tagOf?.(occurrence)}
              {@const name = occurrence.title || t.calendar.untitled}
              <button
                type="button"
                class="block event"
                class:dragging={eventDrag.preview?.subject === occurrence}
                style:--tag={tag?.colorHex}
                data-block
                data-event-id={occurrence.eventId}
                style:top="{top(block.start)}px"
                style:height="{top(block.span) - 2}px"
                style:left="{(block.lane / block.lanes) * 100}%"
                style:width="calc({100 / block.lanes}% - 2px)"
                title={t.calendar.openEvent(name)}
                onpointerdown={(event) =>
                  eventDrag.begin(
                    event,
                    occurrence,
                    "move",
                    column.date,
                    block.start,
                    block.span,
                  )}
                onclick={() => {
                  if (!eventDrag.consumeClick())
                    onOpenEvent?.(occurrence.eventId);
                }}
              >
                <span class="time">{timeRangeOf(occurrence)}</span>
                {name}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                  class="handle"
                  data-resize
                  title={t.events.resizeHandle(name)}
                  onpointerdown={(event) => {
                    event.stopPropagation();
                    eventDrag.begin(
                      event,
                      occurrence,
                      "resize",
                      column.date,
                      block.start,
                      block.span,
                    );
                  }}
                ></span>
              </button>
            {/if}
          {/each}

          {#if eventDrag.preview && eventDrag.preview.date === column.date}
            {@const preview = eventDrag.preview}
            <div
              class="block event ghost"
              data-drag-preview
              style:top="{top(preview.start)}px"
              style:height="{top(preview.end - preview.start) - 2}px"
            >
              <span class="time">
                {t.calendar.span(
                  minutesToTime(preview.start),
                  minutesToTime(preview.end),
                )}
              </span>
              {preview.subject.title || t.calendar.untitled}
            </div>
          {/if}

          {#if taskDrag.preview && taskDrag.preview.date === column.date}
            {@const preview = taskDrag.preview}
            <div
              class="block ghost"
              data-drag-preview
              style:top="{top(preview.start)}px"
              style:height="{top(preview.end - preview.start) - 2}px"
            >
              <span class="time">
                {t.calendar.span(
                  minutesToTime(preview.start),
                  minutesToTime(preview.end),
                )}
              </span>
              {preview.subject.title || t.calendar.untitled}
            </div>
          {/if}

          {#if showNow}
            <div
              class="now {column.nowLine}"
              data-now-line={column.nowLine}
              style:top="{top(nowMinutes)}px"
            ></div>
          {/if}

          {#if week.selection && week.selection.date === column.date}
            <div
              class="selection"
              data-selection
              style:top="{top(week.selection.start)}px"
              style:height="{top(week.selection.end - week.selection.start)}px"
            >
              <span class="span-label">
                {t.calendar.span(
                  minutesToTime(week.selection.start),
                  minutesToTime(week.selection.end),
                )}
              </span>
              {#if week.compose && !week.drag}
                <input
                  class="quick"
                  data-quick-create
                  type="text"
                  autocomplete="off"
                  aria-label={t.calendar.quickTitleLabel}
                  placeholder={t.calendar.quickTitlePlaceholder}
                  bind:value={week.title}
                  use:focusOnMount
                  onkeydown={(event) => week.quickKeydown(event)}
                  onblur={() => void week.commit()}
                />
                <CapturePreview capture={week.capture} {today} floating />
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
    opacity: 0.72;
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

  .chip.event,
  .block.event {
    background: color-mix(in srgb, var(--tag, var(--accent)) 38%, transparent);
    font-weight: 500;
  }

  .block[data-task-id],
  .block.event {
    touch-action: none;
    cursor: grab;
  }

  .block.dragging,
  .block.event.dragging {
    opacity: 0.35;
  }

  .block.ghost {
    right: 2px;
    left: 0;
    border: 1px dashed var(--tag, var(--accent));
    box-shadow: var(--shadow-raised);
    pointer-events: none;
  }

  .handle {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    height: 6px;
    cursor: ns-resize;
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
