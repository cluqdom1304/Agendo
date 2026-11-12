// Connects the task references of a note to the real tasks and tags.

import { formatDisplayDate } from "../../dates";
import type { TagsStore } from "../../stores/tags.svelte";
import type { TasksStore } from "../../stores/tasks.svelte";
import type { TaskRefHost } from "./taskRef";

export interface NoteTaskHost extends TaskRefHost {
  /** Reads the state the references depend on, so an `$effect` redraws them on change. */
  track(): void;
}

export function createTaskHost(
  tasks: TasksStore,
  tags: TagsStore,
  open: (taskId: string) => void,
): NoteTaskHost {
  return {
    lookup(id) {
      const task = tasks.find(id);
      if (!task) return null;
      return {
        title: task.title,
        isCompleted: task.isCompleted,
        due: task.dueDate
          ? task.dueTime
            ? `${formatDisplayDate(task.dueDate)} · ${task.dueTime}`
            : formatDisplayDate(task.dueDate)
          : null,
        tags: tags
          .tagsOf(task)
          .map((tag) => ({ name: tag.name, color: tag.colorHex })),
      };
    },
    toggle(id, completed) {
      void tasks.setCompleted(id, completed);
    },
    open,
    restore: (id) => tasks.restoreById(id),
    track() {
      void tasks.tasks;
      void tags.tags;
    },
  };
}
