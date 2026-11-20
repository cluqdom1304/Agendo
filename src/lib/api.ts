import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  AppInfo,
  ExportFormat,
  ExportKind,
  ExportReport,
  ImportReport,
  Attachment,
  CalendarEvent,
  EffectApplied,
  EffectMode,
  ErrorCode,
  EventInput,
  EventOccurrence,
  EventPatch,
  Folder,
  FolderUpsertInput,
  FolderWithCount,
  Habit,
  HabitInput,
  HabitLog,
  HabitPatch,
  Note,
  NoteCreateInput,
  NotePatch,
  NoteSummary,
  NoteTitle,
  NoteReminder,
  NoteTemplate,
  NoteVersion,
  NoteVersionSummary,
  NotesListQuery,
  ReminderFired,
  ReminderInput,
  FoundItem,
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
  TaskFocusTotal,
  TimeEntry,
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

/** Everything in one call: the undated tasks and every day of the agenda (optionally one tag). */
export function tasksListAll(tagId?: string): Promise<TasksSections> {
  return invoke<TasksSections>("tasks_list_all", { tagId });
}

/** Events (and the repetitions of series) that show between two local days, inclusive (D-94). */
export function eventsList(
  from: string,
  to: string,
): Promise<EventOccurrence[]> {
  return invoke<EventOccurrence[]>("events_list", { from, to });
}

/** Trashed events, most recently deleted first. */
export function eventsTrashList(): Promise<CalendarEvent[]> {
  return invoke<CalendarEvent[]>("events_trash_list");
}

export function eventGet(id: string): Promise<CalendarEvent> {
  return invoke<CalendarEvent>("event_get", { id });
}

export function eventCreate(input: EventInput): Promise<CalendarEvent> {
  return invoke<CalendarEvent>("event_create", { input });
}

export function eventUpdate(
  id: string,
  patch: EventPatch,
): Promise<CalendarEvent> {
  return invoke<CalendarEvent>("event_update", { id, patch });
}

export function eventDelete(id: string): Promise<void> {
  return invoke<void>("event_delete", { id });
}

export function eventRestore(id: string): Promise<CalendarEvent> {
  return invoke<CalendarEvent>("event_restore", { id });
}

export function habitsList(includeArchived = false): Promise<Habit[]> {
  return invoke<Habit[]>("habits_list", { includeArchived });
}

/** Trashed habits, most recently deleted first. */
export function habitsTrashList(): Promise<Habit[]> {
  return invoke<Habit[]>("habits_trash_list");
}

export function habitCreate(input: HabitInput): Promise<Habit> {
  return invoke<Habit>("habit_create", { input });
}

export function habitUpdate(id: string, patch: HabitPatch): Promise<Habit> {
  return invoke<Habit>("habit_update", { id, patch });
}

export function habitDelete(id: string): Promise<void> {
  return invoke<void>("habit_delete", { id });
}

export function habitRestore(id: string): Promise<Habit> {
  return invoke<Habit>("habit_restore", { id });
}

/** Marks or unmarks a local day (`YYYY-MM-DD`, not later than today). */
export function habitSetDone(
  id: string,
  date: string,
  done: boolean,
): Promise<Habit> {
  return invoke<Habit>("habit_set_done", { id, date, done });
}

/** The days marked between two local days, inclusive, for every live habit. */
export function habitLogs(from: string, to: string): Promise<HabitLog[]> {
  return invoke<HabitLog[]>("habit_logs", { from, to });
}

/** The focus session in progress, if any (D-97). */
export function focusCurrent(): Promise<TimeEntry | null> {
  return invoke<TimeEntry | null>("focus_current");
}

export function focusStart(
  taskId: string,
  plannedMinutes: number,
): Promise<TimeEntry> {
  return invoke<TimeEntry>("focus_start", { taskId, plannedMinutes });
}

/** Ends the session now; `null` if none was running. */
export function focusStop(): Promise<TimeEntry | null> {
  return invoke<TimeEntry | null>("focus_stop");
}

/** Sessions that started between two local days, inclusive. */
export function focusList(from: string, to: string): Promise<TimeEntry[]> {
  return invoke<TimeEntry[]>("focus_list", { from, to });
}

/** Puts the session in progress in pause. */
export function focusPause(): Promise<TimeEntry> {
  return invoke<TimeEntry>("focus_pause");
}

export function focusResume(): Promise<TimeEntry> {
  return invoke<TimeEntry>("focus_resume");
}

/** Deletes the session in progress without keeping its time. */
export function focusDiscard(): Promise<void> {
  return invoke<void>("focus_discard");
}

/** Keeps the time spent on the old task and goes on with the rest on another one. */
export function focusSwitch(taskId: string): Promise<TimeEntry> {
  return invoke<TimeEntry>("focus_switch", { taskId });
}

/** Seconds of focus per task; tasks without time are missing. */
export function focusTotals(): Promise<TaskFocusTotal[]> {
  return invoke<TaskFocusTotal[]>("focus_totals");
}

/** The closed sessions and manual entries of a task, newest first. */
export function focusTaskEntries(taskId: string): Promise<TimeEntry[]> {
  return invoke<TimeEntry[]>("focus_task_entries", { taskId });
}

/** Records minutes already spent on a task, ending now. */
export function focusAddManual(
  taskId: string,
  minutes: number,
): Promise<TimeEntry> {
  return invoke<TimeEntry>("focus_add_manual", { taskId, minutes });
}

export function focusDeleteEntry(id: string): Promise<void> {
  return invoke<void>("focus_delete_entry", { id });
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

/** Tasks changed outside the interface, e.g. from a notification button (SPEC §7.4). */
export function onTasksChanged(
  handler: (changed: { ids: string[] }) => void,
): Promise<UnlistenFn> {
  return listen<{ ids: string[] }>("tasks:changed", (event) =>
    handler(event.payload),
  );
}

/** The user clicked a notification: show the task or the note it is about (F13-06). */
export function onReminderOpen(
  handler: (target: {
    taskId?: string | null;
    noteId?: string | null;
    eventId?: string | null;
  }) => void,
): Promise<UnlistenFn> {
  return listen<{
    taskId?: string | null;
    noteId?: string | null;
    eventId?: string | null;
  }>("reminder:open", (event) => handler(event.payload));
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

/** Tasks, notes and tags at once; understands `tag:`, `in:` and `is:` (F11-03). */
export function searchAll(query: string, limit?: number): Promise<FoundItem[]> {
  return invoke<FoundItem[]>("search_all", { query, limit });
}

/** Stores an image and returns its id; the same content gives the same id (D-47). */
export function attachmentAdd(bytes: Uint8Array): Promise<Attachment> {
  return invoke<Attachment>("attachment_add", bytes);
}

/** The bytes of a stored image. */
export async function attachmentRead(id: string): Promise<Uint8Array> {
  const data = await invoke<ArrayBuffer>("attachment_read", { id });
  return new Uint8Array(data);
}

export function noteBacklinks(id: string): Promise<NoteSummary[]> {
  return invoke<NoteSummary[]>("note_backlinks", { id });
}

/** Notes with no link to or from another note. */
export function notesOrphans(): Promise<NoteSummary[]> {
  return invoke<NoteSummary[]>("notes_orphans");
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

const MAX_TITLE_IDS = 1000;

/** Titles of the live notes among `ids`, in as few calls as the backend limit allows. */
export async function noteTitles(ids: readonly string[]): Promise<NoteTitle[]> {
  const found: NoteTitle[] = [];
  for (let from = 0; from < ids.length; from += MAX_TITLE_IDS) {
    found.push(
      ...(await invoke<NoteTitle[]>("note_titles", {
        ids: ids.slice(from, from + MAX_TITLE_IDS),
      })),
    );
  }
  return found;
}

export function noteGet(id: string): Promise<Note> {
  return invoke<Note>("note_get", { id });
}

export function noteCreate(input: NoteCreateInput = {}): Promise<Note> {
  return invoke<Note>("note_create", { ...input });
}

export function noteDailyOpen(date: string, title: string): Promise<Note> {
  return invoke<Note>("note_daily_open", { date, title });
}

export function noteVersionsList(
  noteId: string,
): Promise<NoteVersionSummary[]> {
  return invoke<NoteVersionSummary[]>("note_versions_list", { noteId });
}

export function noteVersionGet(id: string): Promise<NoteVersion> {
  return invoke<NoteVersion>("note_version_get", { id });
}

export function noteVersionRestore(id: string): Promise<Note> {
  return invoke<Note>("note_version_restore", { id });
}

export function noteReminderGet(noteId: string): Promise<NoteReminder | null> {
  return invoke<NoteReminder | null>("note_reminder_get", { noteId });
}

/** Sets the reminder of a note at a UTC instant, or removes it with `null` (F13-04). */
export function noteReminderSet(
  noteId: string,
  notifyAt: string | null,
): Promise<NoteReminder | null> {
  return invoke<NoteReminder | null>("note_reminder_set", { noteId, notifyAt });
}

export function templateList(): Promise<NoteTemplate[]> {
  return invoke<NoteTemplate[]>("template_list");
}

export function templateSave(input: {
  id?: string;
  name: string;
  contentJson?: string;
}): Promise<NoteTemplate> {
  return invoke<NoteTemplate>("template_save", { ...input });
}

export function templateDelete(id: string): Promise<void> {
  return invoke<void>("template_delete", { id });
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

export function exportNotes(
  format: ExportFormat,
  kind: ExportKind,
  id: string | null,
  path: string,
): Promise<ExportReport> {
  return invoke<ExportReport>("export_notes", { format, kind, id, path });
}

export function exportJson(path: string): Promise<ExportReport> {
  return invoke<ExportReport>("export_json", { path });
}

export function importMarkdown(path: string): Promise<ImportReport> {
  return invoke<ImportReport>("import_markdown", { path });
}

export function importEnex(paths: string[]): Promise<ImportReport> {
  return invoke<ImportReport>("import_enex", { paths });
}

export function autostartGet(): Promise<boolean> {
  return invoke<boolean>("autostart_get");
}

export function autostartSet(enabled: boolean): Promise<void> {
  return invoke<void>("autostart_set", { enabled });
}

export function appInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}
