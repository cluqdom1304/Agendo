import { describe, expect, it } from "vitest";

import { EventsStore } from "../../stores/events.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, FakeEventsApi, makeTask } from "../../testing";
import { docToText, eventToTask, taskToEvent } from "./convert";

const NOW = new Date("2026-03-10T10:00:00");

async function setup(tasks = [makeTask({ id: "t1", title: "Reunión" })]) {
  const tasksApi = new FakeApi(tasks);
  const tasksStore = new TasksStore({
    api: tasksApi,
    now: () => NOW,
    settleMs: () => 0,
  });
  await tasksStore.load();
  const eventsApi = new FakeEventsApi();
  const events = new EventsStore({ api: eventsApi });
  return { tasksApi, tasksStore, eventsApi, events };
}

describe("docToText", () => {
  it("joins the text blocks of a document, one per line", () => {
    const doc = JSON.stringify({
      type: "doc",
      content: [
        { type: "paragraph", content: [{ type: "text", text: "Primera" }] },
        {
          type: "bulletList",
          content: [
            {
              type: "listItem",
              content: [
                { type: "paragraph", content: [{ type: "text", text: "Dos" }] },
              ],
            },
          ],
        },
      ],
    });
    expect(docToText(doc)).toBe("Primera\nDos");
  });

  it("keeps plain text and survives garbage", () => {
    expect(docToText("  hola ")).toBe("hola");
    expect(docToText("{roto")).toBe("");
    expect(docToText("")).toBe("");
  });
});

describe("taskToEvent (F21-04)", () => {
  it("creates the event from the task and then trashes the task", async () => {
    const { tasksApi, tasksStore, eventsApi, events } = await setup([
      makeTask({
        id: "t1",
        title: "Reunión",
        dueDate: "2026-03-12",
        dueTime: "16:00",
        durationMinutes: 90,
        tagIds: ["tag1"],
        rrule: "FREQ=WEEKLY;INTERVAL=1",
      }),
    ]);
    const created = await taskToEvent(
      tasksStore.find("t1")!,
      events,
      tasksStore,
      "2026-03-10",
    );
    expect(created).toMatchObject({
      title: "Reunión",
      startDate: "2026-03-12",
      startTime: "16:00",
      endTime: "17:30",
      rrule: "FREQ=WEEKLY;INTERVAL=1",
      tagIds: ["tag1"],
    });
    expect(eventsApi.events).toHaveLength(1);
    expect(tasksApi.calls.some((call) => call.startsWith("taskDelete"))).toBe(
      true,
    );
    expect(events.notice?.message).toBe("Tarea convertida en evento");
  });

  it("an undated task becomes an all-day event today", async () => {
    const { tasksStore, events } = await setup();
    const created = await taskToEvent(
      tasksStore.find("t1")!,
      events,
      tasksStore,
      "2026-03-10",
    );
    expect(created).toMatchObject({
      startDate: "2026-03-10",
      startTime: null,
      allDay: true,
    });
  });

  it("keeps the task when the event cannot be created", async () => {
    const { tasksApi, tasksStore, eventsApi, events } = await setup();
    eventsApi.failWith = "VALIDATION: no";
    const created = await taskToEvent(
      tasksStore.find("t1")!,
      events,
      tasksStore,
      "2026-03-10",
    );
    expect(created).toBeNull();
    expect(tasksApi.calls.some((call) => call.startsWith("taskDelete"))).toBe(
      false,
    );
  });
});

describe("eventToTask (F21-04)", () => {
  it("creates a task with the date, time and length of the event, then trashes it", async () => {
    const { tasksApi, tasksStore, eventsApi, events } = await setup([]);
    const event = eventsApi.makeEvent({
      title: "Dentista",
      startDate: "2026-03-12",
      startTime: "09:00",
      endDate: "2026-03-12",
      endTime: "09:45",
      notes: "Llevar la tarjeta",
      reminderOffsets: [15],
    });
    const task = await eventToTask(event, events, tasksStore);
    expect(task).toMatchObject({
      title: "Dentista",
      dueDate: "2026-03-12",
      dueTime: "09:00",
      durationMinutes: 45,
    });
    expect(tasksApi.calls.some((call) => call.startsWith("taskCreate"))).toBe(
      true,
    );
    expect(eventsApi.events).toHaveLength(0);
    expect(events.deleted).toBeNull();
    expect(events.notice?.message).toBe("Evento convertido en tarea");
  });
});
