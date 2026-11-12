import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  EffectApplied,
  EffectMode,
  ErrorCode,
  Folder,
  FolderUpsertInput,
  FolderWithCount,
  Note,
  NoteCreateInput,
  NotePatch,
  NoteSummary,
  NotesListQuery,
  ReminderFired,
  ReminderInput,
  SearchHit,
  SettingKey,
  Section,
  Tag,
  TagUpsertInput,
  TagWithCount,
  Task,
  TaskCreateInput,
  TaskPatch,
  TasksListQuery,
  TasksSections,
  ThemeMode,
} from "./types";

export type * from "./types";

const ERROR_CODES: readonly ErrorCode[] = [
  "NOT_FOUND",
  "VALIDATION",
  "CONFLICT",
  "DB",
  "IO",
  "NOTIFY",
  "INTERNAL",
];

export interface ParsedError {
  code: ErrorCode;
  message: string;
}

/** Splits the `CODE: message` string the backend rejects with (SPEC §7.1). */
export function parseError(raw: unknown): ParsedError {
  const text = typeof raw === "string" ? raw : String(raw);
  const separator = text.indexOf(": ");
  if (separator > 0) {
    const code = text.slice(0, separator);
    const known = ERROR_CODES.find((candidate) => candidate === code);
    if (known) return { code: known, message: text.slice(separator + 2) };
  }
  return { code: "INTERNAL", message: text };
}

export function startupMark(label: string): Promise<void> {
  return invoke<void>("startup_mark", { label });
}

export function windowSetEffect(
  mode: EffectMode,
  theme?: ThemeMode,
): Promise<EffectApplied> {
  return invoke<EffectApplied>("window_set_effect", { mode, theme });
}

/** Section views return both sections; the range view returns a flat list. */
export function tasksList(
  query: TasksListQuery & { view: "range" },
): Promise<Task[]>;
export function tasksList(query: TasksListQuery): Promise<TasksSections>;
export function tasksList(
  query: TasksListQuery,
): Promise<TasksSections | Task[]> {
  return invoke("tasks_list", { ...query });
}

export function taskCreate(input: TaskCreateInput = {}): Promise<Task> {
  return invoke<Task>("task_create", { ...input });
}

export function taskUpdate(id: string, patch: TaskPatch): Promise<Task> {
  return invoke<Task>("task_update", { id, patch });
}

/** `completedDate` (local `YYYY-MM-DD`) is the day an undated task takes when it is completed. */
export function taskSetCompleted(
  id: string,
  completed: boolean,
  completedDate?: string,
): Promise<Task> {
  return invoke<Task>("task_set_completed", { id, completed, completedDate });
}

export function taskMove(
  id: string,
  toSection: Section,
  options: { date?: string; beforeId?: string } = {},
): Promise<Task> {
  return invoke<Task>("task_move", { id, toSection, ...options });
}

export function taskDelete(id: string): Promise<void> {
  return invoke<void>("task_delete", { id });
}

export function taskRestore(id: string): Promise<Task> {
  return invoke<Task>("task_restore", { id });
}

export function trashList(): Promise<Task[]> {
  return invoke<Task[]>("trash_list");
}

/** Replaces the task's unsent reminders (F-12). */
export function remindersSet(
  taskId: string,
  reminders: ReminderInput[],
): Promise<Task> {
  return invoke<Task>("reminders_set", { taskId, reminders });
}

/** Calls `handler` each time the scheduler delivers a reminder (SPEC §7.4). */
export function onReminderFired(
  handler: (fired: ReminderFired) => void,
): Promise<UnlistenFn> {
  return listen<ReminderFired>("reminder:fired", (event) =>
    handler(event.payload),
  );
}

export function search(query: string, limit?: number): Promise<SearchHit[]> {
  return invoke<SearchHit[]>("search", { query, limit });
}

export function tagList(): Promise<TagWithCount[]> {
  return invoke<TagWithCount[]>("tag_list");
}

export function tagUpsert(input: TagUpsertInput): Promise<Tag> {
  return invoke<Tag>("tag_upsert", { ...input });
}

export function tagDelete(id: string): Promise<void> {
  return invoke<void>("tag_delete", { id });
}

/** Moves everything tagged `sourceId` to `targetId` and deletes `sourceId`. */
export function tagMerge(sourceId: string, targetId: string): Promise<void> {
  return invoke<void>("tag_merge", { sourceId, targetId });
}

export function taskTagsSet(taskId: string, tagIds: string[]): Promise<Task> {
  return invoke<Task>("task_tags_set", { taskId, tagIds });
}

export function folderList(): Promise<FolderWithCount[]> {
  return invoke<FolderWithCount[]>("folder_list");
}

/** Creates a folder, or renames/moves it when `id` is given. */
export function folderUpsert(input: FolderUpsertInput): Promise<Folder> {
  return invoke<Folder>("folder_upsert", { ...input });
}

/** Deletes the folder and its subfolders; resolves with how many notes went to the trash. */
export function folderDelete(id: string): Promise<number> {
  return invoke<number>("folder_delete", { id });
}

export function notesList(query: NotesListQuery): Promise<NoteSummary[]> {
  return invoke<NoteSummary[]>("notes_list", { ...query });
}

export function noteGet(id: string): Promise<Note> {
  return invoke<Note>("note_get", { id });
}

export function noteCreate(input: NoteCreateInput = {}): Promise<Note> {
  return invoke<Note>("note_create", { ...input });
}

export function noteUpdate(id: string, patch: NotePatch): Promise<Note> {
  return invoke<Note>("note_update", { id, patch });
}

export function noteSetPinned(id: string, pinned: boolean): Promise<Note> {
  return invoke<Note>("note_set_pinned", { id, pinned });
}

/** `folderId: null` moves the note to the inbox. */
export function noteMove(id: string, folderId: string | null): Promise<Note> {
  return invoke<Note>("note_move", { id, folderId });
}

export function noteTagsSet(noteId: string, tagIds: string[]): Promise<Note> {
  return invoke<Note>("note_tags_set", { noteId, tagIds });
}

export function noteDelete(id: string): Promise<void> {
  return invoke<void>("note_delete", { id });
}

export function noteRestore(id: string): Promise<Note> {
  return invoke<Note>("note_restore", { id });
}

/** A note of the trash with its document; `note_get` only knows live notes. */
export function noteTrashGet(id: string): Promise<Note> {
  return invoke<Note>("note_trash_get", { id });
}

export function notesTrashList(): Promise<NoteSummary[]> {
  return invoke<NoteSummary[]>("notes_trash_list");
}

export function settingsGet(key: SettingKey): Promise<string | null> {
  return invoke<string | null>("settings_get", { key });
}

export function settingsSet(key: SettingKey, value: string): Promise<void> {
  return invoke<void>("settings_set", { key, value });
}

export function backupCreate(path: string): Promise<void> {
  return invoke<void>("backup_create", { path });
}

export function backupRestore(path: string): Promise<void> {
  return invoke<void>("backup_restore", { path });
}

export function autostartGet(): Promise<boolean> {
  return invoke<boolean>("autostart_get");
}

export function autostartSet(enabled: boolean): Promise<void> {
  return invoke<void>("autostart_set", { enabled });
}
