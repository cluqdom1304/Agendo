// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";

import type { Section } from "../stores/view.svelte";
import type { ViewMode } from "../types";
import { dispatchShortcut, type ShortcutDeps } from "./shortcuts";

function setup(section: Section = "tasks", mode: ViewMode = "list") {
  const deps = {
    section: () => section,
    mode: () => mode,
    setSection: vi.fn(),
    setMode: vi.fn(),
    openPalette: vi.fn(),
    newTaskDate: () => "2026-05-01",
    openNewTask: vi.fn(),
    modalOpen: () => false,
  } satisfies ShortcutDeps;
  return deps;
}

function press(key: string, deps: ShortcutDeps, init: KeyboardEventInit = {}) {
  const event = new KeyboardEvent("keydown", {
    key,
    ctrlKey: true,
    cancelable: true,
    ...init,
  });
  dispatchShortcut(event, deps);
  return event;
}

describe("dispatchShortcut", () => {
  it("opens the search and the quick switcher", () => {
    const deps = setup();
    expect(press("k", deps).defaultPrevented).toBe(true);
    press("p", deps);
    expect(deps.openPalette.mock.calls).toEqual([["all"], ["notes"]]);
  });

  it("starts a new task on the date of the selection in the list", () => {
    const deps = setup();
    press("n", deps);
    expect(deps.openNewTask).toHaveBeenCalledWith("2026-05-01");
  });

  it("starts a new task with no date outside the list", () => {
    const deps = setup("calendar", "week");
    press("n", deps);
    expect(deps.openNewTask).toHaveBeenCalledWith(null);
  });

  it("leaves the keyboard to a dialog that is already open", () => {
    const deps = { ...setup(), modalOpen: () => true };
    const event = press("n", deps);
    expect(event.defaultPrevented).toBe(false);
    expect(deps.openNewTask).not.toHaveBeenCalled();
  });

  it("ignores new task outside the agenda", () => {
    const deps = setup("notes");
    press("n", deps);
    expect(deps.openNewTask).not.toHaveBeenCalled();
  });

  it("ignores keys that are not shortcuts", () => {
    const deps = setup();
    const event = press("q", deps);
    expect(event.defaultPrevented).toBe(false);
  });
});
