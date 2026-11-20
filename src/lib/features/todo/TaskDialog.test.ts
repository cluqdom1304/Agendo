// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, makeTask } from "../../testing";
import type { Task } from "../../types";
import TaskDialog from "./TaskDialog.svelte";

const NOW = new Date(2026, 2, 10, 10, 0);

async function mount(defaultDate: string | null = null) {
  const api = new FakeApi([]);
  const store = new TasksStore({ api, now: () => NOW, settleMs: () => 0 });
  await store.load();
  const onClose = vi.fn();
  render(TaskDialog, { store, defaultDate, now: () => NOW, onClose });
  return { api, store, onClose };
}

const title = () => screen.getByPlaceholderText("¿Qué hay que hacer?");

describe("TaskDialog", () => {
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

  it("creates a task with a priority (F20-02)", async () => {
    const { store, onClose } = await mount();
    await fireEvent.input(title(), { target: { value: "Pagar" } });
    await fireEvent.click(screen.getByRole("button", { name: "Urgente" }));
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.tasks[0]?.priority).toBe(3);
  });

  it("ignores the partial years reported while the date is typed", async () => {
    const { store, onClose } = await mount();
    await fireEvent.input(title(), { target: { value: "Entrega" } });
    const date = screen.getByLabelText("Fecha") as HTMLInputElement;
    for (const value of ["0002-11-23", "0020-11-23", "0202-11-23"]) {
      await fireEvent.change(date, { target: { value } });
    }
    await fireEvent.change(date, { target: { value: "2026-11-23" } });
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.tasks[0].dueDate).toBe("2026-11-23");
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

  it("offers a duration only once there is a start time, and saves it", async () => {
    const { store, onClose } = await mount("2026-03-12");
    expect(screen.queryByRole("button", { name: "30 min" })).toBeNull();
    await fireEvent.input(title(), { target: { value: "Clase" } });
    await fireEvent.input(screen.getByLabelText("Hora"), {
      target: { value: "09:00" },
    });
    const preset = screen.getByRole("button", { name: "30 min" });
    await fireEvent.click(preset);
    expect(preset.getAttribute("aria-pressed")).toBe("true");
    expect(
      (screen.getByLabelText("Termina a las") as HTMLInputElement).value,
    ).toBe("09:30");

    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.tasks[0].durationMinutes).toBe(30);
  });

  it("takes an exact duration from the end time and rejects an end before the start", async () => {
    const { store, onClose } = await mount("2026-03-12");
    await fireEvent.input(title(), { target: { value: "Taller" } });
    await fireEvent.input(screen.getByLabelText("Hora"), {
      target: { value: "10:00" },
    });
    const end = screen.getByLabelText("Termina a las");
    await fireEvent.input(end, { target: { value: "09:00" } });
    expect((await screen.findByRole("alert")).textContent).toContain(
      "posterior",
    );
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));
    expect(onClose).not.toHaveBeenCalled();

    await fireEvent.input(end, { target: { value: "11:20" } });
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.tasks[0].durationMinutes).toBe(80);
  });
});

describe("TaskDialog editing a task", () => {
  async function edit(task: Task) {
    const api = new FakeApi([task]);
    const store = new TasksStore({ api, now: () => NOW, settleMs: () => 0 });
    await store.load();
    const onClose = vi.fn();
    render(TaskDialog, { store, task, now: () => NOW, onClose });
    return { api, store, onClose };
  }

  const timed = makeTask({
    id: "t1",
    title: "Reunión",
    dueDate: "2026-03-12",
    dueTime: "09:00",
    durationMinutes: 60,
  });

  it("starts from the task and saves the changes", async () => {
    const { store, onClose } = await edit(timed);
    expect(screen.getByRole("dialog", { name: "Editar tarea" })).toBeTruthy();
    expect((title() as HTMLInputElement).value).toBe("Reunión");
    expect(
      screen.getByRole("button", { name: "1 h" }).getAttribute("aria-pressed"),
    ).toBe("true");
    expect(screen.queryByLabelText("Descripción")).toBeNull();

    await fireEvent.input(title(), { target: { value: "Reunión larga" } });
    await fireEvent.click(screen.getByRole("button", { name: "15 min" }));
    await fireEvent.click(
      screen.getByRole("button", { name: "Guardar cambios" }),
    );

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.find("t1")).toMatchObject({
      title: "Reunión larga",
      durationMinutes: 15,
      dueTime: "09:00",
    });
  });

  it("shows and changes the priority (F20-02)", async () => {
    const { store, onClose } = await edit({ ...timed, priority: 1 });
    expect(
      screen
        .getByRole("button", { name: "Media" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
    await fireEvent.click(screen.getByRole("button", { name: "Alta" }));
    await fireEvent.click(
      screen.getByRole("button", { name: "Guardar cambios" }),
    );
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.find("t1")?.priority).toBe(2);
  });

  it("drops the duration when the time is removed", async () => {
    const { store, onClose } = await edit(timed);
    await fireEvent.input(screen.getByLabelText("Hora"), {
      target: { value: "" },
    });
    expect(screen.queryByRole("button", { name: "30 min" })).toBeNull();
    await fireEvent.click(
      screen.getByRole("button", { name: "Guardar cambios" }),
    );
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.find("t1")).toMatchObject({
      dueTime: null,
      durationMinutes: null,
    });
  });

  it("does not offer reminders on a task that is done", async () => {
    await edit({ ...timed, isCompleted: true });
    expect(screen.getByText(/no avisa/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "15 min antes" })).toBeNull();
  });

  it("sends reminders only when they were touched", async () => {
    const { api, onClose } = await edit(timed);
    await fireEvent.click(
      screen.getByRole("button", { name: "Guardar cambios" }),
    );
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.calls.some((call) => call.startsWith("remindersSet"))).toBe(
      false,
    );
  });
  it("creates a repeating task with its interval and end date (F13-05)", async () => {
    const { store, onClose } = await mount("2026-03-10");
    await fireEvent.input(title(), { target: { value: "Regar" } });
    await fireEvent.change(screen.getByLabelText("Repetir la tarea"), {
      target: { value: "WEEKLY" },
    });
    await fireEvent.input(screen.getByLabelText(/^Cada/), {
      target: { value: "2" },
    });
    await fireEvent.input(screen.getByLabelText("Hasta (opcional)"), {
      target: { value: "2026-12-31" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.tasks[0]?.rrule).toBe("FREQ=WEEKLY;INTERVAL=2;UNTIL=20261231");
  });

  it("creates a weekly task on chosen weekdays that ends after N times (F17-01)", async () => {
    const { store, onClose } = await mount("2026-03-09");
    await fireEvent.input(title(), { target: { value: "Gimnasio" } });
    await fireEvent.change(screen.getByLabelText("Repetir la tarea"), {
      target: { value: "WEEKLY" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "viernes" }));
    await fireEvent.click(screen.getByRole("button", { name: "lunes" }));
    await fireEvent.input(screen.getByLabelText("Número de veces (opcional)"), {
      target: { value: "8" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(store.tasks[0]?.rrule).toBe(
      "FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,FR;COUNT=8",
    );
  });

  it("does not offer to repeat a task without a date and refuses an end before the task", async () => {
    await mount();
    expect(
      (screen.getByLabelText("Repetir la tarea") as HTMLSelectElement).disabled,
    ).toBe(true);
    expect(
      screen.getByText("Para repetir una tarea, ponle fecha."),
    ).toBeTruthy();
  });

  it("refuses an end date before the task's date", async () => {
    const { api, onClose } = await mount("2026-03-10");
    await fireEvent.input(title(), { target: { value: "Regar" } });
    await fireEvent.change(screen.getByLabelText("Repetir la tarea"), {
      target: { value: "DAILY" },
    });
    await fireEvent.input(screen.getByLabelText("Hasta (opcional)"), {
      target: { value: "2026-03-01" },
    });
    expect((await screen.findByRole("alert")).textContent).toContain("antes");
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));
    expect(api.calls.some((call) => call.startsWith("taskCreate"))).toBe(false);
    expect(onClose).not.toHaveBeenCalled();
  });
});
