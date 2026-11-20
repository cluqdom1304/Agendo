// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

import { FocusStore } from "../../stores/focus.svelte";
import type { FocusApi } from "../../stores/focus.svelte";
import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, installLayoutStubs, makeTask } from "../../testing";
import FocusSwitchDialog from "./FocusSwitchDialog.svelte";

installLayoutStubs();

const running = {
  id: "e1",
  taskId: "a",
  startedAt: "2026-11-20T10:00:00Z",
  endedAt: null,
  plannedMinutes: 25,
  pausedSeconds: 0,
  pausedAt: null,
  spentSeconds: 0,
};

async function mount() {
  const api: FocusApi = {
    focusCurrent: vi.fn(async () => running),
    focusStart: vi.fn(),
    focusStop: vi.fn(),
    focusPause: vi.fn(),
    focusResume: vi.fn(),
    focusDiscard: vi.fn(),
    focusSwitch: vi.fn(async (taskId: string) => ({ ...running, taskId })),
    focusTotals: vi.fn(async () => []),
    focusTaskEntries: vi.fn(async () => []),
    focusAddManual: vi.fn(),
    focusDeleteEntry: vi.fn(),
  };
  const focus = new FocusStore({ api, tickMs: 0 });
  await focus.load();
  const store = new TasksStore({
    api: new FakeApi([
      makeTask({ id: "a", title: "Actual" }),
      makeTask({ id: "b", title: "Informe final" }),
      makeTask({ id: "c", title: "Hecha", isCompleted: true }),
    ]),
    settleMs: () => 0,
  });
  await store.load();
  const onClose = vi.fn();
  render(FocusSwitchDialog, { focus, store, onClose });
  return { api, onClose };
}

describe("FocusSwitchDialog (F25-05)", () => {
  it("lists open tasks, marks the current one and filters by text", async () => {
    await mount();
    expect(
      screen.getByRole("button", { name: /Actual/ }).hasAttribute("disabled"),
    ).toBe(true);
    expect(screen.queryByText("Hecha")).toBeNull();
    await fireEvent.input(screen.getByLabelText("Buscar una tarea"), {
      target: { value: "final" },
    });
    expect(screen.queryByText("Actual")).toBeNull();
    expect(screen.getByRole("button", { name: /Informe final/ })).toBeTruthy();
  });

  it("moves the session to the chosen task and closes", async () => {
    const { api, onClose } = await mount();
    await fireEvent.click(
      screen.getByRole("button", { name: /Informe final/ }),
    );
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.focusSwitch).toHaveBeenCalledWith("b");
  });
});
