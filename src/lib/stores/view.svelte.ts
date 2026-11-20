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

/** The views of the sidebar. Habits join when their view exists (F22). */
export type Section =
  "today" | "upcoming" | "tasks" | "calendar" | "events" | "habits" | "notes";

const SECTIONS: readonly Section[] = [
  "today",
  "upcoming",
  "tasks",
  "calendar",
  "events",
  "habits",
  "notes",
];

/** How many steps back and forward are remembered. */
const HISTORY_LIMIT = 50;

/** Reads the saved section; 2.0.x saved `agenda` for both the list and the calendar. */
export function parseSection(value: string | null, mode: ViewMode): Section {
  if (value === "agenda") return mode === "list" ? "tasks" : "calendar";
  return SECTIONS.find((section) => section === value) ?? "today";
}

/** The sections that show the task list or the calendar. */
export function isAgenda(section: Section): boolean {
  return section === "tasks" || section === "calendar";
}

/** Which view is showing (persisted in `settings`) and which date the calendar is centred on. */
export class ViewStore {
  mode = $state<ViewMode>("list");
  /** The calendar view to return to when switching from the list; week until one is chosen. */
  calendarMode = $state<Exclude<ViewMode, "list">>("week");
  /** `YYYY-MM-DD` the day, week or month view is showing. */
  anchor = $state("");
  /** The view chosen in the sidebar. `tasks` shows the list and `calendar` the calendar mode. */
  section = $state<Section>("today");
  /** Waits for the saved mode so the list does not flash before a saved calendar view. */
  ready = $state(false);
  readonly firstDay: WeekDay;

  /** Sections left behind and the ones to return to, for back and forward (F19-03). */
  #back = $state<Section[]>([]);
  #forward = $state<Section[]>([]);
  #travelling = false;

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
    let saved: string | null = null;
    try {
      saved = await this.#api.settingsGet("app_section");
    } catch {
      // Without the saved section the app opens on «Hoy».
    }
    this.section = parseSection(saved, this.mode);
    if (this.mode !== "list") this.calendarMode = this.mode;
    // The list belongs to «Tareas» and the calendar views to «Calendario».
    if (this.section === "tasks") this.mode = "list";
    else if (this.section === "calendar") this.mode = this.calendarMode;
    if (saved === "agenda") this.#save("app_section", this.section);
    this.anchor = this.today();
    this.ready = true;
  }

  get inAgenda(): boolean {
    return isAgenda(this.section);
  }

  /** Switches at once; the list lives in «Tareas» and the calendar views in «Calendario». */
  setMode(mode: ViewMode): void {
    this.#setSectionOnly(mode === "list" ? "tasks" : "calendar");
    if (mode === this.mode) return;
    this.mode = mode;
    if (mode !== "list") this.calendarMode = mode;
    this.#save("view_mode", mode);
  }

  /** Switches the section at once; saving is best effort, like the view. */
  setSection(section: Section): void {
    if (section === "tasks") this.setMode("list");
    else if (section === "calendar") this.setMode(this.calendarMode);
    else this.#setSectionOnly(section);
  }

  /** Goes to the task list unless the list or the calendar is already showing. */
  showAgenda(): void {
    if (!this.inAgenda) this.setSection("tasks");
  }

  get canGoBack(): boolean {
    return this.#back.length > 0;
  }

  get canGoForward(): boolean {
    return this.#forward.length > 0;
  }

  goBack(): void {
    const target = this.#back.at(-1);
    if (!target) return;
    this.#back = this.#back.slice(0, -1);
    this.#forward = [...this.#forward, this.section];
    this.#travel(target);
  }

  goForward(): void {
    const target = this.#forward.at(-1);
    if (!target) return;
    this.#forward = this.#forward.slice(0, -1);
    this.#back = [...this.#back, this.section];
    this.#travel(target);
  }

  #travel(section: Section): void {
    this.#travelling = true;
    try {
      this.setSection(section);
    } finally {
      this.#travelling = false;
    }
  }

  #setSectionOnly(section: Section): void {
    if (section === this.section) return;
    if (!this.#travelling) {
      this.#back = [...this.#back, this.section].slice(-HISTORY_LIMIT);
      this.#forward = [];
    }
    this.section = section;
    this.#save("app_section", section);
  }

  /** Saving is best effort: the view itself already changed. */
  #save(key: "view_mode" | "app_section", value: string): void {
    void this.#api.settingsSet(key, value).catch(() => undefined);
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
