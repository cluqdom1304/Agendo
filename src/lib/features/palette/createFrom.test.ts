import { describe, expect, it, vi } from "vitest";

import { EventsStore } from "../../stores/events.svelte";
import { HabitsStore } from "../../stores/habits.svelte";
import type { NotesStore } from "../../stores/notes.svelte";
import type { TagsStore } from "../../stores/tags.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import type { ViewStore } from "../../stores/view.svelte";
import { FakeApi, FakeEventsApi, FakeHabitsApi } from "../../testing";
import { createFromText, type CreateDeps } from "./createFrom";

// Tuesday 10 March 2026.
const NOW = new Date(2026, 2, 10, 10, 0);

function setup() {
  const tasksApi = new FakeApi([]);
  const eventsApi = new FakeEventsApi();
  const habitsApi = new FakeHabitsApi();
  const sections: string[] = [];
  const note = { id: "n1" };
  const notes = {
    ensureLoaded: vi.fn(async () => undefined),
    createNote: vi.fn(async () => note),
    saveTitle: vi.fn(async () => undefined),
  };
  const deps = {
    view: {
      setSection: (section: string) => sections.push(section),
      showAgenda: () => sections.push("agenda"),
    } as unknown as ViewStore,
    tasks: new TasksStore({ api: tasksApi, now: () => NOW, settleMs: () => 0 }),
    events: new EventsStore({ api: eventsApi }),
    habits: new HabitsStore({ api: habitsApi }),
    notes: notes as unknown as NotesStore,
    tags: { tags: [{ id: "g1", name: "Salud" }] } as unknown as TagsStore,
    now: () => NOW,
  } satisfies CreateDeps;
  return { deps, tasksApi, eventsApi, habitsApi, notes, sections };
}

describe("createFromText (F23-03)", () => {
  it("creates a task with what the text says", async () => {
    const { deps, tasksApi, sections } = setup();
    await createFromText(deps, "task", "Dentista mañana a las 5 #salud !2");
    expect(tasksApi.tasks.at(-1)).toMatchObject({
      title: "Dentista",
      dueDate: "2026-03-11",
      dueTime: "17:00",
      tagIds: ["g1"],
      priority: 2,
    });
    expect(sections).toEqual(["agenda"]);
  });

  it("creates an event with an end, one hour by default", async () => {
    const { deps, eventsApi, sections } = setup();
    await createFromText(deps, "event", "Cena el viernes a las 21:00");
    expect(eventsApi.events.at(-1)).toMatchObject({
      title: "Cena",
      startDate: "2026-03-13",
      startTime: "21:00",
      endTime: "22:00",
    });
    expect(sections).toEqual(["events"]);
  });

  it("creates an all-day event today when nothing is said", async () => {
    const { deps, eventsApi } = setup();
    await createFromText(deps, "event", "Cumpleaños de Ana");
    expect(eventsApi.events.at(-1)).toMatchObject({
      title: "Cumpleaños de Ana",
      startDate: "2026-03-10",
      allDay: true,
    });
  });

  it("creates a habit on the chosen weekdays and time", async () => {
    const { deps, habitsApi, sections } = setup();
    await createFromText(
      deps,
      "habit",
      "Correr los lunes y jueves a las 7 de la mañana",
    );
    expect(habitsApi.habits.at(-1)).toMatchObject({
      title: "Correr",
      days: 0b0001001,
      time: "07:00",
    });
    expect(sections).toEqual(["habits"]);
  });

  it("creates a note titled with the text as typed", async () => {
    const { deps, notes, sections } = setup();
    await createFromText(deps, "note", "Ideas para mañana #salud");
    expect(notes.createNote).toHaveBeenCalled();
    expect(notes.saveTitle).toHaveBeenCalledWith(
      "n1",
      "Ideas para mañana #salud",
    );
    expect(sections).toEqual(["notes"]);
  });

  it("ignores blank text", async () => {
    const { deps, tasksApi } = setup();
    await createFromText(deps, "task", "   ");
    expect(tasksApi.calls).toEqual([]);
  });
});
