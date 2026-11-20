// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

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
});
