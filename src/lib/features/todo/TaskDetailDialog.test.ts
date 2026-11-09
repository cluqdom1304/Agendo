// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, installLayoutStubs, makeTask } from "../../testing";
import type { Task } from "../../types";
import TaskDetailDialog from "./TaskDetailDialog.svelte";

installLayoutStubs();

const NOW = new Date(2026, 2, 10, 10, 0);

const base = makeTask({
  id: "t1",
  title: "Reunión",
  dueDate: "2026-03-12",
  dueTime: "09:00",
  durationMinutes: 90,
});

async function mount(task: Task = base) {
  const api = new FakeApi([task]);
  const store = new TasksStore({ api, now: () => NOW, settleMs: () => 0 });
  await store.load();
  const onClose = vi.fn();
  const onEdit = vi.fn();
  const onShowInList = vi.fn();
  render(TaskDetailDialog, {
    store,
    taskId: task.id,
    onEdit,
    onShowInList,
    onClose,
  });
  return { api, store, onClose, onEdit, onShowInList };
}

describe("TaskDetailDialog", () => {
  it("shows the task with its schedule and duration", async () => {
    await mount();
    expect(
      screen.getByRole("dialog", { name: "Detalles de la tarea" }),
    ).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Reunión" })).toBeTruthy();
    expect(screen.getByText(/09:00–10:30/)).toBeTruthy();
    expect(screen.getByText(/1 h 30 min/)).toBeTruthy();
    expect(screen.getByText("12/03/2026")).toBeTruthy();
    expect(screen.getByText("Pendiente")).toBeTruthy();
  });

  it("marks the task as done and back", async () => {
    const { store } = await mount();
    await fireEvent.click(
      screen.getByRole("button", { name: "Marcar como hecha" }),
    );
    await waitFor(() => expect(store.find("t1")?.isCompleted).toBe(true));
    expect(await screen.findByRole("button", { name: "Reabrir" })).toBeTruthy();
    expect(screen.getByText("Hecha")).toBeTruthy();
    expect(screen.getByText(/no avisa/)).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Reabrir" }));
    await waitFor(() => expect(store.find("t1")?.isCompleted).toBe(false));
  });

  it("deletes the task with the button and closes", async () => {
    const { api, store, onClose } = await mount();
    await fireEvent.click(screen.getByRole("button", { name: "Eliminar" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.calls).toContain("taskDelete:t1");
    expect(store.find("t1")).toBeUndefined();
  });

  it("deletes with the Delete key unless the user is typing", async () => {
    const { api, onClose } = await mount();
    const button = screen.getByRole("button", { name: "Editar" });
    button.focus();
    await fireEvent.keyDown(button, { key: "Delete" });
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.calls).toContain("taskDelete:t1");
  });

  it("hands the task to the edit dialog and to the list", async () => {
    const { onEdit, onShowInList } = await mount();
    await fireEvent.click(screen.getByRole("button", { name: "Editar" }));
    expect(onEdit).toHaveBeenCalledWith(expect.objectContaining({ id: "t1" }));
    await fireEvent.click(
      screen.getByRole("button", { name: "Ver en la lista" }),
    );
    expect(onShowInList).toHaveBeenCalled();
  });
});
