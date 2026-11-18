// Searches the user keeps in the palette (F17-08): the text of the query, filters included
// (`is:pendiente tag:SMA`). They live in one setting as a JSON list.

import * as backend from "../api";

export const MAX_SAVED = 20;
export const MAX_SAVED_CHARS = 200;

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface SavedSearchesApi {
  settingsGet(key: "saved_searches"): Promise<string | null>;
  settingsSet(key: "saved_searches", value: string): Promise<void>;
}

export class SavedSearches {
  items = $state.raw<string[]>([]);

  readonly #api: SavedSearchesApi;

  constructor(api: SavedSearchesApi = backend) {
    this.#api = api;
  }

  async load(): Promise<void> {
    try {
      const stored = await this.#api.settingsGet("saved_searches");
      const parsed: unknown = stored ? JSON.parse(stored) : [];
      this.items = Array.isArray(parsed)
        ? parsed.filter((item): item is string => typeof item === "string")
        : [];
    } catch {
      this.items = [];
    }
  }

  has(query: string): boolean {
    return this.items.includes(query.trim());
  }

  /** Whether a query can be saved: not empty, not repeated and within the limits. */
  canSave(query: string): boolean {
    const text = query.trim();
    return (
      text !== "" &&
      !text.startsWith(">") &&
      text.length <= MAX_SAVED_CHARS &&
      this.items.length < MAX_SAVED &&
      !this.has(text)
    );
  }

  async add(query: string): Promise<boolean> {
    if (!this.canSave(query)) return false;
    return this.#write([...this.items, query.trim()]);
  }

  async remove(query: string): Promise<boolean> {
    return this.#write(this.items.filter((item) => item !== query));
  }

  async #write(next: string[]): Promise<boolean> {
    const before = this.items;
    this.items = next;
    try {
      await this.#api.settingsSet("saved_searches", JSON.stringify(next));
      return true;
    } catch {
      this.items = before;
      return false;
    }
  }
}

export const savedSearches = new SavedSearches();
