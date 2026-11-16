import { SvelteMap, SvelteSet } from "svelte/reactivity";

import * as backend from "../api";
import { parseError } from "../api";
import { formatDisplayDate } from "../dates";
import { t } from "../i18n";
import { plainQuery } from "../features/notes/linkQuery";
import type {
  FolderUpsertInput,
  FolderWithCount,
  Note,
  NoteCreateInput,
  NotePatch,
  NotesListQuery,
  NoteSummary,
  NoteReminder,
  NoteTemplate,
  NoteVersion,
  NoteVersionSummary,
  FoundItem,
} from "../types";

type NotesSetting =
  "app_section" | "notes_scope" | "notes_open" | "notes_recent";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface NotesApi {
  folderList(): Promise<FolderWithCount[]>;
  folderUpsert(input: FolderUpsertInput): Promise<FolderWithCount | unknown>;
  folderDelete(id: string): Promise<number>;
  notesList(query: NotesListQuery): Promise<NoteSummary[]>;
  noteGet(id: string): Promise<Note>;
  noteCreate(input?: NoteCreateInput): Promise<Note>;
  noteDailyOpen(date: string, title: string): Promise<Note>;
  noteVersionsList(noteId: string): Promise<NoteVersionSummary[]>;
  noteVersionGet(id: string): Promise<NoteVersion>;
  noteVersionRestore(id: string): Promise<Note>;
  noteReminderGet(noteId: string): Promise<NoteReminder | null>;
  noteReminderSet(
    noteId: string,
    notifyAt: string | null,
  ): Promise<NoteReminder | null>;
  templateList(): Promise<NoteTemplate[]>;
  templateSave(input: {
    id?: string;
    name: string;
    contentJson?: string;
  }): Promise<NoteTemplate>;
  templateDelete(id: string): Promise<void>;
  noteUpdate(id: string, patch: NotePatch): Promise<Note>;
  noteSetPinned(id: string, pinned: boolean): Promise<Note>;
  noteMove(id: string, folderId: string | null): Promise<Note>;
  noteTagsSet(noteId: string, tagIds: string[]): Promise<Note>;
  noteDelete(id: string): Promise<void>;
  noteRestore(id: string): Promise<Note>;
  noteTrashGet(id: string): Promise<Note>;
  noteBacklinks(id: string): Promise<NoteSummary[]>;
  notesOrphans(): Promise<NoteSummary[]>;
  searchAll(query: string, limit?: number): Promise<FoundItem[]>;
  notesTrashList(): Promise<NoteSummary[]>;
  settingsGet(key: NotesSetting): Promise<string | null>;
  settingsSet(key: NotesSetting, value: string): Promise<void>;
}

/** What the list shows. `folder:<id>` is a folder with its subfolders. */
export type ScopeKey =
  | "all"
  | "inbox"
  | "pinned"
  | "orphans"
  | "trash"
  | `folder:${string}`
  | `tag:${string}`;

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
/** How many opened notes are remembered for "Ir a…". */
const MAX_RECENT = 8;

function parseScope(value: string | null): ScopeKey {
  if (
    value === "all" ||
    value === "inbox" ||
    value === "pinned" ||
    value === "orphans" ||
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
  /** The open note sits in the trash: it is shown, but cannot be edited. */
  openInTrash = $state(false);
  status = $state<"idle" | "ready" | "error">("idle");
  notice = $state.raw<Notice | null>(null);
  /** Templates for new notes (F13-02), loaded when the dialog opens. */
  templates = $state.raw<NoteTemplate[]>([]);
  templatesOpen = $state(false);
  /** Grows when the open note is replaced from outside the editor (a restored version). */
  revision = $state(0);
  /** Notes sent to the trash in this session, newest last, to bring them back. */
  undoStack = $state.raw<string[]>([]);
  /** Folders whose children are shown. */
  readonly expanded = new SvelteSet<string>();

  /** Titles of notes tasks come from, so a task can name its note without opening it. */
  readonly titles = new SvelteMap<string, string>();

  /** Notes known to be in the trash or gone: links to them show as removed. */
  readonly missing = new SvelteSet<string>();

  /** Ids of the notes opened last, most recent first (F11-05). */
  recent = $state.raw<string[]>([]);

  readonly #api: NotesApi;
  #listToken = 0;
  #loading: Promise<void> | null = null;

  // Rebuilt on change, never mutated.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  byId = $derived(new Map(this.folders.map((folder) => [folder.id, folder])));

  /** The rows of the folder panel: children only under an expanded folder. */
  rows = $derived.by<FolderRow[]>(() => this.#walk(false));

  /** Every folder in tree order, whatever is expanded: for the "move to" menu. */
  tree = $derived.by<FolderRow[]>(() => this.#walk(true));

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
    if (this.scope === "orphans") return t.notesApp.orphans;
    if (this.scope === "trash") return t.notesApp.trash;
    // A tag has no name here; the list takes it from the tags store.
    if (this.scopeTagId) return "";
    return this.byId.get(this.scopeFolderId ?? "")?.name ?? t.notesApp.all;
  });

  constructor(api: NotesApi = backend) {
    this.#api = api;
  }

  #walk(all: boolean): FolderRow[] {
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
        if (all || expanded) walk(folder.id, depth + 1);
      }
    };
    walk(null, 0);
    return result;
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
    const found = await this.#api.searchAll(
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
      const note = await this.#api.noteCreate({
        title,
        ...(folderId ? { folderId } : {}),
      });
      this.titles.set(note.id, note.title);
      this.missing.delete(note.id);
      await this.#refreshAll();
      return note;
    } catch {
      this.#fail(t.notesApp.linkCreateFailed);
      return null;
    }
  }

  /** Notes that link to this one. */
  backlinks(id: string): Promise<NoteSummary[]> {
    return this.#api.noteBacklinks(id);
  }

  /** Notes without links to or from any other. */
  orphans(): Promise<NoteSummary[]> {
    return this.#api.notesOrphans();
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
          this.missing.delete(id);
        } catch {
          // A trashed or purged note has no title to show: its links read as removed.
          this.missing.add(id);
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
      this.#show(note);
      return note;
    } catch {
      this.#fail(t.notesApp.createFailed);
      return null;
    }
  }

  async reminderGet(noteId: string): Promise<NoteReminder | null> {
    try {
      return await this.#api.noteReminderGet(noteId);
    } catch {
      this.#fail(t.notesApp.reminder.failed);
      return null;
    }
  }

  /** Sets (`notifyAt` as a UTC instant) or clears (`null`) the reminder; returns what is set. */
  async reminderSet(
    noteId: string,
    notifyAt: string | null,
  ): Promise<{ ok: boolean; reminder: NoteReminder | null }> {
    try {
      return {
        ok: true,
        reminder: await this.#api.noteReminderSet(noteId, notifyAt),
      };
    } catch (error) {
      this.#fail(parseError(error).message || t.notesApp.reminder.failed);
      return { ok: false, reminder: null };
    }
  }

  /** The scheduler delivered the reminder of a note: say so in the notes section. */
  async onReminderFired(noteId: string): Promise<void> {
    let title = this.titles.get(noteId);
    if (title === undefined) {
      try {
        title = (await this.#api.noteGet(noteId)).title;
      } catch {
        return;
      }
    }
    this.notice = {
      kind: "info",
      message: t.notesApp.reminder.fired(title.trim() || t.notesApp.untitled),
    };
  }

  async versionList(noteId: string): Promise<NoteVersionSummary[]> {
    try {
      return await this.#api.noteVersionsList(noteId);
    } catch {
      this.#fail(t.notesApp.history.failed);
      return [];
    }
  }

  async versionGet(id: string): Promise<NoteVersion | null> {
    try {
      return await this.#api.noteVersionGet(id);
    } catch {
      this.#fail(t.notesApp.history.failed);
      return null;
    }
  }

  /** Puts a version back in its note; the editor is rebuilt with the restored text. */
  async restoreVersion(id: string): Promise<boolean> {
    try {
      const restored = await this.#api.noteVersionRestore(id);
      this.titles.set(restored.id, restored.title);
      this.#adopt(restored);
      this.revision += 1;
      await this.#refreshList().catch(() => undefined);
      this.notice = { kind: "info", message: t.notesApp.history.restored };
      return true;
    } catch {
      this.#fail(t.notesApp.history.failed);
      return false;
    }
  }

  async openTemplates(): Promise<void> {
    this.templatesOpen = true;
    await this.loadTemplates();
  }

  async loadTemplates(): Promise<void> {
    try {
      this.templates = await this.#api.templateList();
    } catch {
      this.#fail(t.notesApp.templatesFailed);
    }
  }

  /** Saves the open note as a template, or rewrites `id` with it. */
  async saveOpenAsTemplate(name: string, id?: string): Promise<boolean> {
    return this.#saveTemplate({
      name,
      ...(id ? { id } : {}),
      ...(this.open ? { contentJson: this.open.contentJson } : {}),
    });
  }

  async renameTemplate(id: string, name: string): Promise<boolean> {
    return this.#saveTemplate({ id, name });
  }

  async #saveTemplate(input: {
    id?: string;
    name: string;
    contentJson?: string;
  }): Promise<boolean> {
    try {
      await this.#api.templateSave(input);
      await this.loadTemplates();
      return true;
    } catch (error) {
      const message = parseError(error).message;
      this.#fail(message || t.notesApp.templatesFailed);
      return false;
    }
  }

  async deleteTemplate(id: string): Promise<void> {
    try {
      await this.#api.templateDelete(id);
      await this.loadTemplates();
    } catch {
      this.#fail(t.notesApp.templatesFailed);
    }
  }

  /** A new note that starts as the template, shown and opened. */
  async createFromTemplate(id: string): Promise<Note | null> {
    const template = this.templates.find((item) => item.id === id);
    if (!template) return null;
    this.templatesOpen = false;
    return this.createNoteWith({
      title: template.name,
      contentJson: template.contentJson,
    });
  }

  /** Opens the note of a day (`YYYY-MM-DD`), creating it the first time (F13-01). */
  async openDaily(date: string): Promise<Note | null> {
    try {
      const note = await this.#api.noteDailyOpen(
        date,
        t.notesApp.dailyTitle(formatDisplayDate(date)),
      );
      this.titles.set(note.id, note.title);
      await this.#refreshAll();
      this.#show(note);
      return note;
    } catch {
      this.#fail(t.notesApp.dailyFailed);
      return null;
    }
  }

  async #load(): Promise<void> {
    try {
      const [scope, openId, recent] = await Promise.all([
        this.#api.settingsGet("notes_scope").catch(() => null),
        this.#api.settingsGet("notes_open").catch(() => null),
        this.#api.settingsGet("notes_recent").catch(() => null),
      ]);
      this.recent = (recent ?? "")
        .split(",")
        .filter((id, at, all) => id !== "" && all.indexOf(id) === at)
        .slice(0, MAX_RECENT);
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
    // A note of the trash only makes sense next to the trash list.
    if (this.openInTrash) await this.openNote(null);
    this.scope = scope;
    this.#remember("notes_scope", scope);
    await this.#tryRefreshList();
  }

  async setSort(sort: NotesSortKey): Promise<void> {
    this.sort = sort;
    await this.#tryRefreshList();
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
      this.openInTrash = false;
      if (options.remember !== false) this.#remember("notes_open", "");
      return;
    }
    try {
      // Only the trash list shows deleted notes, so being in it is what tells them apart.
      const trashed =
        this.scope === "trash" && this.notes.some((item) => item.id === id);
      const note = trashed
        ? await this.#api.noteTrashGet(id)
        : await this.#api.noteGet(id);
      this.open = note;
      this.openInTrash = trashed;
      if (options.remember !== false) {
        this.#remember("notes_open", id);
        if (!trashed) this.#touchRecent(id);
      }
    } catch {
      // A note that no longer exists (purged, or restored from another backup).
      this.open = null;
      this.openInTrash = false;
      this.#remember("notes_open", "");
    }
  }

  async createNote(): Promise<Note | null> {
    const folderId = this.scopeFolderId;
    try {
      if (
        this.scope === "trash" ||
        this.scope === "pinned" ||
        this.scope === "orphans"
      ) {
        await this.selectScope("all");
      }
      const tagId = this.scopeTagId;
      const note = await this.#api.noteCreate({
        ...(folderId ? { folderId } : {}),
        ...(tagId ? { tagIds: [tagId] } : {}),
      });
      await this.#refreshAll();
      this.#show(note);
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
    // The text is saved: a list that fails to refresh must not make the saver retry it.
    await this.#refreshList().catch(() => undefined);
  }

  async saveTitle(id: string, title: string): Promise<void> {
    const saved = await this.#api.noteUpdate(id, { title });
    this.titles.set(id, saved.title);
    this.#adopt(saved);
    await this.#refreshList().catch(() => undefined);
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
      this.#show(copy);
      return copy;
    } catch {
      this.#fail(t.notesApp.actionFailed);
      return null;
    }
  }

  async trashNote(id: string): Promise<void> {
    try {
      await this.#api.noteDelete(id);
      this.missing.add(id);
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
      const restored = await this.#api.noteRestore(id);
      this.missing.delete(id);
      this.titles.set(id, restored.title);
      if (this.open?.id === id) {
        this.open = restored;
        this.openInTrash = false;
      }
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

  #show(note: Note): void {
    this.open = note;
    this.openInTrash = false;
    this.#remember("notes_open", note.id);
    this.#touchRecent(note.id);
  }

  #touchRecent(id: string): void {
    this.recent = [id, ...this.recent.filter((known) => known !== id)].slice(
      0,
      MAX_RECENT,
    );
    this.#remember("notes_recent", this.recent.join(","));
  }

  /** The recent notes that still exist, with their titles, for the quick switcher. */
  recentNotes(limit = MAX_RECENT): { id: string; title: string }[] {
    return this.recent
      .flatMap((id) => {
        const title = this.linkTitle(id);
        return typeof title === "string" ? [{ id, title }] : [];
      })
      .slice(0, limit);
  }

  async #tryRefreshList(): Promise<void> {
    try {
      await this.#refreshList();
    } catch {
      this.#fail(t.notesApp.actionFailed);
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
    else if (scope === "orphans") notes = await this.#api.notesOrphans();
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

  #remember(
    key: "notes_scope" | "notes_open" | "notes_recent",
    value: string,
  ): void {
    void this.#api.settingsSet(key, value).catch(() => undefined);
  }

  #fail(message: string): void {
    this.notice = { kind: "error", message };
  }
}

export const notesStore = new NotesStore();
