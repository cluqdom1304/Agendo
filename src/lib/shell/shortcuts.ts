// Dispatch of the global keyboard shortcuts of the main page.

import type { PaletteMode } from "../features/palette/palette";
import { findShortcut, focusContextOf } from "../keymap";
import { isAgenda, type Section } from "../stores/view.svelte";
import type { ViewMode } from "../types";

const SHORTCUT_VIEWS: Record<string, ViewMode> = {
  viewList: "list",
  viewDay: "day",
  viewWeek: "week",
  viewMonth: "month",
};

const SHORTCUT_SECTIONS: Record<string, Section> = {
  goAgenda: "tasks",
  goNotes: "notes",
  goToday: "today",
};

/** What the shortcuts act on; the page passes its stores and dialogs. */
export interface ShortcutDeps {
  section: () => Section;
  mode: () => ViewMode;
  setSection(section: Section): void;
  setMode(mode: ViewMode): void;
  openPalette(mode: PaletteMode): void;
  /** The day a new task starts on, `null` for none. */
  newTaskDate(): string | null;
  openNewTask(date: string | null): void;
  /** A dialog already on top (settings, a task) owns the keyboard. */
  modalOpen(): boolean;
}

export function dispatchShortcut(
  event: KeyboardEvent,
  deps: ShortcutDeps,
): void {
  const shortcut = findShortcut(event, focusContextOf(event.target));
  if (!shortcut) return;
  if (shortcut === "openSearch" || shortcut === "quickOpen") {
    event.preventDefault();
    deps.openPalette(shortcut === "openSearch" ? "all" : "notes");
    return;
  }
  const section = SHORTCUT_SECTIONS[shortcut];
  if (section) {
    event.preventDefault();
    deps.setSection(section);
    return;
  }
  if (shortcut === "newTask" && isAgenda(deps.section())) {
    if (deps.modalOpen()) return;
    event.preventDefault();
    deps.openNewTask(deps.mode() === "list" ? deps.newTaskDate() : null);
    return;
  }
  const mode = SHORTCUT_VIEWS[shortcut];
  if (!mode || !isAgenda(deps.section())) return;
  event.preventDefault();
  deps.setMode(mode);
}
