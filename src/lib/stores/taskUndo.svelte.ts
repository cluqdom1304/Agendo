// Deleting a task and bringing it back (F-02). Composed into `TasksStore`.

import type { Task } from "../types";
import type { TasksHost } from "./tasksHost";
import type { UndoEntry } from "./tasksTypes";

export class TaskUndo {
  stack = $state.raw<UndoEntry[]>([]);

  readonly #host: TasksHost;
  readonly #ms: number;
  #keys = 0;
  // Timer handles are bookkeeping, not UI state.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  #timers = new Map<number, ReturnType<typeof setTimeout>>();

  constructor(host: TasksHost, ms: number) {
    this.#host = host;
    this.#ms = ms;
  }

  /** The task leaves the list at once; for `ms` it can be brought back. */
  async remove(id: string): Promise<void> {
    const task = this.#host.find(id);
    if (!task) return;
    this.#host.setTasks(this.#host.tasks().filter((other) => other.id !== id));
    this.#host.deselect(id);
    try {
      await this.#host.api.taskDelete(id);
    } catch (error) {
      this.#host.restoreLocally(task);
      this.#host.report(error);
      return;
    }
    const entry: UndoEntry = { key: ++this.#keys, task };
    this.stack = [...this.stack, entry];
    this.#timers.set(
      entry.key,
      setTimeout(() => this.#expire(entry.key), this.#ms),
    );
  }

  /**
   * Brings back a task that is in the trash, for a note that still points at it. Resolves
   * `false` when it is gone for good (purged after 30 days).
   */
  async restoreById(id: string): Promise<boolean> {
    if (this.#host.find(id)) return true;
    try {
      this.#host.restoreLocally(await this.#host.api.taskRestore(id));
      return true;
    } catch {
      return false;
    }
  }

  /** Brings back the most recently deleted task. */
  async undo(): Promise<void> {
    const entry = this.stack.at(-1);
    if (!entry) return;
    this.#expire(entry.key);
    this.#host.restoreLocally(entry.task);
    try {
      this.#host.replace(await this.#host.api.taskRestore(entry.task.id));
    } catch (error) {
      this.#host.setTasks(
        this.#host.tasks().filter((task: Task) => task.id !== entry.task.id),
      );
      this.#host.report(error);
    }
  }

  dispose(): void {
    for (const timer of this.#timers.values()) clearTimeout(timer);
    this.#timers.clear();
  }

  #expire(key: number): void {
    const timer = this.#timers.get(key);
    if (timer !== undefined) clearTimeout(timer);
    this.#timers.delete(key);
    this.stack = this.stack.filter((entry) => entry.key !== key);
  }
}
