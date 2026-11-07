// Test helpers shared by the store and component tests. Not imported by the app.
import { groupByDay, sortTodo } from "./ordering";
import type { TasksApi } from "./stores/tasks.svelte";
import type {
  ReminderInput,
  SearchHit,
  Task,
  TaskPatch,
  TasksSections,
} from "./types";

export function makeTask(overrides: Partial<Task> & { id: string }): Task {
  return {
    title: overrides.id,
    contentJson: "",
    dueDate: null,
    dueTime: null,
    isCompleted: false,
    completedAt: null,
    position: 0,
    tagIds: [],
    reminders: [],
    createdAt: "2026-03-01T08:00:00Z",
    updatedAt: "2026-03-01T08:00:00Z",
    ...overrides,
  };
}

type Method = keyof TasksApi;

/** In-memory stand-in for the backend: same shapes, failures and delays on demand. */
export class FakeApi implements TasksApi {
  tasks: Task[];
  calls: string[] = [];
  /** Methods that reject the next time they are called. */
  failing = new Set<Method>();
  /** When set, every call waits for this promise before answering. */
  gate: Promise<void> | null = null;
  #created = 0;
  #deleted = new Set<string>();

  constructor(tasks: Task[] = []) {
    this.tasks = tasks;
  }

  async #enter(method: Method, detail = ""): Promise<void> {
    this.calls.push(detail ? `${method}:${detail}` : method);
    if (this.gate) await this.gate;
    if (this.failing.delete(method)) throw "DB: fallo simulado";
  }

  #get(id: string): Task {
    const task = this.tasks.find((candidate) => candidate.id === id);
    if (!task) throw `NOT_FOUND: tarea ${id}`;
    return task;
  }

  #put(task: Task): Task {
    this.tasks = this.tasks.map((c) => (c.id === task.id ? task : c));
    return task;
  }

  async tasksList(query: { view: "todo" | "agenda" }): Promise<TasksSections> {
    await this.#enter("tasksList", query.view);
    const live = this.tasks.filter((task) => !this.#deleted.has(task.id));
    return query.view === "todo"
      ? {
          todo: sortTodo(live.filter((task) => task.dueDate === null)),
          agenda: [],
        }
      : { todo: [], agenda: groupByDay(live) };
  }

  async taskCreate(input: { title?: string; dueDate?: string }): Promise<Task> {
    await this.#enter("taskCreate", input.title ?? "");
    const task = makeTask({
      id: `new-${++this.#created}`,
      title: input.title ?? "",
      dueDate: input.dueDate ?? null,
      position: 1000 + this.#created,
    });
    this.tasks = [...this.tasks, task];
    return task;
  }

  async taskUpdate(id: string, patch: TaskPatch): Promise<Task> {
    await this.#enter(
      "taskUpdate",
      patch.contentJson === undefined
        ? `${id}=${patch.title}`
        : `${id}#notes=${patch.contentJson}`,
    );
    const before = this.#get(id);
    return this.#put({
      ...before,
      title: patch.title ?? before.title,
      contentJson: patch.contentJson ?? before.contentJson,
      dueDate: patch.dueDate === undefined ? before.dueDate : patch.dueDate,
      dueTime: patch.dueTime === undefined ? before.dueTime : patch.dueTime,
    });
  }

  async search(query: string): Promise<SearchHit[]> {
    await this.#enter("search", query);
    const fold = (text: string) =>
      text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
    const needle = fold(query.trim());
    if (!needle) return [];
    return this.tasks
      .filter(
        (task) =>
          !this.#deleted.has(task.id) &&
          fold(`${task.title} ${task.contentJson}`).includes(needle),
      )
      .map((task) => ({
        taskId: task.id,
        title: task.title,
        snippet: "",
        dueDate: task.dueDate,
        isCompleted: task.isCompleted,
      }));
  }

  async taskSetCompleted(
    id: string,
    completed: boolean,
    completedDate?: string,
  ): Promise<Task> {
    await this.#enter("taskSetCompleted", `${id}=${completed}`);
    const before = this.#get(id);
    return this.#put({
      ...before,
      dueDate: completed ? (completedDate ?? before.dueDate) : before.dueDate,
      isCompleted: completed,
      completedAt: completed ? "2026-03-10T10:00:00Z" : null,
    });
  }

  async remindersSet(
    taskId: string,
    reminders: ReminderInput[],
  ): Promise<Task> {
    await this.#enter("remindersSet", `${taskId}=${JSON.stringify(reminders)}`);
    const before = this.#get(taskId);
    return this.#put({
      ...before,
      reminders: reminders.map((reminder, index) => ({
        id: `rem-${index}`,
        notifyAt: reminder.notifyAt ?? "2026-03-11T08:00:00Z",
        offsetMinutes: reminder.offsetMinutes ?? null,
        isSent: false,
      })),
    });
  }

  async taskDelete(id: string): Promise<void> {
    await this.#enter("taskDelete", id);
    this.#get(id);
    this.#deleted.add(id);
  }

  async taskRestore(id: string): Promise<Task> {
    await this.#enter("taskRestore", id);
    this.#deleted.delete(id);
    return this.#get(id);
  }
}

/** jsdom has no layout; ProseMirror asks for rectangles when it scrolls the selection into view. */
export function installLayoutStubs(): void {
  const rects = {
    getClientRects: () => [],
    getBoundingClientRect: () => new DOMRect(),
  };
  Object.assign(Range.prototype, rects);
  Object.assign(Text.prototype, rects);
  document.elementFromPoint = () => null;
}
