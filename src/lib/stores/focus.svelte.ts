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
  focusPause(): Promise<TimeEntry>;
  focusResume(): Promise<TimeEntry>;
  focusDiscard(): Promise<void>;
  focusSwitch(taskId: string): Promise<TimeEntry>;
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

/**
 * Milliseconds left of a session, never below zero. Pauses already over do not count and, while
 * paused, the time stands still at the moment the pause began (D-99).
 */
export function remainingMs(entry: TimeEntry, now: number): number {
  const reference = entry.pausedAt ? Date.parse(entry.pausedAt) : now;
  const active =
    reference - Date.parse(entry.startedAt) - entry.pausedSeconds * 1000;
  return Math.max(0, entry.plannedMinutes * 60_000 - active);
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
  /** Seconds of focus per task. */
  totals = $state.raw<Readonly<Record<string, number>>>({});
  #now = $state(0);

  readonly remaining = $derived(
    this.current ? remainingMs(this.current, this.#now) : 0,
  );
  readonly label = $derived(formatCountdown(this.remaining));
  readonly paused = $derived(this.current?.pausedAt != null);

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

  /** Shows the failure of a focus request made outside this store. */
  report(error: unknown): void {
    this.#notices.report(error);
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

  /** Seconds of focus of a task; `0` when it has none. */
  secondsOf(taskId: string): number {
    return this.totals[taskId] ?? 0;
  }

  async loadTotals(): Promise<void> {
    try {
      const rows = await this.#api.focusTotals();
      this.totals = Object.fromEntries(
        rows.map((row) => [row.taskId, row.seconds]),
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

  async pause(): Promise<void> {
    await this.#change(() => this.#api.focusPause());
  }

  async resume(): Promise<void> {
    await this.#change(() => this.#api.focusResume());
  }

  /** Moves the session to another task; the time so far stays on the old one. */
  async switchTo(taskId: string): Promise<void> {
    await this.#change(() => this.#api.focusSwitch(taskId));
    await this.loadTotals();
  }

  /** Deletes the session without keeping its time. */
  async discard(): Promise<void> {
    try {
      await this.#api.focusDiscard();
      this.#set(null);
      this.#notices.inform(t.focus.discarded);
    } catch (error) {
      this.#notices.report(error);
    }
  }

  async #change(action: () => Promise<TimeEntry>): Promise<void> {
    try {
      this.#set(await action());
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
