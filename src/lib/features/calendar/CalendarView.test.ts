// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { TasksStore } from "../../stores/tasks.svelte";
import { ViewStore, type ViewApi } from "../../stores/view.svelte";
import { FakeApi, installLayoutStubs, makeTask } from "../../testing";
import type { Task, ViewMode } from "../../types";
import CalendarView from "./CalendarView.svelte";

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

async function mount(mode: ViewMode, tasks: Task[] = []) {
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
  render(CalendarView, { store, view, onOpenTask });
  return { api, store, view, onOpenTask };
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
