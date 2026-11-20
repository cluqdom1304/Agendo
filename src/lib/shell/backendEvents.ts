// Events the backend sends to the main page: reminders that fired or were clicked, and task
// changes made outside the window.

import { onReminderFired, onReminderOpen, onTasksChanged } from "../api";

export interface BackendEventHandlers {
  taskReminderFired(taskId: string): Promise<void> | void;
  noteReminderFired(noteId: string): Promise<void> | void;
  tasksChanged(): void;
  openTask(taskId: string): void;
  openNote(noteId: string): void;
  openEvent(eventId: string): void;
}

/** Registers the listeners; the returned function releases them. */
export function listenBackendEvents(
  handlers: BackendEventHandlers,
): () => void {
  // A listener that cannot be registered (no backend) must not surface as an unhandled
  // rejection; one registered after the page left is still released by the cleanup.
  const stops = [
    onReminderFired((fired) => {
      if (fired.taskId) return handlers.taskReminderFired(fired.taskId);
      if (fired.noteId) return handlers.noteReminderFired(fired.noteId);
    }),
    onTasksChanged(() => handlers.tasksChanged()),
    onReminderOpen((target) => {
      if (target.taskId) handlers.openTask(target.taskId);
      else if (target.noteId) handlers.openNote(target.noteId);
      else if (target.eventId) handlers.openEvent(target.eventId);
    }),
  ].map((listener) => listener.catch(() => undefined));
  return () => {
    for (const stop of stops) void stop.then((release) => release?.());
  };
}
