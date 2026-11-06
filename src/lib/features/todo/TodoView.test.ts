// @vitest-environment jsdom
import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, installLayoutStubs, makeTask } from "../../testing";
import type { Task } from "../../types";
import TodoView from "./TodoView.svelte";

async function mount(tasks: Task[]) {
  const api = new FakeApi(tasks);
  const store = new TasksStore({
    api,
    now: () => new Date("2026-03-10T10:00:00Z"),
    settleMs: () => 0,
  });
  await store.load();
  render(TodoView, { store });
  return { api, store };
}

const rowTitles = () =>
  screen
    .getAllByRole("row")
    .map((row) => (row.querySelector("input") as HTMLInputElement).value);

const rowFor = (title: string) =>
  screen.getByDisplayValue(title).closest<HTMLElement>("[data-task-row]")!;

const key = (init: KeyboardEventInit, target: Element = document.body) =>
  fireEvent.keyDown(target, init);

installLayoutStubs();
afterEach(() => vi.useRealTimers());

describe("document layout", () => {
  it("shows POR HACER and AGENDA with dates as DD/MM/YYYY, latest first", async () => {
    await mount([
      makeTask({ id: "t", title: "Sin fecha" }),
      makeTask({ id: "o", title: "Antigua", dueDate: "2026-03-01" }),
      makeTask({ id: "n", title: "Reciente", dueDate: "2026-03-12" }),
    ]);
    expect(screen.getByText("POR HACER")).toBeTruthy();
    expect(screen.getByText("AGENDA")).toBeTruthy();
    const days = screen.getAllByRole("heading", { level: 3 });
    expect(days.map((day) => day.textContent)).toEqual([
      "12/03/2026",
      "01/03/2026",
    ]);
    expect(rowTitles()).toEqual(["Sin fecha", "Reciente", "Antigua"]);
  });

  it("orders a day: timed first, untimed by position, completed last", async () => {
    const day = "2026-03-10";
    await mount([
      makeTask({ id: "1", title: "Sin hora B", dueDate: day, position: 2 }),
      makeTask({
        id: "2",
        title: "Hecha",
        dueDate: day,
        isCompleted: true,
        completedAt: "2026-03-10T07:00:00Z",
      }),
      makeTask({ id: "3", title: "A las 18", dueDate: day, dueTime: "18:00" }),
      makeTask({ id: "4", title: "Sin hora A", dueDate: day, position: 1 }),
      makeTask({ id: "5", title: "A las 08", dueDate: day, dueTime: "08:00" }),
    ]);
    expect(rowTitles()).toEqual([
      "A las 08",
      "A las 18",
      "Sin hora A",
      "Sin hora B",
      "Hecha",
    ]);
    expect(screen.getByText("Completadas")).toBeTruthy();
  });

  it("shows the empty state and starts a draft from its button", async () => {
    await mount([]);
    expect(screen.getByText("Todo en orden")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: /Nueva tarea/ }));
    expect(document.activeElement?.getAttribute("aria-label")).toBe(
      "Título de la nueva tarea",
    );
  });
});

describe("completing", () => {
  it("moves the task under Completadas of its day", async () => {
    await mount([
      makeTask({ id: "a", title: "Una", dueDate: "2026-03-10", position: 1 }),
      makeTask({ id: "b", title: "Otra", dueDate: "2026-03-10", position: 2 }),
    ]);
    await fireEvent.click(
      within(rowFor("Una")).getByRole("checkbox", { name: /Completar/ }),
    );
    await vi.waitFor(() =>
      expect(screen.getByText("Completadas")).toBeTruthy(),
    );
    expect(rowTitles()).toEqual(["Otra", "Una"]);
    expect(
      within(rowFor("Una")).getByRole("checkbox").getAttribute("aria-checked"),
    ).toBe("true");
  });

  it("toggles with Space on the focused row", async () => {
    await mount([makeTask({ id: "a", title: "Una" })]);
    const row = rowFor("Una");
    row.focus();
    await key({ key: " " }, row);
    await vi.waitFor(() =>
      expect(screen.getByText("Completadas")).toBeTruthy(),
    );
  });

  it("does not toggle when Space is typed inside the title", async () => {
    await mount([makeTask({ id: "a", title: "Una" })]);
    const input = screen.getByDisplayValue("Una");
    input.focus();
    await key({ key: " " }, input);
    expect(screen.queryByText("Completadas")).toBeNull();
  });
});

describe("deleting with undo", () => {
  it("deletes with Supr and brings the task back from the toast", async () => {
    await mount([
      makeTask({ id: "a", title: "Una", position: 1 }),
      makeTask({ id: "b", title: "Otra", position: 2 }),
    ]);
    const row = rowFor("Una");
    row.focus();
    await key({ key: "Delete" }, row);
    await vi.waitFor(() =>
      expect(screen.getByText("Tarea eliminada")).toBeTruthy(),
    );
    expect(rowTitles()).toEqual(["Otra"]);
    expect(document.activeElement).toBe(rowFor("Otra"));

    await fireEvent.click(screen.getByRole("button", { name: "Deshacer" }));
    await vi.waitFor(() => expect(rowTitles()).toEqual(["Una", "Otra"]));
    expect(screen.queryByText("Tarea eliminada")).toBeNull();
  });

  it("undoes with Ctrl+Z", async () => {
    await mount([makeTask({ id: "a", title: "Una" })]);
    const row = rowFor("Una");
    row.focus();
    await key({ key: "Delete" }, row);
    await vi.waitFor(() =>
      expect(screen.getByText("Tarea eliminada")).toBeTruthy(),
    );
    await key({ key: "z", ctrlKey: true });
    await vi.waitFor(() => expect(rowTitles()).toEqual(["Una"]));
  });
});

describe("creating and editing", () => {
  it("Ctrl+N opens a focused draft; Enter saves it and opens the next one", async () => {
    const { api } = await mount([makeTask({ id: "a", title: "Existente" })]);
    await key({ key: "n", ctrlKey: true });
    const input = screen.getByPlaceholderText(
      "Nueva tarea",
    ) as HTMLInputElement;
    expect(document.activeElement).toBe(input);

    await fireEvent.input(input, { target: { value: "Primera" } });
    await key({ key: "Enter" }, input);
    await vi.waitFor(() => expect(rowTitles()).toContain("Primera"));
    expect(api.calls).toContain("taskCreate:Primera");

    const next = (await screen.findByPlaceholderText(
      "Nueva tarea",
    )) as HTMLInputElement;
    expect(next.value).toBe("");
    await vi.waitFor(() => expect(document.activeElement).toBe(next));
  });

  it("Ctrl+Shift+N creates a task dated today", async () => {
    const { api } = await mount([]);
    await key({ key: "N", ctrlKey: true, shiftKey: true });
    const input = screen.getByPlaceholderText("Nueva tarea");
    await fireEvent.input(input, { target: { value: "Hoy" } });
    await key({ key: "Enter" }, input);
    await screen.findByText("10/03/2026");
    expect(api.tasks.at(-1)?.dueDate).toBe("2026-03-10");
  });

  it("Esc discards a draft", async () => {
    const { api } = await mount([makeTask({ id: "a", title: "Existente" })]);
    await key({ key: "n", ctrlKey: true });
    const input = screen.getByPlaceholderText("Nueva tarea");
    await fireEvent.input(input, { target: { value: "No guardar" } });
    await key({ key: "Escape" }, input);
    expect(screen.queryByPlaceholderText("Nueva tarea")).toBeNull();
    expect(api.calls.some((call) => call.startsWith("taskCreate"))).toBe(false);
  });

  it("saves a renamed title on Enter and on losing focus", async () => {
    const { api } = await mount([
      makeTask({ id: "a", title: "Una", position: 1 }),
      makeTask({ id: "b", title: "Otra", position: 2 }),
    ]);
    const first = screen.getByDisplayValue("Una");
    first.focus();
    await fireEvent.input(first, { target: { value: "Una editada" } });
    await key({ key: "Enter" }, first);
    await vi.waitFor(() =>
      expect(api.calls).toContain("taskUpdate:a=Una editada"),
    );

    const second = screen.getByDisplayValue("Otra");
    second.focus();
    await fireEvent.input(second, { target: { value: "Otra editada" } });
    await fireEvent.blur(second);
    await vi.waitFor(() =>
      expect(api.calls).toContain("taskUpdate:b=Otra editada"),
    );
  });

  it("Esc restores the stored title without saving", async () => {
    const { api } = await mount([makeTask({ id: "a", title: "Una" })]);
    const input = screen.getByDisplayValue("Una") as HTMLInputElement;
    input.focus();
    await fireEvent.input(input, { target: { value: "Cambio" } });
    await key({ key: "Escape" }, input);
    expect(input.value).toBe("Una");
    expect(api.calls.some((call) => call.startsWith("taskUpdate"))).toBe(false);
  });

  it("moves the selection with the arrow keys", async () => {
    await mount([
      makeTask({ id: "a", title: "Una", position: 1 }),
      makeTask({ id: "b", title: "Otra", position: 2 }),
    ]);
    const first = rowFor("Una");
    first.focus();
    await key({ key: "ArrowDown" }, first);
    expect(document.activeElement).toBe(rowFor("Otra"));
    await key({ key: "ArrowUp" }, rowFor("Otra"));
    expect(document.activeElement).toBe(first);
  });
});

describe("rendering cost", () => {
  /** Task whose title counts how many times it is read by the UI. */
  function counted(task: Task, reads: Map<string, number>): Task {
    const { title } = task;
    return Object.defineProperty({ ...task }, "title", {
      enumerable: true,
      get() {
        reads.set(task.id, (reads.get(task.id) ?? 0) + 1);
        return title;
      },
    });
  }

  it("does not re-render rows of tasks that did not change", async () => {
    const reads = new Map<string, number>();
    const tasks = ["a", "b", "c", "d"].map((id, index) =>
      counted(makeTask({ id, position: index }), reads),
    );
    const api = new FakeApi(tasks);
    const store = new TasksStore({ api, settleMs: () => 0 });
    await store.load();
    render(TodoView, { store });
    expect([...reads.keys()].sort()).toEqual(["a", "b", "c", "d"]);

    const snapshot = new Map(reads);
    await store.rename("c", "c renombrada");
    await vi.waitFor(() => expect(screen.getByDisplayValue("c renombrada")));

    for (const id of ["a", "b", "d"]) {
      expect(reads.get(id)).toBe(snapshot.get(id));
    }
    expect(reads.get("c")).toBeGreaterThan(snapshot.get("c") ?? 0);

    const beforeComplete = new Map(reads);
    await store.setCompleted("b", true);
    for (const id of ["a", "d"]) {
      expect(reads.get(id)).toBe(beforeComplete.get(id));
    }
  });
});

describe("notes and search", () => {
  const notes = JSON.stringify({
    type: "doc",
    content: [
      { type: "paragraph", content: [{ type: "text", text: "detalle" }] },
    ],
  });

  it("opens and closes the notes under a task with its toggle", async () => {
    await mount([
      makeTask({ id: "a", title: "Con notas", contentJson: notes }),
    ]);
    expect(
      screen.queryByRole("textbox", { name: "Notas de la tarea" }),
    ).toBeNull();
    await fireEvent.click(
      screen.getByRole("button", { name: /Mostrar notas/ }),
    );
    expect(
      (await screen.findByRole("textbox", { name: "Notas de la tarea" }))
        .textContent,
    ).toBe("detalle");
    await fireEvent.click(
      screen.getByRole("button", { name: /Ocultar notas/ }),
    );
    expect(
      screen.queryByRole("textbox", { name: "Notas de la tarea" }),
    ).toBeNull();
  });

  it("saves edited notes through the store", async () => {
    const { api } = await mount([makeTask({ id: "a", title: "Tarea" })]);
    await fireEvent.click(
      screen.getByRole("button", { name: /Mostrar notas/ }),
    );
    await screen.findByRole("textbox", { name: "Notas de la tarea" });
    await fireEvent.click(
      screen.getByRole("button", { name: "Lista con viñetas" }),
    );
    await fireEvent.click(
      screen.getByRole("button", { name: /Ocultar notas/ }),
    );
    await waitFor(() =>
      expect(
        api.calls.some((call) => call.startsWith("taskUpdate:a#notes=")),
      ).toBe(true),
    );
  });

  it("Ctrl+K opens the search and Enter reveals the chosen task with its notes", async () => {
    const { store } = await mount([
      makeTask({ id: "a", title: "Primera" }),
      makeTask({ id: "b", title: "Estudiar álgebra", contentJson: notes }),
    ]);
    await key({ key: "k", ctrlKey: true });
    const field = await screen.findByRole("combobox");
    await fireEvent.input(field, { target: { value: "algebra" } });
    await screen.findByRole("option");
    await fireEvent.keyDown(field, { key: "Enter" });

    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(store.selectedId).toBe("b");
    expect(store.notesOpenId).toBe("b");
    expect(
      await screen.findByRole("textbox", { name: "Notas de la tarea" }),
    ).toBeTruthy();
  });

  it("does not run list shortcuts while the search is open", async () => {
    const { api } = await mount([makeTask({ id: "a", title: "Primera" })]);
    await key({ key: "k", ctrlKey: true });
    const field = await screen.findByRole("combobox");
    await fireEvent.keyDown(field, { key: "n", ctrlKey: true });
    expect(screen.getAllByRole("row")).toHaveLength(1);
    expect(
      api.calls.filter((call) => call.startsWith("taskCreate")),
    ).toHaveLength(0);
  });
});

describe("row actions", () => {
  it("setting a date moves the task into that day, newest day first", async () => {
    await mount([
      makeTask({ id: "a", title: "Uno", dueDate: "2026-03-01" }),
      makeTask({ id: "b", title: "Dos" }),
    ]);
    await fireEvent.change(screen.getByLabelText("Fecha de «Dos»"), {
      target: { value: "2026-03-15" },
    });
    await waitFor(() =>
      expect(
        screen.getAllByRole("heading", { level: 3 }).map((h) => h.textContent),
      ).toEqual(["15/03/2026", "01/03/2026"]),
    );
  });

  it("clearing the date sends the task back to POR HACER", async () => {
    const { store } = await mount([
      makeTask({ id: "a", title: "Uno", dueDate: "2026-03-01" }),
    ]);
    await fireEvent.change(screen.getByLabelText("Fecha de «Uno»"), {
      target: { value: "" },
    });
    await waitFor(() => expect(store.find("a")?.dueDate).toBeNull());
  });

  it("the delete button removes the task and the toast restores it", async () => {
    await mount([makeTask({ id: "a", title: "Uno" })]);
    await fireEvent.click(
      screen.getByRole("button", { name: "Eliminar «Uno»" }),
    );
    await waitFor(() => expect(screen.queryByDisplayValue("Uno")).toBeNull());
    await fireEvent.click(screen.getByRole("button", { name: "Deshacer" }));
    expect(await screen.findByDisplayValue("Uno")).toBeTruthy();
  });
});
