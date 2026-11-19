// The two suggestion lists of the note editor: `#tag` and `[[note]]`. Each owns the text being
// typed, the highlighted row and the keys that drive the list; the component only draws them.

import type { Tag } from "../../types";
import type { NoteEditorHandle, TagQuery } from "./editorTypes";
import { matchTags } from "./tagQuery";

type EditorHandle = () => NoteEditorHandle | null;

export interface TagSuggestSource {
  tags: Tag[];
  onPick: (tag: Tag) => void;
}

export class TagSuggest {
  query = $state.raw<TagQuery | null>(null);
  highlighted = $state(0);
  // Escape hides the list until the text after the `#` changes.
  dismissedFor = $state<string | null>(null);

  readonly #source: () => TagSuggestSource | undefined;
  readonly #handle: EditorHandle;

  constructor(
    source: () => TagSuggestSource | undefined,
    handle: EditorHandle,
  ) {
    this.#source = source;
    this.#handle = handle;
  }

  suggestions = $derived.by(() => {
    const source = this.#source();
    return source && this.query && this.dismissedFor !== this.query.query
      ? matchTags(source.tags, this.query.query)
      : [];
  });

  /** The editor reports the `#text` being typed, or `null` when there is none. */
  setQuery = (next: TagQuery | null): void => {
    this.query = next;
    this.highlighted = 0;
  };

  pick(tag: Tag): void {
    this.#handle()?.removeTagQuery();
    this.#source()?.onPick(tag);
  }

  /** First look at a key while a `#text` is open; returns true when it was used. */
  onKey = (event: KeyboardEvent): boolean => {
    const list = this.suggestions;
    if (list.length === 0 || event.ctrlKey || event.metaKey) return false;
    if (event.key === "ArrowDown") {
      this.highlighted = (this.highlighted + 1) % list.length;
      return true;
    }
    if (event.key === "ArrowUp") {
      this.highlighted = (this.highlighted - 1 + list.length) % list.length;
      return true;
    }
    if (event.key === "Enter" || event.key === "Tab") {
      const chosen = list[this.highlighted];
      if (chosen) this.pick(chosen);
      return true;
    }
    if (event.key === "Escape") {
      this.dismissedFor = this.query?.query ?? null;
      return true;
    }
    return false;
  };
}

export type LinkChoice = { id: string; title: string } | { create: string };

export interface LinkSuggestSource {
  search: (text: string) => Promise<{ id: string; title: string }[]>;
  /** Makes a note on the spot and resolves with its id. */
  create: (title: string) => Promise<string | null>;
}

/** The notes that match what is typed, plus "create it" when none has that title. */
export class LinkSuggest {
  query = $state.raw<TagQuery | null>(null);
  choices = $state.raw<LinkChoice[]>([]);
  highlighted = $state(0);
  dismissedFor = $state<string | null>(null);
  // Answers can arrive out of order; only the latest request may update the list.
  #token = 0;

  readonly #source: () => LinkSuggestSource | undefined;
  readonly #handle: EditorHandle;

  constructor(
    source: () => LinkSuggestSource | undefined,
    handle: EditorHandle,
  ) {
    this.#source = source;
    this.#handle = handle;
  }

  visible = $derived(
    this.query !== null &&
      this.dismissedFor !== this.query.query &&
      this.choices.length > 0,
  );

  /** The editor reports the `[[text` being typed, or `null` when there is none. */
  setQuery = (next: TagQuery | null): void => {
    this.query = next;
    this.highlighted = 0;
    void this.#refresh(next ? next.query : null);
  };

  async #refresh(text: string | null): Promise<void> {
    const token = ++this.#token;
    const source = this.#source();
    if (text === null || !source) {
      this.choices = [];
      return;
    }
    let found: { id: string; title: string }[];
    try {
      found = await source.search(text);
    } catch {
      found = [];
    }
    if (token !== this.#token) return;
    const wanted = text.trim();
    const exact = found.some(
      (item) => item.title.trim().toLowerCase() === wanted.toLowerCase(),
    );
    this.choices = [
      ...found,
      ...(wanted && !exact ? [{ create: wanted }] : []),
    ];
  }

  async pick(choice: LinkChoice): Promise<void> {
    if ("create" in choice) {
      const id = await this.#source()?.create(choice.create);
      if (id) this.#handle()?.insertNoteLink(id);
    } else {
      this.#handle()?.insertNoteLink(choice.id);
    }
  }

  /** First look at a key while a `[[text` is open; returns true when it was used. */
  onKey = (event: KeyboardEvent): boolean => {
    if (!this.visible || event.ctrlKey || event.metaKey) return false;
    const count = this.choices.length;
    if (event.key === "ArrowDown") {
      this.highlighted = (this.highlighted + 1) % count;
      return true;
    }
    if (event.key === "ArrowUp") {
      this.highlighted = (this.highlighted - 1 + count) % count;
      return true;
    }
    if (event.key === "Enter" || event.key === "Tab") {
      const chosen = this.choices[this.highlighted];
      if (chosen) void this.pick(chosen);
      return true;
    }
    if (event.key === "Escape") {
      this.dismissedFor = this.query?.query ?? null;
      return true;
    }
    return false;
  };
}
