// The draft row and the ways of creating a task. Composed into `TasksStore`.

import type { Task } from "../types";
import type { TasksHost } from "./tasksHost";
import type { Draft, DraftTarget, NewTaskDetails } from "./tasksTypes";

function isSameTarget(a: DraftTarget, b: DraftTarget): boolean {
  return a.section === "todo"
    ? b.section === "todo"
    : b.section === "agenda" && b.date === a.date;
}

export class TaskCreator {
  draft = $state.raw<Draft | null>(null);

  readonly #host: TasksHost;
  #keys = 0;
  #creating = false;

  constructor(host: TasksHost) {
    this.#host = host;
  }

  startDraft(target: DraftTarget): void {
    this.draft = { target, key: ++this.#keys };
  }

  cancelDraft(key: number): void {
    if (this.draft?.key === key) this.draft = null;
  }

  isDraftFor(target: DraftTarget): boolean {
    return this.draft !== null && isSameTarget(this.draft.target, target);
  }

  /**
   * Saves the draft `key`. With `advance` (Enter) a fresh draft opens in the same block so
   * several tasks can be typed in a row. An empty title just closes the draft. If saving
   * fails the draft stays open with its text.
   */
  async commitDraft(
    key: number,
    rawTitle: string,
    advance: boolean,
  ): Promise<void> {
    const draft = this.draft;
    if (!draft || draft.key !== key || this.#creating) return;
    const title = rawTitle.trim();
    if (!title) {
      this.draft = null;
      return;
    }
    this.#creating = true;
    try {
      const created = await this.#host.api.taskCreate({
        title,
        ...(draft.target.section === "agenda"
          ? { dueDate: draft.target.date }
          : {}),
      });
      this.#host.setTasks([...this.#host.tasks(), created]);
      this.draft = advance ? { target: draft.target, key: ++this.#keys } : null;
    } catch (error) {
      this.#host.report(error);
    } finally {
      this.#creating = false;
    }
  }

  /** Creates a task already placed on a day and, optionally, a time (calendar quick creation). */
  async createScheduled(
    rawTitle: string,
    dueDate: string,
    dueTime?: string,
    durationMinutes?: number,
  ): Promise<boolean> {
    const title = rawTitle.trim();
    if (!title || this.#creating) return false;
    this.#creating = true;
    try {
      const created = await this.#host.api.taskCreate({
        title,
        dueDate,
        ...(dueTime ? { dueTime } : {}),
        ...(dueTime && durationMinutes ? { durationMinutes } : {}),
      });
      this.#host.setTasks([...this.#host.tasks(), created]);
      this.#host.warnIfHidden(created);
      return true;
    } catch (error) {
      this.#host.report(error);
      return false;
    } finally {
      this.#creating = false;
    }
  }

  /** Creates a task with everything the creation dialog collects; hands back the task. */
  async create(input: NewTaskDetails): Promise<Task | null> {
    const host = this.#host;
    const title = input.title.trim();
    if (!title || this.#creating) return null;
    this.#creating = true;
    try {
      let created = await host.api.taskCreate({
        title,
        ...(input.dueDate ? { dueDate: input.dueDate } : {}),
        ...(input.dueDate && input.dueTime ? { dueTime: input.dueTime } : {}),
        ...(input.dueDate && input.dueTime && input.durationMinutes
          ? { durationMinutes: input.durationMinutes }
          : {}),
        ...(input.tagIds && input.tagIds.length > 0
          ? { tagIds: input.tagIds }
          : {}),
        ...(input.contentJson ? { contentJson: input.contentJson } : {}),
        ...(input.sourceNoteId ? { sourceNoteId: input.sourceNoteId } : {}),
        ...(input.dueDate && input.rrule ? { rrule: input.rrule } : {}),
        ...(input.priority ? { priority: input.priority } : {}),
      });
      host.setTasks([...host.tasks(), created]);
      host.warnIfHidden(created);
      if (input.reminders && input.reminders.length > 0) {
        try {
          created = await host.api.remindersSet(created.id, input.reminders);
          host.replace(created);
        } catch (error) {
          host.report(error);
        }
      }
      return created;
    } catch (error) {
      host.report(error);
      return null;
    } finally {
      this.#creating = false;
    }
  }
}
