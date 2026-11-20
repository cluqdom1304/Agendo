// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { tagsStore } from "../../stores/tags.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, makeTask } from "../../testing";
import InboxPanel from "./InboxPanel.svelte";

async function mount(tasks = [makeTask({ id: "a", title: "Comprar pan" })]) {
  const api = new FakeApi(tasks);
  const store = new TasksStore({
    api,
    now: () => new Date("2026-03-10T10:00:00"),
    settleMs: () => 0,
  });
  await store.load();
  const opened: string[] = [];
  render(InboxPanel, { store, onOpenTask: (id: string) => opened.push(id) });
  return { api, store, opened };
}

describe("InboxPanel (F20-04)", () => {
  it("lists only the pending tasks without a date and opens one", async () => {
    const { opened } = await mount([
      makeTask({ id: "a", title: "Comprar pan", priority: 2 }),
      makeTask({ id: "b", title: "Con fecha", dueDate: "2026-03-10" }),
      makeTask({ id: "c", title: "Hecha", isCompleted: true }),
    ]);
    expect(
      screen.getByRole("complementary", { name: "Bandeja de entrada" }),
    ).toBeTruthy();
    expect(screen.queryByText("Con fecha")).toBeNull();
    expect(screen.queryByText("Hecha")).toBeNull();
    expect(
      screen.getByRole("checkbox", {
        name: "Completar «Comprar pan», prioridad alta",
      }),
    ).toBeTruthy();
    await fireEvent.click(screen.getByText("Comprar pan"));
    expect(opened).toEqual(["a"]);
  });

  it("adds an undated task from its field", async () => {
    const { store } = await mount([]);
    expect(screen.getByText("Bandeja vacía")).toBeTruthy();
    const field = screen.getByRole("textbox", {
      name: "Añadir a la bandeja de entrada",
    });
    await fireEvent.input(field, { target: { value: "Llamar a Ana" } });
    await fireEvent.submit(field.closest("form") as HTMLFormElement);
    await waitFor(() => expect(screen.getByText("Llamar a Ana")).toBeTruthy());
    expect(store.tasks[0]).toMatchObject({
      title: "Llamar a Ana",
      dueDate: null,
    });
    expect((field as HTMLInputElement).value).toBe("");
  });

  it("reads dates and known tags from the field, with a preview (F23-02)", async () => {
    tagsStore.tags = [
      {
        id: "g1",
        name: "Casa",
        colorHex: "#4C8DFF",
        parentId: null,
        position: 0,
        pendingCount: 0,
        noteCount: 0,
      },
    ];
    try {
      const { api } = await mount([]);
      const field = screen.getByRole("textbox", {
        name: "Añadir a la bandeja de entrada",
      });
      await fireEvent.input(field, {
        target: { value: "Regar plantas pasado mañana #casa #otra" },
      });
      const preview = document.querySelector("[data-capture-preview]")!;
      expect(preview.textContent).toContain("jueves 12/03/2026");
      expect(preview.textContent).toContain("#casa");
      await fireEvent.submit(field.closest("form") as HTMLFormElement);
      await waitFor(() => expect(api.tasks).toHaveLength(1));
      expect(api.tasks[0]).toMatchObject({
        title: "Regar plantas #otra",
        dueDate: "2026-03-12",
        tagIds: ["g1"],
      });
    } finally {
      tagsStore.tags = [];
    }
  });

  it("Esc keeps the field's text as typed", async () => {
    const { api } = await mount([]);
    const field = screen.getByRole("textbox", {
      name: "Añadir a la bandeja de entrada",
    });
    await fireEvent.input(field, { target: { value: "Serie de hoy" } });
    await fireEvent.keyDown(field, { key: "Escape" });
    await fireEvent.submit(field.closest("form") as HTMLFormElement);
    await waitFor(() => expect(api.tasks).toHaveLength(1));
    expect(api.tasks[0]).toMatchObject({
      title: "Serie de hoy",
      dueDate: null,
    });
  });

  it("completes a task from its circle", async () => {
    const { api } = await mount();
    await fireEvent.click(
      screen.getByRole("checkbox", { name: "Completar «Comprar pan»" }),
    );
    await waitFor(() =>
      expect(
        api.calls.some((call) => call.startsWith("taskSetCompleted")),
      ).toBe(true),
    );
  });

  it("dropping a row on a column of the calendar sets day, start and length (F24-01)", async () => {
    const { api, store } = await mount();
    const column = document.createElement("div");
    column.dataset.slotDrop = "2026-03-11";
    column.getBoundingClientRect = () => ({ top: 100 }) as DOMRect;
    document.body.append(column);
    document.elementFromPoint = () => column;
    try {
      const row = screen.getByText("Comprar pan").closest("li") as HTMLElement;
      await fireEvent.pointerDown(row, { button: 0, clientX: 5, clientY: 5 });
      // 148 px below the top of the column at 48 px/hour is 3 h 04 min: the 03:00 quarter.
      await fireEvent.pointerMove(window, { clientX: 50, clientY: 248 });
      await fireEvent.pointerUp(window, { clientX: 50, clientY: 248 });
      await waitFor(() =>
        expect(store.find("a")).toMatchObject({
          dueDate: "2026-03-11",
          dueTime: "03:00",
          durationMinutes: 30,
        }),
      );
      expect(api.calls.some((call) => call.startsWith("taskUpdate"))).toBe(
        true,
      );
    } finally {
      delete (document as { elementFromPoint?: unknown }).elementFromPoint;
      column.remove();
    }
  });

  it("dropping outside the calendar changes nothing", async () => {
    const { api, store } = await mount();
    document.elementFromPoint = () => document.body;
    try {
      const row = screen.getByText("Comprar pan").closest("li") as HTMLElement;
      await fireEvent.pointerDown(row, { button: 0, clientX: 5, clientY: 5 });
      await fireEvent.pointerMove(window, { clientX: 50, clientY: 90 });
      await fireEvent.pointerUp(window, { clientX: 50, clientY: 90 });
      expect(store.find("a")?.dueDate).toBeNull();
      expect(api.calls.some((call) => call.startsWith("taskUpdate"))).toBe(
        false,
      );
    } finally {
      delete (document as { elementFromPoint?: unknown }).elementFromPoint;
    }
  });
});
