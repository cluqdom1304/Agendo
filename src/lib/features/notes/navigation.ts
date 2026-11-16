// Moves between the agenda and the notes: tasks that point at a note, and notes made from
// tasks (F10-05, F10-07).

import { taskRefDoc } from "./content";
import { notesStore } from "../../stores/notes.svelte";
import { tasksStore } from "../../stores/tasks.svelte";
import { viewStore } from "../../stores/view.svelte";
import type { Note, Task } from "../../types";

/** Switches to the notes and opens the note. */
export async function showNote(id: string): Promise<void> {
  viewStore.setSection("notes");
  await notesStore.ensureLoaded();
  await notesStore.openNote(id);
}

/** Switches to the notes and opens the note of that day, creating it if needed (F13-01). */
export async function showDailyNote(date: string): Promise<void> {
  viewStore.setSection("notes");
  await notesStore.ensureLoaded();
  await notesStore.openDaily(date);
}

/** A note named after the task that holds a reference to it (D-45). */
export async function createNoteForTask(task: Task): Promise<Note | null> {
  viewStore.setSection("notes");
  await notesStore.ensureLoaded();
  return notesStore.createNoteWith({
    title: task.title,
    contentJson: taskRefDoc(task.id),
    ...(task.tagIds.length > 0 ? { tagIds: task.tagIds } : {}),
  });
}

/**
 * The task becomes a note with its description; the task goes to the trash, where the
 * undo notice of the agenda can still bring it back.
 */
export async function convertTaskToNote(task: Task): Promise<Note | null> {
  viewStore.setSection("notes");
  await notesStore.ensureLoaded();
  const note = await notesStore.createNoteWith({
    title: task.title,
    ...(task.contentJson.trim().startsWith("{")
      ? { contentJson: task.contentJson }
      : {}),
    ...(task.tagIds.length > 0 ? { tagIds: task.tagIds } : {}),
  });
  if (note) await tasksStore.remove(task.id);
  return note;
}
