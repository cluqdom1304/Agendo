// The reminder of a note (F13). Composed into `NotesStore`.

import { parseError } from "../api";
import { t } from "../i18n";
import type { NoteReminder } from "../types";
import type { NotesHost } from "./notesHost";
import type { NoteLinks } from "./notesLinks.svelte";

export class NoteReminders {
  readonly #host: NotesHost;
  readonly #links: NoteLinks;

  constructor(host: NotesHost, links: NoteLinks) {
    this.#host = host;
    this.#links = links;
  }

  async get(noteId: string): Promise<NoteReminder | null> {
    try {
      return await this.#host.api.noteReminderGet(noteId);
    } catch {
      this.#host.fail(t.notesApp.reminder.failed);
      return null;
    }
  }

  /** Sets (`notifyAt` as a UTC instant) or clears (`null`) the reminder; returns what is set. */
  async set(
    noteId: string,
    notifyAt: string | null,
  ): Promise<{ ok: boolean; reminder: NoteReminder | null }> {
    try {
      return {
        ok: true,
        reminder: await this.#host.api.noteReminderSet(noteId, notifyAt),
      };
    } catch (error) {
      this.#host.fail(parseError(error).message || t.notesApp.reminder.failed);
      return { ok: false, reminder: null };
    }
  }

  /** The scheduler delivered the reminder of a note: say so in the notes section. */
  async onFired(noteId: string): Promise<void> {
    let title = this.#links.titles.get(noteId);
    if (title === undefined) {
      try {
        title = (await this.#host.api.noteGet(noteId)).title;
      } catch {
        return;
      }
    }
    this.#host.inform(
      t.notesApp.reminder.fired(title.trim() || t.notesApp.untitled),
    );
  }
}
