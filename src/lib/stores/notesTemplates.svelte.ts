// Templates for new notes (F13-02): loaded when the dialog opens. Composed into `NotesStore`.

import { parseError } from "../api";
import { t } from "../i18n";
import type { Note, NoteTemplate } from "../types";
import type { NotesHost } from "./notesHost";

export class NoteTemplates {
  items = $state.raw<NoteTemplate[]>([]);
  isOpen = $state(false);

  readonly #host: NotesHost;

  constructor(host: NotesHost) {
    this.#host = host;
  }

  async openDialog(): Promise<void> {
    this.isOpen = true;
    await this.load();
  }

  async load(): Promise<void> {
    try {
      this.items = await this.#host.api.templateList();
    } catch {
      this.#host.fail(t.notesApp.templatesFailed);
    }
  }

  /** Saves the open note as a template, or rewrites `id` with it. */
  async saveOpen(name: string, id?: string): Promise<boolean> {
    const open = this.#host.openNote();
    return this.#save({
      name,
      ...(id ? { id } : {}),
      ...(open ? { contentJson: open.contentJson } : {}),
    });
  }

  async rename(id: string, name: string): Promise<boolean> {
    return this.#save({ id, name });
  }

  async remove(id: string): Promise<void> {
    try {
      await this.#host.api.templateDelete(id);
      await this.load();
    } catch {
      this.#host.fail(t.notesApp.templatesFailed);
    }
  }

  /** A new note that starts as the template, shown and opened. */
  async createNote(id: string): Promise<Note | null> {
    const template = this.items.find((item) => item.id === id);
    if (!template) return null;
    this.isOpen = false;
    return this.#host.createNoteWith({
      title: template.name,
      contentJson: template.contentJson,
    });
  }

  async #save(input: {
    id?: string;
    name: string;
    contentJson?: string;
  }): Promise<boolean> {
    try {
      await this.#host.api.templateSave(input);
      await this.load();
      return true;
    } catch (error) {
      const message = parseError(error).message;
      this.#host.fail(message || t.notesApp.templatesFailed);
      return false;
    }
  }
}
