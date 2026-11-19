// Which dialog of the main page is open. The page and its shortcuts open them; `DialogHost`
// shows them.

import type { PaletteMode } from "../features/palette/palette";
import type { Task } from "../types";

export class DialogState {
  settingsOpen = $state(false);
  newTaskOpen = $state(false);
  /** The day the creation dialog starts on, when it was opened from a day of the calendar. */
  newTaskDate = $state<string | null>(null);
  /** The task whose view is open (opened from the calendar). */
  detailId = $state<string | null>(null);
  /** The command palette (Ctrl+K) or "Ir a…" (Ctrl+P) while it is open. */
  palette = $state<PaletteMode | null>(null);
  /** The task being edited in the creation dialog; the view comes back when it closes. */
  editing = $state.raw<Task | null>(null);

  openNewTask(date: string | null = null): void {
    this.newTaskDate = date;
    this.newTaskOpen = true;
  }
}
