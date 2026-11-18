// Multiple selection and batch actions on the task list (F17-03). Selecting is Ctrl/Shift+click;
// each action runs task by task through the same API calls as the single-task ones and leaves one
// undo entry that reverts the whole batch.

import { SvelteSet } from "svelte/reactivity";

import { t } from "../i18n";
import type { Task } from "../types";
import type { TasksApi } from "./tasks.svelte";

export interface BatchUndo {
  key: number;
  message: string;
  undo: () => Promise<void>;
}

export interface BatchDeps {
  api: TasksApi;
  tasks: () => Task[];
  /** Ids of the visible tasks in the order the list draws them, for Shift+click ranges. */
  order: () => string[];
  /** Local date (`YYYY-MM-DD`) a completed task is filed under. */
  today: () => string;
  refresh: () => Promise<void>;
  report: (error: unknown) => void;
  inform: (message: string) => void;
  undoMs: number;
}

export class TaskBatch {
  ids = $state.raw<string[]>([]);
  entry = $state.raw<BatchUndo | null>(null);
  count = $derived(this.ids.length);
  /** Whether the next «complete» reopens: true when every selected task is done. */
  allDone = $derived.by(() => {
    const chosen = this.#chosen();
    return chosen.length > 0 && chosen.every((task) => task.isCompleted);
  });

  readonly #deps: BatchDeps;
  #anchor: string | null = null;
  #keys = 0;
  #timer: ReturnType<typeof setTimeout> | null = null;

  constructor(deps: BatchDeps) {
    this.#deps = deps;
  }

  isSelected(id: string): boolean {
    return this.ids.includes(id);
  }

  toggle(id: string): void {
    this.ids = this.isSelected(id)
      ? this.ids.filter((candidate) => candidate !== id)
      : [...this.ids, id];
    this.#anchor = id;
  }

  /** Selects from the last clicked task to `id`, both included. */
  extendTo(id: string): void {
    const order = this.#deps.order();
    const from = this.#anchor === null ? -1 : order.indexOf(this.#anchor);
    const to = order.indexOf(id);
    if (from < 0 || to < 0) {
      this.toggle(id);
      return;
    }
    const range = order.slice(Math.min(from, to), Math.max(from, to) + 1);
    this.ids = [...new SvelteSet([...this.ids, ...range])];
  }

  clear(): void {
    this.ids = [];
    this.#anchor = null;
  }

  // ---- actions --------------------------------------------------------------------------

  async complete(): Promise<void> {
    const completed = !this.allDone;
    const date = completed ? this.#deps.today() : undefined;
    const targets = this.#chosen().filter(
      (task) => task.isCompleted !== completed,
    );
    const done = await this.#apply(targets, (task) =>
      this.#deps.api.taskSetCompleted(task.id, completed, date),
    );
    if (done.length === 0) return;
    this.#remember(
      completed
        ? t.batch.completed(done.length)
        : t.batch.reopened(done.length),
      async () => {
        for (const { before, after } of done) {
          await this.#deps.api.taskSetCompleted(before.id, !completed);
          if (before.dueDate !== after.dueDate) {
            await this.#deps.api.taskUpdate(before.id, schedule(before));
          }
        }
      },
    );
  }

  async addTag(tagId: string): Promise<void> {
    const targets = this.#chosen().filter(
      (task) => !task.tagIds.includes(tagId),
    );
    const done = await this.#apply(targets, (task) =>
      this.#deps.api.taskTagsSet(task.id, [...task.tagIds, tagId]),
    );
    if (done.length === 0) return;
    this.#remember(t.batch.tagged(done.length), async () => {
      for (const { before } of done) {
        await this.#deps.api.taskTagsSet(before.id, before.tagIds);
      }
    });
  }

  /** Moves the tasks to a day, or to POR HACER with `null`. */
  async moveTo(date: string | null): Promise<void> {
    const targets = this.#chosen().filter((task) => task.dueDate !== date);
    const done = await this.#apply(targets, (task) =>
      this.#deps.api.taskUpdate(
        task.id,
        date === null ? { dueDate: null, dueTime: null } : { dueDate: date },
      ),
    );
    if (done.length === 0) return;
    this.#remember(t.batch.moved(done.length), async () => {
      for (const { before } of done) {
        await this.#deps.api.taskUpdate(before.id, schedule(before));
      }
    });
  }

  async remove(): Promise<void> {
    const targets = this.#chosen();
    const done = await this.#apply(targets, async (task) => {
      await this.#deps.api.taskDelete(task.id);
      return task;
    });
    if (done.length === 0) return;
    this.#remember(t.batch.deleted(done.length), async () => {
      for (const { before } of done) {
        await this.#deps.api.taskRestore(before.id);
      }
    });
  }

  async undo(): Promise<void> {
    const entry = this.entry;
    if (!entry) return;
    this.#dropEntry();
    try {
      await entry.undo();
    } catch (error) {
      this.#deps.report(error);
    }
    await this.#deps.refresh();
  }

  dismiss(): void {
    this.#dropEntry();
  }

  dispose(): void {
    this.#dropEntry();
  }

  // ---- internals ------------------------------------------------------------------------

  #chosen(): Task[] {
    const wanted = new SvelteSet(this.ids);
    return this.#deps.tasks().filter((task) => wanted.has(task.id));
  }

  /** Runs `action` on each task; keeps going after a failure and reports how many failed. */
  async #apply(
    targets: Task[],
    action: (task: Task) => Promise<Task>,
  ): Promise<{ before: Task; after: Task }[]> {
    const done: { before: Task; after: Task }[] = [];
    let failed = 0;
    let lastError: unknown;
    for (const before of targets) {
      try {
        done.push({ before, after: await action(before) });
      } catch (error) {
        failed += 1;
        lastError = error;
      }
    }
    this.clear();
    await this.#deps.refresh();
    if (failed > 0) {
      if (failed === 1) this.#deps.report(lastError);
      else this.#deps.inform(t.batch.partial(failed));
    }
    return done;
  }

  #remember(message: string, undo: () => Promise<void>): void {
    this.#dropEntry();
    const key = ++this.#keys;
    this.entry = { key, message, undo };
    this.#timer = setTimeout(() => {
      if (this.entry?.key === key) this.entry = null;
    }, this.#deps.undoMs);
  }

  #dropEntry(): void {
    if (this.#timer !== null) clearTimeout(this.#timer);
    this.#timer = null;
    this.entry = null;
  }
}

/** The day and time a task had, as a patch that puts them back. */
function schedule(task: Task) {
  return task.dueDate === null
    ? { dueDate: null, dueTime: null }
    : { dueDate: task.dueDate, dueTime: task.dueTime };
}
