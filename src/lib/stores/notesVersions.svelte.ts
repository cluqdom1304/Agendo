// History of a note (versions to list, read and put back). Composed into `NotesStore`.

import { t } from "../i18n";
import type { NoteVersion, NoteVersionSummary } from "../types";
import type { NotesHost } from "./notesHost";
import type { NoteLinks } from "./notesLinks.svelte";

export class NoteVersions {
  readonly #host: NotesHost;
  readonly #links: NoteLinks;

  constructor(host: NotesHost, links: NoteLinks) {
    this.#host = host;
    this.#links = links;
  }

  async list(noteId: string): Promise<NoteVersionSummary[]> {
    try {
      return await this.#host.api.noteVersionsList(noteId);
    } catch {
      this.#host.fail(t.notesApp.history.failed);
      return [];
    }
  }

  async get(id: string): Promise<NoteVersion | null> {
    try {
      return await this.#host.api.noteVersionGet(id);
    } catch {
      this.#host.fail(t.notesApp.history.failed);
      return null;
    }
  }

  /** Puts a version back in its note; the editor is rebuilt with the restored text. */
  async restore(id: string): Promise<boolean> {
    try {
      const restored = await this.#host.api.noteVersionRestore(id);
      this.#links.titles.set(restored.id, restored.title);
      this.#host.adopt(restored);
      this.#host.bumpRevision();
      await this.#host.refreshList().catch(() => undefined);
      this.#host.inform(t.notesApp.history.restored);
      return true;
    } catch {
      this.#host.fail(t.notesApp.history.failed);
      return false;
    }
  }
}
