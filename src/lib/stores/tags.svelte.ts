import { SvelteSet } from "svelte/reactivity";

import * as backend from "../api";
import { TAG_PRESETS } from "../tagColor";
import type { Tag, TagUpsertInput, TagWithCount, Task } from "../types";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface TagsApi {
  tagList(): Promise<TagWithCount[]>;
  tagUpsert(input: TagUpsertInput): Promise<Tag>;
  tagDelete(id: string): Promise<void>;
}

/** Filter key of the tasks that have no tag. Tag ids never take this form. */
export const NO_TAG = "none";

/**
 * The tags (always in the order the backend keeps them) and which of them the views show.
 * A task has at most one tag in the UI (D-34): the first of its `tagIds`.
 */
export class TagsStore {
  tags = $state.raw<Tag[]>([]);
  /** Filter keys the user has unchecked; empty means everything is shown. */
  readonly hidden = new SvelteSet<string>();
  status = $state<"idle" | "ready" | "error">("idle");

  readonly #api: TagsApi;

  // Rebuilt, never mutated: a plain Map is enough.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  byId = $derived(new Map(this.tags.map((tag) => [tag.id, tag])));
  /** True while any group is unchecked, so the toolbar can warn that the view is filtered. */
  filtered = $derived(this.hidden.size > 0);

  constructor(api: TagsApi = backend) {
    this.#api = api;
  }

  async load(): Promise<void> {
    try {
      this.tags = await this.#api.tagList();
      this.status = "ready";
    } catch {
      this.status = "error";
    }
  }

  tagOf(task: Pick<Task, "tagIds">): Tag | undefined {
    const id = task.tagIds[0];
    return id === undefined ? undefined : this.byId.get(id);
  }

  isVisible(task: Pick<Task, "tagIds">): boolean {
    const tag = this.tagOf(task);
    return !this.hidden.has(tag ? tag.id : NO_TAG);
  }

  isChecked(key: string): boolean {
    return !this.hidden.has(key);
  }

  toggle(key: string): void {
    if (this.hidden.has(key)) this.hidden.delete(key);
    else this.hidden.add(key);
  }

  showAll(): void {
    this.hidden.clear();
  }

  hideAll(): void {
    this.hidden.clear();
    this.hidden.add(NO_TAG);
    for (const tag of this.tags) this.hidden.add(tag.id);
  }

  /** Shows only `key`. */
  only(key: string): void {
    this.hideAll();
    this.hidden.delete(key);
  }

  /** The colour a new tag starts with: the first preset not used yet, else cycling. */
  nextColor(): string {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const used = new Set(this.tags.map((tag) => tag.colorHex.toUpperCase()));
    return (
      TAG_PRESETS.find((color) => !used.has(color)) ??
      TAG_PRESETS[this.tags.length % TAG_PRESETS.length] ??
      "#4C8DFF"
    );
  }

  /** Creates or edits a tag. Returns why it was refused, or `null` when it was saved. */
  async save(input: TagUpsertInput): Promise<"duplicate" | "failed" | null> {
    try {
      const saved = await this.#api.tagUpsert(input);
      const exists = this.tags.some((tag) => tag.id === saved.id);
      this.tags = exists
        ? this.tags.map((tag) => (tag.id === saved.id ? saved : tag))
        : [...this.tags, saved];
      return null;
    } catch (error) {
      return backend.parseError(error).code === "CONFLICT"
        ? "duplicate"
        : "failed";
    }
  }

  /** Deletes the tag. Its tasks lose it, so the caller reloads them. */
  async remove(id: string): Promise<boolean> {
    try {
      await this.#api.tagDelete(id);
    } catch {
      return false;
    }
    this.tags = this.tags.filter((tag) => tag.id !== id);
    this.hidden.delete(id);
    return true;
  }
}

export const tagsStore = new TagsStore();
