// DTOs of the IPC contract (SPEC §7.2). Keep in sync with src-tauri/src/repo/model.rs; the
// fixture test in src/lib/types.test.ts fails if the Rust serialization drifts from these types.

export type EffectMode = "mica" | "acrylic" | "none";

export interface EffectApplied {
  requested: EffectMode;
  applied: EffectMode;
}

export type RepeatUnit = "hour" | "day" | "week" | "month";

/** How a custom-time reminder repeats (D-32). `count` and `until` are never both set. */
export interface Repeat {
  every: number;
  unit: RepeatUnit;
  /** Total number of soundings, first included. */
  count: number | null;
  /** Last local date, `YYYY-MM-DD`, inclusive. */
  until: string | null;
  /** Which sounding of the series this reminder is, starting at 1. */
  occurrence: number;
}

export interface Reminder {
  id: string;
  notifyAt: string;
  offsetMinutes: number | null;
  isSent: boolean;
  repeat: Repeat | null;
}

/** One entry of `reminders_set`: minutes before the task's date and time, or a UTC instant. */
export interface ReminderInput {
  offsetMinutes?: number;
  notifyAt?: string;
  /** Only together with `notifyAt`. */
  repeat?: Omit<Repeat, "occurrence"> & { occurrence?: number };
}

/** Payload of the `reminder:fired` event (SPEC §7.4). */
export interface ReminderFired {
  /** Set for the reminder of a task; `noteId` for the reminder of a note (D-52). */
  taskId?: string;
  noteId?: string;
  /** Set for the reminder of an event (D-94). */
  eventId?: string;
  reminderId: string;
}

/** The time a note will remind its owner (F13-04). */
export interface NoteReminder {
  id: string;
  noteId: string;
  notifyAt: string;
}

/** An appointment with a start and an end; it is not completed (D-94). */
export interface CalendarEvent {
  id: string;
  title: string;
  notes: string;
  startDate: string;
  /** `null` for an all-day event. */
  startTime: string | null;
  endDate: string;
  endTime: string | null;
  allDay: boolean;
  rrule: string | null;
  tagIds: string[];
  /** Minutes before the start at which the event warns. */
  reminderOffsets: number[];
  createdAt: string;
  updatedAt: string;
}

/** One appearance of an event on the calendar: the event itself or a repetition of it. */
export interface EventOccurrence {
  eventId: string;
  title: string;
  startDate: string;
  startTime: string | null;
  endDate: string;
  endTime: string | null;
  allDay: boolean;
  recurring: boolean;
  tagIds: string[];
}

export interface EventInput {
  title?: string;
  notes?: string;
  startDate: string;
  startTime?: string;
  endDate?: string;
  endTime?: string;
  rrule?: string;
  tagIds?: string[];
  reminderOffsets?: number[];
}

/** Absent = leave as is; `null` = clear (`startTime: null` makes it an all-day event). */
export interface EventPatch {
  title?: string;
  notes?: string;
  startDate?: string;
  startTime?: string | null;
  endDate?: string;
  endTime?: string;
  rrule?: string | null;
  tagIds?: string[];
  reminderOffsets?: number[];
}

/** A habit with what depends on today: its streaks come from the days that were marked (D-95). */
export interface Habit {
  id: string;
  title: string;
  /** Scheduled weekdays as a bit mask, bit 0 = Monday ... bit 6 = Sunday (127 = every day). */
  days: number;
  time: string | null;
  tagId: string | null;
  position: number;
  isArchived: boolean;
  scheduledToday: boolean;
  doneToday: boolean;
  currentStreak: number;
  bestStreak: number;
  createdAt: string;
  updatedAt: string;
}

export interface HabitInput {
  title: string;
  days?: number;
  time?: string;
  tagId?: string;
}

/** Absent = leave as is; `null` = clear. */
export interface HabitPatch {
  title?: string;
  days?: number;
  time?: string | null;
  tagId?: string | null;
  archived?: boolean;
}

/** A day on which a habit was marked as done. */
export interface HabitLog {
  habitId: string;
  date: string;
}

/** Seconds of focus a task has accumulated (D-97). */
export interface TaskFocusTotal {
  taskId: string;
  seconds: number;
}

/** A focus session on a task; without `endedAt` it is still running (D-97). */
export interface TimeEntry {
  id: string;
  taskId: string;
  startedAt: string;
  endedAt: string | null;
  plannedMinutes: number;
  /** Pauses already over, in seconds (D-99). */
  pausedSeconds: number;
  /** When the pause in progress began; `null` while the session runs. */
  pausedAt: string | null;
  /** Time really spent: end − start − pauses; `0` while it has not ended. */
  spentSeconds: number;
}

export interface Task {
  id: string;
  title: string;
  contentJson: string;
  dueDate: string | null;
  dueTime: string | null;
  /** Minutes from `dueTime` to the end, within the same day; `null` = no duration (D-33). */
  durationMinutes: number | null;
  /** Note the task was created from (D-45); `null` if none or if that note is gone. */
  sourceNoteId: string | null;
  /** How the task repeats (F13-05): `FREQ=WEEKLY;INTERVAL=1`, with anchors added by the backend. */
  rrule: string | null;
  /** 0 none, 1 medium, 2 high, 3 urgent (D-93). */
  priority: number;
  /** Checked and total checklist items, stored by the backend on save (D-93). */
  checklistDone: number;
  checklistTotal: number;
  isCompleted: boolean;
  completedAt: string | null;
  position: number;
  tagIds: string[];
  reminders: Reminder[];
  createdAt: string;
  updatedAt: string;
}

export interface Tag {
  id: string;
  name: string;
  colorHex: string;
  parentId: string | null;
  position: number;
}

export interface TagWithCount extends Tag {
  pendingCount: number;
  /** Live notes with the tag: tags are shared between tasks and notes (D-44). */
  noteCount: number;
}

export interface DayGroup {
  date: string;
  tasks: Task[];
}

export type ListView = "todo" | "agenda" | "range";
export type Section = "todo" | "agenda";

export interface TasksSections {
  todo: Task[];
  agenda: DayGroup[];
}

export interface TasksListQuery {
  view: ListView;
  from?: string;
  to?: string;
  tagId?: string;
}

export interface TaskCreateInput {
  title?: string;
  dueDate?: string;
  dueTime?: string;
  durationMinutes?: number;
  tagIds?: string[];
  contentJson?: string;
  sourceNoteId?: string;
  rrule?: string;
  priority?: number;
}

/** Absent = leave as is; `null` = clear. */
export interface TaskPatch {
  title?: string;
  contentJson?: string;
  dueDate?: string | null;
  dueTime?: string | null;
  durationMinutes?: number | null;
  rrule?: string | null;
  priority?: number;
}

export interface TagUpsertInput {
  id?: string;
  name: string;
  colorHex: string;
  parentId?: string;
}

/** Folder of notes (D-43). `parentId: null` is the top level. */
export interface Folder {
  id: string;
  name: string;
  parentId: string | null;
  position: number;
}

/** `noteCount` counts the live notes directly inside the folder, not in its subfolders. */
export interface FolderWithCount extends Folder {
  noteCount: number;
}

export interface FolderUpsertInput {
  id?: string;
  name: string;
  parentId?: string;
}

export interface Note {
  id: string;
  /** `null` is the inbox. */
  folderId: string | null;
  title: string;
  contentJson: string;
  isPinned: boolean;
  tagIds: string[];
  createdAt: string;
  updatedAt: string;
}

/** A note as listed: a short plain-text excerpt instead of the document. */
/** A saved version of a note, as listed (F13-03). */
export interface NoteVersionSummary {
  id: string;
  noteId: string;
  title: string;
  size: number;
  createdAt: string;
}

export interface NoteVersion {
  id: string;
  noteId: string;
  title: string;
  contentJson: string;
  createdAt: string;
}

/** What an export wrote (F14). */
export interface ExportReport {
  notes: number;
  /** Images written next to the notes. */
  files: number;
}

/** What an import stored (F14). `skipped` are files, notes or images left out. */
export interface ImportReport {
  notes: number;
  folders: number;
  tags: number;
  skipped: number;
  /** Notes already imported before (same origin and content), left out (F17-06). */
  duplicates: number;
}

export type ExportFormat = "markdown" | "html";
export type ExportKind = "all" | "folder" | "note";

/** A named document new notes can start from (F13-02). */
export interface NoteTemplate {
  id: string;
  name: string;
  contentJson: string;
  createdAt: string;
  updatedAt: string;
}

export interface NoteSummary {
  id: string;
  folderId: string | null;
  title: string;
  excerpt: string;
  isPinned: boolean;
  tagIds: string[];
  createdAt: string;
  updatedAt: string;
}

/** `note_titles`: the title of a live note, to label things that point at it. */
export interface NoteTitle {
  id: string;
  title: string;
}

export type NotesScope = "all" | "inbox" | "pinned" | "folder";
export type NotesSort = "updated" | "created" | "title";

export interface NotesListQuery {
  scope: NotesScope;
  /** Required with `scope: "folder"`; includes the subfolders. */
  folderId?: string;
  tagId?: string;
  sort?: NotesSort;
}

export interface NoteCreateInput {
  folderId?: string;
  title?: string;
  contentJson?: string;
  tagIds?: string[];
}

/** Absent = leave as is. */
export interface NotePatch {
  title?: string;
  contentJson?: string;
}

export type ErrorCode =
  "NOT_FOUND" | "VALIDATION" | "CONFLICT" | "DB" | "IO" | "NOTIFY" | "INTERNAL";

export interface SearchHit {
  taskId: string;
  title: string;
  /** Fragment of the notes around the match; empty when the task has no notes. */
  snippet: string;
  dueDate: string | null;
  isCompleted: boolean;
}

/** A result of the unified search (F11-03): a task, a note or a tag. */
export type FoundItem =
  | {
      kind: "task";
      taskId: string;
      title: string;
      snippet: string;
      dueDate: string | null;
      isCompleted: boolean;
    }
  | {
      kind: "note";
      noteId: string;
      title: string;
      snippet: string;
      folderId: string | null;
      isPinned: boolean;
    }
  | {
      kind: "event";
      eventId: string;
      title: string;
      startDate: string;
      startTime: string | null;
      allDay: boolean;
    }
  | {
      kind: "habit";
      habitId: string;
      title: string;
      isArchived: boolean;
    }
  | {
      kind: "tag";
      tagId: string;
      name: string;
      colorHex: string;
      noteCount: number;
      pendingCount: number;
    };

/** A stored image of a note (D-47); `id` is the SHA-256 of its content. */
export interface Attachment {
  id: string;
  mime: string;
  sizeBytes: number;
}

/** Settings the backend accepts (`settings_get` / `settings_set`). */
export type SettingKey =
  | "view_mode"
  | "theme"
  | "window_effect"
  | "default_notify_time"
  | "app_section"
  | "notes_scope"
  | "notes_open"
  | "notes_recent"
  | "saved_searches"
  | "auto_backup"
  | "auto_backup_keep"
  | "auto_backup_last";

/** Colour theme: follow the system or force one (F6-05). */
export type ThemeMode = "system" | "light" | "dark";

/** What the main area shows (F-10). */
export type ViewMode = "list" | "day" | "week" | "month";

/** Public facts about the installation, shown in Ajustes › Información (`app_info`). */
export interface AppInfo {
  version: string;
  tauriVersion: string;
  webviewVersion: string | null;
  sqliteVersion: string;
  schemaVersion: number;
  os: string;
  arch: string;
  build: "release" | "debug";
  dataDir: string;
  databaseBytes: number;
  tasks: number;
  notes: number;
  tags: number;
  attachments: number;
}
