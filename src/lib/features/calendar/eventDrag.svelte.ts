// Dragging a block of the time grid (events F21-02, tasks F24): the body moves it to another time or day, the
// lower edge changes when it ends. The component owns the markup; this owns the pointer state.
// Nothing is saved until the pointer is released, and Escape drops the drag.

import { MINUTES_PER_DAY, SNAP_MINUTES } from "./grid";

export type DragMode = "move" | "resize";

/** Pixels the pointer must travel before a press is a drag and not a click. */
const THRESHOLD_PX = 4;

export interface EventDragPreview<T> {
  subject: T;
  mode: DragMode;
  date: string;
  start: number;
  end: number;
}

export interface EventDragDeps<T> {
  hourPx: number;
  /** The date of the day column under a point of the screen, if any. */
  dateAt: (x: number, y: number) => string | null;
  onMove: (subject: T, date: string, start: number) => void;
  onResize: (subject: T, end: number) => void;
}

interface Press<T> {
  subject: T;
  mode: DragMode;
  x: number;
  y: number;
  date: string;
  start: number;
  end: number;
}

/** The day column under a point, read from the page. */
export function columnDateAt(x: number, y: number): string | null {
  const element = document.elementFromPoint?.(x, y);
  return (
    element?.closest<HTMLElement>("[role='gridcell'][data-date]")?.dataset
      .date ?? null
  );
}

const snap = (minutes: number) =>
  Math.round(minutes / SNAP_MINUTES) * SNAP_MINUTES;

export class EventDrag<T = unknown> {
  /** What the grid draws while a drag is under way; `null` before the pointer has moved. */
  preview = $state.raw<EventDragPreview<T> | null>(null);

  readonly #deps: EventDragDeps<T>;
  #press: Press<T> | null = null;
  #suppressClick = false;

  constructor(deps: EventDragDeps<T>) {
    this.#deps = deps;
  }

  begin(
    event: PointerEvent,
    subject: T,
    mode: DragMode,
    date: string,
    start: number,
    span: number,
  ): void {
    if (event.button !== 0) return;
    this.#press = {
      subject,
      mode,
      x: event.clientX,
      y: event.clientY,
      date,
      start,
      end: start + span,
    };
    this.preview = null;
  }

  #previewAt(event: PointerEvent, press: Press<T>): EventDragPreview<T> {
    const delta = ((event.clientY - press.y) / this.#deps.hourPx) * 60;
    if (press.mode === "resize") {
      const end = Math.min(
        Math.max(snap(press.end + delta), press.start + SNAP_MINUTES),
        MINUTES_PER_DAY,
      );
      return {
        subject: press.subject,
        mode: "resize",
        date: press.date,
        start: press.start,
        end,
      };
    }
    const span = press.end - press.start;
    const start = Math.min(
      Math.max(snap(press.start + delta), 0),
      MINUTES_PER_DAY - span,
    );
    return {
      subject: press.subject,
      mode: "move",
      date: this.#deps.dateAt(event.clientX, event.clientY) ?? press.date,
      start,
      end: start + span,
    };
  }

  pointerMove(event: PointerEvent): void {
    const press = this.#press;
    if (!press) return;
    const travelled = Math.hypot(
      event.clientX - press.x,
      event.clientY - press.y,
    );
    if (!this.preview && travelled < THRESHOLD_PX) return;
    this.preview = this.#previewAt(event, press);
  }

  pointerUp(event: PointerEvent): void {
    const press = this.#press;
    if (!press) return;
    const moved = this.preview !== null;
    const result = moved ? this.#previewAt(event, press) : null;
    this.cancel();
    if (!result) return;

    // The click that follows the release must not open the event.
    this.#suppressClick = true;
    setTimeout(() => (this.#suppressClick = false), 0);
    const unchanged =
      result.date === press.date &&
      result.start === press.start &&
      result.end === press.end;
    if (unchanged) return;
    if (result.mode === "resize") {
      this.#deps.onResize(result.subject, result.end);
    } else {
      this.#deps.onMove(result.subject, result.date, result.start);
    }
  }

  /** Drops the drag without saving. */
  cancel(): void {
    this.#press = null;
    this.preview = null;
  }

  /** True once after a drag ended: the click that follows it has to be ignored. */
  consumeClick(): boolean {
    const suppressed = this.#suppressClick;
    this.#suppressClick = false;
    return suppressed;
  }
}
