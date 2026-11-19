// Which tasks show their notes, and saving them. Composed into `TasksStore`.

import { SvelteSet } from "svelte/reactivity";

import { hasNotes } from "../features/notes/content";
import type { TasksHost } from "./tasksHost";

/** Where earlier versions remembered hidden notes; cleared so they cannot stay hidden. */
const LEGACY_COLLAPSED_KEY = "agenda.notesCollapsed";

function forgetStoredCollapse(): void {
  try {
    localStorage.removeItem(LEGACY_COLLAPSED_KEY);
  } catch {
    // Storage may be unavailable; there is nothing to forget then.
  }
}

export class TaskNotes {
  /** The task whose notes the user has just opened: its editor takes the focus once. */
  focusId = $state<string | null>(null);

  /** Tasks whose empty notes the user opened to write; tasks with text are open unless collapsed. */
  readonly #opened = new SvelteSet<string>();
  /** Tasks with text whose notes the user hid; every session starts with all of them shown. */
  readonly #collapsed = new SvelteSet<string>();
  /** Notes hidden for an instant so the editor flushes before its row remounts. */
  readonly #suspended = new SvelteSet<string>();
  #saving = 0;
  readonly #host: TasksHost;

  constructor(host: TasksHost) {
    this.#host = host;
    forgetStoredCollapse();
  }

  /** Notes with text show by default (also after a restart); the user can collapse them. */
  isOpen(id: string): boolean {
    if (this.#suspended.has(id)) return false;
    const task = this.#host.find(id);
    if (!task) return false;
    return (
      this.#opened.has(id) ||
      (hasNotes(task.contentJson) && !this.#collapsed.has(id))
    );
  }

  toggle(id: string): void {
    if (this.isOpen(id)) {
      this.#opened.delete(id);
      this.#collapsed.add(id);
      this.focusId = null;
    } else {
      this.#opened.add(id);
      this.#collapsed.delete(id);
      // Only a note the user has just opened takes the cursor; the ones shown at startup
      // must not.
      this.focusId = id;
    }
  }

  /** Puts away notes that were opened to write and left empty. */
  closeEmpty(id: string): void {
    const task = this.#host.find(id);
    if (task && hasNotes(task.contentJson)) return;
    this.#opened.delete(id);
  }

  /** Shows the notes of a task again when it has text (a search result was opened). */
  expand(id: string): void {
    const task = this.#host.find(id);
    if (task && hasNotes(task.contentJson)) this.#collapsed.delete(id);
  }

  suspend(id: string): void {
    if (!this.isOpen(id)) return;
    this.#suspended.add(id);
    setTimeout(() => this.#resume(), 0);
  }

  #resume(): void {
    if (this.#saving === 0) this.#suspended.clear();
  }

  /** Saves the notes document; throws after reporting, so the editor keeps its pending text. */
  async save(id: string, contentJson: string): Promise<void> {
    this.#saving++;
    try {
      this.#host.replace(await this.#host.api.taskUpdate(id, { contentJson }));
    } catch (error) {
      this.#host.report(error);
      throw error;
    } finally {
      this.#saving--;
      this.#resume();
    }
  }
}
