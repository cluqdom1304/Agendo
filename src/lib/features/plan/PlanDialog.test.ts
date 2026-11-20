// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, makeTask } from "../../testing";
import PlanDialog from "./PlanDialog.svelte";

async function mount(tasks = [] as ReturnType<typeof makeTask>[]) {
  const api = new FakeApi(tasks);
  const store = new TasksStore({
    api,
    now: () => new Date("2026-03-10T10:00:00"),
    settleMs: () => 0,
  });
  await store.load();
  let closed = false;
  render(PlanDialog, { store, onClose: () => (closed = true) });
  return { store, isClosed: () => closed };
}

describe("PlanDialog (F24-02)", () => {
  it("sums today's tasks and the chosen ones, and moves the chosen to today", async () => {
    const { store, isClosed } = await mount([
      makeTask({
        id: "now",
        title: "Hoy",
        dueDate: "2026-03-10",
        dueTime: "09:00",
        durationMinutes: 60,
      }),
      makeTask({
        id: "late",
        title: "Vencida",
        dueDate: "2026-03-08",
        durationMinutes: 30,
      }),
      makeTask({ id: "none", title: "Suelta" }),
    ]);
    expect(screen.getByText("2 tareas · 1 h 30 min")).toBeTruthy();
    await fireEvent.click(screen.getByRole("checkbox", { name: /Suelta/ }));
    expect(
      screen.getByText("3 tareas · 1 h 30 min (1 sin estimar)"),
    ).toBeTruthy();
    await fireEvent.click(
      screen.getByRole("button", { name: "Pasar 2 tareas a hoy" }),
    );
    await waitFor(() => expect(isClosed()).toBe(true));
    expect(store.find("late")?.dueDate).toBe("2026-03-10");
    expect(store.find("none")?.dueDate).toBe("2026-03-10");
  });

  it("warns when the day is over eight hours", async () => {
    await mount([
      makeTask({ id: "a", dueDate: "2026-03-10", durationMinutes: 300 }),
      makeTask({ id: "b", dueDate: "2026-03-09", durationMinutes: 240 }),
    ]);
    expect(screen.getByRole("alert").textContent).toContain("8 horas");
  });

  it("leaves everything as it was when cancelled", async () => {
    const { store, isClosed } = await mount([
      makeTask({ id: "late", title: "Vencida", dueDate: "2026-03-08" }),
    ]);
    await fireEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(isClosed()).toBe(true);
    expect(store.find("late")?.dueDate).toBe("2026-03-08");
  });
});
