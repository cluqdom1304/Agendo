// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { EventsStore } from "../../stores/events.svelte";
import { ViewStore } from "../../stores/view.svelte";
import { FakeEventsApi } from "../../testing";
import EventsView from "./EventsView.svelte";

const NOW = new Date("2026-03-10T10:00:00");

function mount(seed: (api: FakeEventsApi) => void = () => undefined) {
  const api = new FakeEventsApi();
  seed(api);
  const store = new EventsStore({ api });
  const view = new ViewStore({
    api: { settingsGet: async () => null, settingsSet: async () => undefined },
    now: () => NOW,
  });
  const onOpenEvent = vi.fn();
  const onNewEvent = vi.fn();
  render(EventsView, { store, view, onOpenEvent, onNewEvent });
  return { api, store, onOpenEvent, onNewEvent };
}

describe("EventsView (F21-01)", () => {
  it("asks for the next 90 days from today", async () => {
    const { store } = mount();
    await waitFor(() => expect(store.from).toBe("2026-03-10"));
    expect(store.to).toBe("2026-06-07");
  });

  it("shows an empty state and a button to create the first event", async () => {
    const { onNewEvent } = mount();
    expect(await screen.findByText("Sin eventos próximos")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Nuevo evento" }));
    expect(onNewEvent).toHaveBeenCalled();
  });

  it("lists the events day by day, in time order, and opens one", async () => {
    const { onOpenEvent } = mount((api) => {
      api.makeEvent({
        id: "b",
        title: "Comida",
        startTime: "14:00",
        endTime: "15:00",
      });
      api.makeEvent({
        id: "a",
        title: "Dentista",
        startTime: "09:00",
        endTime: "09:30",
      });
      api.makeEvent({
        id: "c",
        title: "Congreso",
        startDate: "2026-03-12",
        endDate: "2026-03-12",
        startTime: null,
        endTime: null,
        allDay: true,
      });
    });
    await screen.findByText("Dentista");
    const headings = screen
      .getAllByRole("heading", { level: 3 })
      .map((heading) => heading.firstChild?.textContent?.trim());
    expect(headings).toEqual(["Hoy", "Jueves, 12 de marzo"]);
    const titles = screen
      .getAllByTitle(/^Editar el evento/)
      .map((button) => button.querySelector(".title")?.textContent);
    expect(titles).toEqual(["Dentista", "Comida", "Congreso"]);
    expect(screen.getByText(/Todo el día/)).toBeTruthy();
    await fireEvent.click(screen.getByText("Comida"));
    expect(onOpenEvent).toHaveBeenCalledWith("b");
  });

  it("marks a repeating event", async () => {
    mount((api) =>
      api.makeEvent({ title: "Yoga", rrule: "FREQ=WEEKLY;INTERVAL=1" }),
    );
    expect(await screen.findByText("Se repite")).toBeTruthy();
  });
});
