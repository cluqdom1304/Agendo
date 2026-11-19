// The reminder panel of a task and what happens when a reminder fires. Composed into `TasksStore`.

import { t } from "../i18n";
import type { ReminderInput } from "../types";
import type { TasksHost } from "./tasksHost";

export class TaskReminders {
  /** The task whose reminder panel is open under its row. */
  openId = $state<string | null>(null);

  readonly #host: TasksHost;

  constructor(host: TasksHost) {
    this.#host = host;
  }

  toggle(id: string): void {
    this.openId = this.openId === id ? null : id;
  }

  /** Replaces the task's unsent reminders; the panel shows what the backend stored. */
  async set(id: string, reminders: ReminderInput[]): Promise<void> {
    try {
      this.#host.replace(await this.#host.api.remindersSet(id, reminders));
    } catch (error) {
      this.#host.report(error);
    }
  }

  /** The scheduler delivered a reminder: refresh the task (it is now sent) and tell the user. */
  async onFired(taskId: string): Promise<void> {
    if (!(await this.#host.refresh())) return;
    const title = this.#host.find(taskId)?.title.trim();
    this.#host.inform(t.reminders.fired(title || t.reminders.untitled));
  }
}
