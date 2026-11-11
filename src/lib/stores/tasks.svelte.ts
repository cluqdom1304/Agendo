import { SvelteSet } from "svelte/reactivity";

import * as backend from "../api";
import { toInstant, toLocalIsoDate } from "../dates";
import { t } from "../i18n";
import { hasNotes } from "../features/notes/content";
import { groupByDay, sortTodo, splitCompleted } from "../ordering";
import type {
  ReminderInput,
  SearchHit,
  Task,
  TaskPatch,
  TasksSections,
} from "../types";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface TasksApi {
  tasksList(query: { view: "todo" | "agenda" }): Promise<TasksSections>;
  taskCreate(input: {
    title?: string;
    dueDate?: string;
    dueTime?: string;
    durationMinutes?: number;
    tagIds?: string[];
    contentJson?: string;
  }): Promise<Task>;
  taskUpdate(id: string, patch: TaskPatch): Promise<Task>;
  taskTagsSet(taskId: string, tagIds: string[]): Promise<Task>;
  search(query: string): Promise<SearchHit[]>;
  taskSetCompleted(
    id: string,
    completed: boolean,
    completedDate?: string,
  ): Promise<Task>;
  taskDelete(id: string): Promise<void>;
  taskRestore(id: string): Promise<Task>;
  remindersSet(taskId: string, reminders: ReminderInput[]): Promise<Task>;
}

const realApi: TasksApi = {
  tasksList: (query) => backend.tasksList(query),
  taskCreate: (input) => backend.taskCreate(input),
  taskUpdate: (id, patch) => backend.taskUpdate(id, patch),
  taskTagsSet: (taskId, tagIds) => backend.taskTagsSet(taskId, tagIds),
  taskSetCompleted: (id, completed, completedDate) =>
    backend.taskSetCompleted(id, completed, completedDate),
  taskDelete: (id) => backend.taskDelete(id),
  taskRestore: (id) => backend.taskRestore(id),
  search: (query) => backend.search(query),
  remindersSet: (taskId, reminders) => backend.remindersSet(taskId, reminders),
};

/** Where a new task goes: the "por hacer" list or one day of the agenda. */
export type DraftTarget =
  { section: "todo" } | { section: "agenda"; date: string };

/** An unsaved row at the end of a block, waiting for its title. */
export interface Draft {
  target: DraftTarget;
  /** Changes with every new draft so the input is rebuilt empty. */
  key: number;
}

export interface UndoEntry {
  key: number;
  task: Task;
}

export interface Notice {
  key: number;
  message: string;
  kind: "info" | "error";
}

export interface TaskBlock {
  /** `YYYY-MM-DD`, or `null` for POR HACER. */
  date: string | null;
  open: Task[];
  completed: Task[];
}

export interface TasksStoreOptions {
  api?: TasksApi;
  now?: () => Date;
  /** Exit animation of a completing row (SPEC §8.4); 0 when the user reduces motion. */
  settleMs?: () => number;
  undoMs?: number;
  noticeMs?: number;
}

const COMPLETE_ANIMATION_MS = 220;

function prefersReducedMotion(): boolean {
  return (
    typeof matchMedia === "function" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches
  );
}

function describeError(raw: unknown): string {
  const { code, message } = backend.parseError(raw);
  const base = t.errors[code];
  return code === "VALIDATION" && message ? `${base}: ${message}` : base;
}

function isSameTarget(a: DraftTarget, b: DraftTarget): boolean {
  return a.section === "todo"
    ? b.section === "todo"
    : b.section === "agenda" && b.date === a.date;
}

/** Where earlier versions remembered hidden notes; cleared so they cannot stay hidden. */
const LEGACY_COLLAPSED_KEY = "agenda.notesCollapsed";

function forgetStoredCollapse(): void {
  try {
    localStorage.removeItem(LEGACY_COLLAPSED_KEY);
  } catch {
    // Storage may be unavailable; there is nothing to forget then.
  }
}

export class TasksStore {
  /** Immutable task objects: an update swaps one object, so untouched rows keep their identity. */
  tasks = $state.raw<Task[]>([]);
  status = $state<"idle" | "loading" | "ready" | "error">("idle");
  selectedId = $state<string | null>(null);
  draft = $state.raw<Draft | null>(null);
  undoStack = $state.raw<UndoEntry[]>([]);
  notice = $state.raw<Notice | null>(null);
  /** Tasks whose empty notes the user opened to write; tasks with text are open unless collapsed. */
  readonly #notesOpened = new SvelteSet<string>();
  /** The task whose notes the user has just opened: its editor takes the focus once. */
  notesFocusId = $state<string | null>(null);
  /** Tasks with text whose notes the user hid; every session starts with all of them shown. */
  readonly #notesCollapsed = new SvelteSet<string>();
  /** Notes hidden for an instant so the editor flushes before its row remounts. */
  readonly #notesSuspended = new SvelteSet<string>();
  #notesSaving = 0;
  /** The task whose reminder panel is open under its row. */
  remindersOpenId = $state<string | null>(null);
  /** Rows playing their exit animation; per-key reactivity re-renders only the row concerned. */
  readonly settling = new SvelteSet<string>();

  /** Which tasks the views show (the tag filter); everything until the page sets it. */
  filter = $state.raw<(task: Task) => boolean>(() => true);
  visibleTasks = $derived(this.tasks.filter(this.filter));
  /** There are tasks but the filter hides all of them. */
  allFiltered = $derived(
    this.tasks.length > 0 && this.visibleTasks.length === 0,
  );

  todoBlock = $derived.by<TaskBlock>(() => {
    const undated = this.visibleTasks.filter((task) => task.dueDate === null);
    return { date: null, ...splitCompleted(sortTodo(undated)) };
  });

  /** Days, latest first. The draft's day is included even if it has no tasks yet. */
  agendaBlocks = $derived.by<TaskBlock[]>(() => {
    const blocks: TaskBlock[] = groupByDay(this.visibleTasks).map((day) => ({
      date: day.date,
      ...splitCompleted(day.tasks),
    }));
    const target = this.draft?.target;
    if (
      target?.section === "agenda" &&
      !blocks.some((block) => block.date === target.date)
    ) {
      blocks.push({ date: target.date, open: [], completed: [] });
      blocks.sort((a, b) => ((a.date ?? "") < (b.date ?? "") ? 1 : -1));
    }
    return blocks;
  });

  isEmpty = $derived(this.tasks.length === 0);
  todoPendingCount = $derived(this.todoBlock.open.length);

  readonly #api: TasksApi;
  readonly #now: () => Date;
  readonly #settleMs: () => number;
  readonly #undoMs: number;
  readonly #noticeMs: number;
  #keys = 0;
  #creating = false;
  // Timer handles are bookkeeping, not UI state.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  #undoTimers = new Map<number, ReturnType<typeof setTimeout>>();
  #noticeTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(options: TasksStoreOptions = {}) {
    this.#api = options.api ?? realApi;
    forgetStoredCollapse();
    // A clock reading, not reactive state.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    this.#now = options.now ?? (() => new Date());
    this.#settleMs =
      options.settleMs ??
      (() => (prefersReducedMotion() ? 0 : COMPLETE_ANIMATION_MS));
    this.#undoMs = options.undoMs ?? 5000;
    this.#noticeMs = options.noticeMs ?? 6000;
  }

  async load(): Promise<void> {
    this.status = "loading";
    try {
      this.tasks = await this.#fetchAll();
      this.status = "ready";
    } catch (error) {
      this.status = "error";
      this.#report(error);
    }
  }

  async #fetchAll(): Promise<Task[]> {
    const [todo, agenda] = await Promise.all([
      this.#api.tasksList({ view: "todo" }),
      this.#api.tasksList({ view: "agenda" }),
    ]);
    return [...todo.todo, ...agenda.agenda.flatMap((day) => day.tasks)];
  }

  find(id: string): Task | undefined {
    return this.tasks.find((task) => task.id === id);
  }

  // ---- creation -------------------------------------------------------------------------

  /** Where Ctrl+N puts a task: the day of the selected task, else POR HACER. */
  defaultTarget(): DraftTarget {
    const selected = this.selectedId ? this.find(this.selectedId) : undefined;
    return selected?.dueDate
      ? { section: "agenda", date: selected.dueDate }
      : { section: "todo" };
  }

  todayTarget(): DraftTarget {
    return { section: "agenda", date: toLocalIsoDate(this.#now()) };
  }

  startDraft(target: DraftTarget): void {
    this.draft = { target, key: ++this.#keys };
  }

  cancelDraft(key: number): void {
    if (this.draft?.key === key) this.draft = null;
  }

  isDraftFor(target: DraftTarget): boolean {
    return this.draft !== null && isSameTarget(this.draft.target, target);
  }

  /**
   * Saves the draft `key`. With `advance` (Enter) a fresh draft opens in the same block so
   * several tasks can be typed in a row. An empty title just closes the draft. If saving
   * fails the draft stays open with its text.
   */
  async commitDraft(
    key: number,
    rawTitle: string,
    advance: boolean,
  ): Promise<void> {
    const draft = this.draft;
    if (!draft || draft.key !== key || this.#creating) return;
    const title = rawTitle.trim();
    if (!title) {
      this.draft = null;
      return;
    }
    this.#creating = true;
    try {
      const created = await this.#api.taskCreate({
        title,
        ...(draft.target.section === "agenda"
          ? { dueDate: draft.target.date }
          : {}),
      });
      this.tasks = [...this.tasks, created];
      this.draft = advance ? { target: draft.target, key: ++this.#keys } : null;
    } catch (error) {
      this.#report(error);
    } finally {
      this.#creating = false;
    }
  }

  /** Creates a task already placed on a day and, optionally, a time (calendar quick creation). */
  async createScheduled(
    rawTitle: string,
    dueDate: string,
    dueTime?: string,
    durationMinutes?: number,
  ): Promise<boolean> {
    const title = rawTitle.trim();
    if (!title || this.#creating) return false;
    this.#creating = true;
    try {
      const created = await this.#api.taskCreate({
        title,
        dueDate,
        ...(dueTime ? { dueTime } : {}),
        ...(dueTime && durationMinutes ? { durationMinutes } : {}),
      });
      this.tasks = [...this.tasks, created];
      this.#warnIfHidden(created);
      return true;
    } catch (error) {
      this.#report(error);
      return false;
    } finally {
      this.#creating = false;
    }
  }

  /**
   * Creates a task with everything the creation dialog collects. The task is saved first; if
   * its reminders are then refused the task stays and the error is reported, so nothing typed
   * is lost twice. Returns whether the task was created.
   */
  async createDetailed(input: {
    title: string;
    dueDate?: string;
    dueTime?: string;
    durationMinutes?: number;
    tagId?: string;
    contentJson?: string;
    reminders?: ReminderInput[];
  }): Promise<boolean> {
    const title = input.title.trim();
    if (!title || this.#creating) return false;
    this.#creating = true;
    try {
      let created = await this.#api.taskCreate({
        title,
        ...(input.dueDate ? { dueDate: input.dueDate } : {}),
        ...(input.dueDate && input.dueTime ? { dueTime: input.dueTime } : {}),
        ...(input.dueDate && input.dueTime && input.durationMinutes
          ? { durationMinutes: input.durationMinutes }
          : {}),
        ...(input.tagId ? { tagIds: [input.tagId] } : {}),
        ...(input.contentJson ? { contentJson: input.contentJson } : {}),
      });
      this.tasks = [...this.tasks, created];
      this.#warnIfHidden(created);
      if (input.reminders && input.reminders.length > 0) {
        try {
          created = await this.#api.remindersSet(created.id, input.reminders);
          this.#replace(created);
        } catch (error) {
          this.#report(error);
        }
      }
      return true;
    } catch (error) {
      this.#report(error);
      return false;
    } finally {
      this.#creating = false;
    }
  }

  // ---- editing --------------------------------------------------------------------------

  /**
   * Saves what the edit dialog collects. Not optimistic: the dialog stays open until the
   * backend accepts the change. A task that is done takes no reminders, so `reminders` is
   * ignored for it. Returns whether everything was saved.
   */
  async saveDetails(
    id: string,
    input: {
      title: string;
      dueDate: string | null;
      dueTime: string | null;
      durationMinutes: number | null;
      /** Absent = leave as is; `null` = remove the tag. */
      tagId?: string | null;
      reminders?: ReminderInput[];
    },
  ): Promise<boolean> {
    const before = this.find(id);
    const title = input.title.trim();
    if (!before || !title) return false;
    const patch: TaskPatch = {};
    if (title !== before.title) patch.title = title;
    if (input.dueDate !== before.dueDate) patch.dueDate = input.dueDate;
    const dueTime = input.dueDate ? input.dueTime : null;
    const duration = dueTime ? input.durationMinutes : null;
    if (dueTime !== before.dueTime) patch.dueTime = dueTime;
    if (duration !== before.durationMinutes) patch.durationMinutes = duration;
    this.#suspendNotes(id);
    try {
      if (Object.keys(patch).length > 0) {
        this.#replace(await this.#api.taskUpdate(id, patch));
      }
      if (
        input.tagId !== undefined &&
        input.tagId !== (before.tagIds[0] ?? null)
      ) {
        this.#replace(
          await this.#api.taskTagsSet(id, input.tagId ? [input.tagId] : []),
        );
      }
      if (input.reminders && !before.isCompleted) {
        this.#replace(await this.#api.remindersSet(id, input.reminders));
      }
      const saved = this.find(id);
      if (saved) this.#warnIfHidden(saved);
      return true;
    } catch (error) {
      this.#report(error);
      return false;
    }
  }

  /** Optimistic: the new title shows at once and is restored if the backend refuses it. */
  async rename(id: string, rawTitle: string): Promise<void> {
    const before = this.find(id);
    const title = rawTitle.trim();
    if (!before || !title || title === before.title) return;
    this.#replace({ ...before, title });
    try {
      this.#replace(await this.#api.taskUpdate(id, { title }));
    } catch (error) {
      this.#replace(before);
      this.#report(error);
    }
  }

  /** Optimistic: the task moves to its new day at once; a refusal puts it back. Clearing the date clears the time too (a task without date has no schedule). */
  async setDueDate(id: string, date: string | null): Promise<void> {
    const before = this.find(id);
    if (!before || before.dueDate === date) return;
    this.#suspendNotes(id);
    this.#replace({
      ...before,
      dueDate: date,
      dueTime: date === null ? null : before.dueTime,
    });
    try {
      this.#replace(
        await this.#api.taskUpdate(
          id,
          date === null ? { dueDate: null, dueTime: null } : { dueDate: date },
        ),
      );
    } catch (error) {
      this.#replace(before);
      this.#report(error);
    }
  }

  /** Optimistic like the date; the answer carries the reminders the backend recalculated. */
  async setDueTime(id: string, time: string | null): Promise<void> {
    const before = this.find(id);
    if (!before || before.dueDate === null || before.dueTime === time) return;
    this.#replace({ ...before, dueTime: time });
    try {
      this.#replace(await this.#api.taskUpdate(id, { dueTime: time }));
    } catch (error) {
      this.#replace(before);
      this.#report(error);
    }
  }

  /** Optimistic: the task takes its tag (or loses it) at once; a refusal puts it back. */
  async setTag(id: string, tagId: string | null): Promise<void> {
    const before = this.find(id);
    if (!before || (before.tagIds[0] ?? null) === tagId) return;
    this.#replace({ ...before, tagIds: tagId ? [tagId] : [] });
    try {
      this.#replace(await this.#api.taskTagsSet(id, tagId ? [tagId] : []));
    } catch (error) {
      this.#replace(before);
      this.#report(error);
    }
  }

  // ---- reminders ------------------------------------------------------------------------

  toggleReminders(id: string): void {
    this.remindersOpenId = this.remindersOpenId === id ? null : id;
  }

  /** Replaces the task's unsent reminders; the panel shows what the backend stored. */
  async setReminders(id: string, reminders: ReminderInput[]): Promise<void> {
    try {
      this.#replace(await this.#api.remindersSet(id, reminders));
    } catch (error) {
      this.#report(error);
    }
  }

  /** The scheduler delivered a reminder: refresh the task (it is now sent) and tell the user. */
  async onReminderFired(taskId: string): Promise<void> {
    try {
      const fresh = await this.#fetchAll();
      this.tasks = fresh;
    } catch (error) {
      this.#report(error);
      return;
    }
    const title = this.find(taskId)?.title.trim();
    this.#inform(t.reminders.fired(title || t.reminders.untitled));
  }

  /**
   * Completing plays the exit animation first, then the row moves to "Completadas" without
   * waiting for the backend; the backend's answer replaces the local guess, and a refusal
   * puts the task back as it was.
   */
  async setCompleted(id: string, completed: boolean): Promise<void> {
    const before = this.find(id);
    if (!before || before.isCompleted === completed || this.settling.has(id)) {
      return;
    }
    // The row changes place; closing the notes first lets the editor save before it remounts.
    this.#suspendNotes(id);
    // D-23: a completed task always sits in the "Completadas" of the day it was completed.
    const completedDate = completed ? toLocalIsoDate(this.#now()) : undefined;
    const guess: Task = {
      ...before,
      dueDate: completedDate ?? before.dueDate,
      isCompleted: completed,
      completedAt: completed ? toInstant(this.#now()) : null,
    };
    const answer: { task?: Task; failure?: { error: unknown } } = {};
    // Read through a function: the answer is filled in by callbacks TypeScript cannot see.
    const answered = () => answer;
    this.settling.add(id);
    const request = this.#api
      .taskSetCompleted(id, completed, completedDate)
      .then(
        (task) => {
          answer.task = task;
        },
        (error: unknown) => {
          answer.failure = { error };
        },
      );
    await new Promise((resolve) => setTimeout(resolve, this.#settleMs()));
    this.settling.delete(id);
    const early = answered().failure;
    if (early) {
      this.#report(early.error);
      return;
    }
    this.#replace(guess);
    await request;
    const { task, failure } = answered();
    if (task) {
      this.#replace(task);
    } else if (failure) {
      this.#replace(before);
      this.#report(failure.error);
    }
  }

  // ---- notes and search --------------------------------------------------------------

  /** Notes with text show by default (also after a restart); the user can collapse them. */
  isNotesOpen(id: string): boolean {
    if (this.#notesSuspended.has(id)) return false;
    const task = this.find(id);
    if (!task) return false;
    return (
      this.#notesOpened.has(id) ||
      (hasNotes(task.contentJson) && !this.#notesCollapsed.has(id))
    );
  }

  toggleNotes(id: string): void {
    if (this.isNotesOpen(id)) {
      this.#notesOpened.delete(id);
      this.#notesCollapsed.add(id);
      this.notesFocusId = null;
    } else {
      this.#notesOpened.add(id);
      this.#notesCollapsed.delete(id);
      // Only a note the user has just opened takes the cursor; the ones shown at startup
      // must not.
      this.notesFocusId = id;
    }
  }

  /** Puts away notes that were opened to write and left empty. */
  closeEmptyNotes(id: string): void {
    const task = this.find(id);
    if (task && hasNotes(task.contentJson)) return;
    this.#notesOpened.delete(id);
  }

  #suspendNotes(id: string): void {
    if (!this.isNotesOpen(id)) return;
    this.#notesSuspended.add(id);
    setTimeout(() => this.#resumeNotes(), 0);
  }

  #resumeNotes(): void {
    if (this.#notesSaving === 0) this.#notesSuspended.clear();
  }

  /** Saves the notes document; throws after reporting, so the editor keeps its pending text. */
  async saveNotes(id: string, contentJson: string): Promise<void> {
    this.#notesSaving++;
    try {
      this.#replace(await this.#api.taskUpdate(id, { contentJson }));
    } catch (error) {
      this.#report(error);
      throw error;
    } finally {
      this.#notesSaving--;
      this.#resumeNotes();
    }
  }

  search(query: string): Promise<SearchHit[]> {
    return this.#api.search(query);
  }

  /** Selects a search result and opens its notes when it has any. */
  reveal(id: string): boolean {
    const task = this.find(id);
    if (!task) return false;
    this.selectedId = id;
    if (hasNotes(task.contentJson)) this.#notesCollapsed.delete(id);
    return true;
  }

  // ---- deletion -------------------------------------------------------------------------

  /** The task leaves the list at once; for `undoMs` it can be brought back (F-02). */
  async remove(id: string): Promise<void> {
    const task = this.find(id);
    if (!task) return;
    this.tasks = this.tasks.filter((candidate) => candidate.id !== id);
    if (this.selectedId === id) this.selectedId = null;
    try {
      await this.#api.taskDelete(id);
    } catch (error) {
      this.#restoreLocally(task);
      this.#report(error);
      return;
    }
    const entry: UndoEntry = { key: ++this.#keys, task };
    this.undoStack = [...this.undoStack, entry];
    this.#undoTimers.set(
      entry.key,
      setTimeout(() => this.#expire(entry.key), this.#undoMs),
    );
  }

  /** Brings back the most recently deleted task. */
  async undoDelete(): Promise<void> {
    const entry = this.undoStack.at(-1);
    if (!entry) return;
    this.#expire(entry.key);
    this.#restoreLocally(entry.task);
    try {
      this.#replace(await this.#api.taskRestore(entry.task.id));
    } catch (error) {
      this.tasks = this.tasks.filter((task) => task.id !== entry.task.id);
      this.#report(error);
    }
  }

  dismissNotice(): void {
    this.#clearNotice();
  }

  /** Cancels timers; call when the store is discarded. */
  dispose(): void {
    for (const timer of this.#undoTimers.values()) clearTimeout(timer);
    this.#undoTimers.clear();
    this.#clearNotice();
  }

  // ---- internals ------------------------------------------------------------------------

  #replace(task: Task): void {
    if (!this.tasks.some((candidate) => candidate.id === task.id)) return;
    this.tasks = this.tasks.map((candidate) =>
      candidate.id === task.id ? task : candidate,
    );
  }

  #restoreLocally(task: Task): void {
    if (this.tasks.some((candidate) => candidate.id === task.id)) return;
    this.tasks = [...this.tasks, task];
  }

  #expire(key: number): void {
    const timer = this.#undoTimers.get(key);
    if (timer !== undefined) clearTimeout(timer);
    this.#undoTimers.delete(key);
    this.undoStack = this.undoStack.filter((entry) => entry.key !== key);
  }

  #report(error: unknown): void {
    this.#show("error", describeError(error));
  }

  /** A task the filter hides would seem lost right after saving it. */
  #warnIfHidden(task: Task): void {
    if (!this.filter(task)) this.#inform(t.tags.hiddenByFilter);
  }

  #inform(message: string): void {
    this.#show("info", message);
  }

  #show(kind: Notice["kind"], message: string): void {
    this.#clearNotice();
    const key = ++this.#keys;
    this.notice = { key, message, kind };
    this.#noticeTimer = setTimeout(() => {
      if (this.notice?.key === key) this.notice = null;
    }, this.#noticeMs);
  }

  #clearNotice(): void {
    if (this.#noticeTimer !== null) clearTimeout(this.#noticeTimer);
    this.#noticeTimer = null;
    this.notice = null;
  }
}

export const tasksStore = new TasksStore();
