// Which dialog of the main page is open. The page and its shortcuts open them; `DialogHost`
// shows them.

import type { PaletteMode } from "../features/palette/palette";
import type { Habit, Task } from "../types";

export class DialogState {
  settingsOpen = $state(false);
  /** The task picker of the running focus session (F25-05). */
  focusSwitchOpen = $state(false);
  newTaskOpen = $state(false);
  /** The day the creation dialog starts on, when it was opened from a day of the calendar. */
  newTaskDate = $state<string | null>(null);
  /** The task whose view is open (opened from the calendar). */
  detailId = $state<string | null>(null);
  /** The command palette (Ctrl+K) or "Ir a…" (Ctrl+P) while it is open. */
  palette = $state<PaletteMode | null>(null);
  /** The task being edited in the creation dialog; the view comes back when it closes. */
  editing = $state.raw<Task | null>(null);

  /** The event dialog: a new event (on a day, at a time) or the event with `id`. */
  event = $state.raw<{
    id?: string;
    date: string | null;
    time: string | null;
  } | null>(null);

  openNewEvent(date: string | null = null, time: string | null = null): void {
    this.event = { date, time };
  }

  openEvent(id: string): void {
    this.event = { id, date: null, time: null };
  }

  /** The habit dialog: a new habit (`habit` absent) or the one being edited. */
  habit = $state.raw<{ habit?: Habit } | null>(null);

  openNewHabit(): void {
    this.habit = {};
  }

  openHabit(habit: Habit): void {
    this.habit = { habit };
  }

  /** The «Planificar mi día» assistant (F24-02). */
  planOpen = $state(false);

  openNewTask(date: string | null = null): void {
    this.newTaskDate = date;
    this.newTaskOpen = true;
  }
}
