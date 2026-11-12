import * as backend from "../api";
import { toLocalIsoDate } from "../dates";
import {
  addDays,
  addMonths,
  systemFirstDay,
  type WeekDay,
} from "../features/calendar/grid";
import type { ViewMode } from "../types";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface ViewApi {
  settingsGet(key: "view_mode" | "app_section"): Promise<string | null>;
  settingsSet(key: "view_mode" | "app_section", value: string): Promise<void>;
}

const MODES: readonly ViewMode[] = ["list", "day", "week", "month"];

function parseMode(value: string | null): ViewMode {
  return MODES.find((mode) => mode === value) ?? "list";
}

export interface ViewStoreOptions {
  api?: ViewApi;
  now?: () => Date;
  firstDay?: () => WeekDay;
}

export type Section = "agenda" | "notes";

/** Which view is showing (persisted in `settings`) and which date the calendar is centred on. */
export class ViewStore {
  mode = $state<ViewMode>("list");
  /** The calendar view to return to when switching from the list; week until one is chosen. */
  calendarMode = $state<Exclude<ViewMode, "list">>("week");
  /** `YYYY-MM-DD` the day, week or month view is showing. */
  anchor = $state("");
  /** The section of the app: the agenda (tasks and calendar) or the notes. */
  section = $state<Section>("agenda");
  /** Waits for the saved mode so the list does not flash before a saved calendar view. */
  ready = $state(false);
  readonly firstDay: WeekDay;

  readonly #api: ViewApi;
  readonly #now: () => Date;

  constructor(options: ViewStoreOptions = {}) {
    this.#api = options.api ?? backend;
    // A clock reading, not reactive state.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    this.#now = options.now ?? (() => new Date());
    this.firstDay = (options.firstDay ?? systemFirstDay)();
    this.anchor = this.today();
  }

  now(): Date {
    return this.#now();
  }

  today(): string {
    return toLocalIsoDate(this.#now());
  }

  async load(): Promise<void> {
    try {
      this.mode = parseMode(await this.#api.settingsGet("view_mode"));
    } catch {
      // The saved view is a convenience: without it the app starts on the list.
      this.mode = "list";
    }
    try {
      this.section =
        (await this.#api.settingsGet("app_section")) === "notes"
          ? "notes"
          : "agenda";
    } catch {
      this.section = "agenda";
    }
    if (this.mode !== "list") this.calendarMode = this.mode;
    this.anchor = this.today();
    this.ready = true;
  }

  /** Switches at once; saving is best effort, since the view itself already changed. */
  setMode(mode: ViewMode): void {
    if (mode === this.mode) return;
    this.mode = mode;
    if (mode !== "list") this.calendarMode = mode;
    void this.#api.settingsSet("view_mode", mode).catch(() => undefined);
  }

  /** Switches the section at once; saving is best effort, like the view. */
  setSection(section: Section): void {
    if (section === this.section) return;
    this.section = section;
    void this.#api.settingsSet("app_section", section).catch(() => undefined);
  }

  /** Moves a whole day, week or month, depending on the view. */
  shift(direction: 1 | -1): void {
    if (this.mode === "day") this.anchor = addDays(this.anchor, direction);
    else if (this.mode === "week") {
      this.anchor = addDays(this.anchor, 7 * direction);
    } else if (this.mode === "month") {
      this.anchor = addMonths(this.anchor, direction);
    }
  }

  goToToday(): void {
    this.anchor = this.today();
  }

  showDay(date: string): void {
    this.anchor = date;
    this.setMode("day");
  }
}

export const viewStore = new ViewStore();
