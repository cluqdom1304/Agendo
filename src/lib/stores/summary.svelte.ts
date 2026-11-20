// The weekly summary (F26-02): the week being looked at and its figures. The last request wins,
// so quickly paging through weeks never shows an older one.
import * as backend from "../api";
import { shiftWeek } from "../features/summary/summary";
import type { WeekSummary } from "../types";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface SummaryApi {
  summaryWeek(date: string, firstDay: number): Promise<WeekSummary>;
  summaryExportMarkdown(
    date: string,
    firstDay: number,
    path: string,
  ): Promise<void>;
}

export interface SummaryStoreOptions {
  api?: SummaryApi;
  /** Local `YYYY-MM-DD` of today. */
  today: () => string;
  /** First day of the week, 0 (Sunday) to 6 (Saturday). */
  firstDay: () => number;
}

export class SummaryStore {
  /** A day of the week being shown. */
  anchor = $state("");
  summary = $state.raw<WeekSummary | null>(null);
  status = $state<"idle" | "loading" | "ready" | "error">("idle");
  /** What happened with the last export, for the live region. */
  exportState = $state<"idle" | "done" | "error">("idle");
  error = $state<string | null>(null);

  readonly #api: SummaryApi;
  readonly #today: () => string;
  readonly #firstDay: () => number;
  #request = 0;

  constructor(options: SummaryStoreOptions) {
    this.#api = options.api ?? backend;
    this.#today = options.today;
    this.#firstDay = options.firstDay;
  }

  /** Whether the week shown is the one that contains today. */
  get current(): boolean {
    const today = this.#today();
    return (
      this.summary !== null &&
      today >= this.summary.start &&
      today <= this.summary.end
    );
  }

  /** Opens the week of today. */
  show(): Promise<void> {
    this.anchor = this.#today();
    return this.#load();
  }

  /** Moves `weeks` weeks (negative goes back). */
  go(weeks: number): Promise<void> {
    this.anchor = shiftWeek(this.anchor || this.#today(), weeks);
    return this.#load();
  }

  refresh(): Promise<void> {
    return this.anchor ? this.#load() : this.show();
  }

  async #load(): Promise<void> {
    const request = ++this.#request;
    this.status = "loading";
    this.exportState = "idle";
    try {
      const summary = await this.#api.summaryWeek(
        this.anchor,
        this.#firstDay(),
      );
      if (request !== this.#request) return;
      this.summary = summary;
      this.error = null;
      this.status = "ready";
    } catch (error) {
      if (request !== this.#request) return;
      this.error = String(error);
      this.status = "error";
    }
  }

  /** Writes the week shown as Markdown into `path`. Returns whether it worked. */
  async exportMarkdown(path: string): Promise<boolean> {
    try {
      await this.#api.summaryExportMarkdown(
        this.anchor || this.#today(),
        this.#firstDay(),
        path,
      );
      this.exportState = "done";
      this.error = null;
      return true;
    } catch (error) {
      this.exportState = "error";
      this.error = String(error);
      return false;
    }
  }
}
