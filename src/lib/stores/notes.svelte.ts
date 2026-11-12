import { SvelteMap, SvelteSet } from "svelte/reactivity";

import * as backend from "../api";
import { parseError } from "../api";
import { t } from "../i18n";
import type {
  FolderUpsertInput,
  FolderWithCount,
  Note,
  NoteCreateInput,
  NotePatch,
  NotesListQuery,
  NoteSummary,
} from "../types";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface NotesApi {
  folderList(): Promise<FolderWithCount[]>;
  folderUpsert(input: FolderUpsertInput): Promise<FolderWithCount | unknown>;
  folderDelete(id: string): Promise<number>;
  notesList(query: NotesListQuery): Promise<NoteSummary[]>;
  noteGet(id: string): Promise<Note>;
  noteCreate(input?: NoteCreateInput): Promise<Note>;
  noteUpdate(id: string, patch: NotePatch): Promise<Note>;
  noteSetPinned(id: string, pinned: boolean): Promise<Note>;
  noteMove(id: string, folderId: string | null): Promise<Note>;
  noteTagsSet(noteId: string, tagIds: string[]): Promise<Note>;
  noteDelete(id: string): Promise<void>;
  noteRestore(id: string): Promise<Note>;
  notesTrashList(): Promise<NoteSummary[]>;
  settingsGet(
    key: "app_section" | "notes_scope" | "notes_open",
  ): Promise<string | null>;
  settingsSet(
    key: "app_section" | "notes_scope" | "notes_open",
    value: string,
  ): Promise<void>;
}

/** What the list shows. `folder:<id>` is a folder with its subfolders. */
export type ScopeKey =
  "all" | "inbox" | "pinned" | "trash" | `folder:${string}` | `tag:${string}`;

export type NotesSortKey = "updated" | "created" | "title";

/** One visible row of the folder tree. */
export interface FolderRow {
  folder: FolderWithCount;
  depth: number;
  hasChildren: boolean;
  expanded: boolean;
}

export interface Notice {
  kind: "info" | "error";
  message: string;
}

const MAX_UNDO = 20;

function parseScope(value: string | null): ScopeKey {
  if (
    value === "all" ||
    value === "inbox" ||
    value === "pinned" ||
    value === "trash"
  ) {
    return value;
  }
  if (value?.startsWith("folder:") && value.length > "folder:".length) {
    return value as ScopeKey;
  }
  if (value?.startsWith("tag:") && value.length > "tag:".length) {
    return value as ScopeKey;
  }
  return "all";
}

/** Folders, the note list of the chosen scope and the note open in the editor. */
export class NotesStore {
  folders = $state.raw<FolderWithCount[]>([]);
  notes = $state.raw<NoteSummary[]>([]);
  scope = $state<ScopeKey>("all");
  sort = $state<NotesSortKey>("updated");
  /** The open note, with its document; `null` while none is chosen. */
  open = $state.raw<Note | null>(null);
  status = $state<"idle" | "ready" | "error">("idle");
  notice = $state.raw<Notice | null>(null);
  /** Notes sent to the trash in this session, newest last, to bring them back. */
  undoStack = $state.raw<string[]>([]);
  /** Folders whose children are shown. */
  readonly expanded = new SvelteSet<string>();

  /** Titles of notes tasks come from, so a task can name its note without opening it. */
  readonly titles = new SvelteMap<string, string>();

  readonly #api: NotesApi;
  #listToken = 0;
  #loading: Promise<void> | null = null;

  // Rebuilt on change, never mutated.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  byId = $derived(new Map(this.folders.map((folder) => [folder.id, folder])));

  rows = $derived.by<FolderRow[]>(() => {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    const children = new Map<string | null, FolderWithCount[]>();
    for (const folder of this.folders) {
      const list = children.get(folder.parentId) ?? [];
      list.push(folder);
      children.set(folder.parentId, list);
    }
    const result: FolderRow[] = [];
    const walk = (parent: string | null, depth: number) => {
      for (const folder of children.get(parent) ?? []) {
        const hasChildren = children.has(folder.id);
        const expanded = hasChildren && this.expanded.has(folder.id);
        result.push({ folder, depth, hasChildren, expanded });
        if (expanded) walk(folder.id, depth + 1);
      }
    };
    walk(null, 0);
    return result;
  });

  /** Id of the folder the scope points at, if it is one. */
  scopeFolderId = $derived(
    this.scope.startsWith("folder:")
      ? this.scope.slice("folder:".length)
      : null,
  );

  /** Id of the tag the scope points at, if it is one. */
  scopeTagId = $derived(
    this.scope.startsWith("tag:") ? this.scope.slice("tag:".length) : null,
  );

  scopeTitle = $derived.by(() => {
    if (this.scope === "all") return t.notesApp.all;
    if (this.scope === "inbox") return t.notesApp.inbox;
    if (this.scope === "pinned") return t.notesApp.pinned;
    if (this.scope === "trash") return t.notesApp.trash;
    // A tag has no name here; the list takes it from the tags store.
    if (this.scopeTagId) return "";
    return this.byId.get(this.scopeFolderId ?? "")?.name ?? t.notesApp.all;
  });

  constructor(api: NotesApi = backend) {
    this.#api = api;
  }

  /** Loads once at a time: callers that arrive while it runs share the same work. */
  load(): Promise<void> {
    this.#loading ??= this.#load().finally(() => {
      this.#loading = null;
    });
    return this.#loading;
  }

  /** Loads the notes the first time they are needed. */
  async ensureLoaded(): Promise<void> {
    if (this.status === "idle") await this.load();
  }

  /** The title of a note known by id, if it has been looked up. */
  titleOf(id: string): string | undefined {
    return this.titles.get(id);
  }

  /** Looks up the titles of notes not known yet; unknown or removed notes stay unknown. */
  async ensureTitles(ids: readonly string[]): Promise<void> {
    const missing = ids.filter(
      (id, at) => ids.indexOf(id) === at && !this.titles.has(id),
    );
    await Promise.all(
      missing.map(async (id) => {
        try {
          this.titles.set(id, (await this.#api.noteGet(id)).title);
        } catch {
          // A purged note has no title to show.
        }
      }),
    );
  }

  /** Creates a note with these contents, shows it and opens it. */
  async createNoteWith(input: NoteCreateInput): Promise<Note | null> {
    try {
      const note = await this.#api.noteCreate(input);
      this.titles.set(note.id, note.title);
      await this.#refreshAll();
      this.open = note;
      this.#remember("notes_open", note.id);
      return note;
    } catch {
      this.#fail(t.notesApp.createFailed);
      return null;
    }
  }

  async #load(): Promise<void> {
    try {
      const [scope, openId] = await Promise.all([
        this.#api.settingsGet("notes_scope").catch(() => null),
        this.#api.settingsGet("notes_open").catch(() => null),
      ]);
      this.folders = await this.#api.folderList();
      const wanted = parseScope(scope);
      // A folder deleted since the last run falls back to everything.
      this.scope =
        wanted.startsWith("folder:") &&
        !this.byId.has(wanted.slice("folder:".length))
          ? "all"
          : wanted;
      this.#expandAncestors(this.scopeFolderId);
      await this.#refreshList();
      if (openId) await this.openNote(openId, { remember: false });
      this.status = "ready";
    } catch {
      this.status = "error";
    }
  }

  async selectScope(scope: ScopeKey): Promise<void> {
    if (scope === this.scope) return;
    this.scope = scope;
    this.#remember("notes_scope", scope);
    await this.#refreshList();
  }

  setSort(sort: NotesSortKey): Promise<void> {
    this.sort = sort;
    return this.#refreshList();
  }

  toggleExpanded(id: string): void {
    if (this.expanded.has(id)) this.expanded.delete(id);
    else this.expanded.add(id);
  }

  async openNote(
    id: string | null,
    options: { remember?: boolean } = {},
  ): Promise<void> {
    if (id === null) {
      this.open = null;
      if (options.remember !== false) this.#remember("notes_open", "");
      return;
    }
    try {
      this.open = await this.#api.noteGet(id);
      if (options.remember !== false) this.#remember("notes_open", id);
    } catch {
      // A note that no longer exists (purged, or restored from another backup).
      this.open = null;
      this.#remember("notes_open", "");
    }
  }

  async createNote(): Promise<Note | null> {
    const folderId = this.scopeFolderId;
    try {
      if (this.scope === "trash" || this.scope === "pinned") {
        await this.selectScope("all");
      }
      const tagId = this.scopeTagId;
      const note = await this.#api.noteCreate({
        ...(folderId ? { folderId } : {}),
        ...(tagId ? { tagIds: [tagId] } : {}),
      });
      await this.#refreshAll();
      this.open = note;
      this.#remember("notes_open", note.id);
      return note;
    } catch {
      this.#fail(t.notesApp.createFailed);
      return null;
    }
  }

  /** Saves the document of the open note; rejects so the saver keeps the text. */
  async saveContent(id: string, contentJson: string): Promise<void> {
    const saved = await this.#api.noteUpdate(id, { contentJson });
    this.#adopt(saved);
    await this.#refreshList();
  }

  async saveTitle(id: string, title: string): Promise<void> {
    const saved = await this.#api.noteUpdate(id, { title });
    this.titles.set(id, saved.title);
    this.#adopt(saved);
    await this.#refreshList();
  }

  async setPinned(id: string, pinned: boolean): Promise<void> {
    try {
      this.#adopt(await this.#api.noteSetPinned(id, pinned));
      await this.#refreshList();
    } catch {
      this.#fail(t.notesApp.actionFailed);
    }
  }

  async setTags(id: string, tagIds: string[]): Promise<void> {
    try {
      this.#adopt(await this.#api.noteTagsSet(id, tagIds));
      await this.#refreshList();
    } catch {
      this.#fail(t.notesApp.actionFailed);
    }
  }

  async moveNote(id: string, folderId: string | null): Promise<void> {
    try {
      this.#adopt(await this.#api.noteMove(id, folderId));
      await this.#refreshAll();
    } catch {
      this.#fail(t.notesApp.actionFailed);
    }
  }

  async duplicateNote(id: string): Promise<Note | null> {
    try {
      const source =
        this.open?.id === id ? this.open : await this.#api.noteGet(id);
      const copy = await this.#api.noteCreate({
        ...(source.folderId ? { folderId: source.folderId } : {}),
        title: t.notesApp.copyOf(source.title),
        contentJson: source.contentJson,
        tagIds: source.tagIds,
      });
      await this.#refreshAll();
      this.open = copy;
      this.#remember("notes_open", copy.id);
      return copy;
    } catch {
      this.#fail(t.notesApp.actionFailed);
      return null;
    }
  }

  async trashNote(id: string): Promise<void> {
    try {
      await this.#api.noteDelete(id);
      this.undoStack = [...this.undoStack, id].slice(-MAX_UNDO);
      if (this.open?.id === id) await this.openNote(null);
      await this.#refreshAll();
    } catch {
      this.#fail(t.notesApp.actionFailed);
    }
  }

  async undoTrash(): Promise<void> {
    const id = this.undoStack.at(-1);
    if (id === undefined) return;
    this.undoStack = this.undoStack.slice(0, -1);
    await this.restoreNote(id);
  }

  async restoreNote(id: string): Promise<void> {
    try {
      await this.#api.noteRestore(id);
      await this.#refreshAll();
    } catch {
      this.#fail(t.notesApp.actionFailed);
    }
  }

  async createFolder(name: string, parentId?: string): Promise<boolean> {
    return this.#folderCall(
      () => this.#api.folderUpsert({ name, ...(parentId ? { parentId } : {}) }),
      parentId,
    );
  }

  async renameFolder(id: string, name: string): Promise<boolean> {
    const folder = this.byId.get(id);
    if (!folder) return false;
    return this.#folderCall(() =>
      this.#api.folderUpsert({
        id,
        name,
        ...(folder.parentId ? { parentId: folder.parentId } : {}),
      }),
    );
  }

  /** `parentId: null` moves the folder to the top level. */
  async moveFolder(id: string, parentId: string | null): Promise<boolean> {
    const folder = this.byId.get(id);
    if (!folder || folder.parentId === parentId) return false;
    return this.#folderCall(
      () =>
        this.#api.folderUpsert({
          id,
          name: folder.name,
          ...(parentId ? { parentId } : {}),
        }),
      parentId ?? undefined,
    );
  }

  async deleteFolder(id: string): Promise<void> {
    try {
      const trashed = await this.#api.folderDelete(id);
      const stillThere = (folderId: string | null): boolean => {
        for (let at = folderId; at; at = this.byId.get(at)?.parentId ?? null) {
          if (at === id) return false;
        }
        return true;
      };
      if (!stillThere(this.scopeFolderId)) await this.selectScope("all");
      this.folders = await this.#api.folderList();
      if (this.open && !stillThere(this.open.folderId))
        await this.openNote(null);
      await this.#refreshAll();
      this.notice = {
        kind: "info",
        message: t.notesApp.folderDeleted(trashed),
      };
    } catch {
      this.#fail(t.notesApp.actionFailed);
    }
  }

  dismissNotice(): void {
    this.notice = null;
  }

  async #folderCall(
    call: () => Promise<unknown>,
    reveal?: string,
  ): Promise<boolean> {
    try {
      await call();
      if (reveal) this.expanded.add(reveal);
      this.folders = await this.#api.folderList();
      return true;
    } catch (error) {
      const { code } = parseError(error);
      this.#fail(
        code === "CONFLICT"
          ? t.notesApp.folderNameTaken
          : code === "VALIDATION"
            ? t.notesApp.folderInvalid
            : t.notesApp.actionFailed,
      );
      return false;
    }
  }

  #expandAncestors(id: string | null): void {
    for (
      let at = id ? this.byId.get(id)?.parentId : null;
      at;
      at = this.byId.get(at)?.parentId
    ) {
      this.expanded.add(at);
    }
  }

  // Keeps the open note in step with what the backend now holds.
  #adopt(note: Note): void {
    if (this.open?.id === note.id) this.open = note;
  }

  async #refreshAll(): Promise<void> {
    this.folders = await this.#api.folderList();
    await this.#refreshList();
  }

  // Only the latest request may fill the list: a slow answer must not overwrite a newer one.
  async #refreshList(): Promise<void> {
    const token = ++this.#listToken;
    const scope = this.scope;
    let notes: NoteSummary[];
    if (scope === "trash") notes = await this.#api.notesTrashList();
    else {
      const query: NotesListQuery = { scope: "all", sort: this.sort };
      if (scope === "inbox" || scope === "pinned") query.scope = scope;
      else if (scope.startsWith("folder:")) {
        query.scope = "folder";
        query.folderId = scope.slice("folder:".length);
      } else if (scope.startsWith("tag:")) {
        query.tagId = scope.slice("tag:".length);
      }
      notes = await this.#api.notesList(query);
    }
    if (token === this.#listToken) this.notes = notes;
    for (const note of notes) this.titles.set(note.id, note.title);
  }

  #remember(key: "notes_scope" | "notes_open", value: string): void {
    void this.#api.settingsSet(key, value).catch(() => undefined);
  }

  #fail(message: string): void {
    this.notice = { kind: "error", message };
  }
}

export const notesStore = new NotesStore();
