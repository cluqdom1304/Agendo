import { SvelteSet } from "svelte/reactivity";

import * as backend from "../api";
import { toInstant, toLocalIsoDate } from "../dates";
import { t } from "../i18n";
import { hasNotes } from "../features/notes/content";
import { groupByDay, sortTodo, splitCompleted } from "../ordering";
import type { SearchHit, Task, TaskPatch, TasksSections } from "../types";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface TasksApi {
  tasksList(query: { view: "todo" | "agenda" }): Promise<TasksSections>;
  taskCreate(input: { title?: string; dueDate?: string }): Promise<Task>;
  taskUpdate(id: string, patch: TaskPatch): Promise<Task>;
  search(query: string): Promise<SearchHit[]>;
  taskSetCompleted(id: string, completed: boolean): Promise<Task>;
  taskDelete(id: string): Promise<void>;
  taskRestore(id: string): Promise<Task>;
}

const realApi: TasksApi = {
  tasksList: (query) => backend.tasksList(query),
  taskCreate: (input) => backend.taskCreate(input),
  taskUpdate: (id, patch) => backend.taskUpdate(id, patch),
  taskSetCompleted: (id, completed) => backend.taskSetCompleted(id, completed),
  taskDelete: (id) => backend.taskDelete(id),
  taskRestore: (id) => backend.taskRestore(id),
  search: (query) => backend.search(query),
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

export class TasksStore {
  /** Immutable task objects: an update swaps one object, so untouched rows keep their identity. */
  tasks = $state.raw<Task[]>([]);
  status = $state<"idle" | "loading" | "ready" | "error">("idle");
  selectedId = $state<string | null>(null);
  draft = $state.raw<Draft | null>(null);
  undoStack = $state.raw<UndoEntry[]>([]);
  notice = $state.raw<Notice | null>(null);
  /** The task whose notes are open under its row; one at a time keeps the document short. */
  notesOpenId = $state<string | null>(null);
  /** Rows playing their exit animation; per-key reactivity re-renders only the row concerned. */
  readonly settling = new SvelteSet<string>();

  todoBlock = $derived.by<TaskBlock>(() => {
    const undated = this.tasks.filter((task) => task.dueDate === null);
    return { date: null, ...splitCompleted(sortTodo(undated)) };
  });

  /** Days, latest first. The draft's day is included even if it has no tasks yet. */
  agendaBlocks = $derived.by<TaskBlock[]>(() => {
    const blocks: TaskBlock[] = groupByDay(this.tasks).map((day) => ({
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
      const [todo, agenda] = await Promise.all([
        this.#api.tasksList({ view: "todo" }),
        this.#api.tasksList({ view: "agenda" }),
      ]);
      this.tasks = [...todo.todo, ...agenda.agenda.flatMap((day) => day.tasks)];
      this.status = "ready";
    } catch (error) {
      this.status = "error";
      this.#report(error);
    }
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

  // ---- editing --------------------------------------------------------------------------

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
    if (this.notesOpenId === id) this.notesOpenId = null;
    const guess: Task = {
      ...before,
      isCompleted: completed,
      completedAt: completed ? toInstant(this.#now()) : null,
    };
    const answer: { task?: Task; failure?: { error: unknown } } = {};
    // Read through a function: the answer is filled in by callbacks TypeScript cannot see.
    const answered = () => answer;
    this.settling.add(id);
    const request = this.#api.taskSetCompleted(id, completed).then(
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

  toggleNotes(id: string): void {
    this.notesOpenId = this.notesOpenId === id ? null : id;
  }

  /** Saves the notes document; throws after reporting, so the editor keeps its pending text. */
  async saveNotes(id: string, contentJson: string): Promise<void> {
    try {
      this.#replace(await this.#api.taskUpdate(id, { contentJson }));
    } catch (error) {
      this.#report(error);
      throw error;
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
    this.notesOpenId = hasNotes(task.contentJson) ? id : null;
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
    this.#clearNotice();
    const key = ++this.#keys;
    this.notice = { key, message: describeError(error) };
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
