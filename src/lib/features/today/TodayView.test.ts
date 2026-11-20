// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { EventsStore } from "../../stores/events.svelte";
import { HabitsStore } from "../../stores/habits.svelte";
import { NotesStore } from "../../stores/notes.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { ViewStore } from "../../stores/view.svelte";
import { FakeApi, FakeEventsApi, FakeHabitsApi, makeTask } from "../../testing";
import { FakeNotesApi, note } from "../notes/testing";
import TodayView from "./TodayView.svelte";

const NOW = new Date("2026-03-10T10:00:00");

async function mount(
  tasks = [makeTask({ id: "a", title: "Entregar", dueDate: "2026-03-10" })],
  pinned: string[] = [],
  seed: (api: FakeEventsApi) => void = () => undefined,
  seedHabits: (api: FakeHabitsApi) => void = () => undefined,
) {
  const api = new FakeApi(tasks);
  const store = new TasksStore({ api, now: () => NOW, settleMs: () => 0 });
  await store.load();
  const notesApi = new FakeNotesApi();
  notesApi.notes = pinned.map((title, index) =>
    note(`p${index}`, { title, isPinned: true }),
  );
  const notes = new NotesStore(notesApi);
  const view = new ViewStore({
    api: { settingsGet: async () => null, settingsSet: async () => undefined },
    now: () => NOW,
  });
  const eventsApi = new FakeEventsApi();
  seed(eventsApi);
  const habitsApi = new FakeHabitsApi();
  seedHabits(habitsApi);
  const habits = new HabitsStore({ api: habitsApi });
  const opened: string[] = [];
  render(TodayView, {
    store,
    eventStore: new EventsStore({ api: eventsApi }),
    habitStore: habits,
    notes,
    view,
    onOpenTask: (id: string) => opened.push(`task:${id}`),
    onOpenEvent: (id: string) => opened.push(`event:${id}`),
    onOpenNote: (id: string) => opened.push(`note:${id}`),
    onOpenDaily: (date: string) => opened.push(`daily:${date}`),
  });
  return { api, opened, habitsApi, habits };
}

describe("TodayView", () => {
  it("lists only the tasks of today, open ones first, and opens one", async () => {
    const { opened } = await mount([
      makeTask({
        id: "done",
        title: "Hecha",
        dueDate: "2026-03-10",
        isCompleted: true,
      }),
      makeTask({
        id: "late",
        title: "Tarde",
        dueDate: "2026-03-10",
        dueTime: "18:00",
      }),
      makeTask({
        id: "early",
        title: "Pronto",
        dueDate: "2026-03-10",
        dueTime: "08:00",
      }),
      makeTask({ id: "other", title: "Mañana", dueDate: "2026-03-11" }),
    ]);
    const titles = screen
      .getAllByRole("button")
      .map((button) => button.textContent ?? "");
    const shown = ["Pronto", "Tarde", "Hecha"].map((title) =>
      titles.findIndex((text) => text.includes(title)),
    );
    expect(shown.every((index) => index >= 0)).toBe(true);
    expect(shown).toEqual([...shown].sort((a, b) => a - b));
    expect(screen.queryByText("Mañana")).toBeNull();
    expect(screen.getByText("2 pendientes")).toBeTruthy();
    await fireEvent.click(screen.getByText("Pronto"));
    expect(opened).toEqual(["task:early"]);
  });

  it("completes a task from its checkbox", async () => {
    const { api } = await mount();
    await fireEvent.click(
      screen.getByRole("checkbox", { name: "Marcar «Entregar» como hecha" }),
    );
    await waitFor(() =>
      expect(
        api.calls.some((call) => call.startsWith("taskSetCompleted")),
      ).toBe(true),
    );
  });

  it("says when the day is empty and opens the daily note", async () => {
    const { opened } = await mount([]);
    expect(screen.getByText("Nada para hoy")).toBeTruthy();
    await fireEvent.click(
      screen.getByRole("button", { name: "Abrir la nota de hoy" }),
    );
    expect(opened).toEqual(["daily:2026-03-10"]);
  });

  it("lists the pinned notes, or says there are none", async () => {
    const { opened } = await mount([], ["Plan semanal"]);
    await waitFor(() => expect(screen.getByText("Plan semanal")).toBeTruthy());
    await fireEvent.click(screen.getByText("Plan semanal"));
    expect(opened).toEqual(["note:p0"]);
  });

  it("shows an empty message without pinned notes", async () => {
    await mount([], []);
    await waitFor(() =>
      expect(screen.getByText("No tienes notas fijadas.")).toBeTruthy(),
    );
  });

  it("summarizes the day with its progress and the overdue tasks", async () => {
    await mount([
      makeTask({ id: "a", dueDate: "2026-03-10", isCompleted: true }),
      makeTask({ id: "b", dueDate: "2026-03-10" }),
      makeTask({ id: "c", dueDate: "2026-03-08" }),
    ]);
    const banner = screen.getByRole("region", { name: "Resumen del día" });
    expect(banner.textContent).toContain("Te queda 1 tarea para hoy.");
    expect(banner.textContent).toContain("1 tarea vencida de días anteriores.");
    const bar = screen.getByRole("progressbar", { name: "1 de 2 hechas" });
    expect(bar.getAttribute("aria-valuenow")).toBe("1");
  });

  it("shows another day picked in the strip and moves with the arrows", async () => {
    const { opened } = await mount([
      makeTask({ id: "a", title: "Hoy mismo", dueDate: "2026-03-10" }),
      makeTask({ id: "b", title: "Pasado", dueDate: "2026-03-11" }),
    ]);
    const strip = screen.getByRole("group", { name: "Próximos 14 días" });
    const days = strip.querySelectorAll("button");
    expect(days).toHaveLength(14);
    expect(days[0].getAttribute("aria-pressed")).toBe("true");
    expect(days[1].getAttribute("aria-label")).toBe(
      "miércoles, 11 de marzo, 1 pendiente",
    );

    await fireEvent.click(days[1]);
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe(
      "Mañana",
    );
    expect(screen.getByText("Pasado")).toBeTruthy();
    expect(screen.queryByText("Hoy mismo")).toBeNull();
    await fireEvent.click(
      screen.getByRole("button", { name: "Abrir la nota del día" }),
    );
    expect(opened).toEqual(["daily:2026-03-11"]);

    await fireEvent.keyDown(days[1], { key: "ArrowRight" });
    expect(days[2].getAttribute("aria-pressed")).toBe("true");
    expect(document.activeElement).toBe(days[2]);
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe(
      "Jueves",
    );
    expect(screen.getByText("Nada para este día")).toBeTruthy();
    await fireEvent.keyDown(days[2], { key: "Home" });
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Hoy");
  });
});

describe("TodayView with events (F21-03)", () => {
  it("lists the events of the day before its tasks, counts them in the banner and opens one", async () => {
    const { opened } = await mount(undefined, [], (api) => {
      api.makeEvent({
        id: "e1",
        title: "Dentista",
        startTime: "09:00",
        endTime: "09:30",
      });
      api.makeEvent({
        id: "e2",
        title: "Mañana libre",
        startDate: "2026-03-11",
        endDate: "2026-03-11",
      });
    });
    expect(await screen.findByText("Dentista")).toBeTruthy();
    expect(screen.getByText("1 evento")).toBeTruthy();
    expect(screen.queryByText("Mañana libre")).toBeNull();
    const headings = screen
      .getAllByRole("heading", { level: 3 })
      .map((heading) => heading.firstChild?.textContent?.trim());
    expect(headings.indexOf("Eventos del día")).toBeLessThan(
      headings.indexOf("Tareas de hoy"),
    );
    await fireEvent.click(screen.getByText("Dentista"));
    expect(opened).toEqual(["event:e1"]);
  });

  it("marks the days of the strip that have events", async () => {
    await mount([], [], (api) =>
      api.makeEvent({ startDate: "2026-03-12", endDate: "2026-03-12" }),
    );
    await waitFor(() =>
      expect(
        document
          .querySelector('[data-date="2026-03-12"] .mark')
          ?.classList.contains("shown"),
      ).toBe(true),
    );
    expect(
      document
        .querySelector('[data-date="2026-03-11"] .mark')
        ?.classList.contains("shown"),
    ).toBe(false);
  });
});

describe("TodayView with habits (F22-02)", () => {
  const seed = (api: FakeHabitsApi) => {
    api.makeHabit({ id: "h1", title: "Leer", currentStreak: 2 });
    // Hoy es martes: este hábito solo toca los lunes.
    api.makeHabit({ id: "h2", title: "Gimnasio", days: 0b1 });
    api.makeHabit({ id: "h3", title: "Viejo", isArchived: true });
  };

  it("lists the habits scheduled today with the counter in the banner", async () => {
    await mount(undefined, [], undefined, seed);
    expect(await screen.findByText("Leer")).toBeTruthy();
    expect(screen.queryByText("Gimnasio")).toBeNull();
    expect(screen.queryByText("Viejo")).toBeNull();
    expect(screen.getAllByText("0 de 1 hábitos").length).toBeGreaterThan(0);
    expect(screen.getByText("Racha de 2 días")).toBeTruthy();
  });

  it("ticks a habit in one click and counts it", async () => {
    const { habitsApi, habits } = await mount(undefined, [], undefined, seed);
    await fireEvent.click(
      await screen.findByRole("checkbox", {
        name: "Marcar «Leer» como hecho hoy",
      }),
    );
    await waitFor(() =>
      expect(habitsApi.logs).toEqual([{ habitId: "h1", date: "2026-03-10" }]),
    );
    expect(habits.marked?.habit.title).toBe("Leer");
    expect(screen.getAllByText("1 de 1 hábitos").length).toBeGreaterThan(0);
    await fireEvent.click(
      screen.getByRole("checkbox", {
        name: "Quitar la marca de «Leer» de hoy",
      }),
    );
    await waitFor(() => expect(habitsApi.logs).toEqual([]));
  });

  it("shows no habits for another day", async () => {
    await mount(undefined, [], undefined, seed);
    await screen.findByText("Leer");
    await fireEvent.click(
      document.querySelector('[data-date="2026-03-11"]') as Element,
    );
    expect(screen.queryByText("Leer")).toBeNull();
  });
});
