// The transient message of the task views (an error or a hint). Composed into `TasksStore`.

import * as backend from "../api";
import { t } from "../i18n";
import type { Notice } from "./tasksTypes";

function describeError(raw: unknown): string {
  const { code, message } = backend.parseError(raw);
  const base = t.errors[code];
  return code === "VALIDATION" && message ? `${base}: ${message}` : base;
}

export class TaskNotices {
  notice = $state.raw<Notice | null>(null);

  readonly #ms: number;
  #keys = 0;
  #timer: ReturnType<typeof setTimeout> | null = null;

  constructor(ms: number) {
    this.#ms = ms;
  }

  report(error: unknown): void {
    this.#show("error", describeError(error));
  }

  inform(message: string): void {
    this.#show("info", message);
  }

  dismiss(): void {
    if (this.#timer !== null) clearTimeout(this.#timer);
    this.#timer = null;
    this.notice = null;
  }

  #show(kind: Notice["kind"], message: string): void {
    this.dismiss();
    const key = ++this.#keys;
    this.notice = { key, message, kind };
    this.#timer = setTimeout(() => {
      if (this.notice?.key === key) this.notice = null;
    }, this.#ms);
  }
}
