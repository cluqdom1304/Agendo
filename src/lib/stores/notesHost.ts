import type { Note, NoteCreateInput } from "../types";
import type { NotesApi } from "./notes.svelte";

/** What the pieces composed into `NotesStore` need from it, so none of them imports the store. */
export interface NotesHost {
  api: NotesApi;
  /** Shows an error notice. */
  fail(message: string): void;
  /** Shows an informative notice. */
  inform(message: string): void;
  /** Reloads folders and the list of the current scope. */
  refreshAll(): Promise<void>;
  /** Reloads the list of the current scope. */
  refreshList(): Promise<void>;
  /** Opens a note that was just created or fetched. */
  show(note: Note): void;
  /** Replaces the open note when `note` is the same one. */
  adopt(note: Note): void;
  /** Forces the editor to rebuild from the open note. */
  bumpRevision(): void;
  openNote(): Note | null;
  createNoteWith(input: NoteCreateInput): Promise<Note | null>;
}
