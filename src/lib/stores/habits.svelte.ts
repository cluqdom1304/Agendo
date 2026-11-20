// The habits (D-95) and the days marked in the window the views show. Marking is optimistic:
// the day is ticked at once and goes back if the backend refuses.
import * as backend from "../api";
import { doneKeys, logKey } from "../features/habits/habits";
import { TaskNotices } from "./taskNotices.svelte";
import type { Habit, HabitInput, HabitLog, HabitPatch } from "../types";
import type { Notice } from "./tasksTypes";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface HabitsApi {
  habitsList(includeArchived?: boolean): Promise<Habit[]>;
  habitLogs(from: string, to: string): Promise<HabitLog[]>;
  habitCreate(input: HabitInput): Promise<Habit>;
  habitUpdate(id: string, patch: HabitPatch): Promise<Habit>;
  habitDelete(id: string): Promise<void>;
  habitRestore(id: string): Promise<Habit>;
  habitSetDone(id: string, date: string, done: boolean): Promise<Habit>;
}

export interface HabitsStoreOptions {
  api?: HabitsApi;
  noticeMs?: number;
}

export class HabitsStore {
  /** Live habits, archived ones included. */
  habits = $state.raw<Habit[]>([]);
  /** The marks of the window `from`..`to`. */
  logs = $state.raw<HabitLog[]>([]);
  from = $state("");
  to = $state("");
  /** The tag filter of the sidebar, set by the page. */
  filter = $state.raw<(habit: Habit) => boolean>(() => true);
  /** The habit just deleted, for «Deshacer». */
  deleted = $state.raw<Habit | null>(null);
  /** The last day marked as done from «Hoy», for «Deshacer». */
  marked = $state.raw<{ habit: Habit; date: string } | null>(null);

  readonly active = $derived(
    this.habits.filter((habit) => !habit.isArchived && this.filter(habit)),
  );
  readonly archived = $derived(
    this.habits.filter((habit) => habit.isArchived && this.filter(habit)),
  );
  readonly done = $derived(doneKeys(this.logs));

  readonly #api: HabitsApi;
  readonly #notices: TaskNotices;
  #request = 0;

  constructor(options: HabitsStoreOptions = {}) {
    this.#api = options.api ?? backend;
    this.#notices = new TaskNotices(options.noticeMs ?? 6000);
  }

  get notice(): Notice | null {
    return this.#notices.notice;
  }

  dismissNotice(): void {
    this.#notices.dismiss();
  }

  isDone(habitId: string, date: string): boolean {
    return this.done.has(logKey(habitId, date));
  }

  /** Loads the habits and their marks between two local days, inclusive. The last request wins. */
  async show(from: string, to: string): Promise<void> {
    this.from = from;
    this.to = to;
    await this.#load();
  }

  refresh(): Promise<void> {
    return this.from ? this.#load() : Promise.resolve();
  }

  async #load(): Promise<void> {
    const request = ++this.#request;
    try {
      const [habits, logs] = await Promise.all([
        this.#api.habitsList(true),
        this.#api.habitLogs(this.from, this.to),
      ]);
      if (request !== this.#request) return;
      this.habits = habits;
      this.logs = logs;
    } catch (error) {
      if (request === this.#request) this.#notices.report(error);
    }
  }

  /** Marks or unmarks `date`. `offerUndo` is for the quick tick of «Hoy». */
  async setDone(
    habit: Habit,
    date: string,
    done: boolean,
    offerUndo = false,
  ): Promise<boolean> {
    const before = this.logs;
    this.logs = done
      ? [...before, { habitId: habit.id, date }]
      : before.filter(
          (log) => !(log.habitId === habit.id && log.date === date),
        );
    try {
      const saved = await this.#api.habitSetDone(habit.id, date, done);
      this.habits = this.habits.map((item) =>
        item.id === saved.id ? saved : item,
      );
      this.marked = done && offerUndo ? { habit: saved, date } : null;
      return true;
    } catch (error) {
      this.logs = before;
      this.#notices.report(error);
      return false;
    }
  }

  async undoMark(): Promise<void> {
    const mark = this.marked;
    if (!mark) return;
    this.marked = null;
    await this.setDone(mark.habit, mark.date, false);
  }

  dismissMarked(): void {
    this.marked = null;
  }

  async create(input: HabitInput): Promise<Habit | null> {
    try {
      const created = await this.#api.habitCreate(input);
      await this.refresh();
      return created;
    } catch (error) {
      this.#notices.report(error);
      return null;
    }
  }

  async update(id: string, patch: HabitPatch): Promise<Habit | null> {
    try {
      const saved = await this.#api.habitUpdate(id, patch);
      await this.refresh();
      return saved;
    } catch (error) {
      this.#notices.report(error);
      return null;
    }
  }

  async remove(habit: Habit): Promise<boolean> {
    try {
      await this.#api.habitDelete(habit.id);
    } catch (error) {
      this.#notices.report(error);
      return false;
    }
    this.deleted = habit;
    await this.refresh();
    return true;
  }

  async undoDelete(): Promise<void> {
    const habit = this.deleted;
    if (!habit) return;
    this.deleted = null;
    try {
      await this.#api.habitRestore(habit.id);
      await this.refresh();
    } catch (error) {
      this.#notices.report(error);
    }
  }

  dismissDeleted(): void {
    this.deleted = null;
  }
}

export const habitsStore = new HabitsStore();
