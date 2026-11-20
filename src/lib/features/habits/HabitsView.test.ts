// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { HabitsStore } from "../../stores/habits.svelte";
import { ViewStore } from "../../stores/view.svelte";
import { FakeHabitsApi } from "../../testing";
import HabitDialog from "./HabitDialog.svelte";
import HabitsView from "./HabitsView.svelte";

const NOW = new Date("2026-03-10T10:00:00");

function mountView(seed: (api: FakeHabitsApi) => void = () => undefined) {
  const api = new FakeHabitsApi();
  seed(api);
  const store = new HabitsStore({ api });
  const view = new ViewStore({
    api: { settingsGet: async () => null, settingsSet: async () => undefined },
    now: () => NOW,
  });
  const onOpenHabit = vi.fn();
  const onNewHabit = vi.fn();
  render(HabitsView, { store, view, onOpenHabit, onNewHabit });
  return { api, store, onOpenHabit, onNewHabit };
}

describe("HabitsView (F22-01)", () => {
  it("asks for the last seven days and shows the empty state", async () => {
    const { store, onNewHabit } = mountView();
    await waitFor(() => expect(store.from).toBe("2026-03-04"));
    expect(store.to).toBe("2026-03-10");
    expect(await screen.findByText("Aún no tienes hábitos")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Nuevo hábito" }));
    expect(onNewHabit).toHaveBeenCalled();
  });

  it("shows the streak and the seven days, and ticks a day", async () => {
    const { api } = mountView((fake) => {
      fake.makeHabit({
        id: "h1",
        title: "Leer",
        days: 0b0011111,
        currentStreak: 3,
        bestStreak: 9,
      });
    });
    expect(await screen.findByText("Racha de 3 días · Mejor: 9")).toBeTruthy();
    expect(screen.getByText("lun, mar, mié, jue, vie")).toBeTruthy();
    const boxes = screen.getAllByRole("checkbox");
    expect(boxes).toHaveLength(7);
    // El último es hoy.
    await fireEvent.click(boxes[6]);
    await waitFor(() =>
      expect(api.logs).toEqual([{ habitId: "h1", date: "2026-03-10" }]),
    );
    expect(boxes[6].getAttribute("aria-checked")).toBe("true");
  });

  it("a day that was marked can be unmarked", async () => {
    const { api } = mountView((fake) => {
      const habit = fake.makeHabit({ id: "h1", title: "Leer" });
      fake.logs.push({ habitId: habit.id, date: "2026-03-09" });
    });
    const boxes = await screen.findAllByRole("checkbox");
    await waitFor(() =>
      expect(boxes[5].getAttribute("aria-checked")).toBe("true"),
    );
    await fireEvent.click(boxes[5]);
    await waitFor(() => expect(api.logs).toEqual([]));
  });

  it("opens a habit and lists the archived ones apart", async () => {
    const { onOpenHabit } = mountView((fake) => {
      fake.makeHabit({ title: "Leer" });
      fake.makeHabit({ title: "Correr", isArchived: true });
    });
    expect(await screen.findByText("Archivados")).toBeTruthy();
    await fireEvent.click(screen.getByText("Leer"));
    expect(onOpenHabit).toHaveBeenCalledWith(
      expect.objectContaining({ title: "Leer" }),
    );
  });
});

describe("HabitDialog (F22-01)", () => {
  function mountDialog(seed?: (api: FakeHabitsApi) => void, edit = false) {
    const api = new FakeHabitsApi();
    seed?.(api);
    const store = new HabitsStore({ api, noticeMs: 60_000 });
    const onClose = vi.fn();
    render(HabitDialog, {
      store,
      habit: edit ? api.habits[0] : undefined,
      onClose,
    });
    return { api, store, onClose };
  }

  it("needs a name and at least one day", async () => {
    const { api, onClose } = mountDialog();
    await fireEvent.click(screen.getByRole("button", { name: "Crear" }));
    expect(
      await screen.findByText("Escribe un nombre para el hábito."),
    ).toBeTruthy();
    await fireEvent.input(screen.getByLabelText("Nombre"), {
      target: { value: "Leer" },
    });
    for (const name of [
      "lunes",
      "martes",
      "miércoles",
      "jueves",
      "viernes",
      "sábado",
      "domingo",
    ]) {
      await fireEvent.click(screen.getByRole("button", { name }));
    }
    await fireEvent.click(screen.getByRole("button", { name: "Crear" }));
    expect(await screen.findByText("Elige al menos un día.")).toBeTruthy();
    expect(api.habits).toHaveLength(0);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("creates a habit for the chosen days", async () => {
    const { api, onClose } = mountDialog();
    await fireEvent.input(screen.getByLabelText("Nombre"), {
      target: { value: "Correr" },
    });
    for (const name of ["sábado", "domingo"]) {
      await fireEvent.click(screen.getByRole("button", { name }));
    }
    await fireEvent.click(screen.getByRole("button", { name: "Crear" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.habits[0]).toMatchObject({ title: "Correr", days: 0b0011111 });
  });

  it("edits, archives and deletes an existing habit", async () => {
    const { api, store, onClose } = mountDialog(
      (fake) => fake.makeHabit({ title: "Leer" }),
      true,
    );
    const name = screen.getByLabelText("Nombre") as HTMLInputElement;
    expect(name.value).toBe("Leer");
    await fireEvent.input(name, { target: { value: "Leer 20 min" } });
    await fireEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
    expect(api.habits[0].title).toBe("Leer 20 min");

    await fireEvent.click(screen.getByRole("button", { name: "Archivar" }));
    await waitFor(() => expect(api.habits[0].isArchived).toBe(true));

    await fireEvent.click(screen.getByRole("button", { name: "Eliminar" }));
    await waitFor(() => expect(api.habits).toHaveLength(0));
    expect(store.deleted?.title).toBe("Leer 20 min");
  });
});
