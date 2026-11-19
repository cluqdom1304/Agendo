// What the entries of the context menu need from the application.

import type { NotesStore } from "../../stores/notes.svelte";
import type { TagsStore } from "../../stores/tags.svelte";
import type { TasksStore } from "../../stores/tasks.svelte";
import type { ViewStore } from "../../stores/view.svelte";
import type { ContextMenuStore } from "./context.svelte";
import type { Note, Task } from "../../types";

export interface ContextDeps {
  view: ViewStore;
  tasks: TasksStore;
  tags: TagsStore;
  notes: NotesStore;
  menu: Pick<ContextMenuStore, "request">;
  /** Opens the creation dialog, on a day if one is given. */
  newTask(date: string | null): void;
  editTask(task: Task): void;
  /** Shows the view of one task. */
  openTask(taskId: string): void;
  /** Switches to the notes and opens the note. */
  showNote(noteId: string): Promise<void>;
  noteForTask(task: Task): Promise<Note | null>;
  taskToNote(task: Task): Promise<Note | null>;
  /** Cut, copy and paste in the editor (the clipboard belongs to the browser). */
  clipboard: {
    cut(): void;
    copy(): void;
    paste(): void;
    selectAll(): void;
  };
}
