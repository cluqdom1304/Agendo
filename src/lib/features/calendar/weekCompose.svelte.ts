// Dragging out a span on a day column and typing the title of the task it creates. The
// component owns the markup and styles; this owns the pointer and keyboard state.

import {
  parseNatural,
  type Capture,
  type CaptureOptions,
} from "../../naturalInput";
import { BLOCK_MINUTES, SNAP_MINUTES, minutesToTime, snapSpan } from "./grid";

/** Where a task made from the keyboard on a day other than today starts. */
const DEFAULT_START_MINUTES = 9 * 60;

interface Drag {
  date: string;
  anchor: number;
  current: number;
}

export interface Compose {
  date: string;
  start: number;
  end: number;
  /** Dragged over more than one step: the length is the task's duration. */
  sized: boolean;
}

export interface WeekComposeDeps {
  hourPx: number;
  today: () => string;
  /** Minutes since midnight of the current time. */
  nowMinutes: () => number;
  /** Reads dates, tags… from the title (F23-02); the slot's day is the fallback. */
  capture?: (date: string) => CaptureOptions;
  /** `duration` is only given when the span was dragged out (D-33) or typed. */
  onCreate: (
    title: string,
    date: string,
    time: string,
    duration?: number,
    capture?: Capture,
  ) => Promise<boolean>;
}

export class WeekCompose {
  drag = $state.raw<Drag | null>(null);
  compose = $state.raw<Compose | null>(null);
  title = $state("");
  /** Esc on a title with something detected keeps it as typed. */
  ignored = $state(false);

  readonly dragSpan = $derived(
    this.drag
      ? {
          date: this.drag.date,
          ...snapSpan(this.drag.anchor, this.drag.current),
        }
      : null,
  );
  readonly capture = $derived.by(() => {
    const options = this.#deps.capture;
    return this.compose && options && !this.ignored && this.title.trim()
      ? parseNatural(this.title, options(this.compose.date))
      : null;
  });
  /** What the grid highlights: the span being dragged, else the one waiting for a title. */
  readonly selection = $derived(this.dragSpan ?? this.compose);

  readonly #deps: WeekComposeDeps;
  #column: HTMLElement | null = null;
  #committing = false;

  constructor(deps: WeekComposeDeps) {
    this.#deps = deps;
  }

  #minutesAt(event: MouseEvent, element: HTMLElement): number {
    return (
      ((event.clientY - element.getBoundingClientRect().top) /
        this.#deps.hourPx) *
      60
    );
  }

  pointerDown(event: PointerEvent, date: string): void {
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
    this.#column = element;
    const minutes = this.#minutesAt(event, element);
    this.compose = null;
    this.drag = { date, anchor: minutes, current: minutes };
    event.preventDefault();
  }

  pointerMove(event: PointerEvent): void {
    if (!this.drag || !this.#column) return;
    this.drag = { ...this.drag, current: this.#minutesAt(event, this.#column) };
  }

  pointerUp(event: PointerEvent): void {
    if (!this.drag || !this.#column) return;
    const finished = {
      ...this.drag,
      current: this.#minutesAt(event, this.#column),
    };
    this.drag = null;
    this.#column = null;
    const span = snapSpan(finished.anchor, finished.current);
    this.compose = {
      date: finished.date,
      ...span,
      sized: span.end - span.start > SNAP_MINUTES,
    };
    this.title = "";
    this.ignored = false;
  }

  /** Keyboard route to what a drag does: Enter on a day column opens the quick title. */
  columnKeydown(event: KeyboardEvent, date: string): void {
    if (event.target !== event.currentTarget) return;
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    const start =
      date === this.#deps.today()
        ? Math.min(
            Math.ceil(this.#deps.nowMinutes() / BLOCK_MINUTES) * BLOCK_MINUTES,
            24 * 60 - BLOCK_MINUTES,
          )
        : DEFAULT_START_MINUTES;
    this.drag = null;
    this.compose = { date, start, end: start + BLOCK_MINUTES, sized: false };
    this.title = "";
    this.ignored = false;
  }

  cancel(): void {
    this.compose = null;
    this.title = "";
    this.ignored = false;
  }

  async commit(): Promise<void> {
    const pending = this.compose;
    const text = this.title.trim();
    if (!pending || this.#committing) return;
    if (!text) {
      this.cancel();
      return;
    }
    const capture = this.capture ?? undefined;
    this.#committing = true;
    try {
      // A dragged span becomes the duration; a plain click only picks the start (B-04, D-33).
      // What is typed in the title wins over the slot.
      const slotDuration = pending.sized
        ? pending.end - pending.start
        : undefined;
      if (
        await this.#deps.onCreate(
          capture?.title ?? text,
          capture?.date ?? pending.date,
          capture?.time ?? minutesToTime(pending.start),
          capture?.time ? (capture.duration ?? undefined) : slotDuration,
          capture,
        )
      ) {
        this.cancel();
      }
    } finally {
      this.#committing = false;
    }
  }

  quickKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      void this.commit();
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      if (this.capture) this.ignored = true;
      else this.cancel();
    }
  }
}
