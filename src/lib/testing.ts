// Test helpers shared by the store and component tests. Not imported by the app.
import { groupByDay, sortTodo } from "./ordering";
import type { TasksApi } from "./stores/tasks.svelte";
import type { Task, TasksSections } from "./types";

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

  async taskUpdate(id: string, patch: { title: string }): Promise<Task> {
    await this.#enter("taskUpdate", `${id}=${patch.title}`);
    return this.#put({ ...this.#get(id), title: patch.title });
  }

  async taskSetCompleted(id: string, completed: boolean): Promise<Task> {
    await this.#enter("taskSetCompleted", `${id}=${completed}`);
    return this.#put({
      ...this.#get(id),
      isCompleted: completed,
      completedAt: completed ? "2026-03-10T10:00:00Z" : null,
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
