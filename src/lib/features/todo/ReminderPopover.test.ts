// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import type { Reminder } from "../../types";
import ReminderPopover from "./ReminderPopover.svelte";

const NOW = new Date(2026, 2, 10, 10, 0);

function mount(overrides: {
  dueDate?: string | null;
  dueTime?: string | null;
  reminders?: Reminder[];
}) {
  const handlers = {
    onSetTime: vi.fn(),
    onChange: vi.fn(),
    onClose: vi.fn(),
  };
  render(ReminderPopover, {
    title: "Clase",
    dueDate: "2026-03-11",
    dueTime: "09:00",
    reminders: [],
    now: () => NOW,
    ...overrides,
    ...handlers,
  });
  return handlers;
}

describe("ReminderPopover", () => {
  it("offers the four options of F-12 and adds one on click", async () => {
    const { onChange } = mount({});
    for (const name of [
      "En el momento",
      "15 min antes",
      "1 hora antes",
      "1 día antes",
    ]) {
      expect(screen.getByRole("button", { name })).toBeTruthy();
    }
    await fireEvent.click(screen.getByRole("button", { name: "15 min antes" }));
    expect(onChange).toHaveBeenCalledWith([{ offsetMinutes: 15 }]);
  });

  it("marks active options and removes them on click", async () => {
    const { onChange } = mount({
      reminders: [
        {
          id: "r",
          notifyAt: "2026-03-11T07:45:00Z",
          offsetMinutes: 15,
          isSent: false,
          repeat: null,
        },
      ],
    });
    const chip = screen.getByRole("button", { name: "15 min antes" });
    expect(chip.getAttribute("aria-pressed")).toBe("true");
    await fireEvent.click(chip);
    expect(onChange).toHaveBeenCalledWith([]);
  });

  it("disables relative options without a time and explains why", () => {
    mount({ dueTime: null });
    const chip = screen.getByRole("button", { name: "En el momento" });
    expect((chip as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(/Pon una hora a la tarea/)).toBeTruthy();
  });

  it("disables options whose moment has passed", () => {
    mount({ dueDate: "2026-03-10", dueTime: "10:10" });
    expect(
      (
        screen.getByRole("button", {
          name: "En el momento",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false);
    expect(
      (
        screen.getByRole("button", {
          name: "15 min antes",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
  });

  it("sets the time of the task", async () => {
    const { onSetTime } = mount({ dueTime: null });
    const input =
      document.querySelector<HTMLInputElement>('input[type="time"]')!;
    await fireEvent.change(input, { target: { value: "18:30" } });
    expect(onSetTime).toHaveBeenCalledWith("18:30");
  });

  it("adds a custom reminder only while it is in the future", async () => {
    const { onChange } = mount({ dueDate: null, dueTime: null });
    const add = screen.getByRole("button", { name: "Añadir aviso" });
    const input = document.querySelector<HTMLInputElement>(
      'input[type="datetime-local"]',
    )!;
    expect((add as HTMLButtonElement).disabled).toBe(true);

    await fireEvent.input(input, { target: { value: "2026-03-10T09:00" } });
    expect((add as HTMLButtonElement).disabled).toBe(true);

    await fireEvent.input(input, { target: { value: "2026-03-12T09:30" } });
    expect((add as HTMLButtonElement).disabled).toBe(false);
    await fireEvent.click(add);
    expect(onChange).toHaveBeenCalledWith([
      {
        notifyAt: new Date(2026, 2, 12, 9, 30)
          .toISOString()
          .replace(/\.\d{3}Z$/, "Z"),
      },
    ]);
  });

  it("closes with Escape", async () => {
    const { onClose } = mount({});
    await fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("closes when clicking outside, but not inside", async () => {
    const { onClose } = mount({});
    await fireEvent.pointerDown(screen.getByRole("dialog"));
    expect(onClose).not.toHaveBeenCalled();
    await fireEvent.pointerDown(document.body);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("ignores the bell that toggles it", async () => {
    const { onClose } = mount({});
    const bell = document.createElement("button");
    bell.setAttribute("data-reminders-toggle", "");
    document.body.append(bell);
    await fireEvent.pointerDown(bell);
    expect(onClose).not.toHaveBeenCalled();
    bell.remove();
  });
});

describe("sounded reminders (history)", () => {
  const sent = (id: string, notifyAt: string): Reminder => ({
    id,
    notifyAt,
    offsetMinutes: null,
    isSent: true,
    repeat: null,
  });

  it("shows them greyed out under their own heading, without a remove button", () => {
    mount({
      reminders: [
        sent("s1", "2026-03-09T08:00:00Z"),
        {
          id: "p1",
          notifyAt: "2026-03-11T07:45:00Z",
          offsetMinutes: 15,
          isSent: false,
          repeat: null,
        },
      ],
    });
    const history = screen.getByRole("list", { name: "Ya avisados" });
    const rows = history.querySelectorAll("[data-sent]");
    expect(rows).toHaveLength(1);
    expect(rows[0].textContent).toContain("Avisado");
    expect(history.querySelector("button")).toBeNull();
    expect(
      screen.getAllByRole("button", { name: /^Quitar el aviso/ }),
    ).toHaveLength(1);
  });

  it("keeps history out of what is sent back when something changes", async () => {
    const { onChange } = mount({
      reminders: [sent("s1", "2026-03-09T08:00:00Z")],
    });
    await fireEvent.click(screen.getByRole("button", { name: "15 min antes" }));
    expect(onChange).toHaveBeenCalledWith([{ offsetMinutes: 15 }]);
  });

  it("summarises a long history instead of listing all of it", () => {
    mount({
      reminders: Array.from({ length: 8 }, (_, i) =>
        sent(`s${i}`, `2026-03-0${i + 1}T08:00:00Z`),
      ),
    });
    expect(document.querySelectorAll("[data-sent]")).toHaveLength(5);
    expect(screen.getByText("3 avisos anteriores")).toBeTruthy();
  });
});

describe("repeating reminders", () => {
  async function fillCustom() {
    await fireEvent.input(screen.getByLabelText("Aviso a una hora concreta"), {
      target: { value: "2026-03-12T08:00" },
    });
  }

  it("adds a daily repeat that never ends", async () => {
    const { onChange } = mount({});
    await fillCustom();
    await fireEvent.change(screen.getByLabelText("Repetir"), {
      target: { value: "daily" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Añadir aviso" }));

    const [inputs] = onChange.mock.calls[0];
    expect(inputs).toHaveLength(1);
    expect(inputs[0].repeat).toEqual({
      every: 1,
      unit: "day",
      count: null,
      until: null,
    });
  });

  it("adds a custom interval that ends after a number of times", async () => {
    const { onChange } = mount({});
    await fillCustom();
    await fireEvent.change(screen.getByLabelText("Repetir"), {
      target: { value: "custom" },
    });
    await fireEvent.input(screen.getByLabelText("Cada"), {
      target: { value: "2" },
    });
    await fireEvent.change(screen.getByLabelText("Unidad de la repetición"), {
      target: { value: "week" },
    });
    await fireEvent.change(screen.getByLabelText("Termina"), {
      target: { value: "count" },
    });
    await fireEvent.input(screen.getByLabelText("Número de veces"), {
      target: { value: "4" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Añadir aviso" }));

    expect(onChange.mock.calls[0][0][0].repeat).toEqual({
      every: 2,
      unit: "week",
      count: 4,
      until: null,
    });
  });

  it("adds a repeat that ends on a date, and refuses a date before the first aviso", async () => {
    const { onChange } = mount({});
    await fillCustom();
    await fireEvent.change(screen.getByLabelText("Repetir"), {
      target: { value: "weekly" },
    });
    await fireEvent.change(screen.getByLabelText("Termina"), {
      target: { value: "until" },
    });
    const add = screen.getByRole("button", { name: "Añadir aviso" });

    await fireEvent.input(screen.getByLabelText("Fecha final"), {
      target: { value: "2026-03-01" },
    });
    expect((add as HTMLButtonElement).disabled).toBe(true);

    await fireEvent.input(screen.getByLabelText("Fecha final"), {
      target: { value: "2026-04-30" },
    });
    await fireEvent.click(add);
    expect(onChange.mock.calls[0][0][0].repeat).toMatchObject({
      unit: "week",
      until: "2026-04-30",
      count: null,
    });
  });

  it("does not allow fewer than two times", async () => {
    mount({});
    await fillCustom();
    await fireEvent.change(screen.getByLabelText("Repetir"), {
      target: { value: "daily" },
    });
    await fireEvent.change(screen.getByLabelText("Termina"), {
      target: { value: "count" },
    });
    await fireEvent.input(screen.getByLabelText("Número de veces"), {
      target: { value: "1" },
    });
    expect(
      (
        screen.getByRole("button", {
          name: "Añadir aviso",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
  });

  it("describes a repeating reminder in the list and keeps its rule when others change", async () => {
    const { onChange } = mount({
      reminders: [
        {
          id: "r",
          notifyAt: "2026-03-12T08:00:00Z",
          offsetMinutes: null,
          isSent: false,
          repeat: {
            every: 2,
            unit: "week",
            count: 5,
            until: null,
            occurrence: 2,
          },
        },
      ],
    });
    expect(screen.getByText(/Cada 2 semanas, 5 veces \(2 de 5\)/)).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "15 min antes" }));
    expect(onChange.mock.calls[0][0]).toContainEqual({
      notifyAt: "2026-03-12T08:00:00Z",
      repeat: {
        every: 2,
        unit: "week",
        count: 5,
        until: null,
        occurrence: 2,
      },
    });
  });
});
