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
