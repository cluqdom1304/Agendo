// Test helpers shared by the store and component tests. Not imported by the app.
import { groupByDay, sortTodo } from "./ordering";
import type { EventsApi } from "./stores/events.svelte";
import type { TasksApi } from "./stores/tasks.svelte";
import type {
  CalendarEvent,
  EventInput,
  EventOccurrence,
  EventPatch,
  FoundItem,
  Note,
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
    durationMinutes: null,
    sourceNoteId: null,
    rrule: null,
    priority: 0,
    checklistDone: 0,
    checklistTotal: 0,
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

const STAMP = "2026-03-10T10:00:00Z";

export function makeNote(id: string, overrides: Partial<Note> = {}): Note {
  return {
    id,
    folderId: null,
    title: id,
    contentJson: '{"type":"doc","content":[]}',
    isPinned: false,
    tagIds: [],
    createdAt: STAMP,
    updatedAt: STAMP,
    ...overrides,
  };
}

type Hit<K extends FoundItem["kind"]> = Extract<FoundItem, { kind: K }>;

/** A search result for a note, task or tag; the rest of its fields take neutral values. */
export function makeNoteHit(
  noteId: string,
  title: string,
  overrides: Partial<Hit<"note">> = {},
): FoundItem {
  return {
    kind: "note",
    noteId,
    title,
    snippet: "",
    folderId: null,
    isPinned: false,
    ...overrides,
  };
}

export function makeTaskHit(
  taskId: string,
  title: string,
  overrides: Partial<Hit<"task">> = {},
): FoundItem {
  return {
    kind: "task",
    taskId,
    title,
    snippet: "",
    dueDate: null,
    isCompleted: false,
    ...overrides,
  };
}

export function makeTagHit(
  tagId: string,
  name: string,
  overrides: Partial<Hit<"tag">> = {},
): FoundItem {
  return {
    kind: "tag",
    tagId,
    name,
    colorHex: "#4C8DFF",
    noteCount: 1,
    pendingCount: 1,
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

  async tasksListAll(): Promise<TasksSections> {
    await this.#enter("tasksListAll");
    const live = this.tasks.filter((task) => !this.#deleted.has(task.id));
    return {
      todo: sortTodo(live.filter((task) => task.dueDate === null)),
      agenda: groupByDay(live),
    };
  }

  async taskCreate(input: {
    title?: string;
    dueDate?: string;
    dueTime?: string;
    durationMinutes?: number;
    tagIds?: string[];
    contentJson?: string;
    sourceNoteId?: string;
    rrule?: string;
    priority?: number;
  }): Promise<Task> {
    await this.#enter(
      "taskCreate",
      input.dueTime
        ? `${input.title ?? ""}@${input.dueTime}`
        : (input.title ?? ""),
    );
    const task = makeTask({
      id: `new-${++this.#created}`,
      title: input.title ?? "",
      dueDate: input.dueDate ?? null,
      dueTime: input.dueTime ?? null,
      durationMinutes: input.durationMinutes ?? null,
      tagIds: input.tagIds ?? [],
      contentJson: input.contentJson ?? "",
      sourceNoteId: input.sourceNoteId ?? null,
      rrule: input.rrule ?? null,
      priority: input.priority ?? 0,
      position: 1000 + this.#created,
    });
    this.tasks = [...this.tasks, task];
    return task;
  }

  async taskTagsSet(taskId: string, tagIds: string[]): Promise<Task> {
    await this.#enter("taskTagsSet", `${taskId}=${tagIds.join(",")}`);
    return this.#put({ ...this.#get(taskId), tagIds });
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
      durationMinutes:
        patch.durationMinutes === undefined
          ? before.durationMinutes
          : patch.durationMinutes,
      priority: patch.priority ?? before.priority,
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
        repeat: reminder.repeat ? { occurrence: 1, ...reminder.repeat } : null,
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

/** In-memory events API for tests: no repetitions, every event shows on the days it covers. */
export class FakeEventsApi implements EventsApi {
  events: CalendarEvent[] = [];
  trashed: CalendarEvent[] = [];
  failWith: string | null = null;
  #ids = 0;

  makeEvent(overrides: Partial<CalendarEvent> = {}): CalendarEvent {
    const event: CalendarEvent = {
      id: `e${++this.#ids}`,
      title: "Evento",
      notes: "",
      startDate: "2026-03-10",
      startTime: "10:00",
      endDate: "2026-03-10",
      endTime: "11:00",
      allDay: false,
      rrule: null,
      tagIds: [],
      reminderOffsets: [],
      createdAt: "2026-03-01T00:00:00Z",
      updatedAt: "2026-03-01T00:00:00Z",
      ...overrides,
    };
    this.events.push(event);
    return event;
  }

  #check(): void {
    if (this.failWith) throw this.failWith;
  }

  async eventsList(from: string, to: string): Promise<EventOccurrence[]> {
    this.#check();
    return this.events
      .filter((event) => event.startDate <= to && event.endDate >= from)
      .map((event) => ({
        eventId: event.id,
        title: event.title,
        startDate: event.startDate,
        startTime: event.startTime,
        endDate: event.endDate,
        endTime: event.endTime,
        allDay: event.allDay,
        recurring: event.rrule !== null,
        tagIds: event.tagIds,
      }));
  }

  async eventGet(id: string): Promise<CalendarEvent> {
    this.#check();
    const event = this.events.find((item) => item.id === id);
    if (!event) throw "NOT_FOUND: evento no encontrado";
    return event;
  }

  async eventCreate(input: EventInput): Promise<CalendarEvent> {
    this.#check();
    return this.makeEvent({
      title: input.title ?? "",
      notes: input.notes ?? "",
      startDate: input.startDate,
      startTime: input.startTime ?? null,
      endDate: input.endDate ?? input.startDate,
      endTime: input.startTime ? (input.endTime ?? null) : null,
      allDay: !input.startTime,
      rrule: input.rrule ?? null,
      tagIds: input.tagIds ?? [],
      reminderOffsets: input.reminderOffsets ?? [],
    });
  }

  async eventUpdate(id: string, patch: EventPatch): Promise<CalendarEvent> {
    this.#check();
    const event = await this.eventGet(id);
    const { startTime, rrule, ...rest } = patch;
    Object.assign(event, rest);
    if (startTime !== undefined) {
      event.startTime = startTime;
      event.allDay = startTime === null;
      if (startTime === null) event.endTime = null;
    }
    if (rrule !== undefined) event.rrule = rrule;
    return event;
  }

  async eventDelete(id: string): Promise<void> {
    this.#check();
    const event = await this.eventGet(id);
    this.events = this.events.filter((item) => item.id !== id);
    this.trashed.push(event);
  }

  async eventRestore(id: string): Promise<CalendarEvent> {
    this.#check();
    const event = this.trashed.find((item) => item.id === id);
    if (!event) throw "NOT_FOUND: evento no encontrado";
    this.trashed = this.trashed.filter((item) => item.id !== id);
    this.events.push(event);
    return event;
  }
}
