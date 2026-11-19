// Dragging out a span on a day column and typing the title of the task it creates. The
// component owns the markup and styles; this owns the pointer and keyboard state.

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
  /** `duration` is only given when the span was dragged out (D-33). */
  onCreate: (
    title: string,
    date: string,
    time: string,
    duration?: number,
  ) => Promise<boolean>;
}

export class WeekCompose {
  drag = $state.raw<Drag | null>(null);
  compose = $state.raw<Compose | null>(null);
  title = $state("");

  readonly dragSpan = $derived(
    this.drag
      ? {
          date: this.drag.date,
          ...snapSpan(this.drag.anchor, this.drag.current),
        }
      : null,
  );
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
  }

  cancel(): void {
    this.compose = null;
    this.title = "";
  }

  async commit(): Promise<void> {
    const pending = this.compose;
    const text = this.title.trim();
    if (!pending || this.#committing) return;
    if (!text) {
      this.cancel();
      return;
    }
    this.#committing = true;
    try {
      // A dragged span becomes the duration; a plain click only picks the start (B-04, D-33).
      if (
        await this.#deps.onCreate(
          text,
          pending.date,
          minutesToTime(pending.start),
          pending.sized ? pending.end - pending.start : undefined,
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
      this.cancel();
    }
  }
}
