// Context menu entries for tasks and the agenda background.

import { formatDisplayDate } from "../../dates";
import { addDays } from "../calendar/grid";
import { t } from "../../i18n";
import type { ContextEntry } from "./context.svelte";
import type { ContextDeps } from "./deps";
import { tagChoices } from "./tagChoices";
import type { Task, ViewMode } from "../../types";

const VIEW_MODES: { mode: ViewMode; label: string }[] = [
  { mode: "list", label: t.views.list },
  { mode: "day", label: t.views.day },
  { mode: "week", label: t.views.week },
  { mode: "month", label: t.views.month },
];

export function taskEntries(taskId: string, deps: ContextDeps): ContextEntry[] {
  const task = deps.tasks.find(taskId);
  if (!task) return [];
  const { tasks, view } = deps;
  const today = view.today();
  const tomorrow = addDays(today, 1);

  const dates: ContextEntry[] = [
    {
      id: "today",
      label: t.context.today,
      checked: task.dueDate === today,
      run: () => void tasks.setDueDate(task.id, today),
    },
    {
      id: "tomorrow",
      label: t.context.tomorrow,
      checked: task.dueDate === tomorrow,
      run: () => void tasks.setDueDate(task.id, tomorrow),
    },
    {
      id: "pick",
      label: t.context.pickDate,
      run: () => deps.editTask(task),
    },
  ];
  if (task.dueDate) {
    dates.push({
      id: "clear-date",
      label: t.context.clearDate,
      separated: true,
      run: () => void tasks.setDueDate(task.id, null),
    });
  }

  const entries: ContextEntry[] = [
    {
      id: "toggle",
      label: task.isCompleted ? t.context.reopen : t.context.done,
      run: () => void tasks.setCompleted(task.id, !task.isCompleted),
    },
    { id: "open", label: t.context.open, run: () => deps.openTask(task.id) },
    { id: "edit", label: t.context.edit, run: () => deps.editTask(task) },
    {
      id: "date",
      label: task.dueDate
        ? `${t.context.date} · ${formatDisplayDate(task.dueDate)}`
        : t.context.date,
      separated: true,
      children: dates,
    },
    {
      id: "priority",
      label: t.priority.label,
      children: t.priority.levels.map((name, level) => ({
        id: `priority:${level}`,
        label: name,
        checked: task.priority === level,
        run: () => void tasks.setPriority(task.id, level),
      })),
    },
  ];
  if (!task.isCompleted) {
    entries.push({
      id: "reminder",
      label: t.context.reminder,
      // The reminder panel lives under the row of the list; elsewhere the edit dialog has them.
      run: () =>
        view.mode === "list"
          ? tasks.toggleReminders(task.id)
          : deps.editTask(task),
    });
  }
  entries.push(
    {
      id: "tags",
      label: t.context.tags,
      children: tagChoices(
        deps,
        () => deps.tasks.find(task.id)?.tagIds ?? [],
        (next) => void tasks.setTags(task.id, next),
      ),
    },
    {
      id: "duplicate",
      label: t.context.duplicate,
      run: () => void duplicate(task, deps),
    },
  );
  if (task.sourceNoteId) {
    const noteId = task.sourceNoteId;
    entries.push({
      id: "source",
      label: t.context.openNote,
      separated: true,
      run: () => void deps.showNote(noteId),
    });
  } else {
    entries.push({
      id: "create-note",
      label: t.context.createNote,
      separated: true,
      run: () => void deps.noteForTask(task),
    });
  }
  entries.push(
    {
      id: "convert",
      label: t.context.convertNote,
      run: () => void deps.taskToNote(task),
    },
    {
      id: "convert-event",
      label: t.context.convertEvent,
      run: () => void deps.taskToEvent(task),
    },
    {
      id: "delete",
      label: t.context.delete,
      danger: true,
      separated: true,
      run: () => void tasks.remove(task.id),
    },
  );
  return entries;
}

export function eventEntries(
  eventId: string,
  deps: ContextDeps,
): ContextEntry[] {
  return [
    {
      id: "edit-event",
      label: t.context.openEventTask,
      run: () => deps.openEvent(eventId),
    },
    {
      id: "event-to-task",
      label: t.context.convertTask,
      run: () => void deps.eventToTask(eventId),
    },
    {
      id: "delete-event",
      label: t.context.delete,
      danger: true,
      separated: true,
      run: () => void deps.deleteEvent(eventId),
    },
  ];
}

export async function duplicate(task: Task, deps: ContextDeps): Promise<void> {
  await deps.tasks.create({
    title: task.title,
    ...(task.dueDate ? { dueDate: task.dueDate } : {}),
    ...(task.dueTime ? { dueTime: task.dueTime } : {}),
    ...(task.durationMinutes !== null
      ? { durationMinutes: task.durationMinutes }
      : {}),
    ...(task.tagIds.length > 0 ? { tagIds: task.tagIds } : {}),
    ...(task.priority > 0 ? { priority: task.priority } : {}),
    ...(task.contentJson.trim().startsWith("{")
      ? { contentJson: task.contentJson }
      : {}),
  });
}

export function agendaEntries(
  date: string | null,
  deps: ContextDeps,
): ContextEntry[] {
  const { view } = deps;
  const entries: ContextEntry[] = [
    {
      id: "new-task",
      label: date
        ? t.context.newTaskOn(formatDisplayDate(date))
        : t.context.newTask,
      run: () => deps.newTask(date),
    },
    {
      id: "new-event",
      label: date
        ? t.context.newEventOn(formatDisplayDate(date))
        : t.context.newEvent,
      run: () => deps.newEvent(date),
    },
  ];
  if (date) {
    entries.push({
      id: "go-day",
      label: t.context.goToDay,
      run: () => view.showDay(date),
    });
  }
  entries.push(
    {
      id: "today",
      label: t.context.goToToday,
      run: () => view.goToToday(),
    },
    {
      id: "view",
      label: t.context.view,
      separated: true,
      children: VIEW_MODES.map(({ mode, label }) => ({
        id: `view:${mode}`,
        label,
        checked: view.mode === mode,
        run: () => view.setMode(mode),
      })),
    },
  );
  return entries;
}
