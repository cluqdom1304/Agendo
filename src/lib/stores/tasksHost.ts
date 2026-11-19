import type { Task } from "../types";
import type { TasksApi } from "./tasks.svelte";

/** What the pieces composed into `TasksStore` need from it, so none of them imports the store. */
export interface TasksHost {
  readonly api: TasksApi;
  tasks(): readonly Task[];
  setTasks(tasks: Task[]): void;
  find(id: string): Task | undefined;
  /** Swaps the task with the same id; ignored when the list does not hold it. */
  replace(task: Task): void;
  /** Puts a task back in the list unless it is there. */
  restoreLocally(task: Task): void;
  /** Clears the selection when it is on `id`. */
  deselect(id: string): void;
  report(error: unknown): void;
  inform(message: string): void;
  /** Warns when the tag filter hides a task that was just saved. */
  warnIfHidden(task: Task): void;
  now(): Date;
  /** Reads everything again; resolves `false` when the read failed. */
  refresh(): Promise<boolean>;
  /** Closes the notes of a task for an instant so its editor flushes before the row remounts. */
  suspendNotes(id: string): void;
}
