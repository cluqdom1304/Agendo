// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi } from "../../testing";
import NewTaskDialog from "./NewTaskDialog.svelte";

const NOW = new Date(2026, 2, 10, 10, 0);

async function mount(defaultDate: string | null = null) {
  const api = new FakeApi([]);
  const store = new TasksStore({ api, now: () => NOW, settleMs: () => 0 });
  await store.load();
  const onClose = vi.fn();
  render(NewTaskDialog, { store, defaultDate, now: () => NOW, onClose });
  return { api, store, onClose };
}

const title = () => screen.getByPlaceholderText("¿Qué hay que hacer?");

describe("NewTaskDialog", () => {
  it("is a modal dialog with the title focused", async () => {
    await mount();
    expect(screen.getByRole("dialog", { name: "Nueva tarea" })).toBeTruthy();
    expect(document.activeElement).toBe(title());
  });

  it("asks for a title and creates nothing without one", async () => {
    const { api, onClose } = await mount();
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));
    expect((await screen.findByRole("alert")).textContent).toContain("título");
    expect(api.calls.some((call) => call.startsWith("taskCreate"))).toBe(false);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("creates a task with only a title on Enter", async () => {
    const { api, store, onClose } = await mount();
    await fireEvent.input(title(), { target: { value: "Llamar al banco" } });
    await fireEvent.keyDown(title(), { key: "Enter" });

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.calls).toContain("taskCreate:Llamar al banco");
    expect(store.tasks.map((task) => task.title)).toEqual(["Llamar al banco"]);
  });

  it("creates a task with date, time, description and reminders", async () => {
    const { api, store, onClose } = await mount();
    await fireEvent.input(title(), { target: { value: "Entrega" } });
    await fireEvent.change(screen.getByLabelText("Fecha"), {
      target: { value: "2026-03-12" },
    });
    await fireEvent.input(screen.getByLabelText("Hora"), {
      target: { value: "09:30" },
    });
    await fireEvent.input(screen.getByLabelText("Descripción"), {
      target: { value: "Primera línea\n\nSegunda línea" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "15 min antes" }));
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.calls).toContain("taskCreate:Entrega@09:30");
    expect(api.calls.some((call) => call.startsWith("remindersSet"))).toBe(
      true,
    );
    const task = store.tasks[0];
    expect(task).toMatchObject({ dueDate: "2026-03-12", dueTime: "09:30" });
    expect(JSON.parse(task.contentJson).content).toHaveLength(2);
    expect(task.reminders).toHaveLength(1);
    expect(task.reminders[0].offsetMinutes).toBe(15);
  });

  it("pre-fills the day it is opened from and disables the time without a date", async () => {
    await mount("2026-03-14");
    expect((screen.getByLabelText("Fecha") as HTMLInputElement).value).toBe(
      "2026-03-14",
    );
    await fireEvent.click(screen.getByRole("button", { name: "Quitar fecha" }));
    expect((screen.getByLabelText("Hora") as HTMLInputElement).disabled).toBe(
      true,
    );
  });

  it("refuses a reminder that would already have passed", async () => {
    const { api, onClose } = await mount();
    await fireEvent.input(title(), { target: { value: "Tarde" } });
    await fireEvent.change(screen.getByLabelText("Fecha"), {
      target: { value: "2026-03-10" },
    });
    await fireEvent.input(screen.getByLabelText("Hora"), {
      target: { value: "10:10" },
    });
    await fireEvent.click(
      screen.getByRole("button", { name: "En el momento" }),
    );
    await fireEvent.change(screen.getByLabelText("Fecha"), {
      target: { value: "2026-03-09" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));

    expect((await screen.findByRole("alert")).textContent).toContain(
      "ya habría pasado",
    );
    expect(api.calls.some((call) => call.startsWith("taskCreate"))).toBe(false);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("closes with Escape and with Cancel without creating anything", async () => {
    const { api, onClose } = await mount();
    await fireEvent.input(title(), { target: { value: "Descartada" } });
    await fireEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(onClose).toHaveBeenCalled();
    expect(api.calls.some((call) => call.startsWith("taskCreate"))).toBe(false);
  });
});
