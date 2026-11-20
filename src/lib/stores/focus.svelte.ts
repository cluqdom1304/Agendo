// The focus session in progress (D-97). The backend marks start and end and closes the session at
// its planned end; this store only shows the countdown and asks the backend again once it is over.
import * as backend from "../api";
import { t } from "../i18n";
import { TaskNotices } from "./taskNotices.svelte";
import type { Notice } from "./tasksTypes";
import type { TaskFocusTotal, TimeEntry } from "../types";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface FocusApi {
  focusCurrent(): Promise<TimeEntry | null>;
  focusStart(taskId: string, plannedMinutes: number): Promise<TimeEntry>;
  focusStop(): Promise<TimeEntry | null>;
  focusTotals(): Promise<TaskFocusTotal[]>;
  focusTaskEntries(taskId: string): Promise<TimeEntry[]>;
  focusAddManual(taskId: string, minutes: number): Promise<TimeEntry>;
  focusDeleteEntry(id: string): Promise<void>;
}

export interface FocusStoreOptions {
  api?: FocusApi;
  now?: () => number;
  /** Interval of the countdown; `0` leaves it to the caller (tests). */
  tickMs?: number;
  noticeMs?: number;
}

/** Milliseconds left of a session, never below zero. */
export function remainingMs(entry: TimeEntry, now: number): number {
  const end = Date.parse(entry.startedAt) + entry.plannedMinutes * 60_000;
  return Math.max(0, end - now);
}

/** `mm:ss`, or `h:mm:ss` from one hour. */
export function formatCountdown(ms: number): string {
  const total = Math.ceil(ms / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const two = (value: number) => String(value).padStart(2, "0");
  return hours > 0
    ? `${hours}:${two(minutes)}:${two(seconds)}`
    : `${two(minutes)}:${two(seconds)}`;
}

export class FocusStore {
  current = $state.raw<TimeEntry | null>(null);
  /** Minutes of focus per task. */
  totals = $state.raw<Readonly<Record<string, number>>>({});
  #now = $state(0);

  readonly remaining = $derived(
    this.current ? remainingMs(this.current, this.#now) : 0,
  );
  readonly label = $derived(formatCountdown(this.remaining));

  readonly #api: FocusApi;
  readonly #clock: () => number;
  readonly #notices: TaskNotices;
  #timer: ReturnType<typeof setInterval> | null = null;
  #tickMs: number;
  #checking = false;

  constructor(options: FocusStoreOptions = {}) {
    this.#api = options.api ?? backend;
    this.#clock = options.now ?? Date.now;
    this.#tickMs = options.tickMs ?? 1000;
    this.#notices = new TaskNotices(options.noticeMs ?? 6000);
    this.#now = this.#clock();
  }

  get notice(): Notice | null {
    return this.#notices.notice;
  }

  dismissNotice(): void {
    this.#notices.dismiss();
  }

  async load(): Promise<void> {
    try {
      const wasRunning = this.current !== null;
      this.#set(await this.#api.focusCurrent());
      if (!wasRunning || this.current === null) await this.loadTotals();
    } catch (error) {
      this.#notices.report(error);
    }
  }

  /** Minutes of focus of a task; `0` when it has none. */
  minutesOf(taskId: string): number {
    return this.totals[taskId] ?? 0;
  }

  async loadTotals(): Promise<void> {
    try {
      const rows = await this.#api.focusTotals();
      this.totals = Object.fromEntries(
        rows.map((row) => [row.taskId, row.minutes]),
      );
    } catch (error) {
      this.#notices.report(error);
    }
  }

  /** The closed sessions and manual entries of a task, newest first. */
  async entriesOf(taskId: string): Promise<TimeEntry[]> {
    try {
      return await this.#api.focusTaskEntries(taskId);
    } catch (error) {
      this.#notices.report(error);
      return [];
    }
  }

  async addManual(taskId: string, minutes: number): Promise<boolean> {
    try {
      await this.#api.focusAddManual(taskId, minutes);
      await this.loadTotals();
      return true;
    } catch (error) {
      this.#notices.report(error);
      return false;
    }
  }

  async deleteEntry(id: string): Promise<boolean> {
    try {
      await this.#api.focusDeleteEntry(id);
      await this.loadTotals();
      return true;
    } catch (error) {
      this.#notices.report(error);
      return false;
    }
  }

  async start(taskId: string, plannedMinutes: number): Promise<void> {
    try {
      this.#set(await this.#api.focusStart(taskId, plannedMinutes));
    } catch (error) {
      this.#notices.report(error);
    }
  }

  async stop(): Promise<void> {
    try {
      const closed = await this.#api.focusStop();
      this.#set(null);
      if (closed) this.#notices.inform(t.focus.stopped);
      await this.loadTotals();
    } catch (error) {
      this.#notices.report(error);
    }
  }

  /** One step of the countdown. Once the time is over the backend closes the session. */
  async tick(): Promise<void> {
    this.#now = this.#clock();
    if (!this.current || this.remaining > 0 || this.#checking) return;
    this.#checking = true;
    try {
      await this.load();
      // The scheduler may not have closed it yet; the next tick asks again.
    } finally {
      this.#checking = false;
    }
  }

  #set(entry: TimeEntry | null): void {
    this.#now = this.#clock();
    this.current = entry && !entry.endedAt ? entry : null;
    if (this.current) this.#run();
    else this.#halt();
  }

  #run(): void {
    if (this.#timer !== null || this.#tickMs <= 0) return;
    this.#timer = setInterval(() => void this.tick(), this.#tickMs);
  }

  #halt(): void {
    if (this.#timer !== null) clearInterval(this.#timer);
    this.#timer = null;
  }
}

export const focusStore = new FocusStore();
