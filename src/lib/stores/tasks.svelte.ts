import * as backend from "../api";
import { toLocalIsoDate } from "../dates";
import { t } from "../i18n";
import type { Capture } from "../naturalInput";
import { sameRule } from "../recurrence";
import { TaskBatch } from "./taskBatch.svelte";
import { TaskCompletion } from "./taskCompletion.svelte";
import { TaskCreator } from "./taskCreator.svelte";
import { flattenSections, mergeTasks } from "./taskMerge";
import { TaskNotes } from "./taskNotes.svelte";
import { TaskNotices } from "./taskNotices.svelte";
import { TaskReminders } from "./taskReminders.svelte";
import { TaskUndo } from "./taskUndo.svelte";
import type { TasksHost } from "./tasksHost";
import type {
  Draft,
  DraftTarget,
  NewTaskDetails,
  Notice,
  TaskBlock,
  UndoEntry,
} from "./tasksTypes";
import { groupByDay, sortTodo, splitCompleted } from "../ordering";
import {
  isFiltering,
  matchesFilter,
  NO_FILTER,
  type ListFilter,
} from "./listFilter";
import type {
  ReminderInput,
  SearchHit,
  Task,
  TaskPatch,
  TasksSections,
} from "../types";

export type {
  Draft,
  DraftTarget,
  NewTaskDetails,
  Notice,
  TaskBlock,
  UndoEntry,
} from "./tasksTypes";

/** The slice of `api.ts` the store uses; tests pass a fake. */
export interface TasksApi {
  tasksListAll(tagId?: string): Promise<TasksSections>;
  taskCreate(input: {
    title?: string;
    dueDate?: string;
    dueTime?: string;
    durationMinutes?: number;
    tagIds?: string[];
    contentJson?: string;
    sourceNoteId?: string;
    rrule?: string;
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
  tasksListAll: (tagId) => backend.tasksListAll(tagId),
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

function sameSet(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((id) => b.includes(id));
}

export class TasksStore {
  /** Immutable task objects: an update swaps one object, so untouched rows keep their identity. */
  tasks = $state.raw<Task[]>([]);
  status = $state<"idle" | "loading" | "ready" | "error">("idle");
  selectedId = $state<string | null>(null);

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

  /** Filters of the «Tareas» view (F20-05); the other views ignore them. */
  listFilter = $state.raw<ListFilter>(NO_FILTER);
  listFiltering = $derived(isFiltering(this.listFilter));

  /** POR HACER as the list draws it, after its filters. */
  listTodoBlock = $derived.by<TaskBlock>(() =>
    this.listFiltering ? this.#narrow(this.todoBlock) : this.todoBlock,
  );

  /** The days as the list draws them: a day left empty by the filters is dropped. */
  listAgendaBlocks = $derived.by<TaskBlock[]>(() => {
    if (!this.listFiltering) return this.agendaBlocks;
    const target = this.draft?.target;
    return this.agendaBlocks
      .map((block) => this.#narrow(block))
      .filter(
        (block) =>
          block.open.length + block.completed.length > 0 ||
          (target?.section === "agenda" && target.date === block.date),
      );
  });

  /** Ids in the order the list draws them (POR HACER first, then each day). */
  visibleOrder = $derived.by<string[]>(() =>
    [this.listTodoBlock, ...this.listAgendaBlocks].flatMap((block) =>
      [...block.open, ...block.completed].map((task) => task.id),
    ),
  );

  /** Multiple selection and batch actions (F17-03). */
  readonly batch: TaskBatch;

  isEmpty = $derived(this.tasks.length === 0);

  /** Rows playing their exit animation; per-key reactivity re-renders only the row concerned. */
  readonly settling: TaskCompletion["settling"];

  readonly #api: TasksApi;
  readonly #now: () => Date;
  readonly #notices: TaskNotices;
  readonly #notes: TaskNotes;
  readonly #undo: TaskUndo;
  readonly #reminders: TaskReminders;
  readonly #completion: TaskCompletion;
  readonly #creator: TaskCreator;
  #refreshing: Promise<boolean> | null = null;
  #refreshAgain = false;

  constructor(options: TasksStoreOptions = {}) {
    this.#api = options.api ?? realApi;
    // A clock reading, not reactive state.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
    this.#now = options.now ?? (() => new Date());
    const settleMs =
      options.settleMs ??
      (() => (prefersReducedMotion() ? 0 : COMPLETE_ANIMATION_MS));
    const undoMs = options.undoMs ?? 5000;
    this.#notices = new TaskNotices(options.noticeMs ?? 6000);
    const host = this.#makeHost();
    this.#notes = new TaskNotes(host);
    this.#undo = new TaskUndo(host, undoMs);
    this.#reminders = new TaskReminders(host);
    this.#completion = new TaskCompletion(host, settleMs);
    this.#creator = new TaskCreator(host);
    this.settling = this.#completion.settling;
    this.batch = new TaskBatch({
      api: this.#api,
      tasks: () => this.tasks,
      order: () => this.visibleOrder,
      today: () => toLocalIsoDate(this.#now()),
      refresh: async () => {
        await this.refresh();
      },
      report: (error) => this.#notices.report(error),
      inform: (message) => this.#notices.inform(message),
      undoMs,
    });
  }

  #makeHost(): TasksHost {
    return {
      api: this.#api,
      tasks: () => this.tasks,
      setTasks: (tasks) => {
        this.tasks = tasks;
      },
      find: (id) => this.find(id),
      replace: (task) => this.#replace(task),
      restoreLocally: (task) => this.#restoreLocally(task),
      deselect: (id) => {
        if (this.selectedId === id) this.selectedId = null;
      },
      report: (error) => this.#notices.report(error),
      inform: (message) => this.#notices.inform(message),
      warnIfHidden: (task) => this.#warnIfHidden(task),
      now: () => this.#now(),
      refresh: () => this.refresh(),
      suspendNotes: (id) => this.#notes.suspend(id),
    };
  }

  // ---- state owned by the composed pieces ---------------------------------------------

  get draft(): Draft | null {
    return this.#creator.draft;
  }

  get undoStack(): UndoEntry[] {
    return this.#undo.stack;
  }

  set undoStack(stack: UndoEntry[]) {
    this.#undo.stack = stack;
  }

  get notice(): Notice | null {
    return this.#notices.notice;
  }

  set notice(notice: Notice | null) {
    if (notice === null) this.#notices.dismiss();
  }

  /** The task whose notes the user has just opened: its editor takes the focus once. */
  get notesFocusId(): string | null {
    return this.#notes.focusId;
  }

  set notesFocusId(id: string | null) {
    this.#notes.focusId = id;
  }

  /** The task whose reminder panel is open under its row. */
  get remindersOpenId(): string | null {
    return this.#reminders.openId;
  }

  set remindersOpenId(id: string | null) {
    this.#reminders.openId = id;
  }

  // ---- loading --------------------------------------------------------------------------

  async load(): Promise<void> {
    this.status = "loading";
    try {
      this.tasks = flattenSections(await this.#api.tasksListAll());
      this.status = "ready";
    } catch (error) {
      this.status = "error";
      this.#notices.report(error);
    }
  }

  /**
   * Reads everything again, after a change made behind the interface's back. Calls that arrive
   * while a read is in flight share one follow-up read, so a burst costs two reads at most and
   * the last change is never missed. Resolves `false` when the read failed (already reported).
   */
  refresh(): Promise<boolean> {
    if (this.#refreshing) {
      this.#refreshAgain = true;
      return this.#refreshing;
    }
    const run = (async () => {
      let ok = true;
      try {
        do {
          this.#refreshAgain = false;
          const fresh = flattenSections(await this.#api.tasksListAll());
          this.tasks = mergeTasks(this.tasks, fresh);
        } while (this.#refreshAgain);
      } catch (error) {
        this.#notices.report(error);
        ok = false;
      } finally {
        this.#refreshing = null;
      }
      return ok;
    })();
    this.#refreshing = run;
    return run;
  }

  /** A block with only the tasks the list filters let through. */
  #narrow(block: TaskBlock): TaskBlock {
    const today = toLocalIsoDate(this.#now());
    const keep = (task: Task) => matchesFilter(task, this.listFilter, today);
    return {
      date: block.date,
      open: block.open.filter(keep),
      completed: block.completed.filter(keep),
    };
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

  /** Today's date, `YYYY-MM-DD`, from the store's clock. */
  today(): string {
    return toLocalIsoDate(this.#now());
  }

  todayTarget(): DraftTarget {
    return { section: "agenda", date: toLocalIsoDate(this.#now()) };
  }

  startDraft(target: DraftTarget): void {
    this.#creator.startDraft(target);
  }

  cancelDraft(key: number): void {
    this.#creator.cancelDraft(key);
  }

  isDraftFor(target: DraftTarget): boolean {
    return this.#creator.isDraftFor(target);
  }

  commitDraft(
    key: number,
    rawTitle: string,
    advance: boolean,
    capture?: Capture | null,
  ): Promise<void> {
    return this.#creator.commitDraft(key, rawTitle, advance, capture);
  }

  createScheduled(
    rawTitle: string,
    dueDate: string,
    dueTime?: string,
    durationMinutes?: number,
  ): Promise<boolean> {
    return this.#creator.createScheduled(
      rawTitle,
      dueDate,
      dueTime,
      durationMinutes,
    );
  }

  /**
   * Creates a task with everything the creation dialog collects. The task is saved first; if
   * its reminders are then refused the task stays and the error is reported, so nothing typed
   * is lost twice. Returns whether the task was created.
   */
  async createDetailed(input: NewTaskDetails): Promise<boolean> {
    return (await this.create(input)) !== null;
  }

  /** Like `createDetailed`, but hands back the task (a note needs its id to point at it). */
  create(input: NewTaskDetails): Promise<Task | null> {
    return this.#creator.create(input);
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
      /** Absent = leave as is; `[]` = remove every tag. */
      tagIds?: string[];
      reminders?: ReminderInput[];
      /** Absent = leave as is; `null` = stop repeating (F13-05). */
      rrule?: string | null;
      /** Absent = leave as is (D-93). */
      priority?: number;
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
    if (input.rrule !== undefined) {
      const rule = input.dueDate ? input.rrule : null;
      if (!sameRule(rule, before.rrule)) patch.rrule = rule;
    }
    if (input.priority !== undefined && input.priority !== before.priority) {
      patch.priority = input.priority;
    }
    this.#notes.suspend(id);
    try {
      if (Object.keys(patch).length > 0) {
        this.#replace(await this.#api.taskUpdate(id, patch));
      }
      if (input.tagIds !== undefined && !sameSet(input.tagIds, before.tagIds)) {
        this.#replace(await this.#api.taskTagsSet(id, input.tagIds));
      }
      if (input.reminders && !before.isCompleted) {
        this.#replace(await this.#api.remindersSet(id, input.reminders));
      }
      const saved = this.find(id);
      if (saved) this.#warnIfHidden(saved);
      return true;
    } catch (error) {
      this.#notices.report(error);
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
      this.#notices.report(error);
    }
  }

  /** Optimistic: the task moves to its new day at once; a refusal puts it back. Clearing the date clears the time too (a task without date has no schedule). */
  async setDueDate(id: string, date: string | null): Promise<void> {
    const before = this.find(id);
    if (!before || before.dueDate === date) return;
    this.#notes.suspend(id);
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
      this.#notices.report(error);
    }
  }

  /**
   * Puts a task on a time slot (F24-01): day, start and length in one update, so the backend
   * recalculates its reminders once. A task without length takes `minutes`. Optimistic; a
   * refusal puts it back.
   */
  async schedule(
    id: string,
    date: string,
    time: string,
    minutes: number,
  ): Promise<void> {
    const before = this.find(id);
    if (!before || before.isCompleted) return;
    const durationMinutes = before.durationMinutes ?? minutes;
    this.#notes.suspend(id);
    this.#replace({ ...before, dueDate: date, dueTime: time, durationMinutes });
    try {
      this.#replace(
        await this.#api.taskUpdate(id, {
          dueDate: date,
          dueTime: time,
          durationMinutes,
        }),
      );
    } catch (error) {
      this.#replace(before);
      this.#notices.report(error);
    }
  }

  /** Optimistic: a scheduled task takes this length at once; a refusal puts it back (F24). */
  async setDuration(id: string, minutes: number): Promise<void> {
    const before = this.find(id);
    if (!before || before.dueTime === null) return;
    if (before.durationMinutes === minutes) return;
    this.#replace({ ...before, durationMinutes: minutes });
    try {
      this.#replace(
        await this.#api.taskUpdate(id, { durationMinutes: minutes }),
      );
    } catch (error) {
      this.#replace(before);
      this.#notices.report(error);
    }
  }

  /** Optimistic: the ring and the order change at once; a refusal puts them back (F20-02). */
  async setPriority(id: string, priority: number): Promise<void> {
    const before = this.find(id);
    if (!before || before.priority === priority) return;
    this.#replace({ ...before, priority });
    try {
      this.#replace(await this.#api.taskUpdate(id, { priority }));
    } catch (error) {
      this.#replace(before);
      this.#notices.report(error);
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
      this.#notices.report(error);
    }
  }

  /** Optimistic: the task takes exactly these tags at once; a refusal puts it back. */
  async setTags(id: string, tagIds: string[]): Promise<void> {
    const before = this.find(id);
    if (!before || sameSet(tagIds, before.tagIds)) return;
    this.#replace({ ...before, tagIds });
    try {
      this.#replace(await this.#api.taskTagsSet(id, tagIds));
    } catch (error) {
      this.#replace(before);
      this.#notices.report(error);
    }
  }

  /** Adds one tag to the ones the task has (dropping it on a sidebar tag). */
  async addTag(id: string, tagId: string): Promise<void> {
    const before = this.find(id);
    if (!before || before.tagIds.includes(tagId)) return;
    await this.setTags(id, [...before.tagIds, tagId]);
  }

  setCompleted(id: string, completed: boolean): Promise<void> {
    return this.#completion.set(id, completed);
  }

  // ---- reminders ------------------------------------------------------------------------

  toggleReminders(id: string): void {
    this.#reminders.toggle(id);
  }

  setReminders(id: string, reminders: ReminderInput[]): Promise<void> {
    return this.#reminders.set(id, reminders);
  }

  onReminderFired(taskId: string): Promise<void> {
    return this.#reminders.onFired(taskId);
  }

  // ---- notes and search -----------------------------------------------------------------

  isNotesOpen(id: string): boolean {
    return this.#notes.isOpen(id);
  }

  toggleNotes(id: string): void {
    this.#notes.toggle(id);
  }

  closeEmptyNotes(id: string): void {
    this.#notes.closeEmpty(id);
  }

  saveNotes(id: string, contentJson: string): Promise<void> {
    return this.#notes.save(id, contentJson);
  }

  search(query: string): Promise<SearchHit[]> {
    return this.#api.search(query);
  }

  /** Selects a search result and opens its notes when it has any. */
  reveal(id: string): boolean {
    if (!this.find(id)) return false;
    this.selectedId = id;
    this.#notes.expand(id);
    return true;
  }

  // ---- deletion -------------------------------------------------------------------------

  remove(id: string): Promise<void> {
    return this.#undo.remove(id);
  }

  restoreById(id: string): Promise<boolean> {
    return this.#undo.restoreById(id);
  }

  undoDelete(): Promise<void> {
    return this.#undo.undo();
  }

  dismissNotice(): void {
    this.#notices.dismiss();
  }

  /** Cancels timers; call when the store is discarded. */
  dispose(): void {
    this.#undo.dispose();
    this.batch.dispose();
    this.#notices.dismiss();
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

  /** A task the filter hides would seem lost right after saving it. */
  #warnIfHidden(task: Task): void {
    if (!this.filter(task)) this.#notices.inform(t.tags.hiddenByFilter);
  }
}

export const tasksStore = new TasksStore();
