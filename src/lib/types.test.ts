import { describe, expect, it } from "vitest";

import fixtureJson from "../../src-tauri/fixtures/dto.json";
import { parseError } from "./api";
import type {
  DayGroup,
  EffectApplied,
  Reminder,
  Repeat,
  SearchHit,
  Tag,
  TagWithCount,
  Task,
  TasksSections,
} from "./types";

// Written by `cargo test contract` from the real Rust DTOs (src-tauri/src/contract_tests.rs).
interface Fixture {
  task: Task;
  taskWithoutOptionals: Task;
  tag: Tag;
  tagWithoutParent: Tag;
  tagWithCount: TagWithCount;
  dayGroup: DayGroup;
  sectionsOutput: TasksSections;
  rangeOutput: Task[];
  effectApplied: EffectApplied;
  searchHit: SearchHit;
}

const fixture = fixtureJson as unknown as Fixture;

// `Record<keyof T, true>` fails to compile if a field is missing or extra, so adding a field to
// a TypeScript type forces an update here, and the key comparison catches drift from Rust.
const taskKeys: Record<keyof Task, true> = {
  id: true,
  title: true,
  contentJson: true,
  dueDate: true,
  dueTime: true,
  durationMinutes: true,
  isCompleted: true,
  completedAt: true,
  position: true,
  tagIds: true,
  reminders: true,
  createdAt: true,
  updatedAt: true,
};
const reminderKeys: Record<keyof Reminder, true> = {
  id: true,
  notifyAt: true,
  offsetMinutes: true,
  isSent: true,
  repeat: true,
};
const repeatKeys: Record<keyof Repeat, true> = {
  every: true,
  unit: true,
  count: true,
  until: true,
  occurrence: true,
};
const tagKeys: Record<keyof Tag, true> = {
  id: true,
  name: true,
  colorHex: true,
  parentId: true,
  position: true,
};
const tagWithCountKeys: Record<keyof TagWithCount, true> = {
  ...tagKeys,
  pendingCount: true,
};
const dayGroupKeys: Record<keyof DayGroup, true> = { date: true, tasks: true };
const sectionsKeys: Record<keyof TasksSections, true> = {
  todo: true,
  agenda: true,
};
const effectKeys: Record<keyof EffectApplied, true> = {
  requested: true,
  applied: true,
};

const searchHitKeys: Record<keyof SearchHit, true> = {
  taskId: true,
  title: true,
  snippet: true,
  dueDate: true,
  isCompleted: true,
};

function keys(value: object): string[] {
  return Object.keys(value).sort();
}

describe("Rust DTO serialization matches the TypeScript types", () => {
  it("Task and Reminder", () => {
    for (const task of [
      fixture.task,
      fixture.taskWithoutOptionals,
      ...fixture.rangeOutput,
    ]) {
      expect(keys(task)).toEqual(keys(taskKeys));
      for (const reminder of task.reminders) {
        expect(keys(reminder)).toEqual(keys(reminderKeys));
        if (reminder.repeat) {
          expect(keys(reminder.repeat)).toEqual(keys(repeatKeys));
        }
      }
    }
  });

  it("serializes absent optionals as null, never omitting the key", () => {
    const task = fixture.taskWithoutOptionals;
    expect([task.dueDate, task.dueTime, task.completedAt]).toEqual([
      null,
      null,
      null,
    ]);
    expect(task.reminders[0].offsetMinutes).toBeNull();
    expect(fixture.tagWithoutParent.parentId).toBeNull();
  });

  it("uses the primitive types the interfaces declare", () => {
    const task = fixture.task;
    expect(typeof task.isCompleted).toBe("boolean");
    expect(typeof task.position).toBe("number");
    expect(typeof task.contentJson).toBe("string");
    expect(typeof task.dueDate).toBe("string");
    expect(Array.isArray(task.tagIds)).toBe(true);
    expect(typeof task.reminders[0].offsetMinutes).toBe("number");
    expect(typeof task.reminders[0].isSent).toBe("boolean");
  });

  it("Tag and TagWithCount (flattened pendingCount)", () => {
    expect(keys(fixture.tag)).toEqual(keys(tagKeys));
    expect(keys(fixture.tagWithCount)).toEqual(keys(tagWithCountKeys));
    expect(fixture.tagWithCount.pendingCount).toBe(4);
  });

  it("task list outputs", () => {
    expect(keys(fixture.sectionsOutput)).toEqual(keys(sectionsKeys));
    expect(keys(fixture.dayGroup)).toEqual(keys(dayGroupKeys));
    expect(keys(fixture.sectionsOutput.agenda[0])).toEqual(keys(dayGroupKeys));
    expect(Array.isArray(fixture.rangeOutput)).toBe(true);
  });

  it("EffectApplied", () => {
    expect(keys(fixture.effectApplied)).toEqual(keys(effectKeys));
    expect(fixture.effectApplied).toEqual({
      requested: "mica",
      applied: "acrylic",
    });
  });
});

describe("SearchHit", () => {
  it("matches the Rust serialization", () => {
    expect(keys(fixture.searchHit)).toEqual(keys(searchHitKeys));
    expect(fixture.searchHit.dueDate).toBeNull();
  });
});

describe("parseError", () => {
  it("splits the stable code from the message", () => {
    expect(parseError("NOT_FOUND: tarea x no encontrada")).toEqual({
      code: "NOT_FOUND",
      message: "tarea x no encontrada",
    });
    expect(parseError("VALIDATION: a: b")).toEqual({
      code: "VALIDATION",
      message: "a: b",
    });
  });

  it("falls back to INTERNAL for unknown or malformed errors", () => {
    expect(parseError("BOOM: algo")).toEqual({
      code: "INTERNAL",
      message: "BOOM: algo",
    });
    expect(parseError("sin código")).toEqual({
      code: "INTERNAL",
      message: "sin código",
    });
    expect(parseError(new Error("x")).code).toBe("INTERNAL");
  });
});
