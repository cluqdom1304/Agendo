// Turning a task into an event and back (F21-04, D-94). It is an explicit operation: the
// new item is created first and the old one only goes to the trash once that worked, so a
// failure never loses anything.
import { t } from "../../i18n";
import { durationBetween, endTimeOf } from "../../schedule";
import type { EventsStore } from "../../stores/events.svelte";
import type { TasksStore } from "../../stores/tasks.svelte";
import type { CalendarEvent, ReminderInput, Task } from "../../types";
import { textToDoc } from "../notes/content";

interface DocNode {
  type?: string;
  text?: string;
  content?: DocNode[];
}

/** The text of a notes document, one line per text block; plain text is kept as it is. */
export function docToText(contentJson: string): string {
  if (!contentJson.trim().startsWith("{")) return contentJson.trim();
  let root: DocNode;
  try {
    root = JSON.parse(contentJson) as DocNode;
  } catch {
    return "";
  }
  const inline = (node: DocNode): string =>
    typeof node.text === "string"
      ? node.text
      : (node.content ?? []).map(inline).join("");
  const lines: string[] = [];
  const visit = (node: DocNode) => {
    const children = node.content ?? [];
    // A block whose children are all text (or breaks) is one line; anything else holds blocks.
    if (children.every((child) => child.content === undefined)) {
      lines.push(inline(node));
    } else {
      children.forEach(visit);
    }
  };
  visit(root);
  return lines.filter((line) => line.trim() !== "").join("\n");
}

/** Reminders "N minutes before" are the ones an event can keep; fixed instants are not. */
function offsetsOf(task: Task): number[] {
  return task.reminders.flatMap((reminder) =>
    !reminder.isSent && reminder.offsetMinutes !== null
      ? [reminder.offsetMinutes]
      : [],
  );
}

export async function taskToEvent(
  task: Task,
  events: EventsStore,
  tasks: TasksStore,
  today: string,
): Promise<CalendarEvent | null> {
  const notes = docToText(task.contentJson);
  const created = await events.create({
    title: task.title,
    ...(notes ? { notes } : {}),
    startDate: task.dueDate ?? today,
    ...(task.dueTime
      ? {
          startTime: task.dueTime,
          ...(task.durationMinutes
            ? { endTime: endTimeOf(task.dueTime, task.durationMinutes) }
            : {}),
        }
      : {}),
    ...(task.rrule && task.dueDate ? { rrule: task.rrule } : {}),
    ...(task.tagIds.length > 0 ? { tagIds: task.tagIds } : {}),
    ...(offsetsOf(task).length > 0 ? { reminderOffsets: offsetsOf(task) } : {}),
  });
  if (!created) return null;
  await tasks.remove(task.id);
  events.inform(t.events.convertedToEvent);
  return created;
}

export async function eventToTask(
  event: CalendarEvent,
  events: EventsStore,
  tasks: TasksStore,
): Promise<Task | null> {
  const timed = event.startTime !== null && !event.allDay;
  const duration =
    timed &&
    event.endTime !== null &&
    event.endDate === event.startDate &&
    event.startTime !== null
      ? durationBetween(event.startTime, event.endTime)
      : null;
  const reminders: ReminderInput[] = event.reminderOffsets.map((minutes) => ({
    offsetMinutes: minutes,
  }));
  const contentJson = textToDoc(event.notes);
  const created = await tasks.create({
    title: event.title,
    dueDate: event.startDate,
    ...(timed && event.startTime ? { dueTime: event.startTime } : {}),
    ...(duration ? { durationMinutes: duration } : {}),
    ...(event.tagIds.length > 0 ? { tagIds: event.tagIds } : {}),
    ...(contentJson ? { contentJson } : {}),
    ...(event.rrule ? { rrule: event.rrule } : {}),
    reminders,
  });
  if (!created) return null;
  await events.remove(event, false);
  events.inform(t.events.convertedToTask);
  return created;
}
