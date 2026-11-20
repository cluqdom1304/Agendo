// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { EventsStore } from "../../stores/events.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { ViewStore } from "../../stores/view.svelte";
import { FakeApi, FakeEventsApi, makeTask } from "../../testing";
import UpcomingView from "./UpcomingView.svelte";

const NOW = new Date("2026-03-10T10:00:00");

async function mount(
  tasks = [makeTask({ id: "a", dueDate: "2026-03-10" })],
  seed: (api: FakeEventsApi) => void = () => undefined,
) {
  const api = new FakeApi(tasks);
  const store = new TasksStore({ api, now: () => NOW, settleMs: () => 0 });
  await store.load();
  const view = new ViewStore({
    api: { settingsGet: async () => null, settingsSet: async () => undefined },
    now: () => NOW,
  });
  const eventsApi = new FakeEventsApi();
  seed(eventsApi);
  const opened: string[] = [];
  render(UpcomingView, {
    store,
    eventStore: new EventsStore({ api: eventsApi }),
    view,
    onOpenTask: (id: string) => opened.push(id),
    onOpenEvent: (id: string) => opened.push(`event:${id}`),
  });
  return { api, opened };
}

describe("UpcomingView (F20-05)", () => {
  it("shows overdue, then each day by name, and opens a task", async () => {
    const { opened } = await mount([
      makeTask({ id: "a", title: "Atrasada", dueDate: "2026-03-08" }),
      makeTask({ id: "b", title: "Para hoy", dueDate: "2026-03-10" }),
      makeTask({ id: "c", title: "Mañana sin falta", dueDate: "2026-03-11" }),
      makeTask({ id: "d", title: "El jueves", dueDate: "2026-03-12" }),
    ]);
    const headings = screen
      .getAllByRole("heading", { level: 3 })
      .map((heading) => heading.firstChild?.textContent?.trim());
    expect(headings).toEqual([
      "Vencidas",
      "Hoy",
      "Mañana",
      "Jueves, 12 de marzo",
    ]);
    await fireEvent.click(screen.getByText("El jueves"));
    expect(opened).toEqual(["d"]);
  });

  it("completes a task from its circle", async () => {
    const { api } = await mount([
      makeTask({ id: "a", title: "Entregar", dueDate: "2026-03-11" }),
    ]);
    await fireEvent.click(
      screen.getByRole("checkbox", { name: "Marcar «Entregar» como hecha" }),
    );
    await waitFor(() =>
      expect(
        api.calls.some((call) => call.startsWith("taskSetCompleted")),
      ).toBe(true),
    );
  });

  it("says when there is nothing ahead", async () => {
    await mount([makeTask({ id: "a", title: "Sin fecha" })]);
    expect(screen.getByText("Nada por delante")).toBeTruthy();
  });
});

describe("UpcomingView with events (F21-03)", () => {
  it("shows events in their day, with a heading for a day that only has events", async () => {
    const { opened } = await mount(
      [makeTask({ id: "a", title: "Tarea", dueDate: "2026-03-11" })],
      (api) => {
        api.makeEvent({
          id: "e1",
          title: "Cena",
          startDate: "2026-03-12",
          endDate: "2026-03-12",
        });
        api.makeEvent({
          id: "e2",
          title: "Reunión",
          startDate: "2026-03-11",
          endDate: "2026-03-11",
        });
      },
    );
    await screen.findByText("Cena");
    const headings = screen
      .getAllByRole("heading", { level: 3 })
      .map((heading) => heading.firstChild?.textContent?.trim());
    expect(headings).toEqual(["Mañana", "Jueves, 12 de marzo"]);
    await fireEvent.click(screen.getByText("Cena"));
    expect(opened).toEqual(["event:e1"]);
  });

  it("is not empty when there are only events", async () => {
    await mount([], (api) =>
      api.makeEvent({
        title: "Cena",
        startDate: "2026-03-12",
        endDate: "2026-03-12",
      }),
    );
    await screen.findByText("Cena");
    expect(screen.queryByText("Nada por delante")).toBeNull();
  });
});
