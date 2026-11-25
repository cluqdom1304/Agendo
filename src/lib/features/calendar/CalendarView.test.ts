// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { EventsStore } from "../../stores/events.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { ViewStore, type ViewApi } from "../../stores/view.svelte";
import {
  FakeApi,
  FakeEventsApi,
  installLayoutStubs,
  makeTask,
} from "../../testing";
import type { Task, ViewMode } from "../../types";
import CalendarView from "./CalendarView.svelte";
import { FocusStore } from "../../stores/focus.svelte";
import type { TimeEntry } from "../../types";
import { FocusHistory } from "./focusHistory.svelte";

installLayoutStubs();

const HOUR_PX = 48;
/** Pixel offset inside a day column of a given time. */
const at = (hours: number, minutes = 0) => (hours + minutes / 60) * HOUR_PX;

class FakeViewApi implements ViewApi {
  saved: string | null = null;
  async settingsGet() {
    return this.saved;
  }
  async settingsSet(_key: "view_mode", value: string) {
    this.saved = value;
  }
}

async function mount(
  mode: ViewMode,
  tasks: Task[] = [],
  seed: (api: FakeEventsApi) => void = () => undefined,
  sessions: TimeEntry[] = [],
) {
  const api = new FakeApi(tasks);
  const store = new TasksStore({
    api,
    now: () => new Date("2026-03-10T10:00:00"),
  });
  await store.load();
  const view = new ViewStore({
    api: new FakeViewApi(),
    now: () => new Date("2026-03-10T10:00:00"),
    firstDay: () => 1,
  });
  view.mode = mode;
  const onOpenTask = vi.fn();
  const eventsApi = new FakeEventsApi();
  seed(eventsApi);
  const events = new EventsStore({ api: eventsApi });
  const onOpenEvent = vi.fn();
  const onNewEvent = vi.fn();
  const focus = new FocusStore({ tickMs: 0 });
  const focusList = vi.fn(async () => sessions);
  const history = new FocusHistory({ api: { focusList } });
  render(CalendarView, {
    focus,
    history,
    store,
    eventStore: events,
    view,
    onOpenTask,
    onOpenEvent,
    onNewEvent,
  });
  return {
    api,
    store,
    view,
    onOpenTask,
    events,
    eventsApi,
    onOpenEvent,
    onNewEvent,
    focus,
    focusList,
  };
}

/** Column of a day: the grid has no layout in jsdom, so its top is pinned to 0. */
function column(date: string): HTMLElement {
  const element = document.querySelector<HTMLElement>(
    `.column[data-date="${date}"]`,
  );
  if (!element) throw new Error(`no column for ${date}`);
  element.getBoundingClientRect = () => new DOMRect(0, 0, 100, 24 * HOUR_PX);
  return element;
}

const pointer = (type: string, target: Element | Window, clientY: number) =>
  fireEvent(
    target,
    new MouseEvent(type, { bubbles: true, clientY, button: 0 }),
  );

async function drag(date: string, from: number, to: number) {
  const target = column(date);
  await pointer("pointerdown", target, from);
  await pointer("pointermove", window, to);
  await pointer("pointerup", window, to);
}

describe("week view", () => {
  it("shows seven days starting on the configured first day, with today marked", async () => {
    await mount("week");
    const heads = screen.getAllByRole("columnheader");
    expect(
      heads.map((head) => head.textContent?.replace(/\s+/g, " ").trim()),
    ).toEqual([
      "lun 9",
      "mar 10",
      "mié 11",
      "jue 12",
      "vie 13",
      "sáb 14",
      "dom 15",
    ]);
    expect(heads[1].classList.contains("today")).toBe(true);
    expect(screen.getByText("09/03/2026 – 15/03/2026")).toBeTruthy();
  });

  it("puts timed tasks in the grid and untimed ones in the all-day row", async () => {
    await mount("week", [
      makeTask({
        id: "a",
        title: "Clase",
        dueDate: "2026-03-10",
        dueTime: "09:30",
      }),
      makeTask({ id: "b", title: "Entrega", dueDate: "2026-03-11" }),
    ]);
    const block = screen
      .getByText("Clase")
      .closest<HTMLElement>("[data-block]");
    expect(block?.style.top).toBe(`${at(9, 30)}px`);
    expect(screen.getByText("Entrega").closest("[data-block]")).toBeNull();
  });

  it("hands a clicked task to the list", async () => {
    const { onOpenTask } = await mount("week", [
      makeTask({
        id: "a",
        title: "Clase",
        dueDate: "2026-03-10",
        dueTime: "09:30",
      }),
    ]);
    await fireEvent.click(screen.getByText("Clase"));
    expect(onOpenTask).toHaveBeenCalledWith(
      expect.objectContaining({ id: "a" }),
    );
  });

  it("moves a week at a time and back to today", async () => {
    const { view } = await mount("week");
    await fireEvent.click(screen.getByRole("button", { name: "Siguiente" }));
    expect(view.anchor).toBe("2026-03-17");
    await fireEvent.click(screen.getByRole("button", { name: "Anterior" }));
    await fireEvent.click(screen.getByRole("button", { name: "Anterior" }));
    expect(view.anchor).toBe("2026-03-03");
    await fireEvent.click(screen.getByRole("button", { name: "Hoy" }));
    expect(view.anchor).toBe("2026-03-10");
  });
});

describe("quick creation by dragging (F-11)", () => {
  it("creates a task at the start of the dragged span", async () => {
    const { api, store } = await mount("week");
    await drag("2026-03-11", at(9), at(10, 30));

    expect(screen.getByText("09:00 – 10:45")).toBeTruthy();
    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, { target: { value: "Tutoría" } });
    await fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() =>
      expect(api.calls).toContain("taskCreate:Tutoría@09:00"),
    );
    await waitFor(() =>
      expect(
        store.tasks.find((task) => task.title === "Tutoría"),
      ).toMatchObject({
        dueDate: "2026-03-11",
        dueTime: "09:00",
      }),
    );
    await waitFor(() =>
      expect(screen.queryByLabelText("Título de la nueva tarea")).toBeNull(),
    );
    expect(document.querySelector("[data-block]")?.textContent).toContain(
      "Tutoría",
    );
  });

  it("saves a dragged span as the task's duration, but not a plain click", async () => {
    const { store } = await mount("week");
    await drag("2026-03-11", at(9), at(10, 30));
    let input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, { target: { value: "Tutoría" } });
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(store.tasks).toHaveLength(1));
    expect(store.tasks[0].durationMinutes).toBe(105);

    await waitFor(() =>
      expect(screen.queryByLabelText("Título de la nueva tarea")).toBeNull(),
    );
    await drag("2026-03-12", at(16, 5), at(16, 5));
    input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, { target: { value: "Rápida" } });
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(store.tasks).toHaveLength(2));
    expect(store.tasks[1].durationMinutes).toBeNull();
  });

  it("draws a task as tall as its duration", async () => {
    await mount("day", [
      makeTask({
        id: "t",
        title: "Clase",
        dueDate: "2026-03-10",
        dueTime: "09:00",
        durationMinutes: 120,
      }),
    ]);
    const block = document.querySelector<HTMLElement>("[data-block]");
    expect(block?.style.height).toBe(`${2 * HOUR_PX - 2}px`);
    expect(block?.textContent).toContain("09:00–11:00");
  });

  it("works dragging upwards and snaps to quarter hours", async () => {
    const { api } = await mount("week");
    await drag("2026-03-09", at(14, 40), at(13, 20));

    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, { target: { value: "Repaso" } });
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(api.calls).toContain("taskCreate:Repaso@13:15"));
  });

  it("creates on a plain click too, using that quarter hour", async () => {
    const { api } = await mount("day");
    const target = column("2026-03-10");
    await pointer("pointerdown", target, at(16, 5));
    await pointer("pointerup", window, at(16, 5));
    expect(screen.getByText("16:00 – 16:15")).toBeTruthy();

    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, { target: { value: "Llamar" } });
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(api.calls).toContain("taskCreate:Llamar@16:00"));
  });

  it("Escape and an empty title create nothing", async () => {
    const { api } = await mount("week");
    await drag("2026-03-10", at(8), at(9));
    await fireEvent.keyDown(screen.getByLabelText("Título de la nueva tarea"), {
      key: "Escape",
    });
    expect(screen.queryByLabelText("Título de la nueva tarea")).toBeNull();

    await drag("2026-03-10", at(8), at(9));
    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.keyDown(input, { key: "Enter" });
    expect(screen.queryByLabelText("Título de la nueva tarea")).toBeNull();
    expect(api.calls.filter((call) => call.startsWith("taskCreate"))).toEqual(
      [],
    );
  });

  it("keeps the text when the backend refuses, and does not start a drag on a task", async () => {
    const { api, store } = await mount("day", [
      makeTask({
        id: "a",
        title: "Clase",
        dueDate: "2026-03-10",
        dueTime: "09:00",
      }),
    ]);
    await pointer("pointerdown", screen.getByText("Clase"), at(9, 10));
    await pointer("pointerup", window, at(9, 10));
    expect(screen.queryByLabelText("Título de la nueva tarea")).toBeNull();

    api.failing.add("taskCreate");
    await drag("2026-03-10", at(12), at(13));
    const input = screen.getByLabelText<HTMLInputElement>(
      "Título de la nueva tarea",
    );
    await fireEvent.input(input, { target: { value: "Fallará" } });
    await fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() => expect(store.notice?.kind).toBe("error"));
    expect(
      screen.getByLabelText<HTMLInputElement>("Título de la nueva tarea").value,
    ).toBe("Fallará");
    expect(store.tasks).toHaveLength(1);
  });
});

describe("scale selector", () => {
  it("switches between day, week and month from inside the calendar", async () => {
    const { view } = await mount("week");
    const group = screen.getByRole("radiogroup", { name: "Vista" });
    expect(group.textContent).toBe("DíaSemanaMes");
    await fireEvent.click(screen.getByRole("radio", { name: "Mes" }));
    expect(view.mode).toBe("month");
    await fireEvent.click(screen.getByRole("radio", { name: "Día" }));
    expect(view.mode).toBe("day");
  });
});

describe("current time line", () => {
  const lines = () =>
    [...document.querySelectorAll<HTMLElement>("[data-now-line]")].map(
      (line) => [
        line.closest<HTMLElement>(".column")?.dataset.date,
        line.dataset.nowLine,
        line.style.top,
      ],
    );

  it("starts on today's column in the week and runs to the end, faint before it", async () => {
    await mount("week");
    // Tuesday 10:00: Monday faint, Tuesday with the dot, Wednesday to Sunday solid.
    expect(lines()).toEqual([
      ["2026-03-09", "faint", `${at(10)}px`],
      ["2026-03-10", "start", `${at(10)}px`],
      ["2026-03-11", "full", `${at(10)}px`],
      ["2026-03-12", "full", `${at(10)}px`],
      ["2026-03-13", "full", `${at(10)}px`],
      ["2026-03-14", "full", `${at(10)}px`],
      ["2026-03-15", "full", `${at(10)}px`],
    ]);
  });

  it("shows one line with its dot in the day view, only when the day is today", async () => {
    const { view } = await mount("day");
    expect(lines()).toEqual([["2026-03-10", "start", `${at(10)}px`]]);
    view.anchor = "2026-03-11";
    await waitFor(() => expect(lines()).toEqual([]));
  });

  it("is absent in a week that does not contain today and in the month view", async () => {
    const { view } = await mount("week");
    view.anchor = "2026-03-24";
    await waitFor(() => expect(lines()).toEqual([]));
    view.mode = "month";
    await waitFor(() => expect(document.querySelector(".month")).toBeTruthy());
    expect(lines()).toEqual([]);
  });
});

describe("day view", () => {
  it("shows a single column for the anchor day", async () => {
    await mount("day");
    expect(screen.getAllByRole("columnheader")).toHaveLength(1);
    expect(screen.getByText("Martes, 10/03/2026")).toBeTruthy();
  });
});

describe("month view", () => {
  it("shows whole weeks, the tasks of each day and «+N más» when a day is full", async () => {
    await mount("month", [
      ...["a", "b", "c", "d", "e"].map((id) =>
        makeTask({
          id,
          title: `Tarea ${id}`,
          dueDate: "2026-03-10",
          position: id.charCodeAt(0),
        }),
      ),
      makeTask({ id: "z", title: "Otro día", dueDate: "2026-03-31" }),
    ]);
    expect(screen.getByText("Marzo de 2026")).toBeTruthy();
    expect(document.querySelectorAll(".cell")).toHaveLength(42);
    expect(screen.getByText("Otro día")).toBeTruthy();
    expect(screen.getAllByText(/^Tarea [a-e]$/)).toHaveLength(3);
    expect(screen.getByText("+2 más")).toBeTruthy();
  });

  it("opens the day view from a day number", async () => {
    const { view } = await mount("month");
    await fireEvent.click(
      screen.getByRole("button", { name: "Ver el día 2026-03-18" }),
    );
    expect(view.mode).toBe("day");
    expect(view.anchor).toBe("2026-03-18");
  });

  it("moves a month at a time, clamping the day", async () => {
    const { view } = await mount("month");
    view.anchor = "2026-01-31";
    await fireEvent.click(screen.getByRole("button", { name: "Siguiente" }));
    expect(view.anchor).toBe("2026-02-28");
  });
});

describe("keyboard creation (N-08)", () => {
  it("opens the quick title from a focused day column and saves at 09:00", async () => {
    const { api } = await mount("week");
    const target = column("2026-03-11");
    target.focus();
    expect(document.activeElement).toBe(target);
    await fireEvent.keyDown(target, { key: "Enter" });

    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, { target: { value: "Teclado" } });
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() =>
      expect(api.calls).toContain("taskCreate:Teclado@09:00"),
    );
  });

  it("lets the typed day, time and length win over the slot (F23-02)", async () => {
    const { api } = await mount("week");
    await fireEvent.keyDown(column("2026-03-11"), { key: "Enter" });
    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, {
      target: { value: "Revisión a las 16:00 durante 45 min !3" },
    });
    expect(
      document.querySelector("[data-capture-preview]")?.textContent,
    ).toContain("16:00–16:45");
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() =>
      expect(api.calls).toContain("taskCreate:Revisión@16:00"),
    );
    expect(api.tasks.at(-1)).toMatchObject({
      dueDate: "2026-03-11",
      durationMinutes: 45,
      priority: 3,
    });
  });

  it("Esc on a detected title keeps it; a second Esc closes the quick title", async () => {
    const { api } = await mount("week");
    await fireEvent.keyDown(column("2026-03-11"), { key: "Enter" });
    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, { target: { value: "Cine mañana" } });
    await fireEvent.keyDown(input, { key: "Escape" });
    expect(screen.getByLabelText("Título de la nueva tarea")).toBe(input);
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() =>
      expect(api.calls).toContain("taskCreate:Cine mañana@09:00"),
    );
    expect(api.tasks.at(-1)?.dueDate).toBe("2026-03-11");
  });

  it("starts at the next free block on today's column", async () => {
    const { api } = await mount("week");
    const target = column("2026-03-10");
    await fireEvent.keyDown(target, { key: " " });
    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.input(input, { target: { value: "Ahora" } });
    await fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(api.calls).toContain("taskCreate:Ahora@10:00"));
  });

  it("does not hijack typing inside the quick title", async () => {
    await mount("week");
    await fireEvent.keyDown(column("2026-03-11"), { key: "Enter" });
    const input = screen.getByLabelText("Título de la nueva tarea");
    await fireEvent.keyDown(input, { key: " " });
    expect(screen.getAllByLabelText("Título de la nueva tarea")).toHaveLength(
      1,
    );
  });

  it("moves the now line when the minute changes and reads the clock every tick", async () => {
    vi.useFakeTimers();
    try {
      let clock = new Date("2026-03-10T10:00:10");
      const store = new TasksStore({
        api: new FakeApi([]),
        now: () => clock,
      });
      await store.load();
      const view = new ViewStore({
        api: new FakeViewApi(),
        now: () => clock,
        firstDay: () => 1,
      });
      view.mode = "day";
      render(CalendarView, {
        store,
        eventStore: new EventsStore({ api: new FakeEventsApi() }),
        view,
        onOpenTask: vi.fn(),
        onOpenEvent: vi.fn(),
        onNewEvent: vi.fn(),
      });
      const line = () =>
        document.querySelector<HTMLElement>("[data-now-line]")?.style.top;
      const before = line();
      clock = new Date("2026-03-10T10:00:40");
      await vi.advanceTimersByTimeAsync(30_000);
      expect(line()).toBe(before);
      clock = new Date("2026-03-10T11:30:40");
      await vi.advanceTimersByTimeAsync(30_000);
      expect(line()).not.toBe(before);
    } finally {
      vi.useRealTimers();
    }
  });
});

describe("events in the calendar (F21-02)", () => {
  const meeting = (api: FakeEventsApi) =>
    api.makeEvent({
      id: "m",
      title: "Reunión",
      startTime: "10:00",
      endTime: "11:00",
    });

  it("shows a timed event as a block of the week and opens it on click", async () => {
    const { onOpenEvent } = await mount("week", [], meeting);
    const block = await screen.findByTitle("Editar el evento «Reunión»");
    expect(block.dataset.eventId).toBe("m");
    expect(block.textContent).toContain("10:00 – 11:00");
    await fireEvent.click(block);
    expect(onOpenEvent).toHaveBeenCalledWith("m");
  });

  it("puts an all-day event in the all-day row", async () => {
    await mount("week", [], (api) =>
      api.makeEvent({
        title: "Vacaciones",
        startTime: null,
        endTime: null,
        allDay: true,
        startDate: "2026-03-10",
        endDate: "2026-03-11",
      }),
    );
    const chips = await screen.findAllByTitle("Editar el evento «Vacaciones»");
    expect(chips).toHaveLength(2);
    expect(chips[0].closest("[data-block]")).toBeNull();
  });

  it("shows events in the month before the tasks", async () => {
    await mount(
      "month",
      [makeTask({ id: "t", title: "Tarea", dueDate: "2026-03-10" })],
      meeting,
    );
    const chip = await screen.findByTitle("Editar el evento «Reunión»");
    const task = screen.getByText("Tarea");
    expect(
      chip.compareDocumentPosition(task) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  it("dragging the block changes its time and a drag does not open it", async () => {
    const { eventsApi, onOpenEvent } = await mount("week", [], meeting);
    const block = await screen.findByTitle("Editar el evento «Reunión»");
    await pointer("pointerdown", block, 0);
    await pointer("pointermove", window, HOUR_PX);
    expect(document.querySelector("[data-drag-preview]")).not.toBeNull();
    await pointer("pointerup", window, HOUR_PX);
    await fireEvent.click(block);
    await waitFor(() => expect(eventsApi.events[0].startTime).toBe("11:00"));
    expect(eventsApi.events[0].endTime).toBe("12:00");
    expect(onOpenEvent).not.toHaveBeenCalled();
  });

  it("dragging the lower edge changes when it ends", async () => {
    const { eventsApi } = await mount("week", [], meeting);
    await screen.findByTitle("Editar el evento «Reunión»");
    const handle = document.querySelector<HTMLElement>("[data-resize]");
    if (!handle) throw new Error("no handle");
    await pointer("pointerdown", handle, 0);
    await pointer("pointermove", window, HOUR_PX);
    await pointer("pointerup", window, HOUR_PX);
    await waitFor(() => expect(eventsApi.events[0].endTime).toBe("12:00"));
    expect(eventsApi.events[0].startTime).toBe("10:00");
  });

  it("asks for a new event from the button of the bar", async () => {
    const { onNewEvent } = await mount("week");
    await fireEvent.click(screen.getByRole("button", { name: "Nuevo evento" }));
    expect(onNewEvent).toHaveBeenCalledWith("2026-03-10", null);
  });
});

describe("timed tasks in the grid (F24)", () => {
  const timed = () =>
    makeTask({
      id: "t",
      title: "Estudiar",
      dueDate: "2026-03-10",
      dueTime: "09:00",
      durationMinutes: 60,
    });

  it("dragging the block moves it in steps of 15 minutes and shows the time", async () => {
    const { store, onOpenTask } = await mount("week", [timed()]);
    const block = await screen.findByTitle("Ver los detalles de «Estudiar»");
    await pointer("pointerdown", block, 0);
    // 1 h 20 min down: snaps to 10:15.
    await pointer("pointermove", window, HOUR_PX * (80 / 60));
    expect(
      document.querySelector("[data-drag-preview]")?.textContent,
    ).toContain("10:15 – 11:15");
    await pointer("pointerup", window, HOUR_PX * (80 / 60));
    await fireEvent.click(block);
    await waitFor(() =>
      expect(store.find("t")).toMatchObject({
        dueTime: "10:15",
        durationMinutes: 60,
      }),
    );
    expect(onOpenTask).not.toHaveBeenCalled();
  });

  it("dragging the lower edge changes the duration", async () => {
    const { store } = await mount("week", [timed()]);
    await screen.findByTitle("Ver los detalles de «Estudiar»");
    const handle = document.querySelector<HTMLElement>("[data-resize]");
    if (!handle) throw new Error("no handle");
    await pointer("pointerdown", handle, 0);
    await pointer("pointermove", window, HOUR_PX / 2);
    await pointer("pointerup", window, HOUR_PX / 2);
    await waitFor(() =>
      expect(store.find("t")).toMatchObject({
        dueTime: "09:00",
        durationMinutes: 90,
      }),
    );
  });

  it("a click without dragging still opens the task", async () => {
    const { onOpenTask } = await mount("week", [timed()]);
    const block = await screen.findByTitle("Ver los detalles de «Estudiar»");
    await pointer("pointerdown", block, 0);
    await pointer("pointerup", window, 0);
    await fireEvent.click(block);
    expect(onOpenTask).toHaveBeenCalled();
  });
});

describe("focus time", () => {
  // Built from local times so the expectations do not depend on the zone of the machine.
  const session = (
    id: string,
    from: string,
    to: string,
    seconds: number,
  ): TimeEntry => ({
    id,
    taskId: "a",
    startedAt: new Date(from).toISOString(),
    endedAt: new Date(to).toISOString(),
    plannedMinutes: 25,
    pausedSeconds: 0,
    pausedAt: null,
    spentSeconds: seconds,
  });
  const study = makeTask({
    id: "a",
    title: "Estudiar",
    dueDate: "2026-03-10",
    dueTime: "09:00",
    durationMinutes: 60,
  });
  const morning = session(
    "s1",
    "2026-03-10T09:00:00",
    "2026-03-10T09:30:00",
    1800,
  );
  const slots = (id: string) =>
    document.querySelectorAll<HTMLElement>(`[data-focus-id="${id}"]`);

  it("draws each session behind the blocks and opens its task", async () => {
    const { onOpenTask } = await mount("week", [study], undefined, [morning]);
    await waitFor(() => expect(slots("s1")).toHaveLength(1));
    const [element] = slots("s1");
    expect(element.style.top).toBe(`${at(9)}px`);
    expect(element.style.height).toBe(`${at(0, 30) - 2}px`);
    expect(element.textContent).toContain("Enfoque · Estudiar");
    expect(element.textContent).toContain("09:00 – 09:30 · 30 min");
    const block = document.querySelector<HTMLElement>(".block[data-task-id]");
    expect(block?.style.left).toContain("8px");

    await fireEvent.click(element);
    expect(onOpenTask).toHaveBeenCalledWith(
      expect.objectContaining({ id: "a" }),
    );
  });

  it("splits a session that crosses midnight between its two days", async () => {
    const night = session(
      "s2",
      "2026-03-10T23:30:00",
      "2026-03-11T00:30:00",
      3600,
    );
    await mount("week", [study], undefined, [night]);
    await waitFor(() => expect(slots("s2")).toHaveLength(2));
    const [before, after] = slots("s2");
    expect(before.style.top).toBe(`${at(23, 30)}px`);
    expect(after.style.top).toBe("0px");
  });

  it("hides the time of tasks that are not shown", async () => {
    const { focusList } = await mount("day", [], undefined, [morning]);
    await waitFor(() => expect(focusList).toHaveBeenCalled());
    expect(slots("s1")).toHaveLength(0);
  });

  it("adds up each day in the month view", async () => {
    const later = session(
      "s3",
      "2026-03-10T16:00:00",
      "2026-03-10T17:10:00",
      4200,
    );
    await mount("month", [study], undefined, [morning, later]);
    const total = await screen.findByText("Enfoque: 1 h 40 min");
    expect(total.closest("[data-date]")?.getAttribute("data-date")).toBe(
      "2026-03-10",
    );
  });

  it("reads the history again when the focus time changes", async () => {
    const { focus, focusList } = await mount("week", [study]);
    await waitFor(() => expect(focusList).toHaveBeenCalledTimes(1));
    focus.totals = { a: 60 };
    await waitFor(() => expect(focusList).toHaveBeenCalledTimes(2));
  });
});
