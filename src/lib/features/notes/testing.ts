import type {
  FolderUpsertInput,
  FoundItem,
  FolderWithCount,
  Note,
  NoteCreateInput,
  NotePatch,
  NotesListQuery,
  NoteSummary,
} from "../../types";
import type { NotesApi } from "../../stores/notes.svelte";

const NOW = "2026-03-10T10:00:00Z";
const DOC = '{"type":"doc","content":[]}';

export function folder(
  id: string,
  parentId: string | null = null,
  noteCount = 0,
): FolderWithCount {
  return { id, name: id, parentId, position: 0, noteCount };
}

export function note(id: string, overrides: Partial<Note> = {}): Note {
  return {
    id,
    folderId: null,
    title: id,
    contentJson: DOC,
    isPinned: false,
    tagIds: [],
    createdAt: NOW,
    updatedAt: NOW,
    ...overrides,
  };
}

export function summary(value: Note): NoteSummary {
  return {
    id: value.id,
    folderId: value.folderId,
    title: value.title,
    excerpt: "",
    isPinned: value.isPinned,
    tagIds: value.tagIds,
    createdAt: value.createdAt,
    updatedAt: value.updatedAt,
  };
}

export class FakeNotesApi implements NotesApi {
  folders: FolderWithCount[] = [];
  notes: Note[] = [];
  trash: Note[] = [];
  settings = new Map<string, string>();
  /** `from` -> `to`: the links between notes the fake reports as backlinks. */
  links: [string, string][] = [];
  searches: string[] = [];
  queries: NotesListQuery[] = [];
  failWith: string | null = null;
  #next = 0;
  daily = new Map<string, string>();

  #guard() {
    if (this.failWith) throw this.failWith;
  }

  async folderList() {
    return this.folders;
  }
  async folderUpsert(input: FolderUpsertInput) {
    this.#guard();
    const saved = folder(
      input.id ?? `f${++this.#next}`,
      input.parentId ?? null,
    );
    saved.name = input.name;
    this.folders = [...this.folders.filter((f) => f.id !== saved.id), saved];
    return saved;
  }
  async folderDelete(id: string) {
    this.#guard();
    const gone = this.notes.filter((n) => n.folderId === id);
    this.trash.push(...gone);
    this.notes = this.notes.filter((n) => n.folderId !== id);
    this.folders = this.folders.filter((f) => f.id !== id);
    return gone.length;
  }
  async notesList(query: NotesListQuery) {
    this.queries.push(query);
    return this.notes.map(summary);
  }
  async noteGet(id: string) {
    const found = this.notes.find((n) => n.id === id);
    if (!found) throw "NOT_FOUND: no existe";
    return found;
  }
  async noteCreate(input: NoteCreateInput = {}) {
    this.#guard();
    const created = note(`n${++this.#next}`, {
      title: input.title ?? "",
      folderId: input.folderId ?? null,
      ...(input.contentJson ? { contentJson: input.contentJson } : {}),
    });
    this.notes = [created, ...this.notes];
    return created;
  }
  async noteDailyOpen(date: string, title: string) {
    this.#guard();
    const known = this.daily.get(date);
    if (known) return this.noteGet(known);
    const created = note(`n${++this.#next}`, { title });
    this.notes = [created, ...this.notes];
    this.daily.set(date, created.id);
    return created;
  }
  async noteUpdate(id: string, patch: NotePatch) {
    this.#guard();
    const current = await this.noteGet(id);
    const next = {
      ...current,
      ...(patch.title === undefined ? {} : { title: patch.title }),
      ...(patch.contentJson === undefined
        ? {}
        : { contentJson: patch.contentJson }),
    };
    this.notes = this.notes.map((n) => (n.id === id ? next : n));
    return next;
  }
  async noteSetPinned(id: string, pinned: boolean) {
    const next = { ...(await this.noteGet(id)), isPinned: pinned };
    this.notes = this.notes.map((n) => (n.id === id ? next : n));
    return next;
  }
  async noteMove(id: string, folderId: string | null) {
    const next = { ...(await this.noteGet(id)), folderId };
    this.notes = this.notes.map((n) => (n.id === id ? next : n));
    return next;
  }
  async noteTagsSet(id: string, tagIds: string[]) {
    const next = { ...(await this.noteGet(id)), tagIds };
    this.notes = this.notes.map((n) => (n.id === id ? next : n));
    return next;
  }
  async noteDelete(id: string) {
    const found = await this.noteGet(id);
    this.trash.push(found);
    this.notes = this.notes.filter((n) => n.id !== id);
  }
  async noteRestore(id: string) {
    const found = this.trash.find((n) => n.id === id);
    if (!found) throw "NOT_FOUND: no existe";
    this.trash = this.trash.filter((n) => n.id !== id);
    this.notes = [found, ...this.notes];
    return found;
  }
  async noteTrashGet(id: string) {
    const found = this.trash.find((n) => n.id === id);
    if (!found) throw "NOT_FOUND: no existe";
    return found;
  }
  async notesTrashList() {
    return this.trash.map(summary);
  }
  async noteBacklinks(id: string) {
    const from = this.links
      .filter(([, to]) => to === id)
      .map(([source]) => source);
    return this.notes.filter((n) => from.includes(n.id)).map(summary);
  }
  async notesOrphans() {
    const linked = new Set(this.links.flat());
    return this.notes.filter((n) => !linked.has(n.id)).map(summary);
  }
  async searchAll(query: string): Promise<FoundItem[]> {
    this.searches.push(query);
    const text = query
      .replace(/\b(is|in|tag):\S*/g, "")
      .trim()
      .toLowerCase();
    return this.notes
      .filter((n) => n.title.toLowerCase().includes(text))
      .map((n) => ({
        kind: "note" as const,
        noteId: n.id,
        title: n.title,
        snippet: "",
        folderId: n.folderId,
        isPinned: n.isPinned,
      }));
  }
  async settingsGet(key: string) {
    return this.settings.get(key) ?? null;
  }
  async settingsSet(key: string, value: string) {
    this.settings.set(key, value);
  }
}
