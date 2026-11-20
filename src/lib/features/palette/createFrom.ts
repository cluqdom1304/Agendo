// «Crear cualquier cosa» desde la paleta (F23-03): turns the typed text into a task, an event,
// a habit or a note with the same reader the creation fields use.

import { toLocalIsoDate } from "../../dates";
import { parseNatural, taskFieldsOf } from "../../naturalInput";
import { DAY_CODES } from "../../recurrence";
import { endTimeOf, maxDuration } from "../../schedule";
import type { EventsStore } from "../../stores/events.svelte";
import type { HabitsStore } from "../../stores/habits.svelte";
import type { NotesStore } from "../../stores/notes.svelte";
import type { TagsStore } from "../../stores/tags.svelte";
import type { TasksStore } from "../../stores/tasks.svelte";
import type { ViewStore } from "../../stores/view.svelte";
import { captureOptions } from "../capture/capture.svelte";
import { maskOf } from "../habits/habits";
import type { CreateKind } from "./palette";

export interface CreateDeps {
  view: ViewStore;
  tasks: TasksStore;
  events: EventsStore;
  habits: HabitsStore;
  notes: NotesStore;
  tags: TagsStore;
  now?: () => Date;
}

const DEFAULT_EVENT_MINUTES = 60;

export async function createFromText(
  deps: CreateDeps,
  what: CreateKind,
  rawText: string,
): Promise<void> {
  const text = rawText.trim();
  if (!text) return;
  const today = toLocalIsoDate((deps.now ?? (() => new Date()))());
  const capture =
    what === "note"
      ? null
      : parseNatural(text, {
          ...captureOptions(today),
          tagId: (name) => {
            const wanted = name.toLocaleLowerCase();
            return deps.tags.tags.find(
              (tag) => tag.name.toLocaleLowerCase() === wanted,
            )?.id;
          },
        });
  const title = capture?.title ?? text;

  if (what === "task") {
    deps.view.showAgenda();
    await deps.tasks.create({
      title,
      ...(capture ? taskFieldsOf(capture) : {}),
    });
  } else if (what === "event") {
    const start = capture?.time;
    const room = start ? maxDuration(start) : 0;
    const length = Math.min(capture?.duration ?? DEFAULT_EVENT_MINUTES, room);
    const created = await deps.events.create({
      title,
      startDate: capture?.date ?? today,
      ...(start ? { startTime: start } : {}),
      ...(start && length > 0
        ? {
            endDate: capture?.date ?? today,
            endTime: endTimeOf(start, length),
          }
        : {}),
      ...(capture?.rrule ? { rrule: capture.rrule } : {}),
      ...(capture && capture.tags.length > 0
        ? { tagIds: capture.tags.map((tag) => tag.id) }
        : {}),
    });
    if (created) deps.view.setSection("events");
  } else if (what === "habit") {
    const days = (capture?.days ?? []).map((code) => DAY_CODES.indexOf(code));
    const created = await deps.habits.create({
      title,
      ...(days.length > 0 ? { days: maskOf(days) } : {}),
      ...(capture?.time ? { time: capture.time } : {}),
      ...(capture?.tags[0] ? { tagId: capture.tags[0].id } : {}),
    });
    if (created) deps.view.setSection("habits");
  } else {
    deps.view.setSection("notes");
    await deps.notes.ensureLoaded();
    const note = await deps.notes.createNote();
    if (note) await deps.notes.saveTitle(note.id, text);
  }
}
