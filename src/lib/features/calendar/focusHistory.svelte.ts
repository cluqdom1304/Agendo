// The focus sessions (D-102) of the days the calendar shows. Read only.
import * as backend from "../../api";
import type { TimeEntry } from "../../types";

/** The slice of `api.ts` the history uses; tests pass a fake. */
export interface FocusHistoryApi {
  focusList(from: string, to: string): Promise<TimeEntry[]>;
}

export interface FocusHistoryOptions {
  api?: FocusHistoryApi;
  onError?: (error: unknown) => void;
}

export class FocusHistory {
  entries = $state.raw<TimeEntry[]>([]);

  readonly #api: FocusHistoryApi;
  readonly #onError: (error: unknown) => void;
  #request = 0;

  constructor(options: FocusHistoryOptions = {}) {
    this.#api = options.api ?? backend;
    this.#onError = options.onError ?? (() => undefined);
  }

  /** Loads the sessions between two local days, inclusive. The last request wins. */
  async show(from: string, to: string): Promise<void> {
    const request = ++this.#request;
    try {
      const list = await this.#api.focusList(from, to);
      if (request === this.#request) this.entries = list;
    } catch (error) {
      if (request === this.#request) this.#onError(error);
    }
  }
}
