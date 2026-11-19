// Titles of the notes known by id, which ones are gone, and the link queries (backlinks, orphans,
// suggestions). Composed into `NotesStore`; the components reach it through the store.

import { SvelteMap, SvelteSet } from "svelte/reactivity";

import { t } from "../i18n";
import { plainQuery } from "../features/notes/linkQuery";
import type { Note, NoteSummary } from "../types";
import type { NotesHost } from "./notesHost";

export class NoteLinks {
  /** Titles of notes tasks come from, so a task can name its note without opening it. */
  readonly titles = new SvelteMap<string, string>();

  /** Notes known to be in the trash or gone: links to them show as removed. */
  readonly missing = new SvelteSet<string>();

  readonly #host: NotesHost;

  constructor(host: NotesHost) {
    this.#host = host;
  }

  titleOf(id: string): string | undefined {
    return this.titles.get(id);
  }

  /** The title a link shows: `null` for a removed note, `undefined` while it is unknown. */
  linkTitle(id: string): string | null | undefined {
    if (this.missing.has(id)) return null;
    return this.titles.get(id);
  }

  /** Notes to offer when a link is being typed: the title (or, with no text, the recent ones). */
  async suggest(
    text: string,
    limit = 6,
  ): Promise<{ id: string; title: string }[]> {
    const found = await this.#host.api.searchAll(
      `is:nota ${plainQuery(text)}`,
      limit,
    );
    return found.flatMap((item) =>
      item.kind === "note" ? [{ id: item.noteId, title: item.title }] : [],
    );
  }

  /** A note made while linking to it: it is shown in the list but the open note stays open. */
  async createLinkedNote(
    title: string,
    folderId: string | null,
  ): Promise<Note | null> {
    try {
      const note = await this.#host.api.noteCreate({
        title,
        ...(folderId ? { folderId } : {}),
      });
      this.titles.set(note.id, note.title);
      this.missing.delete(note.id);
      await this.#host.refreshAll();
      return note;
    } catch {
      this.#host.fail(t.notesApp.linkCreateFailed);
      return null;
    }
  }

  /** Notes that link to this one. */
  backlinks(id: string): Promise<NoteSummary[]> {
    return this.#host.api.noteBacklinks(id);
  }

  /** Notes without links to or from any other. */
  orphans(): Promise<NoteSummary[]> {
    return this.#host.api.notesOrphans();
  }

  /** Looks up, in one batch, the titles of notes not known yet; unknown or removed notes stay unknown. */
  async ensureTitles(ids: readonly string[]): Promise<void> {
    const unknown = ids.filter(
      (id, at) => ids.indexOf(id) === at && !this.titles.has(id),
    );
    if (unknown.length === 0) return;
    try {
      const found = await this.#host.api.noteTitles(unknown);
      const live = new SvelteSet<string>();
      for (const { id, title } of found) {
        this.titles.set(id, title);
        this.missing.delete(id);
        live.add(id);
      }
      // A trashed or purged note has no title to show: its links read as removed.
      for (const id of unknown) if (!live.has(id)) this.missing.add(id);
    } catch {
      // Without an answer nothing is known about them; they are asked for again next time.
    }
  }
}
