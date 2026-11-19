// Completing and reopening a task, with the exit animation of the row. Composed into `TasksStore`.

import { SvelteSet } from "svelte/reactivity";

import { toInstant, toLocalIsoDate } from "../dates";
import type { Task } from "../types";
import type { TasksHost } from "./tasksHost";

export class TaskCompletion {
  /** Rows playing their exit animation; per-key reactivity re-renders only the row concerned. */
  readonly settling = new SvelteSet<string>();

  readonly #host: TasksHost;
  readonly #settleMs: () => number;

  constructor(host: TasksHost, settleMs: () => number) {
    this.#host = host;
    this.#settleMs = settleMs;
  }

  /**
   * Completing plays the exit animation first, then the row moves to "Completadas" without
   * waiting for the backend; the backend's answer replaces the local guess, and a refusal
   * puts the task back as it was.
   */
  async set(id: string, completed: boolean): Promise<void> {
    const host = this.#host;
    const before = host.find(id);
    if (!before || before.isCompleted === completed || this.settling.has(id)) {
      return;
    }
    // The row changes place; closing the notes first lets the editor save before it remounts.
    host.suspendNotes(id);
    // D-23: a completed task always sits in the "Completadas" of the day it was completed.
    const completedDate = completed ? toLocalIsoDate(host.now()) : undefined;
    const guess: Task = {
      ...before,
      dueDate: completedDate ?? before.dueDate,
      isCompleted: completed,
      completedAt: completed ? toInstant(host.now()) : null,
    };
    const answer: { task?: Task; failure?: { error: unknown } } = {};
    // Read through a function: the answer is filled in by callbacks TypeScript cannot see.
    const answered = () => answer;
    this.settling.add(id);
    const request = host.api
      .taskSetCompleted(id, completed, completedDate)
      .then(
        (task) => {
          answer.task = task;
        },
        (error: unknown) => {
          answer.failure = { error };
        },
      );
    await new Promise((resolve) => setTimeout(resolve, this.#settleMs()));
    this.settling.delete(id);
    const early = answered().failure;
    if (early) {
      host.report(early.error);
      return;
    }
    host.replace(guess);
    await request;
    const { task, failure } = answered();
    if (task) {
      host.replace(task);
      // Completing a repeating task created the next one behind the interface's back.
      if (completed && before.rrule) await host.refresh();
    } else if (failure) {
      host.replace(before);
      host.report(failure.error);
    }
  }
}
